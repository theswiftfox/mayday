// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//! Tauri IPC commands — thin wrappers around the myday-server service layer.
//! These replace the HTTP route handlers when running as a Tauri desktop app.

use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use myday_server::config::{
    CalendarConfig, DashboardConfig, GeneralConfig, GitHubConfig, GitLabConfig, GitLabProject,
    JiraConfig,
};
use myday_server::responses::{
    AuthResult, CalendarAuthStartResponse, CalendarAuthStatusResponse,
    CalendarDeviceCodeStartResponse, DataResponse, DeviceCodePollResponse,
    GhDeviceCodeStartResponse, MaskedCalendarConfig, MaskedConfig, MaskedGitHubConfig,
    MaskedGitLabConfig, MaskedJiraConfig, ResolvedGitLabProject, StatusResponse,
};
use myday_server::services;
use myday_server::state::AppState;

use crate::error::CommandError;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn to_value<T: serde::Serialize>(v: T) -> Result<Value, CommandError> {
    serde_json::to_value(v).map_err(Into::into)
}

/// Clamp poll interval to a safe range (30s to 3600s).
fn clamp_poll_interval(secs: u64) -> u64 {
    secs.clamp(30, 3600)
}

// ─── Dashboard ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let response = services::dashboard::aggregate(state.inner()).await?;
    Ok(response)
}

// ─── Config ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let config = state.config.read().await;

    let masked = MaskedConfig {
        github: config.github.as_ref().map(MaskedGitHubConfig::from),
        jira: config.jira.as_ref().map(MaskedJiraConfig::from),
        gitlab: config.gitlab.as_ref().map(MaskedGitLabConfig::from),
        calendar: config.calendar.as_ref().map(MaskedCalendarConfig::from),
        general: config.general.clone(),
    };

    to_value(masked)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
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
    repos: String,
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
    project_keys: String,
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

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    request: UpdateConfigRequest,
) -> Result<Value, CommandError> {
    {
        let mut config = state.config.write().await;

        if let Some(gh) = request.github {
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
                token: if gh.token.is_empty() {
                    existing.token
                } else {
                    gh.token
                },
                username: if gh.username.is_empty() {
                    existing.username
                } else {
                    gh.username
                },
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

        if let Some(jira) = request.jira {
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
                    api_token: if jira.api_token.is_empty() {
                        existing_token
                    } else {
                        jira.api_token
                    },
                    project_keys,
                    poll_interval_secs: clamp_poll_interval(jira.poll_interval),
                });
            }
        }

        if let Some(gl) = request.gitlab {
            if !gl.host.is_empty() {
                let projects: Vec<GitLabProject> = gl
                    .projects
                    .into_iter()
                    .map(|p| GitLabProject {
                        id: p.id,
                        path: p.path,
                    })
                    .collect();

                let existing_token = config
                    .gitlab
                    .as_ref()
                    .map(|c| c.token.clone())
                    .unwrap_or_default();

                config.gitlab = Some(GitLabConfig {
                    host: gl.host,
                    token: if gl.token.is_empty() {
                        existing_token
                    } else {
                        gl.token
                    },
                    username: gl.username,
                    projects,
                    poll_interval_secs: clamp_poll_interval(gl.poll_interval),
                });
            }
        }

        if let Some(cal) = request.calendar {
            let source = if cal.source.is_empty() {
                "ics".to_string()
            } else {
                cal.source
            };

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

        if let Some(general) = request.general {
            config.general = GeneralConfig {
                refresh_on_focus: general.refresh_on_focus,
                theme: general.theme,
            };
        }
    }

    state.save_config().await?;
    state.api_cache.invalidate_all();
    to_value(StatusResponse { status: "saved".to_string() })
}

// ─── Dashboard Config ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_dashboard_config(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let config = state.config.read().await;
    to_value(&config.dashboard)
}

