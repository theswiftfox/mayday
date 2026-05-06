use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::{CalendarConfig, GeneralConfig, GitHubConfig, GitLabConfig, JiraConfig};
use crate::error::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_config).put(update_config))
}

async fn get_config(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;

    // Return config with secrets masked
    let masked = json!({
        "github": config.github.as_ref().map(|c| json!({
            "username": c.username,
            "repos": c.repos,
            "poll_interval_secs": c.poll_interval_secs,
            "has_token": !c.token.is_empty(),
            "oauth_client_id": c.oauth_client_id,
            "token_source": c.token_source,
        })),
        "jira": config.jira.as_ref().map(|c| json!({
            "host": c.host,
            "email": c.email,
            "project_keys": c.project_keys,
            "poll_interval_secs": c.poll_interval_secs,
            "has_token": !c.api_token.is_empty(),
        })),
        "gitlab": config.gitlab.as_ref().map(|c| json!({
            "host": c.host,
            "username": c.username,
            "project_ids": c.project_ids,
            "poll_interval_secs": c.poll_interval_secs,
            "has_token": !c.token.is_empty(),
        })),
        "calendar": config.calendar.as_ref().map(|c| json!({
            "source": c.source,
            "ics_url": c.ics_url,
            "ms_client_id": c.ms_client_id,
            "ms_tenant_id": c.ms_tenant_id,
            "ms_redirect_uri": c.ms_redirect_uri,
            "has_ms_refresh_token": c.ms_refresh_token.is_some(),
            "poll_interval_secs": c.poll_interval_secs,
        })),
        "general": config.general,
    });

    Ok(Json(masked))
}

/// The shape the frontend sends when saving settings
#[derive(Debug, Deserialize)]
struct UpdateConfigRequest {
    github: Option<GitHubFormData>,
    jira: Option<JiraFormData>,
    gitlab: Option<GitLabFormData>,
    calendar: Option<CalendarFormData>,
    general: Option<GeneralFormData>,
}

#[derive(Debug, Deserialize)]
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
struct GitLabFormData {
    #[serde(default)]
    host: String,
    #[serde(default)]
    token: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    project_ids: String, // comma-separated
    #[serde(default = "default_poll")]
    poll_interval: u64,
}

#[derive(Debug, Deserialize)]
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
                poll_interval_secs: gh.poll_interval,
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
                    poll_interval_secs: jira.poll_interval,
                });
            }
        }

        // Update GitLab config
        if let Some(gl) = form.gitlab {
            if !gl.host.is_empty() {
                let project_ids: Vec<u64> = gl
                    .project_ids
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u64>().ok())
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
                    project_ids,
                    poll_interval_secs: gl.poll_interval,
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
                poll_interval_secs: cal.poll_interval,
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

    Ok(Json(json!({ "status": "saved" })))
}
