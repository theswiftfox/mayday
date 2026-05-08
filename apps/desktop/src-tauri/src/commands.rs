//! Tauri IPC commands — thin wrappers around the myday-server service layer.
//! These replace the HTTP route handlers when running as a Tauri desktop app.

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

use myday_server::config::{
    CalendarConfig, DashboardConfig, GeneralConfig, GitHubConfig, GitLabConfig, GitLabProject,
    JiraConfig,
};
use myday_server::error::AppError;
use myday_server::services;
use myday_server::state::AppState;

// ─── Error handling ──────────────────────────────────────────────────────────

fn map_err(e: AppError) -> String {
    e.to_string()
}

// ─── Dashboard ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<Value, String> {
    let cache_key = "dashboard".to_string();

    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let client = &state.http_client;

    let (gh_result, jira_result, gl_mr_result, gl_pipe_result, cal_result) = tokio::join!(
        async {
            match &config.github {
                Some(gh_config) => Some(services::github::fetch_prs(client, gh_config).await),
                None => None,
            }
        },
        async {
            match &config.jira {
                Some(jira_config) => {
                    Some(services::jira::fetch_tickets(client, jira_config).await)
                }
                None => None,
            }
        },
        async {
            match &config.gitlab {
                Some(gl_config) => Some(services::gitlab::fetch_mrs(client, gl_config).await),
                None => None,
            }
        },
        async {
            match &config.gitlab {
                Some(gl_config) => {
                    Some(services::gitlab::fetch_pipelines(client, gl_config).await)
                }
                None => None,
            }
        },
        async {
            match &config.calendar {
                Some(cal_config) => {
                    Some(services::calendar::fetch_todays_events(client, cal_config).await)
                }
                None => None,
            }
        },
    );

    let mut items = Vec::new();
    let mut errors = Vec::new();

    if let Some(result) = gh_result {
        match result {
            Ok(prs) => {
                for pr in prs {
                    items.push(json!({ "type": "github_pr", "data": pr }));
                }
            }
            Err(e) => errors.push(json!({ "source": "github", "message": e.to_string() })),
        }
    }

    if let Some(result) = jira_result {
        match result {
            Ok(tickets) => {
                for ticket in tickets {
                    items.push(json!({ "type": "jira_ticket", "data": ticket }));
                }
            }
            Err(e) => errors.push(json!({ "source": "jira", "message": e.to_string() })),
        }
    }

    if let Some(result) = gl_mr_result {
        match result {
            Ok(mrs) => {
                for mr in mrs {
                    items.push(json!({ "type": "gitlab_mr", "data": mr }));
                }
            }
            Err(e) => errors.push(json!({ "source": "gitlab", "message": e.to_string() })),
        }
    }

    if let Some(result) = gl_pipe_result {
        match result {
            Ok(pipelines) => {
                for pipeline in pipelines {
                    items.push(json!({ "type": "gitlab_pipeline", "data": pipeline }));
                }
            }
            Err(e) => errors.push(json!({ "source": "gitlab", "message": e.to_string() })),
        }
    }

    if let Some(result) = cal_result {
        match result {
            Ok(events) => {
                for event in events {
                    items.push(json!({ "type": "calendar_event", "data": event }));
                }
            }
            Err(e) => errors.push(json!({ "source": "calendar", "message": e.to_string() })),
        }
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let response = json!({
        "items": items,
        "errors": errors,
        "last_updated": timestamp,
    });

    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

// ─── Config ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.read().await;

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
            "projects": c.projects,
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

    Ok(masked)
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
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
    repos: String,
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
    project_keys: String,
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
    projects: Vec<GitLabProjectForm>,
    #[serde(default = "default_poll")]
    poll_interval: u64,
}

#[derive(Debug, Deserialize)]
struct GitLabProjectForm {
    id: u64,
    path: String,
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

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    request: UpdateConfigRequest,
) -> Result<Value, String> {
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
                poll_interval_secs: gh.poll_interval,
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
                    poll_interval_secs: jira.poll_interval,
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
                    poll_interval_secs: gl.poll_interval,
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
                poll_interval_secs: cal.poll_interval,
            });
        }

        if let Some(general) = request.general {
            config.general = GeneralConfig {
                refresh_on_focus: general.refresh_on_focus,
                theme: general.theme,
            };
        }
    }

    state.save_config().await.map_err(|e| e.to_string())?;
    Ok(json!({ "status": "saved" }))
}

