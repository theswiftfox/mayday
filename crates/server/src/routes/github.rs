use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/prs", get(list_prs))
        .route("/prs/{owner}/{repo}/{number}", get(get_pr_detail))
        .route("/auth/detect-gh-cli", post(detect_gh_cli))
        .route("/auth/device-code/start", post(start_device_code))
        .route("/auth/device-code/poll", post(poll_device_code))
}

async fn list_prs(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("github".to_string()))?;

    let prs = services::github::fetch_prs(&state.http_client, gh_config).await?;
    Ok(Json(json!({ "data": prs })))
}

async fn get_pr_detail(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, u64)>,
) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("github".to_string()))?;

    let detail = services::github::fetch_pr_detail(
        &state.http_client,
        gh_config,
        &owner,
        &repo,
        number,
    )
    .await?;

    Ok(Json(json!({ "data": detail })))
}

/// Attempt to detect and use the existing `gh` CLI token
async fn detect_gh_cli(State(state): State<AppState>) -> AppResult<Json<Value>> {
    match services::github::detect_gh_cli_token() {
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

            Ok(Json(json!({
                "success": true,
                "username": actual_username,
                "source": "gh_cli",
            })))
        }
        None => Ok(Json(json!({
            "success": false,
            "message": "gh CLI not found or not authenticated. Run `gh auth login` first, or use the device code flow.",
        }))),
    }
}

#[derive(Deserialize)]
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

    Ok(Json(json!({
        "device_code": device_code.device_code,
        "user_code": device_code.user_code,
        "verification_uri": device_code.verification_uri,
        "expires_in": device_code.expires_in,
        "interval": device_code.interval,
    })))
}

#[derive(Deserialize)]
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

            Ok(Json(json!({
                "status": "complete",
                "username": username,
            })))
        }
        None => Ok(Json(json!({
            "status": "pending",
        }))),
    }
}
