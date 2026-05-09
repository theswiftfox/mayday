// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
#[cfg(feature = "http-server")]
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Integration not configured: {0}")]
    NotConfigured(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[cfg(feature = "http-server")]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::{http::StatusCode, Json};

        let (status, code, message) = match &self {
            AppError::NotConfigured(msg) => {
                (StatusCode::BAD_REQUEST, "not_configured", msg.clone())
            }
            AppError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg.clone())
            }
            AppError::ExternalApi(msg) => {
                tracing::error!(error = %msg, "External API error");
                (StatusCode::BAD_GATEWAY, "external_api_error", "An external service request failed".to_string())
            }
            AppError::Request(e) => {
                tracing::error!(error = %e, "External request failed");
                (StatusCode::BAD_GATEWAY, "request_failed", "An external service request failed".to_string())
            }
            AppError::Internal(e) => {
                // Log the real error but return a generic message to the client
                tracing::error!(error = %e, "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "An internal error occurred".to_string())
            }
        };

        tracing::error!(%status, %code, %message, "Request error");

        let body = json!({
            "error": message,
            "code": code,
        });

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