// ─── Dashboard Config ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_dashboard_config(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.read().await;
    Ok(serde_json::to_value(&config.dashboard).unwrap_or(json!({})))
}

#[tauri::command]
pub async fn update_dashboard_config(
    state: State<'_, AppState>,
    dashboard: DashboardConfig,
) -> Result<Value, String> {
    {
        let mut config = state.config.write().await;
        config.dashboard = dashboard;
    }
    state.save_config().await.map_err(|e| e.to_string())?;
    Ok(json!({ "status": "saved" }))
}

// ─── GitHub ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_github_prs(state: State<'_, AppState>) -> Result<Value, String> {
    let cache_key = "github_prs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| "github not configured".to_string())?;

    let prs = services::github::fetch_prs(&state.http_client, gh_config)
        .await
        .map_err(map_err)?;
    let response = json!({ "data": prs });
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_github_pr_detail(
    state: State<'_, AppState>,
    owner: String,
    repo: String,
    number: u64,
) -> Result<Value, String> {
    let config = state.config.read().await;
    let gh_config = config
        .github
        .as_ref()
        .ok_or_else(|| "github not configured".to_string())?;

    let detail =
        services::github::fetch_pr_detail(&state.http_client, gh_config, &owner, &repo, number)
            .await
            .map_err(map_err)?;

    Ok(json!({ "data": detail }))
}

#[tauri::command]
pub async fn detect_gh_cli(state: State<'_, AppState>) -> Result<Value, String> {
    match services::github::detect_gh_cli_token() {
        Some((token, username)) => {
            let actual_username = if username.is_empty() {
                services::github::fetch_authenticated_user(&state.http_client, &token)
                    .await
                    .map_err(map_err)?
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
            state.save_config().await.map_err(|e| e.to_string())?;

            Ok(json!({
                "success": true,
                "username": actual_username,
                "source": "gh_cli",
            }))
        }
        None => Ok(json!({
            "success": false,
            "message": "gh CLI not found or not authenticated. Run `gh auth login` first.",
        })),
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
) -> Result<Value, String> {
    let token = request.token.trim().to_string();
    if token.is_empty() {
        return Ok(json!({ "success": false, "message": "Token cannot be empty." }));
    }

    let username = services::github::fetch_authenticated_user(&state.http_client, &token)
        .await
        .map_err(map_err)?;

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
    state.save_config().await.map_err(|e| e.to_string())?;

    Ok(json!({ "success": true, "username": username, "source": "manual" }))
}

#[derive(Debug, Deserialize)]
pub struct GhDeviceCodeStartRequest {
    client_id: String,
}

#[tauri::command]
pub async fn start_github_device_code(
    state: State<'_, AppState>,
    request: GhDeviceCodeStartRequest,
) -> Result<Value, String> {
    let device_code =
        services::github::start_device_code_flow(&state.http_client, &request.client_id)
            .await
            .map_err(map_err)?;

    Ok(json!({
        "device_code": device_code.device_code,
        "user_code": device_code.user_code,
        "verification_uri": device_code.verification_uri,
        "expires_in": device_code.expires_in,
        "interval": device_code.interval,
    }))
}

#[derive(Debug, Deserialize)]
pub struct GhDeviceCodePollRequest {
    client_id: String,
    device_code: String,
}

#[tauri::command]
pub async fn poll_github_device_code(
    state: State<'_, AppState>,
    request: GhDeviceCodePollRequest,
) -> Result<Value, String> {
    let result = services::github::poll_device_code_token(
        &state.http_client,
        &request.client_id,
        &request.device_code,
    )
    .await
    .map_err(map_err)?;

    match result {
        Some(token_response) => {
            let username = services::github::fetch_authenticated_user(
                &state.http_client,
                &token_response.access_token,
            )
            .await
            .map_err(map_err)?;

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
            state.save_config().await.map_err(|e| e.to_string())?;

            Ok(json!({ "status": "complete", "username": username }))
        }
        None => Ok(json!({ "status": "pending" })),
    }
}

// ─── GitLab ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_gitlab_mrs(state: State<'_, AppState>) -> Result<Value, String> {
    let cache_key = "gitlab_mrs".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| "gitlab not configured".to_string())?;

    let mrs = services::gitlab::fetch_mrs(&state.http_client, gl_config)
        .await
        .map_err(map_err)?;
    let response = json!({ "data": mrs });
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_gitlab_mr_detail(
    state: State<'_, AppState>,
    project_id: u64,
    iid: u64,
) -> Result<Value, String> {
    let config = state.config.read().await;
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| "gitlab not configured".to_string())?;

    let detail =
        services::gitlab::fetch_mr_detail(&state.http_client, gl_config, project_id, iid)
            .await
            .map_err(map_err)?;

    Ok(json!({ "data": detail }))
}

