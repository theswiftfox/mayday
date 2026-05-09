// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::responses::DataResponse;
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
    let response =
        serde_json::to_value(DataResponse { data: &tickets }).map_err(|e| AppError::Internal(e.into()))?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(Json(response))
}

async fn get_ticket_detail(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> AppResult<Json<Value>> {
    // Validate ticket key format (e.g., PROJ-123)
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        || key.is_empty()
        || key.len() > 50
    {
        return Err(AppError::Validation("Invalid ticket key format".to_string()));
    }

    let config = state.config.read().await;
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("jira".to_string()))?;

    let detail = services::jira::fetch_ticket_detail(&state.http_client, jira_config, &key).await?;
    Ok(Json(
        serde_json::to_value(DataResponse { data: &detail }).map_err(|e| AppError::Internal(e.into()))?,
    ))
}
