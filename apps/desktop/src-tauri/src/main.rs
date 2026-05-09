// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;

mod commands;

#[cfg(target_os = "macos")]
mod auth_session;

/// Tauri command: open an OAuth auth URL using ASWebAuthenticationSession on macOS.
/// This uses the system auth session which benefits from the Enterprise SSO extension
/// (device compliance via Company Portal), and intercepts the OOB redirect.
#[tauri::command]
async fn open_auth_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, myday_server::state::AppState>,
    url: String,
) -> Result<(), String> {
    eprintln!(
        "[myday] open_auth_window called with url: {}...",
        &url[..url.len().min(80)]
    );

    #[cfg(target_os = "macos")]
    {
        let state_inner = state.inner().clone();
        let app_handle = app.clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();

        let url_clone = url.clone();
        std::thread::spawn(move || {
            auth_session::run_auth_session(&url_clone, tx);
        });

        match rx.await {
            Ok(Ok(callback_url)) => {
                let code = extract_code_from_url(&callback_url);
                if let Some(code) = code {
                    // Exchange the code directly via the service layer (no HTTP)
                    let exchange_result = exchange_code_internal(&state_inner, &code).await;
                    match exchange_result {
                        Ok(_) => {
                            let _ = app_handle.emit("calendar-auth-complete", "connected");
                        }
                        Err(e) => {
                            eprintln!("Auth code exchange failed: {}", e);
                            let _ = app_handle.emit("calendar-auth-error", e);
                        }
                    }
                } else {
                    let _ = app_handle.emit("calendar-auth-error", "No code in callback URL");
                }
            }
            Ok(Err(e)) => {
                eprintln!("Auth session error: {}", e);
                let _ = app_handle.emit("calendar-auth-error", e);
            }
            Err(_) => {
                let _ = app_handle.emit("calendar-auth-error", "Auth session cancelled");
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        let _ = state;
        let _ = url;
    }

    Ok(())
}

/// Exchange an auth code directly via the service layer (no HTTP round-trip)
async fn exchange_code_internal(
    state: &myday_server::state::AppState,
    code: &str,
) -> Result<(), String> {
    use myday_server::services;

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| "calendar not configured".to_string())?;

    let redirect_uri = calendar_config
        .ms_redirect_uri
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("urn:ietf:wg:oauth:2.0:oob")
        .to_string();

    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);

    let token_resp = if use_v1 {
        services::calendar::exchange_auth_code_v1(
            &state.http_client,
            calendar_config,
            code,
            &redirect_uri,
        )
        .await
        .map_err(|e| e.to_string())?
    } else {
        let code_verifier = {
            let verifier = state.pkce_verifier.read().await;
            verifier.clone()
        };

        let client_id = calendar_config
            .ms_client_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(myday_server::config::DEFAULT_MS_CLIENT_ID);

        let tenant = calendar_config.ms_tenant_id.as_deref().unwrap_or("common");
        let scope = services::calendar::scopes_for_source(&calendar_config.source);

        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant
        );

        let mut params: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
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

        let resp_body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

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
    Ok(())
}

fn extract_code_from_url(url_str: &str) -> Option<String> {
    if let Some(query_start) = url_str.find('?') {
        let query = &url_str[query_start + 1..];
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("code=") {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("myday=info".parse().unwrap()),
        )
        .init();

    // Initialize application state (same AppState used by the service layer)
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let state = rt
        .block_on(myday_server::state::AppState::new())
        .expect("failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            open_auth_window,
            // Dashboard
            commands::get_dashboard,
            // Config
            commands::get_config,
            commands::update_config,
            commands::get_dashboard_config,
            commands::update_dashboard_config,
            // GitHub
            commands::get_github_prs,
            commands::get_github_pr_detail,
            commands::detect_gh_cli,
            commands::use_manual_github_token,
            commands::start_github_device_code,
            commands::poll_github_device_code,
            // GitLab
            commands::get_gitlab_mrs,
            commands::get_gitlab_mr_detail,
            commands::get_gitlab_pipelines,
            commands::resolve_gitlab_project,
            // Jira
            commands::get_jira_tickets,
            commands::get_jira_ticket_detail,
            // Calendar
            commands::get_calendar_events,
            commands::start_calendar_auth,
            commands::get_calendar_auth_status,
            commands::start_calendar_device_code,
            commands::poll_calendar_device_code,
            commands::exchange_calendar_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
