// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_dashboard))
}

async fn get_dashboard(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let response = services::dashboard::aggregate(&state)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(response))
}