#[tauri::command]
pub async fn get_gitlab_pipelines(state: State<'_, AppState>) -> Result<Value, String> {
    let cache_key = "gitlab_pipelines".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let gl_config = config
        .gitlab
        .as_ref()
        .ok_or_else(|| "gitlab not configured".to_string())?;

    let pipelines = services::gitlab::fetch_pipelines(&state.http_client, gl_config)
        .await
        .map_err(map_err)?;
    let response = json!({ "data": pipelines });
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
) -> Result<Value, String> {
    let config = state.config.read().await;
    let token = config
        .gitlab
        .as_ref()
        .map(|c| c.token.clone())
        .unwrap_or_default();

    if token.is_empty() {
        return Err("gitlab not configured (no token)".to_string());
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
    let url = format!("https://{}/api/v4/projects/{}", host, encoded_path);

    let resp = state
        .http_client
        .get(&url)
        .header("PRIVATE-TOKEN", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitLab API returned {} for project '{}'",
            resp.status(),
            request.path
        ));
    }

    let project: Value = resp.json().await.map_err(|e| e.to_string())?;
    let id = project["id"]
        .as_u64()
        .ok_or_else(|| "No 'id' field in GitLab project response".to_string())?;
    let path_with_namespace = project["path_with_namespace"]
        .as_str()
        .unwrap_or(&request.path)
        .to_string();

    Ok(json!({ "id": id, "path": path_with_namespace }))
}

// ─── Jira ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_jira_tickets(state: State<'_, AppState>) -> Result<Value, String> {
    let cache_key = "jira_tickets".to_string();
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(cached);
    }

    let config = state.config.read().await.clone();
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| "jira not configured".to_string())?;

    let tickets = services::jira::fetch_tickets(&state.http_client, jira_config)
        .await
        .map_err(map_err)?;
    let response = json!({ "data": tickets });
    state.api_cache.insert(cache_key, response.clone()).await;
    Ok(response)
}

#[tauri::command]
pub async fn get_jira_ticket_detail(
    state: State<'_, AppState>,
    key: String,
) -> Result<Value, String> {
    let config = state.config.read().await;
    let jira_config = config
        .jira
        .as_ref()
        .ok_or_else(|| "jira not configured".to_string())?;

    let detail = services::jira::fetch_ticket_detail(&state.http_client, jira_config, &key)
        .await
        .map_err(map_err)?;

    Ok(json!({ "data": detail }))
}

// ─── Calendar ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_calendar_events(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| "calendar not configured".to_string())?;

    let events = services::calendar::fetch_todays_events(&state.http_client, calendar_config)
        .await
        .map_err(map_err)?;

    Ok(json!({ "data": events }))
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
) -> Result<Value, String> {
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

    let auth_url = if use_v1 {
        services::calendar::build_auth_url_v1(calendar_config, &redirect_uri)
    } else {
        let code_verifier = services::calendar::generate_pkce_verifier();
        {
            let mut verifier = state.pkce_verifier.write().await;
            *verifier = Some(code_verifier.clone());
        }
        services::calendar::build_auth_url_with_redirect(
            calendar_config,
            &redirect_uri,
            &code_verifier,
        )
    };

    Ok(json!({
        "auth_url": auth_url,
        "source": source,
        "flow": flow,
    }))
}

#[tauri::command]
pub async fn get_calendar_auth_status(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.read().await;
    let connected = config
        .calendar
        .as_ref()
        .and_then(|c| c.ms_refresh_token.as_ref())
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    Ok(json!({ "connected": connected }))
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
) -> Result<Value, String> {
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
        .map_err(map_err)?;

    {
        let mut dc = state.device_code.write().await;
        *dc = Some(resp.device_code.clone());
    }

    Ok(json!({
        "user_code": resp.user_code,
        "verification_uri": resp.verification_uri,
        "expires_in": resp.expires_in,
        "interval": resp.interval,
    }))
}

