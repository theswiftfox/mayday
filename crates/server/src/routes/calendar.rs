use axum::{
    extract::{Query, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::CalendarConfig;
use crate::error::{AppError, AppResult};
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route("/auth/start", post(start_auth))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/status", get(auth_status))
}

#[derive(Debug, Deserialize)]
struct StartAuthBody {
    /// Optional: "microsoft" or "ews" (defaults to "ews")
    source: Option<String>,
}

async fn list_events(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?;

    let events =
        services::calendar::fetch_todays_events(&state.http_client, calendar_config).await?;
    Ok(Json(json!({ "data": events })))
}

/// Returns the Microsoft authorization URL for the frontend to open
async fn start_auth(
    State(state): State<AppState>,
    body: Option<Json<StartAuthBody>>,
) -> AppResult<Json<Value>> {
    let source = body
        .and_then(|b| b.source.clone())
        .unwrap_or_else(|| "ews".to_string());

    // Ensure a calendar config exists with the chosen source
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
                poll_interval_secs: 300,
            });
        } else if let Some(cal) = config.calendar.as_mut() {
            cal.source = source.clone();
        }
    }

    let config = state.config.read().await;
    let calendar_config = config.calendar.as_ref().unwrap();

    // Build redirect base from the server's port
    let port = std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
    let redirect_base = format!("http://localhost:{}", port);

    // Generate PKCE verifier and store it for the callback
    let code_verifier = services::calendar::generate_pkce_verifier();
    {
        let mut verifier = state.pkce_verifier.write().await;
        *verifier = Some(code_verifier.clone());
    }

    let auth_url =
        services::calendar::build_auth_url(calendar_config, &redirect_base, &code_verifier);

    Ok(Json(json!({
        "auth_url": auth_url,
        "source": source,
    })))
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Handles the OAuth redirect from Microsoft
async fn auth_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Html<String>, AppError> {
    // Check for errors from Microsoft
    if let Some(error) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown error");
        let html = format!(
            r#"<!DOCTYPE html>
<html><head><title>Calendar Auth Failed</title>
<style>body {{ font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #1a1a2e; color: #e0e0e0; }}
.card {{ text-align: center; padding: 2rem; border-radius: 12px; background: #16213e; border: 1px solid #e94560; max-width: 400px; }}
h1 {{ color: #e94560; font-size: 1.5rem; }} p {{ color: #a0a0a0; }}</style></head>
<body><div class="card"><h1>Authentication Failed</h1><p>{}: {}</p><p>You can close this tab and try again.</p></div></body></html>"#,
            error, desc
        );
        return Ok(Html(html));
    }

    let code = query
        .code
        .as_ref()
        .ok_or_else(|| AppError::ExternalApi("No authorization code received".to_string()))?;

    // Retrieve the stored PKCE verifier
    let code_verifier = {
        let verifier = state.pkce_verifier.read().await;
        verifier.clone().ok_or_else(|| {
            AppError::ExternalApi(
                "No PKCE verifier found. Please restart the auth flow.".to_string(),
            )
        })?
    };

    // Exchange the code for tokens
    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?;

    let port = std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
    let redirect_base = format!("http://localhost:{}", port);

    let token_resp = services::calendar::exchange_auth_code(
        &state.http_client,
        calendar_config,
        code,
        &redirect_base,
        &code_verifier,
    )
    .await?;

    drop(config);

    // Clear the stored PKCE verifier
    {
        let mut verifier = state.pkce_verifier.write().await;
        *verifier = None;
    }

    // Save the refresh token
    {
        let mut config = state.config.write().await;
        if let Some(cal) = config.calendar.as_mut() {
            cal.ms_refresh_token = token_resp.refresh_token;
            cal.source = "microsoft".to_string();
        }
    }

    state
        .save_config()
        .await
        .map_err(crate::error::AppError::Internal)?;

    let html = r#"<!DOCTYPE html>
<html><head><title>Calendar Connected</title>
<style>body { font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #1a1a2e; color: #e0e0e0; }
.card { text-align: center; padding: 2rem; border-radius: 12px; background: #16213e; border: 1px solid #0f3460; max-width: 400px; }
h1 { color: #4ecca3; font-size: 1.5rem; } p { color: #a0a0a0; }</style></head>
<body><div class="card"><h1>Calendar Connected</h1><p>Microsoft 365 calendar is now connected to myday.</p><p>You can close this tab.</p></div></body></html>"#;

    Ok(Html(html.to_string()))
}

/// Returns whether the Microsoft auth is complete (for frontend polling)
async fn auth_status(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let connected = config
        .calendar
        .as_ref()
        .and_then(|c| c.ms_refresh_token.as_ref())
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    Ok(Json(json!({
        "connected": connected,
    })))
}
