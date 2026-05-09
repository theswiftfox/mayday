// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::{CalendarConfig, DashboardConfig, GeneralConfig, GitHubConfig, GitLabConfig, GitLabProject, JiraConfig};
use crate::error::{AppError, AppResult};
use crate::responses::{
    MaskedCalendarConfig, MaskedConfig, MaskedGitHubConfig, MaskedGitLabConfig, MaskedJiraConfig,
    StatusResponse,
};
use crate::state::AppState;

/// Clamp poll interval to a safe range (30s to 3600s).
fn clamp_poll_interval(secs: u64) -> u64 {
    secs.clamp(30, 3600)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_config).put(update_config))
        .route("/dashboard", get(get_dashboard_config).put(update_dashboard_config))
}

async fn get_config(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;

    let masked = MaskedConfig {
        github: config.github.as_ref().map(MaskedGitHubConfig::from),
        jira: config.jira.as_ref().map(MaskedJiraConfig::from),
        gitlab: config.gitlab.as_ref().map(MaskedGitLabConfig::from),
        calendar: config.calendar.as_ref().map(MaskedCalendarConfig::from),
        general: config.general.clone(),
    };

    Ok(Json(
        serde_json::to_value(masked).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

/// The shape the frontend sends when saving settings
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateConfigRequest {
    github: Option<GitHubFormData>,
    jira: Option<JiraFormData>,
    gitlab: Option<GitLabFormData>,
    calendar: Option<CalendarFormData>,
    general: Option<GeneralFormData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubFormData {
    #[serde(default)]
    token: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    repos: String, // comma-separated
    #[serde(default = "default_poll")]
    poll_interval: u64,
    #[serde(default)]
    oauth_client_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraFormData {
    #[serde(default)]
    host: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    api_token: String,
    #[serde(default)]
    project_keys: String, // comma-separated
    #[serde(default = "default_poll")]
    poll_interval: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitLabFormData {
    #[serde(default)]
    host: String,
    #[serde(default)]
    token: String,
    #[serde(default)]
    username: String,
    /// Resolved projects with both id and path
    #[serde(default)]
    projects: Vec<GitLabProjectForm>,
    #[serde(default = "default_poll")]
    poll_interval: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitLabProjectForm {
    id: u64,
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarFormData {
    #[serde(default)]
    source: String,
    #[serde(default)]
    ics_url: String,
    #[serde(default)]
    ms_client_id: String,
    #[serde(default)]
    ms_tenant_id: String,
    #[serde(default)]
    ms_redirect_uri: String,
    #[serde(default = "default_poll")]
    poll_interval: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeneralFormData {
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default = "default_true")]
    refresh_on_focus: bool,
}

fn default_poll() -> u64 {
    300
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_true() -> bool {
    true
}

async fn update_config(
    State(state): State<AppState>,
    Json(form): Json<UpdateConfigRequest>,
) -> AppResult<Json<Value>> {
    {
        let mut config = state.config.write().await;

        // Update GitHub config - merge with existing (preserve token from OAuth flows)
        if let Some(gh) = form.github {
            let existing = config.github.take().unwrap_or_else(|| GitHubConfig {
                token: String::new(),
                username: String::new(),
                repos: vec![],
                poll_interval_secs: 300,
                oauth_client_id: None,
                token_source: "manual".to_string(),
            });

            let repos: Vec<String> = gh
                .repos
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            config.github = Some(GitHubConfig {
                // Only update token if the user provided one (non-empty)
                token: if gh.token.is_empty() { existing.token } else { gh.token },
                username: if gh.username.is_empty() { existing.username } else { gh.username },
                repos,
                poll_interval_secs: clamp_poll_interval(gh.poll_interval),
                oauth_client_id: if gh.oauth_client_id.is_empty() {
                    existing.oauth_client_id
                } else {
                    Some(gh.oauth_client_id)
                },
                token_source: existing.token_source,
            });
        }

        // Update JIRA config
        if let Some(jira) = form.jira {
            if !jira.host.is_empty() {
                let project_keys: Vec<String> = jira
                    .project_keys
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let existing_token = config
                    .jira
                    .as_ref()
                    .map(|c| c.api_token.clone())
                    .unwrap_or_default();

                config.jira = Some(JiraConfig {
                    host: jira.host,
                    email: jira.email,
                    api_token: if jira.api_token.is_empty() { existing_token } else { jira.api_token },
                    project_keys,
                    poll_interval_secs: clamp_poll_interval(jira.poll_interval),
                });
            }
        }

        // Update GitLab config
        if let Some(gl) = form.gitlab {
            if !gl.host.is_empty() {
                let projects: Vec<GitLabProject> = gl.projects.into_iter()
                    .map(|p| GitLabProject { id: p.id, path: p.path })
                    .collect();

                let existing_token = config
                    .gitlab
                    .as_ref()
                    .map(|c| c.token.clone())
                    .unwrap_or_default();

                config.gitlab = Some(GitLabConfig {
                    host: gl.host,
                    token: if gl.token.is_empty() { existing_token } else { gl.token },
                    username: gl.username,
                    projects,
                    poll_interval_secs: clamp_poll_interval(gl.poll_interval),
                });
            }
        }

        // Update Calendar config
        if let Some(cal) = form.calendar {
            let source = if cal.source.is_empty() {
                "ics".to_string()
            } else {
                cal.source
            };

            // Preserve existing refresh token if not provided
            let existing_refresh = config
                .calendar
                .as_ref()
                .and_then(|c| c.ms_refresh_token.clone());

            config.calendar = Some(CalendarConfig {
                source,
                ics_url: if cal.ics_url.is_empty() {
                    None
                } else {
                    Some(cal.ics_url)
                },
                ms_client_id: if cal.ms_client_id.is_empty() {
                    None
                } else {
                    Some(cal.ms_client_id)
                },
                ms_tenant_id: if cal.ms_tenant_id.is_empty() {
                    None
                } else {
                    Some(cal.ms_tenant_id)
                },
                ms_refresh_token: existing_refresh,
                ews_url: None,
                ms_redirect_uri: if cal.ms_redirect_uri.is_empty() {
                    None
                } else {
                    Some(cal.ms_redirect_uri)
                },
                poll_interval_secs: clamp_poll_interval(cal.poll_interval),
            });
        }

        // Update general config
        if let Some(general) = form.general {
            config.general = GeneralConfig {
                refresh_on_focus: general.refresh_on_focus,
                theme: general.theme,
            };
        }
    }

    state.save_config().await.map_err(crate::error::AppError::Internal)?;
    state.api_cache.invalidate_all();

    Ok(Json(
        serde_json::to_value(StatusResponse { status: "saved".to_string() })
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

// ---- Dashboard config endpoints ----

async fn get_dashboard_config(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    Ok(Json(
        serde_json::to_value(&config.dashboard).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

async fn update_dashboard_config(
    State(state): State<AppState>,
    Json(dashboard): Json<DashboardConfig>,
) -> AppResult<Json<Value>> {
    // Validate calendar layout
    if !["sidebar", "inline"].contains(&dashboard.calendar_layout.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid calendar layout: {}",
            dashboard.calendar_layout
        )));
    }

    // Validate section types
    const VALID_SECTIONS: &[&str] = &[
        "important",
        "github_pr",
        "gitlab",
        "gitlab_mr",
        "gitlab_pipeline",
        "jira_ticket",
        "calendar_event",
    ];
    for section in &dashboard.section_order {
        if !VALID_SECTIONS.contains(&section.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid section type: {section}"
            )));
        }
    }
    for section in &dashboard.visible_sections {
        if !VALID_SECTIONS.contains(&section.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid section type: {section}"
            )));
        }
    }

    {
        let mut config = state.config.write().await;
        config.dashboard = dashboard;
    }

    state.save_config().await.map_err(crate::error::AppError::Internal)?;

    Ok(Json(
        serde_json::to_value(StatusResponse { status: "saved".to_string() })
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}
