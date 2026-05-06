// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;

#[cfg(target_os = "macos")]
mod auth_session;

/// Tauri command: open an OAuth auth URL using ASWebAuthenticationSession on macOS.
/// This uses the system auth session which benefits from the Enterprise SSO extension
/// (device compliance via Company Portal), and intercepts the OOB redirect.
#[tauri::command]
async fn open_auth_window(app: tauri::AppHandle, url: String) -> Result<(), String> {
    eprintln!("[myday] open_auth_window called with url: {}...", &url[..url.len().min(80)]);

    #[cfg(target_os = "macos")]
    {
        let app_handle = app.clone();
        // Run ASWebAuthenticationSession on the main thread (required by AppKit)
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();

        // We need to dispatch to main thread for ASWebAuthenticationSession
        let url_clone = url.clone();
        std::thread::spawn(move || {
            auth_session::run_auth_session(&url_clone, tx);
        });

        // Wait for the result
        match rx.await {
            Ok(Ok(callback_url)) => {
                // Extract code from the callback URL
                let code = extract_code_from_url(&callback_url);
                if let Some(code) = code {
                    // Exchange the code via the server
                    let client = reqwest::Client::new();
                    let port =
                        std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
                    let resp = client
                        .post(format!(
                            "http://localhost:{}/api/calendar/auth/exchange-code",
                            port
                        ))
                        .json(&serde_json::json!({
                            "code": code,
                            "redirect_uri": "urn:ietf:wg:oauth:2.0:oob"
                        }))
                        .send()
                        .await;

                    match resp {
                        Ok(r) if r.status().is_success() => {
                            let _ = app_handle.emit("calendar-auth-complete", "connected");
                        }
                        Ok(r) => {
                            let body = r.text().await.unwrap_or_default();
                            eprintln!("Auth code exchange failed: {}", body);
                            let _ = app_handle.emit("calendar-auth-error", body);
                        }
                        Err(e) => {
                            eprintln!("Auth code exchange request failed: {}", e);
                            let _ = app_handle.emit("calendar-auth-error", e.to_string());
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
        // Fallback for non-macOS: open in system browser (user will paste code)
        let _ = app;
        let _ = url;
    }

    Ok(())
}

fn extract_code_from_url(url_str: &str) -> Option<String> {
    // Handle urn:ietf:wg:oauth:2.0:oob?code=...&session_state=...
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
    // Spawn the API server on a dedicated thread with its own tokio runtime
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = myday_server::run_server().await {
                eprintln!("Server error: {}", e);
            }
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![open_auth_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
