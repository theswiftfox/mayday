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
use crate::responses::{DataResponse, ResolvedGitLabProject};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mrs", get(list_mrs))
        .route("/mrs/{project_id}/{iid}", get(get_mr_detail))
        .route("/pipelines", get(list_pipelines))
        .route("/resolve-project", post(resolve_project))
}

async fn list_mrs(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let cache_key = "gitlab_mrs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?;

    let mrs = services::gitlab::fetch_mrs(&state.http_client, gl_config).await?;
    let response =
        serde_json::to_value(DataResponse { data: &mrs }).map_err(|e| AppError::Internal(e.into()))?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(Json(response))
}

async fn get_mr_detail(
    State(state): State<AppState>,
    Path((project_id, iid)): Path<(u64, u64)>,
) -> AppResult<Json<Value>> {
    let gl_config = {
        let config = state.config.read().await;
        config
            .gitlab
            .clone()
            .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?
    };

    let detail = services::gitlab::fetch_mr_detail(
        &state.http_client,
        &gl_config,
        project_id,
        iid,
    )
    .await?;

    Ok(Json(
        serde_json::to_value(DataResponse { data: &detail }).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

async fn list_pipelines(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let cache_key = "gitlab_pipelines".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?;

    let pipelines = services::gitlab::fetch_pipelines(&state.http_client, gl_config).await?;
    let response =
        serde_json::to_value(DataResponse { data: &pipelines }).map_err(|e| AppError::Internal(e.into()))?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct ResolveProjectRequest {
    /// The GitLab host (e.g., "gitlab.com")
    host: String,
    /// Project path (e.g., "apricot/cedar")
    path: String,
}

/// Resolve a GitLab project path to its numeric ID via the GitLab API
async fn resolve_project(
    State(state): State<AppState>,
    Json(req): Json<ResolveProjectRequest>,
) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let token = config
        .gitlab
        .as_ref()
        .map(|c| c.token.clone())
        .unwrap_or_default();

    if token.is_empty() {
        return Err(AppError::NotConfigured("gitlab (no token)".to_string()));
    }

    let host = if req.host.is_empty() {
        config
            .gitlab
            .as_ref()
            .map(|c| c.host.clone())
            .unwrap_or_else(|| "gitlab.com".to_string())
    } else {
        req.host
    };

    // URL-encode the project path for the API call
    let encoded_path = urlencoding::encode(&req.path);
    let url = format!("https://{host}/api/v4/projects/{encoded_path}");

    let resp = state
        .http_client
        .get(&url)
        .header("PRIVATE-TOKEN", &token)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "GitLab API returned {} for project '{}'",
            resp.status(),
            req.path
        )));
    }

    let project: Value = resp.json().await.map_err(|e| AppError::Internal(e.into()))?;
    let id = project["id"].as_u64().ok_or_else(|| {
        AppError::Internal(anyhow::anyhow!("No 'id' field in GitLab project response"))
    })?;
    let path_with_namespace = project["path_with_namespace"]
        .as_str()
        .unwrap_or(&req.path)
        .to_string();

    Ok(Json(
        serde_json::to_value(ResolvedGitLabProject {
            id,
            path: path_with_namespace,
        })
        .map_err(|e| AppError::Internal(e.into()))?,
    ))
}
