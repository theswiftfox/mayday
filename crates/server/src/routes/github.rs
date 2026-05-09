// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::responses::{AuthResult, DataResponse, DeviceCodePollResponse, GhDeviceCodeStartResponse};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/prs", get(list_prs))
        .route("/prs/{owner}/{repo}/{number}", get(get_pr_detail))
        .route("/auth/detect-gh-cli", post(detect_gh_cli))
        .route("/auth/token", post(use_manual_token))
        .route("/auth/device-code/start", post(start_device_code))
        .route("/auth/device-code/poll", post(poll_device_code))
}

async fn list_prs(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let cache_key = "github_prs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    let config = state.config.read().await.clone();
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("github".to_string()))?;

    let prs = services::github::fetch_prs(&state.http_client, gh_config).await?;
    let response =
        serde_json::to_value(DataResponse { data: &prs }).map_err(|e| AppError::Internal(e.into()))?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(Json(response))
}

async fn get_pr_detail(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, u64)>,
) -> AppResult<Json<Value>> {
    let gh_config = {
        let config = state.config.read().await;
        config
            .github
            .clone()
            .ok_or_else(|| AppError::NotConfigured("github".to_string()))?
    };

    let detail = services::github::fetch_pr_detail(
        &state.http_client,
        &gh_config,
        &owner,
        &repo,
        number,
    )
    .await?;

    Ok(Json(
        serde_json::to_value(DataResponse { data: &detail }).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

/// Attempt to detect and use the existing `gh` CLI token
async fn detect_gh_cli(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let detected = tokio::task::spawn_blocking(services::github::detect_gh_cli_token)
        .await
        .unwrap_or(None);
    match detected {
        Some((token, username)) => {
            // Verify the token works and get username if empty
            let actual_username = if username.is_empty() {
                services::github::fetch_authenticated_user(&state.http_client, &token).await?
            } else {
                username
            };

            // Save to config
            {
                let mut config = state.config.write().await;
                let gh_config = config.github.get_or_insert(crate::config::GitHubConfig {
                    token: String::new(),
                    username: String::new(),
                    repos: vec![],
                    poll_interval_secs: 300,
                    oauth_client_id: None,
                    token_source: "gh_cli".to_string(),
                });
                gh_config.token = token;
                gh_config.username = actual_username.clone();
                gh_config.token_source = "gh_cli".to_string();
            }
            state.save_config().await.map_err(crate::error::AppError::Internal)?;
            state.api_cache.invalidate_all();

            Ok(Json(
                serde_json::to_value(AuthResult {
                    success: true,
                    username: actual_username,
                    source: "gh_cli".to_string(),
                })
                .map_err(|e| AppError::Internal(e.into()))?,
            ))
        }
        None => Err(crate::error::AppError::NotConfigured(
            "gh CLI not found or not authenticated. Run `gh auth login` first, or use the device code flow.".to_string()
        )),
    }
}

#[derive(Deserialize)]
struct ManualTokenRequest {
    token: String,
}

/// Validate and save a manually-provided GitHub token
async fn use_manual_token(
    State(state): State<AppState>,
    Json(body): Json<ManualTokenRequest>,
) -> AppResult<Json<Value>> {
    let token = body.token.trim().to_string();
    if token.is_empty() {
        return Err(crate::error::AppError::NotConfigured(
            "Token cannot be empty.".to_string()
        ));
    }

    let username = services::github::fetch_authenticated_user(&state.http_client, &token).await?;

    {
        let mut config = state.config.write().await;
        let gh_config = config.github.get_or_insert(crate::config::GitHubConfig {
            token: String::new(),
            username: String::new(),
            repos: vec![],
            poll_interval_secs: 300,
            oauth_client_id: None,
            token_source: "manual".to_string(),
        });
        gh_config.token = token;
        gh_config.username = username.clone();
        gh_config.token_source = "manual".to_string();
    }
    state.save_config().await.map_err(crate::error::AppError::Internal)?;
    state.api_cache.invalidate_all();

    Ok(Json(
        serde_json::to_value(AuthResult {
            success: true,
            username,
            source: "manual".to_string(),
        })
        .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceCodeStartRequest {
    client_id: String,
}

/// Start the GitHub device code OAuth flow
async fn start_device_code(
    State(state): State<AppState>,
    Json(body): Json<DeviceCodeStartRequest>,
) -> AppResult<Json<Value>> {
    let device_code =
        services::github::start_device_code_flow(&state.http_client, &body.client_id).await?;

    Ok(Json(
        serde_json::to_value(GhDeviceCodeStartResponse::from(&device_code))
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceCodePollRequest {
    client_id: String,
    device_code: String,
}

/// Poll for the access token (frontend calls this repeatedly until success)
async fn poll_device_code(
    State(state): State<AppState>,
    Json(body): Json<DeviceCodePollRequest>,
) -> AppResult<Json<Value>> {
    let result = services::github::poll_device_code_token(
        &state.http_client,
        &body.client_id,
        &body.device_code,
    )
    .await?;

    match result {
        Some(token_response) => {
            // Get the authenticated user
            let username = services::github::fetch_authenticated_user(
                &state.http_client,
                &token_response.access_token,
            )
            .await?;

            // Save to config
            {
                let mut config = state.config.write().await;
                let gh_config = config.github.get_or_insert(crate::config::GitHubConfig {
                    token: String::new(),
                    username: String::new(),
                    repos: vec![],
                    poll_interval_secs: 300,
                    oauth_client_id: Some(body.client_id.clone()),
                    token_source: "device_code".to_string(),
                });
                gh_config.token = token_response.access_token;
                gh_config.username = username.clone();
                gh_config.token_source = "device_code".to_string();
                gh_config.oauth_client_id = Some(body.client_id);
            }
            state.save_config().await.map_err(crate::error::AppError::Internal)?;
            state.api_cache.invalidate_all();

            Ok(Json(
                serde_json::to_value(DeviceCodePollResponse {
                    status: "complete".to_string(),
                    username: Some(username),
                    error: None,
                })
                .map_err(|e| AppError::Internal(e.into()))?,
            ))
        }
        None => Ok(Json(
            serde_json::to_value(DeviceCodePollResponse {
                status: "pending".to_string(),
                username: None,
                error: None,
            })
            .map_err(|e| AppError::Internal(e.into()))?,
        )),
    }
}
