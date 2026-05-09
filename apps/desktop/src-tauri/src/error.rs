// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//! Typed error enum for Tauri IPC commands.
//!
//! Serialized as `{ "code": "...", "message": "..." }` so the frontend can
//! programmatically distinguish error categories.

use myday_core::error::AppError;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotConfigured,
    Validation,
    NetworkError,
    ExternalApi,
    AuthFailed,
    Internal,
}

impl CommandError {
    pub fn not_configured(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::NotConfigured, message: msg.into() }
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::Validation, message: msg.into() }
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::NetworkError, message: msg.into() }
    }

    pub fn external_api(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::ExternalApi, message: msg.into() }
    }

    pub fn auth_failed(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::AuthFailed, message: msg.into() }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self { code: ErrorCode::Internal, message: msg.into() }
    }
}

impl From<AppError> for CommandError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::NotConfigured(msg) => Self::not_configured(msg),
            AppError::Validation(msg) => Self::validation(msg),
            AppError::ExternalApi(msg) => Self::external_api(msg),
            AppError::Request(e) => Self::network(e.to_string()),
            AppError::Internal(e) => Self::internal(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(e: serde_json::Error) -> Self {
        Self::internal(e.to_string())
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(e: anyhow::Error) -> Self {
        Self::internal(e.to_string())
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
