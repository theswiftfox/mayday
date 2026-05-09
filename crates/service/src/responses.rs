// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner

//! Typed API response structs shared by both axum route handlers and Tauri IPC commands.
//!
//! Every response struct derives `Serialize` with `rename_all = "camelCase"`,
//! ensuring all JSON keys sent to the frontend are consistently camelCase
//! without any manual `json!()` key strings.

use serde::Serialize;

use crate::config::{
    CalendarConfig, GeneralConfig, GitHubConfig, GitLabConfig, GitLabProject, JiraConfig,
};
use crate::services::calendar::{CalendarEvent, DeviceCodeResponse as CalDeviceCodeResponse};
use crate::services::github::{DeviceCodeResponse as GhDeviceCodeResponse, GitHubPR};
use crate::services::gitlab::{GitLabMR, GitLabPipeline};
use crate::services::jira::JiraTicket;

// ─── Generic Wrappers ────────────────────────────────────────────────────────

/// Wraps a single `data` field — used by list and detail endpoints.
#[derive(Debug, Serialize)]
pub struct DataResponse<T: Serialize> {
    pub data: T,
}

/// Simple `{ "status": "..." }` acknowledgement.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
}

// ─── Dashboard ───────────────────────────────────────────────────────────────

/// Top-level dashboard response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
    pub items: Vec<DashboardItem>,
    pub errors: Vec<DashboardError>,
    pub last_updated: String,
}

/// A tagged-union item in the dashboard feed.
///
/// Serializes as `{ "type": "github_pr", "data": { ... } }`.
/// The variant tag values are kept as snake_case strings because they are
/// discriminator values, not field names.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DashboardItem {
    #[serde(rename = "github_pr")]
    GitHubPr(GitHubPR),
    #[serde(rename = "jira_ticket")]
    JiraTicket(JiraTicket),
    #[serde(rename = "gitlab_mr")]
    GitLabMr(GitLabMR),
    #[serde(rename = "gitlab_pipeline")]
    GitLabPipeline(GitLabPipeline),
    #[serde(rename = "calendar_event")]
    CalendarEvent(CalendarEvent),
}

/// An error from a single integration source.
#[derive(Debug, Serialize)]
pub struct DashboardError {
    pub source: String,
    pub message: String,
}

// ─── Config (masked — secrets stripped) ──────────────────────────────────────

/// The full config response with tokens replaced by `hasToken` booleans.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedConfig {
    pub github: Option<MaskedGitHubConfig>,
    pub jira: Option<MaskedJiraConfig>,
    pub gitlab: Option<MaskedGitLabConfig>,
    pub calendar: Option<MaskedCalendarConfig>,
    pub general: GeneralConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedGitHubConfig {
    pub username: String,
    pub repos: Vec<String>,
    pub poll_interval_secs: u64,
    pub has_token: bool,
    pub oauth_client_id: Option<String>,
    pub token_source: String,
}

impl From<&GitHubConfig> for MaskedGitHubConfig {
    fn from(c: &GitHubConfig) -> Self {
        Self {
            username: c.username.clone(),
            repos: c.repos.clone(),
            poll_interval_secs: c.poll_interval_secs,
            has_token: !c.token.is_empty(),
            oauth_client_id: c.oauth_client_id.clone(),
            token_source: c.token_source.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedJiraConfig {
    pub host: String,
    pub email: String,
    pub project_keys: Vec<String>,
    pub poll_interval_secs: u64,
    pub has_token: bool,
}

impl From<&JiraConfig> for MaskedJiraConfig {
    fn from(c: &JiraConfig) -> Self {
        Self {
            host: c.host.clone(),
            email: c.email.clone(),
            project_keys: c.project_keys.clone(),
            poll_interval_secs: c.poll_interval_secs,
            has_token: !c.api_token.is_empty(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedGitLabConfig {
    pub host: String,
    pub username: String,
    pub projects: Vec<GitLabProject>,
    pub poll_interval_secs: u64,
    pub has_token: bool,
}

impl From<&GitLabConfig> for MaskedGitLabConfig {
    fn from(c: &GitLabConfig) -> Self {
        Self {
            host: c.host.clone(),
            username: c.username.clone(),
            projects: c.projects.clone(),
            poll_interval_secs: c.poll_interval_secs,
            has_token: !c.token.is_empty(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedCalendarConfig {
    pub source: String,
    pub ics_url: Option<String>,
    pub ms_client_id: Option<String>,
    pub ms_tenant_id: Option<String>,
    pub ms_redirect_uri: Option<String>,
    pub has_ms_refresh_token: bool,
    pub poll_interval_secs: u64,
}

impl From<&CalendarConfig> for MaskedCalendarConfig {
    fn from(c: &CalendarConfig) -> Self {
        Self {
            source: c.source.clone(),
            ics_url: c.ics_url.clone(),
            ms_client_id: c.ms_client_id.clone(),
            ms_tenant_id: c.ms_tenant_id.clone(),
            ms_redirect_uri: c.ms_redirect_uri.clone(),
            has_ms_refresh_token: c.ms_refresh_token.as_ref().map(|t| !t.is_empty()).unwrap_or(false),
            poll_interval_secs: c.poll_interval_secs,
        }
    }
}

// ─── Auth Responses ──────────────────────────────────────────────────────────

/// Successful authentication result (GitHub CLI detect / manual token / device code).
#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub success: bool,
    pub username: String,
    pub source: String,
}

/// GitHub device code flow start — all fields from the GitHub API response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GhDeviceCodeStartResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

impl From<&GhDeviceCodeResponse> for GhDeviceCodeStartResponse {
    fn from(r: &GhDeviceCodeResponse) -> Self {
        Self {
            device_code: r.device_code.clone(),
            user_code: r.user_code.clone(),
            verification_uri: r.verification_uri.clone(),
            expires_in: r.expires_in,
            interval: r.interval,
        }
    }
}

/// Device code poll result — shared by GitHub and Calendar.
///
/// GitHub sets `username` on completion; Calendar sets `error` on failure.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePollResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ─── Calendar Auth ───────────────────────────────────────────────────────────

/// Calendar OAuth start response — contains the auth URL to open in the browser.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAuthStartResponse {
    pub auth_url: String,
    pub source: String,
    pub flow: String,
    /// Random state parameter for correlating the PKCE verifier on callback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Calendar device code flow start — user_code and verification_uri for user.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDeviceCodeStartResponse {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

impl From<&CalDeviceCodeResponse> for CalendarDeviceCodeStartResponse {
    fn from(r: &CalDeviceCodeResponse) -> Self {
        Self {
            user_code: r.user_code.clone(),
            verification_uri: r.verification_uri.clone(),
            expires_in: r.expires_in,
            interval: r.interval,
        }
    }
}

/// Calendar auth status — whether the Microsoft calendar is connected.
#[derive(Debug, Serialize)]
pub struct CalendarAuthStatusResponse {
    pub connected: bool,
}

// ─── GitLab ──────────────────────────────────────────────────────────────────

/// Resolved GitLab project (id + path) returned by project resolution endpoint.
#[derive(Debug, Serialize)]
pub struct ResolvedGitLabProject {
    pub id: u64,
    pub path: String,
}
