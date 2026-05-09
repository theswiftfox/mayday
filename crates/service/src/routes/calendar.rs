// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use axum::{
    extract::{Query, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::CalendarConfig;
use crate::error::{AppError, AppResult};
use crate::responses::{
    CalendarAuthStartResponse, CalendarAuthStatusResponse, CalendarDeviceCodeStartResponse,
    DataResponse, DeviceCodePollResponse, StatusResponse,
};
use crate::services;
use crate::state::AppState;

/// Escape HTML special characters to prevent XSS in rendered HTML responses.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route("/auth/start", post(start_auth))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/status", get(auth_status))
        .route("/auth/device-code/start", post(device_code_start))
        .route("/auth/device-code/poll", post(device_code_poll))
        .route("/auth/exchange-code", post(exchange_manual_code))
}

#[derive(Debug, Deserialize)]
struct StartAuthBody {
    /// Optional: "microsoft" or "ews" (defaults to "ews")
    source: Option<String>,
    /// Optional: "redirect" or "manual" (defaults to "redirect")
    flow: Option<String>,
}

async fn list_events(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let calendar_config = {
        let config = state.config.read().await;
        config
            .calendar
            .clone()
            .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?
    };

    let events =
        services::calendar::fetch_todays_events(&state.http_client, &calendar_config).await?;
    Ok(Json(
        serde_json::to_value(DataResponse { data: &events }).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

/// Returns the Microsoft authorization URL for the frontend to open
async fn start_auth(
    State(state): State<AppState>,
    body: Option<Json<StartAuthBody>>,
) -> AppResult<Json<Value>> {
    let (source, flow) = match body {
        Some(Json(b)) => (
            b.source.unwrap_or_else(|| "ews".to_string()),
            b.flow.unwrap_or_else(|| "redirect".to_string()),
        ),
        None => ("ews".to_string(), "redirect".to_string()),
    };

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
                ms_redirect_uri: None,
                poll_interval_secs: 300,
            });
        } else if let Some(cal) = config.calendar.as_mut() {
            cal.source = source.clone();
        }
    }

    let config = state.config.read().await;
    let calendar_config = config.calendar.as_ref().unwrap();

    // For "manual" flow, use nativeclient redirect (or the configured redirect URI)
    // For "redirect" flow, use localhost callback
    let redirect_base = if flow == "manual" {
        // Use configured redirect URI or default to nativeclient
        let override_uri = calendar_config
            .ms_redirect_uri
            .as_deref()
            .filter(|s| !s.is_empty());
        override_uri
            .unwrap_or("https://login.microsoftonline.com/common/oauth2/nativeclient")
            .to_string()
    } else {
        let port = std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{port}")
    };

    // For v1.0 (OOB redirect), no PKCE is used; for v2.0, generate PKCE verifier
    let use_v1 = flow == "manual" && services::calendar::is_v1_flow(&redirect_base);

    let (auth_url, oauth_state) = if use_v1 {
        // v1.0 endpoint: no PKCE, uses resource param; shows code on page
        (services::calendar::build_auth_url_v1(calendar_config, &redirect_base), None)
    } else {
        // v2.0 endpoint: PKCE required
        let code_verifier = services::calendar::generate_pkce_verifier();
        let oauth_state = uuid::Uuid::new_v4().to_string();
        state.pkce_verifiers.insert(oauth_state.clone(), code_verifier.clone()).await;
        let url = if flow == "manual" {
            // For manual flow, the redirect_base IS the full redirect_uri (not base + path)
            services::calendar::build_auth_url_with_redirect(
                calendar_config,
                &redirect_base,
                &code_verifier,
            )
        } else {
            services::calendar::build_auth_url(calendar_config, &redirect_base, &code_verifier)
        };
        (url, Some(oauth_state))
    };

    Ok(Json(
        serde_json::to_value(CalendarAuthStartResponse {
            auth_url,
            source,
            flow,
            state: oauth_state,
        })
        .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
    state: Option<String>,
}

/// Handles the OAuth redirect from Microsoft
async fn auth_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Html<String>, AppError> {
    // Check for errors from Microsoft
    if let Some(error) = &query.error {
        let desc = query.error_description.as_deref().unwrap_or("Unknown error");
        let safe_error = html_escape(error);
        let safe_desc = html_escape(desc);
        let html = format!(
            r#"<!DOCTYPE html>
<html><head><title>Calendar Auth Failed</title>
<style>body {{ font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #1a1a2e; color: #e0e0e0; }}
.card {{ text-align: center; padding: 2rem; border-radius: 12px; background: #16213e; border: 1px solid #e94560; max-width: 400px; }}
h1 {{ color: #e94560; font-size: 1.5rem; }} p {{ color: #a0a0a0; }}</style></head>
<body><div class="card"><h1>Authentication Failed</h1><p>{safe_error}: {safe_desc}</p><p>You can close this tab and try again.</p></div></body></html>"#
        );
        return Ok(Html(html));
    }

    let code = query
        .code
        .as_ref()
        .ok_or_else(|| AppError::ExternalApi("No authorization code received".to_string()))?;

    // Retrieve the stored PKCE verifier using the state parameter
    let oauth_state = query.state.as_deref().unwrap_or("active");
    let code_verifier = state.pkce_verifiers.get(&oauth_state.to_string()).await
        .ok_or_else(|| {
            AppError::ExternalApi(
                "No PKCE verifier found. Please restart the auth flow.".to_string(),
            )
        })?;

    // Exchange the code for tokens
    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?;

    let port = std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
    let redirect_base = format!("http://localhost:{port}");

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
    state.pkce_verifiers.invalidate(&oauth_state.to_string()).await;

    // Save the refresh token
    {
        let mut config = state.config.write().await;
        if let Some(cal) = config.calendar.as_mut() {
            cal.source = "microsoft".to_string();
        }
    }
    services::calendar::save_refresh_token(&state, token_resp.refresh_token)
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

    Ok(Json(
        serde_json::to_value(CalendarAuthStatusResponse { connected })
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

// ─── Device Code Flow ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DeviceCodeStartBody {
    source: Option<String>,
}

/// Initiates a device code flow — returns user_code and verification_uri
async fn device_code_start(
    State(state): State<AppState>,
    body: Option<Json<DeviceCodeStartBody>>,
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
                ms_redirect_uri: None,
                poll_interval_secs: 300,
            });
        } else if let Some(cal) = config.calendar.as_mut() {
            cal.source = source.clone();
        }
    }

    let config = state.config.read().await;
    let calendar_config = config.calendar.as_ref().unwrap();

    let resp =
        services::calendar::start_device_code_flow(&state.http_client, calendar_config).await?;

    // Store the device_code for polling
    state.device_codes.insert("calendar".to_string(), resp.device_code.clone()).await;

    Ok(Json(
        serde_json::to_value(CalendarDeviceCodeStartResponse::from(&resp))
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

/// Polls for device code flow completion
async fn device_code_poll(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let device_code = state.device_codes.get(&"calendar".to_string()).await
        .ok_or_else(|| {
            AppError::ExternalApi(
                "No device code flow in progress. Please start again.".to_string(),
            )
        })?;

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?;

    let result = services::calendar::poll_device_code_flow(
        &state.http_client,
        calendar_config,
        &device_code,
    )
    .await?;

    if result.status == "completed" {
        drop(config);

        // Clear the stored device code
        state.device_codes.invalidate(&"calendar".to_string()).await;

        // Save the refresh token
        if let Some(token) = &result.token {
            services::calendar::save_refresh_token(&state, token.refresh_token.clone())
                .await
                .map_err(crate::error::AppError::Internal)?;
        }
    }

    Ok(Json(
        serde_json::to_value(DeviceCodePollResponse {
            status: result.status,
            username: None,
            error: result.error,
        })
        .map_err(|e| AppError::Internal(e.into()))?,
    ))
}

// ─── Manual Code Exchange ────────────────────────────────────────────────────

/// Extract an authorization code from various input formats:
/// - Raw code string (e.g. "M.C507_SN1.2.U.abc...")
/// - Full HTTP URL with ?code= query param
/// - OOB URN: urn:ietf:wg:oauth:2.0:oob?code=...
fn extract_auth_code(input: &str) -> String {
    services::calendar::extract_auth_code(input)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangeCodeBody {
    /// The authorization code (or full redirect URL containing the code)
    code: String,
    /// The redirect_uri used in the authorization request
    redirect_uri: Option<String>,
    /// The OAuth state parameter used to correlate the PKCE verifier
    state: Option<String>,
}

/// Exchange a manually-entered authorization code for tokens.
/// Used when the redirect URI is urn:ietf:wg:oauth:2.0:oob or nativeclient
/// (code appears in the browser, user pastes it here).
async fn exchange_manual_code(
    State(state): State<AppState>,
    Json(body): Json<ExchangeCodeBody>,
) -> AppResult<Json<Value>> {
    // Extract the code from either a raw code, a full URL, or a urn:ietf:wg:oauth:2.0:oob?code=... redirect
    let code = extract_auth_code(&body.code);

    let config = state.config.read().await;
    let calendar_config = config
        .calendar
        .as_ref()
        .ok_or_else(|| AppError::NotConfigured("calendar".to_string()))?;

    // Determine redirect_uri: use the one from the request, or from config, or default OOB
    let redirect_uri = body
        .redirect_uri
        .or_else(|| calendar_config.ms_redirect_uri.clone())
        .unwrap_or_else(|| "https://login.microsoftonline.com/common/oauth2/nativeclient".to_string());

    // If using OOB redirect, use v1.0 token endpoint (resource-based, no PKCE)
    let use_v1 = services::calendar::is_v1_flow(&redirect_uri);

    // Resolve PKCE verifier key for later invalidation
    let verifier_key = body.state.as_deref().unwrap_or("active");

    let token_resp = if use_v1 {
        services::calendar::exchange_auth_code_v1(
            &state.http_client,
            calendar_config,
            &code,
            &redirect_uri,
        )
        .await?
    } else {
        // Retrieve PKCE verifier if one was stored (keyed by state param)
        let code_verifier = state.pkce_verifiers.get(&verifier_key.to_string()).await;

        services::calendar::exchange_code_v2(
            &state.http_client,
            calendar_config,
            &code,
            &redirect_uri,
            code_verifier.as_deref(),
        )
        .await?
    };

    let refresh_token = token_resp.refresh_token;

    drop(config);

    // Clear the stored PKCE verifier
    state.pkce_verifiers.invalidate(&verifier_key.to_string()).await;

    // Save the refresh token
    services::calendar::save_refresh_token(&state, refresh_token)
        .await
        .map_err(crate::error::AppError::Internal)?;

    Ok(Json(
        serde_json::to_value(StatusResponse { status: "connected".to_string() })
            .map_err(|e| AppError::Internal(e.into()))?,
    ))
}