#[tauri::command]
pub async fn update_dashboard_config(
    state: State<'_, AppState>,
    dashboard: DashboardConfig,
) -> Result<Value, CommandError> {
    {
        let mut config = state.config.write().await;
        config.dashboard = dashboard;
    }
    state.save_config().await?;
    to_value(StatusResponse { status: "saved".to_string() })
}

// ─── GitHub ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_github_prs(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let cache_key = "github_prs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("github not configured"))?;

    let prs = services::github::fetch_prs(&state.http_client, gh_config)
        .await
        ?;
    let response = to_value(DataResponse { data: &prs })?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_github_pr_detail(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: u64,
) -> Result<Value, CommandError> {
    let gh_config = {
        let config = state.config.read().await;
        config
            .github
            .clone()
            .ok_or_else(|| CommandError::not_configured("github not configured"))?
    };

    let detail =
        services::github::fetch_pr_detail(&state.http_client, &gh_config, &owner, &repo, number)
            .await
            ?;

    to_value(DataResponse { data: &detail })
}

#[tauri::command]
pub async fn detect_gh_cli(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let detected = tokio::task::spawn_blocking(services::github::detect_gh_cli_token)
        .await
        .unwrap_or(None);
    match detected {
        Some((token, username)) => {
            let actual_username = if username.is_empty() {
                services::github::fetch_authenticated_user(&state.http_client, &token)
                    .await
                    ?
            } else {
                username
            };

            {
                let mut config = state.config.write().await;
                let gh_config = config.github.get_or_insert(GitHubConfig {
                    token: String::new(),
                    username: String::new(),
                    repos: vec![],
                    poll_interval_secs: 300,
                    oauth_client_id: None,
                    token_source: "gh_cli".to_string(),
                });
                gh_config.token = token;
                gh_config.username = actual_username.clone();
                gh_config.token_source = "gh_cli".to_string();
            }
            state.save_config().await?;
            state.api_cache.invalidate_all();

            to_value(AuthResult {
                success: true,
                username: actual_username,
                source: "gh_cli".to_string(),
            })
        }
        None => Err(
            CommandError::not_configured("gh CLI not found or not authenticated. Run `gh auth login` first."),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ManualTokenRequest {
    token: String,
}

#[tauri::command]
pub async fn use_manual_github_token(
    state: State<'_, AppState>,
    request: ManualTokenRequest,
) -> Result<Value, CommandError> {
    let token = request.token.trim().to_string();
    if token.is_empty() {
        return Err(CommandError::validation("Token cannot be empty."));
    }

    let username = services::github::fetch_authenticated_user(&state.http_client, &token)
        .await
        ?;

    {
        let mut config = state.config.write().await;
        let gh_config = config.github.get_or_insert(GitHubConfig {
            token: String::new(),
            username: String::new(),
            repos: vec![],
            poll_interval_secs: 300,
            oauth_client_id: None,
            token_source: "manual".to_string(),
        });
        gh_config.token = token;
        gh_config.username = username.clone();
        gh_config.token_source = "manual".to_string();
    }
    state.save_config().await?;
    state.api_cache.invalidate_all();

    to_value(AuthResult {
        success: true,
        username,
        source: "manual".to_string(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhDeviceCodeStartRequest {
    client_id: String,
}

#[tauri::command]
pub async fn start_github_device_code(
    state: State<'_, AppState>,
    request: GhDeviceCodeStartRequest,
) -> Result<Value, CommandError> {
    let device_code =
        services::github::start_device_code_flow(&state.http_client, &request.client_id)
            .await
            ?;

    to_value(GhDeviceCodeStartResponse::from(&device_code))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhDeviceCodePollRequest {
    client_id: String,
    device_code: String,
}

#[tauri::command]
pub async fn poll_github_device_code(
    state: State<'_, AppState>,
    request: GhDeviceCodePollRequest,
) -> Result<Value, CommandError> {
    let result = services::github::poll_device_code_token(
        &state.http_client,
        &request.client_id,
        &request.device_code,
    )
    .await
    ?;

    match result {
        Some(token_response) => {
            let username = services::github::fetch_authenticated_user(
                &state.http_client,
                &token_response.access_token,
            )
            .await
            ?;

            {
                let mut config = state.config.write().await;
                let gh_config = config.github.get_or_insert(GitHubConfig {
                    token: String::new(),
                    username: String::new(),
                    repos: vec![],
                    poll_interval_secs: 300,
                    oauth_client_id: Some(request.client_id.clone()),
                    token_source: "device_code".to_string(),
                });
                gh_config.token = token_response.access_token;
                gh_config.username = username.clone();
                gh_config.token_source = "device_code".to_string();
                gh_config.oauth_client_id = Some(request.client_id);
            }
            state.save_config().await?;
            state.api_cache.invalidate_all();

            to_value(DeviceCodePollResponse {
                status: "complete".to_string(),
                username: Some(username),
                error: None,
            })
        }
        None => to_value(DeviceCodePollResponse {
            status: "pending".to_string(),
            username: None,
            error: None,
        }),
    }
}

// ─── GitLab ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_gitlab_mrs(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let cache_key = "gitlab_mrs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("gitlab not configured"))?;

    let mrs = services::gitlab::fetch_mrs(&state.http_client, gl_config)
        .await
        ?;
    let response = to_value(DataResponse { data: &mrs })?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_gitlab_mr_detail(
    state: State<'_, AppState>,
    project_id: u64,
    iid: u64,
) -> Result<Value, CommandError> {
    let gl_config = {
        let config = state.config.read().await;
        config
            .gitlab
            .clone()
            .ok_or_else(|| CommandError::not_configured("gitlab not configured"))?
    };

    let detail =
        services::gitlab::fetch_mr_detail(&state.http_client, &gl_config, project_id, iid)
            .await
            ?;

    to_value(DataResponse { data: &detail })
}

#[tauri::command]
pub async fn get_gitlab_pipelines(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let cache_key = "gitlab_pipelines".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("gitlab not configured"))?;

    let pipelines = services::gitlab::fetch_pipelines(&state.http_client, gl_config)
        .await
        ?;
    let response = to_value(DataResponse { data: &pipelines })?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct ResolveProjectRequest {
    host: String,
    path: String,
}

#[tauri::command]
pub async fn resolve_gitlab_project(
    state: State<'_, AppState>,
    request: ResolveProjectRequest,
) -> Result<Value, CommandError> {
    let config = state.config.read().await;
    let token = config
        .gitlab
        .as_ref()
        .map(|c| c.token.clone())
        .unwrap_or_default();

    if token.is_empty() {
        return Err(CommandError::not_configured("gitlab not configured (no token)"));
    }

    let host = if request.host.is_empty() {
        config
            .gitlab
            .as_ref()
            .map(|c| c.host.clone())
            .unwrap_or_else(|| "gitlab.com".to_string())
    } else {
        request.host
    };
    drop(config);

    let encoded_path = urlencoding::encode(&request.path);
    let url = format!("https://{host}/api/v4/projects/{encoded_path}");

    let resp = state
        .http_client
        .get(&url)
        .header("PRIVATE-TOKEN", &token)
        .send()
        .await
        .map_err(|e| CommandError::network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(CommandError::external_api(format!(
            "GitLab API returned {} for project '{}'",
            resp.status(),
            request.path
        )));
    }

    let project: Value = resp.json().await.map_err(|e| CommandError::network(e.to_string()))?;
    let id = project["id"]
        .as_u64()
        .ok_or_else(|| CommandError::external_api("No 'id' field in GitLab project response"))?;
    let path_with_namespace = project["path_with_namespace"]
        .as_str()
        .unwrap_or(&request.path)
        .to_string();

    to_value(ResolvedGitLabProject {
        id,
        path: path_with_namespace,
    })
}

// ─── Jira ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_jira_tickets(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let cache_key = "jira_tickets".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("jira not configured"))?;

    let tickets = services::jira::fetch_tickets(&state.http_client, jira_config)
        .await
        ?;
    let response = to_value(DataResponse { data: &tickets })?;
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_jira_ticket_detail(
    state: State<'_, AppState>,
    key: String,
) -> Result<Value, CommandError> {
    let config = state.config.read().await;
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("jira not configured"))?;

    let detail = services::jira::fetch_ticket_detail(&state.http_client, jira_config, &key)
        .await
        ?;

    to_value(DataResponse { data: &detail })
}

// ─── Calendar ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_calendar_events(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("calendar not configured"))?;

    let events = services::calendar::fetch_todays_events(&state.http_client, calendar_config)
        .await
        ?;

    to_value(DataResponse { data: &events })
}

#[derive(Debug, Deserialize)]
pub struct StartCalendarAuthRequest {
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    flow: Option<String>,
}

#[tauri::command]
pub async fn start_calendar_auth(
    state: State<'_, AppState>,
    request: StartCalendarAuthRequest,
) -> Result<Value, CommandError> {
    let source = request.source.unwrap_or_else(|| "ews".to_string());
    let flow = request.flow.unwrap_or_else(|| "manual".to_string());

    {
        let mut config = state.config.write().await;
        if config.calendar.is_none() {
            config.calendar = Some(CalendarConfig {
                source: source.clone(),
                ics_url: None,
                ms_client_id: None,
                ms_tenant_id: None,
                ms_refresh_token: None,
                ews_url: None,
                ms_redirect_uri: None,
                poll_interval_secs: 300,
            });
        } else if let Some(cal) = config.calendar.as_mut() {
            cal.source = source.clone();
        }
    }

    let config = state.config.read().await;
    let calendar_config = config.calendar.as_ref().unwrap();

    // In Tauri mode, default to OOB redirect since there's no local HTTP callback server
    let redirect_uri = calendar_config
        .ms_redirect_uri
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("urn:ietf:wg:oauth:2.0:oob")
        .to_string();

    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);

    let (auth_url, oauth_state) = if use_v1 {
        (services::calendar::build_auth_url_v1(calendar_config, &redirect_uri), None)
    } else {
        let code_verifier = services::calendar::generate_pkce_verifier();
        let oauth_state = uuid::Uuid::new_v4().to_string();
        state.pkce_verifiers.insert(oauth_state.clone(), code_verifier.clone()).await;
        let url = services::calendar::build_auth_url_with_redirect(
            calendar_config,
            &redirect_uri,
            &code_verifier,
        );
        (url, Some(oauth_state))
    };

    to_value(CalendarAuthStartResponse {
        auth_url,
        source,
        flow,
        state: oauth_state,
    })
}

#[tauri::command]
pub async fn get_calendar_auth_status(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let config = state.config.read().await;
    let connected = config
        .calendar
        .as_ref()
        .and_then(|c| c.ms_refresh_token.as_ref())
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    to_value(CalendarAuthStatusResponse { connected })
}

#[derive(Debug, Deserialize)]
pub struct CalendarDeviceCodeStartRequest {
    #[serde(default)]
    source: Option<String>,
}

#[tauri::command]
pub async fn start_calendar_device_code(
    state: State<'_, AppState>,
    request: CalendarDeviceCodeStartRequest,
) -> Result<Value, CommandError> {
    let source = request.source.unwrap_or_else(|| "ews".to_string());

    {
        let mut config = state.config.write().await;
        if config.calendar.is_none() {
            config.calendar = Some(CalendarConfig {
                source: source.clone(),
                ics_url: None,
                ms_client_id: None,
                ms_tenant_id: None,
                ms_refresh_token: None,
                ews_url: None,
                ms_redirect_uri: None,
                poll_interval_secs: 300,
            });
        } else if let Some(cal) = config.calendar.as_mut() {
            cal.source = source;
        }
    }

    let config = state.config.read().await;
    let calendar_config = config.calendar.as_ref().unwrap();

    let resp = services::calendar::start_device_code_flow(&state.http_client, calendar_config)
        .await
        ?;

    {
        state.device_codes.insert("calendar".to_string(), resp.device_code.clone()).await;
    }

    to_value(CalendarDeviceCodeStartResponse::from(&resp))
}

#[tauri::command]
pub async fn poll_calendar_device_code(state: State<'_, AppState>) -> Result<Value, CommandError> {
    let device_code = state.device_codes.get(&"calendar".to_string()).await
        .ok_or_else(|| CommandError::validation("No device code flow in progress. Please start again."))?;

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| CommandError::not_configured("calendar not configured"))?;

    let result =
        services::calendar::poll_device_code_flow(&state.http_client, calendar_config, &device_code)
            .await
            ?;

    if result.status == "completed" {
        drop(config);

        state.device_codes.invalidate(&"calendar".to_string()).await;

        if let Some(token) = &result.token {
            let mut config = state.config.write().await;
            if let Some(cal) = config.calendar.as_mut() {
                cal.ms_refresh_token = token.refresh_token.clone();
            }
        }

        state.save_config().await?;
        state.api_cache.invalidate_all();
    }

    to_value(DeviceCodePollResponse {
        status: result.status,
        username: None,
        error: result.error,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeCodeRequest {
    code: String,
    redirect_uri: Option<String>,
    /// The OAuth state parameter used to look up the PKCE verifier
    state: Option<String>,
}

/// Exchange a manually-entered authorization code for tokens (used by Tauri auth window)
#[tauri::command]
pub async fn exchange_calendar_code(
    state: State<'_, AppState>,
    request: ExchangeCodeRequest,
) -> Result<Value, CommandError> {
    let code = extract_auth_code(&request.code);

    let redirect_uri = {
        let config = state.config.read().await;
        let calendar_config = config
            .calendar
            .as_ref()
            .ok_or_else(|| CommandError::not_configured("calendar not configured"))?;

        request
            .redirect_uri
            .clone()
            .or_else(|| calendar_config.ms_redirect_uri.clone())
            .unwrap_or_else(|| {
                "https://login.microsoftonline.com/common/oauth2/nativeclient".to_string()
            })
    };

    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);

    let token_resp = if use_v1 {
        let calendar_config = {
            let config = state.config.read().await;
            config
                .calendar
                .clone()
                .ok_or_else(|| CommandError::not_configured("calendar not configured"))?
        };
        services::calendar::exchange_auth_code_v1(
            &state.http_client,
            &calendar_config,
            &code,
            &redirect_uri,
        )
        .await
        ?
    } else {
        exchange_code_v2(&state, &code, &redirect_uri).await?
    };

    save_calendar_token(&state, token_resp, request.state.as_deref()).await?;
    to_value(StatusResponse { status: "connected".to_string() })
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Exchange an authorization code for tokens using the v2 OAuth flow.
/// Shared between `open_auth_window` (main.rs) and `exchange_calendar_code`.
pub async fn exchange_code_v2(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
) -> Result<services::calendar::TokenResponse, CommandError> {
    let calendar_config = {
        let config = state.config.read().await;
        config
            .calendar
            .clone()
            .ok_or_else(|| CommandError::not_configured("calendar not configured"))?
    };

    let code_verifier = state.pkce_verifiers.get(&"active".to_string()).await;

    services::calendar::exchange_code_v2(
        &state.http_client,
        &calendar_config,
        code,
        redirect_uri,
        code_verifier.as_deref(),
    )
    .await
    .map_err(Into::into)
}

/// Save a token response (refresh token) to config and clear PKCE state.
pub async fn save_calendar_token(
    state: &AppState,
    token_resp: services::calendar::TokenResponse,
    verifier_key: Option<&str>,
) -> Result<(), CommandError> {
    let key = verifier_key.unwrap_or("active");
    state.pkce_verifiers.invalidate(&key.to_string()).await;
    services::calendar::save_refresh_token(state, token_resp.refresh_token).await?;
    Ok(())
}

fn extract_auth_code(input: &str) -> String {
    services::calendar::extract_auth_code(input)
}
