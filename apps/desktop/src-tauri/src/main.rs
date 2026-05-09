// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;

mod commands;
mod error;

#[cfg(target_os = "macos")]
mod auth_session;

/// Tauri command: open an OAuth auth URL using ASWebAuthenticationSession on macOS.
/// This uses the system auth session which benefits from the Enterprise SSO extension
/// (device compliance via Company Portal), and intercepts the OOB redirect.
#[tauri::command]
async fn open_auth_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, myday_core::state::AppState>,
    url: String,
) -> Result<(), error::CommandError> {
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
                    let exchange_result = exchange_code_internal(&state_inner, &code).await;
                    match exchange_result {
                        Ok(_) => {
                            let _ = app_handle.emit("calendar-auth-complete", "connected");
                        }
                        Err(e) => {
                            let msg = e.message.clone();
                            let _ = app_handle.emit("calendar-auth-error", &msg);
                            return Err(e);
                        }
                    }
                } else {
                    let msg = "No authorization code found in callback URL";
                    let _ = app_handle.emit("calendar-auth-error", msg);
                    return Err(error::CommandError::external_api(msg));
                }
            }
            Ok(Err(e)) => {
                eprintln!("Auth session error: {e}");
                let _ = app_handle.emit("calendar-auth-error", &e);
                return Err(error::CommandError::auth_failed(e));
            }
            Err(_) => {
                let msg = "Auth session channel closed unexpectedly";
                let _ = app_handle.emit("calendar-auth-error", msg);
                return Err(error::CommandError::internal(msg));
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        let _ = state;
        let _ = url;
        return Err(error::CommandError::internal("OAuth auth sessions are only supported on macOS"));
    }

    Ok(())
}

/// Exchange an auth code directly via the service layer (no HTTP round-trip)
async fn exchange_code_internal(
    state: &myday_core::state::AppState,
    code: &str,
) -> Result<(), error::CommandError> {
    use myday_core::services;

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| error::CommandError::not_configured("calendar not configured"))?;

    let redirect_uri = calendar_config
        .ms_redirect_uri
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("urn:ietf:wg:oauth:2.0:oob")
        .to_string();

    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);
    drop(config);

    let token_resp = if use_v1 {
        let config = state.config.read().await;
        let calendar_config = config
            .calendar
            .as_ref()
            .ok_or_else(|| error::CommandError::not_configured("calendar not configured"))?;
        services::calendar::exchange_auth_code_v1(
            &state.http_client,
            calendar_config,
            code,
            &redirect_uri,
        )
        .await
        .map_err(error::CommandError::from)?
    } else {
        commands::exchange_code_v2(state, code, &redirect_uri).await?
    };

    commands::save_calendar_token(state, token_resp, None).await?;
    Ok(())
}

fn extract_code_from_url(url_str: &str) -> Option<String> {
    let url = url::Url::parse(url_str).ok()?;
    url.query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
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
        .block_on(myday_core::state::AppState::new())
        .expect("failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
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
