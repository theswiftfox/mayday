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
        .route("/tickets", get(list_tickets))
        .route("/tickets/{key}", get(get_ticket_detail))
}

async fn list_tickets(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let cache_key = "jira_tickets".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    let config = state.config.read().await.clone();
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("jira".to_string()))?;

    let tickets = services::jira::fetch_tickets(&state.http_client, jira_config).await?;
    let response = json!({ "data": tickets });
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(Json(response))
}

async fn get_ticket_detail(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("jira".to_string()))?;

    let detail = services::jira::fetch_ticket_detail(&state.http_client, jira_config, &key).await?;
    Ok(Json(json!({ "data": detail })))
}
