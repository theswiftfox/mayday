use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mrs", get(list_mrs))
        .route("/mrs/{project_id}/{iid}", get(get_mr_detail))
        .route("/pipelines", get(list_pipelines))
}

async fn list_mrs(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?;

    let mrs = services::gitlab::fetch_mrs(&state.http_client, gl_config).await?;
    Ok(Json(json!({ "data": mrs })))
}

async fn get_mr_detail(
    State(state): State<AppState>,
    Path((project_id, iid)): Path<(u64, u64)>,
) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?;

    let detail = services::gitlab::fetch_mr_detail(
        &state.http_client,
        gl_config,
        project_id,
        iid,
    )
    .await?;

    Ok(Json(json!({ "data": detail })))
}

async fn list_pipelines(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("gitlab".to_string()))?;

    let pipelines = services::gitlab::fetch_pipelines(&state.http_client, gl_config).await?;
    Ok(Json(json!({ "data": pipelines })))
}