#[tauri::command]
pub async fn poll_calendar_device_code(state: State<'_, AppState>) -> Result<Value, String> {
    let device_code = {
        let dc = state.device_code.read().await;
        dc.clone()
            .ok_or_else(|| "No device code flow in progress. Please start again.".to_string())?
    };

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| "calendar not configured".to_string())?;

    let result =
        services::calendar::poll_device_code_flow(&state.http_client, calendar_config, &device_code)
            .await
            .map_err(map_err)?;

    if result.status == "completed" {
        drop(config);

        {
            let mut dc = state.device_code.write().await;
            *dc = None;
        }

        if let Some(token) = &result.token {
            let mut config = state.config.write().await;
            if let Some(cal) = config.calendar.as_mut() {
                cal.ms_refresh_token = token.refresh_token.clone();
            }
        }

        state.save_config().await.map_err(|e| e.to_string())?;
    }

    Ok(json!({ "status": result.status, "error": result.error }))
}

#[derive(Debug, Deserialize)]
pub struct ExchangeCodeRequest {
    code: String,
    redirect_uri: Option<String>,
}

/// Exchange a manually-entered authorization code for tokens (used by Tauri auth window)
#[tauri::command]
pub async fn exchange_calendar_code(
    state: State<'_, AppState>,
    request: ExchangeCodeRequest,
) -> Result<Value, String> {
    let code = extract_auth_code(&request.code);

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| "calendar not configured".to_string())?;

    let redirect_uri = request
        .redirect_uri
        .or_else(|| calendar_config.ms_redirect_uri.clone())
        .unwrap_or_else(|| {
            "https://login.microsoftonline.com/common/oauth2/nativeclient".to_string()
        });

    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);

    let token_resp = if use_v1 {
        services::calendar::exchange_auth_code_v1(
            &state.http_client,
            calendar_config,
            &code,
            &redirect_uri,
        )
        .await
        .map_err(map_err)?
    } else {
        let client_id = calendar_config
            .ms_client_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(myday_server::config::DEFAULT_MS_CLIENT_ID);

        let tenant = calendar_config.ms_tenant_id.as_deref().unwrap_or("common");
        let scope = services::calendar::scopes_for_source(&calendar_config.source);

        let code_verifier = {
            let verifier = state.pkce_verifier.read().await;
            verifier.clone()
        };

        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant
        );

        let mut params: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("scope", scope),
        ];

        let verifier_str;
        if let Some(ref v) = code_verifier {
            verifier_str = v.clone();
            params.push(("code_verifier", &verifier_str));
        }

        let resp = state
            .http_client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let resp_body: Value = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(error) = resp_body["error"].as_str() {
            let desc = resp_body["error_description"]
                .as_str()
                .unwrap_or("Unknown error");
            return Err(format!("Token exchange failed: {} - {}", error, desc));
        }

        services::calendar::TokenResponse {
            access_token: resp_body["access_token"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            refresh_token: resp_body["refresh_token"].as_str().map(String::from),
            expires_in: resp_body["expires_in"].as_u64().unwrap_or(3600),
        }
    };

    let refresh_token = token_resp.refresh_token;
    drop(config);

    {
        let mut verifier = state.pkce_verifier.write().await;
        *verifier = None;
    }

    {
        let mut config = state.config.write().await;
        if let Some(cal) = config.calendar.as_mut() {
            cal.ms_refresh_token = refresh_token;
        }
    }

    state.save_config().await.map_err(|e| e.to_string())?;
    Ok(json!({ "status": "connected" }))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_auth_code(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.starts_with("http") {
        if let Ok(u) = url::Url::parse(trimmed) {
            if let Some((_, v)) = u.query_pairs().find(|(k, _)| k == "code") {
                return v.to_string();
            }
        }
        return trimmed.to_string();
    }

    if trimmed.starts_with("urn:") {
        if let Some(query_start) = trimmed.find('?') {
            let query = &trimmed[query_start + 1..];
            for pair in query.split('&') {
                if let Some(value) = pair.strip_prefix("code=") {
                    return value.to_string();
                }
            }
        }
        return trimmed.to_string();
    }

    trimmed.to_string()
}
