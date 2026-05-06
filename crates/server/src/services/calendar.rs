use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::config::{CalendarConfig, DEFAULT_MS_CLIENT_ID};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub subject: String,
    pub start_time: String,
    pub end_time: String,
    pub is_all_day: bool,
    pub location: Option<String>,
    pub organizer: Option<String>,
    pub is_online: bool,
    pub online_url: Option<String>,
    pub description: Option<String>,
}

/// Fetch today's calendar events, dispatching based on config source
pub async fn fetch_todays_events(
    client: &Client,
    config: &CalendarConfig,
) -> AppResult<Vec<CalendarEvent>> {
    match config.source.as_str() {
        "none" => Ok(vec![]),
        "microsoft" => fetch_todays_events_microsoft(client, config).await,
        "ews" => fetch_todays_events_ews(client, config).await,
        _ => fetch_todays_events_ics(client, config).await,
    }
}

// ─── ICS Source ───────────────────────────────────────────────────────────────

/// Fetch today's calendar events from an ICS feed URL
async fn fetch_todays_events_ics(
    client: &Client,
    config: &CalendarConfig,
) -> AppResult<Vec<CalendarEvent>> {
    let ics_url = config
        .ics_url
        .as_ref()
        .filter(|u| !u.is_empty())
        .ok_or_else(|| AppError::NotConfigured("Calendar ICS URL not set".to_string()))?;

    let ics_text = client
        .get(ics_url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("Calendar ICS fetch: {}", e)))?
        .text()
        .await?;

    let today = chrono::Local::now().date_naive();
    let events = parse_ics_events(&ics_text, today);

    Ok(events)
}

// ─── Microsoft Graph Source ───────────────────────────────────────────────────

/// Fetch today's events from Microsoft Graph API
async fn fetch_todays_events_microsoft(
    client: &Client,
    config: &CalendarConfig,
) -> AppResult<Vec<CalendarEvent>> {
    let access_token = get_ms_access_token(client, config).await?;

    let now = chrono::Utc::now();
    let start_of_day = now.format("%Y-%m-%dT00:00:00Z").to_string();
    let end_of_day = now.format("%Y-%m-%dT23:59:59Z").to_string();

    let resp = client
        .get("https://graph.microsoft.com/v1.0/me/calendarview")
        .query(&[
            ("startDateTime", start_of_day.as_str()),
            ("endDateTime", end_of_day.as_str()),
            ("$orderby", "start/dateTime"),
            ("$top", "50"),
        ])
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Prefer", "outlook.timezone=\"UTC\"")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("Microsoft Graph: {}", e)))?;

    let body: Value = resp.json().await?;
    let empty = vec![];
    let events = body["value"].as_array().unwrap_or(&empty);

    let results = events
        .iter()
        .map(|event| {
            let online_url = event["onlineMeeting"]["joinUrl"]
                .as_str()
                .map(String::from);
            let is_online = event["isOnlineMeeting"].as_bool().unwrap_or(false);

            CalendarEvent {
                id: event["id"].as_str().unwrap_or("").to_string(),
                subject: event["subject"]
                    .as_str()
                    .unwrap_or("(No subject)")
                    .to_string(),
                start_time: event["start"]["dateTime"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                end_time: event["end"]["dateTime"].as_str().unwrap_or("").to_string(),
                is_all_day: event["isAllDay"].as_bool().unwrap_or(false),
                location: event["location"]["displayName"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from),
                organizer: event["organizer"]["emailAddress"]["name"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from),
                is_online,
                online_url,
                description: event["body"]["content"].as_str().map(String::from),
            }
        })
        .collect();

    Ok(results)
}

/// Refresh the Microsoft access token using the stored refresh token
async fn get_ms_access_token(client: &Client, config: &CalendarConfig) -> AppResult<String> {
    let refresh_token = config
        .ms_refresh_token
        .as_ref()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            AppError::NotConfigured(
                "Microsoft 365 not connected. Please authenticate first.".to_string(),
            )
        })?;

    let client_id = config
        .ms_client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MS_CLIENT_ID);

    let tenant = config.ms_tenant_id.as_deref().unwrap_or("common");
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant
    );

    let scope = scopes_for_source(&config.source);

    let resp = client
        .post(&url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("scope", scope),
        ])
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("Microsoft OAuth refresh: {}", e)))?;

    let body: Value = resp.json().await?;
    let access_token = body["access_token"]
        .as_str()
        .ok_or_else(|| AppError::ExternalApi("No access token in response".to_string()))?;

    Ok(access_token.to_string())
}

// ─── Microsoft Authorization Code Flow ───────────────────────────────────────

const REDIRECT_PATH: &str = "/api/calendar/auth/callback";
const MS_GRAPH_SCOPES: &str = "Calendars.Read offline_access";
const EWS_SCOPES: &str = "https://outlook.office365.com/EWS.AccessAsUser.All offline_access";

/// Get the appropriate OAuth scope for a given source type
pub fn scopes_for_source(source: &str) -> &'static str {
    match source {
        "ews" => EWS_SCOPES,
        _ => MS_GRAPH_SCOPES,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

// ─── PKCE Helpers ────────────────────────────────────────────────────────────

/// Generate a cryptographically random PKCE code_verifier (43-128 chars, base64url)
pub fn generate_pkce_verifier() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Derive the PKCE code_challenge from a code_verifier using S256
fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Build the Microsoft authorization URL for the browser redirect (with PKCE)
pub fn build_auth_url(config: &CalendarConfig, redirect_base: &str, code_verifier: &str) -> String {
    let client_id = config
        .ms_client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MS_CLIENT_ID);

    let tenant = config.ms_tenant_id.as_deref().unwrap_or("common");
    let redirect_uri = format!("{}{}", redirect_base, REDIRECT_PATH);
    let code_challenge = pkce_challenge(code_verifier);
    let scope = scopes_for_source(&config.source);

    format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?\
         client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&prompt=consent\
         &code_challenge={}&code_challenge_method=S256",
        tenant,
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scope),
        urlencoding::encode(&code_challenge),
    )
}

/// Exchange an authorization code for access + refresh tokens (with PKCE)
pub async fn exchange_auth_code(
    client: &Client,
    config: &CalendarConfig,
    code: &str,
    redirect_base: &str,
    code_verifier: &str,
) -> AppResult<TokenResponse> {
    let client_id = config
        .ms_client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MS_CLIENT_ID);

    let tenant = config.ms_tenant_id.as_deref().unwrap_or("common");
    let redirect_uri = format!("{}{}", redirect_base, REDIRECT_PATH);
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant
    );
    let scope = scopes_for_source(&config.source);

    let resp = client
        .post(&url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", &redirect_uri),
            ("scope", scope),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?;

    let body: Value = resp.json().await?;

    if let Some(error) = body["error"].as_str() {
        let desc = body["error_description"]
            .as_str()
            .unwrap_or("Unknown error");
        return Err(AppError::ExternalApi(format!(
            "Microsoft OAuth error: {} - {}",
            error, desc
        )));
    }

    let access_token = body["access_token"]
        .as_str()
        .ok_or_else(|| AppError::ExternalApi("No access token in response".to_string()))?;

    Ok(TokenResponse {
        access_token: access_token.to_string(),
        refresh_token: body["refresh_token"].as_str().map(String::from),
        expires_in: body["expires_in"].as_u64().unwrap_or(3600),
    })
}

// ─── EWS (Exchange Web Services) Source ──────────────────────────────────────

const EWS_ENDPOINT: &str = "https://outlook.office365.com/EWS/Exchange.asmx";

/// Fetch today's events via EWS SOAP API
async fn fetch_todays_events_ews(
    client: &Client,
    config: &CalendarConfig,
) -> AppResult<Vec<CalendarEvent>> {
    let access_token = get_ms_access_token(client, config).await?;
    let ews_url = config.ews_url.as_deref().unwrap_or(EWS_ENDPOINT);

    let now = chrono::Utc::now();
    let start_of_day = now.format("%Y-%m-%dT00:00:00Z").to_string();
    let end_of_day = now.format("%Y-%m-%dT23:59:59Z").to_string();

    // Build the SOAP envelope for FindItem with CalendarView
    let soap_body = build_ews_find_item_request(&start_of_day, &end_of_day);

    let resp = client
        .post(ews_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "text/xml; charset=utf-8")
        .body(soap_body)
        .send()
        .await?;

    let status = resp.status();
    let xml_body = resp.text().await?;

    if !status.is_success() {
        return Err(AppError::ExternalApi(format!(
            "EWS request failed ({}): {}",
            status,
            xml_body.chars().take(500).collect::<String>()
        )));
    }

    parse_ews_find_item_response(&xml_body)
}

/// Build a SOAP FindItem request with CalendarView for today's events
fn build_ews_find_item_request(start: &str, end: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
               xmlns:m="http://schemas.microsoft.com/exchange/services/2006/messages"
               xmlns:t="http://schemas.microsoft.com/exchange/services/2006/types"
               xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
  <soap:Header>
    <t:RequestServerVersion Version="Exchange2013_SP1" />
  </soap:Header>
  <soap:Body>
    <m:FindItem Traversal="Shallow">
      <m:ItemShape>
        <t:BaseShape>Default</t:BaseShape>
        <t:AdditionalProperties>
          <t:FieldURI FieldURI="item:Subject" />
          <t:FieldURI FieldURI="calendar:Start" />
          <t:FieldURI FieldURI="calendar:End" />
          <t:FieldURI FieldURI="calendar:IsAllDayEvent" />
          <t:FieldURI FieldURI="calendar:Location" />
          <t:FieldURI FieldURI="calendar:Organizer" />
          <t:FieldURI FieldURI="item:Body" />
          <t:FieldURI FieldURI="calendar:IsOnlineMeeting" />
          <t:FieldURI FieldURI="calendar:OnlineMeetingSettings" />
        </t:AdditionalProperties>
      </m:ItemShape>
      <m:CalendarView MaxEntriesReturned="50" StartDate="{}" EndDate="{}" />
      <m:ParentFolderIds>
        <t:DistinguishedFolderId Id="calendar" />
      </m:ParentFolderIds>
    </m:FindItem>
  </soap:Body>
</soap:Envelope>"#,
        start, end
    )
}

/// Parse the EWS FindItem SOAP response XML into CalendarEvents
fn parse_ews_find_item_response(xml: &str) -> AppResult<Vec<CalendarEvent>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut events = Vec::new();

    // State machine for parsing CalendarItem elements
    let mut in_calendar_item = false;
    let mut current_field: Option<String> = None;
    let mut item_id = String::new();
    let mut subject = String::new();
    let mut start_time = String::new();
    let mut end_time = String::new();
    let mut is_all_day = false;
    let mut location = String::new();
    let mut organizer_name = String::new();
    let mut body_content = String::new();
    let mut in_organizer = false;
    let mut in_mailbox = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_string();

                match local_name.as_str() {
                    "CalendarItem" => {
                        in_calendar_item = true;
                        item_id.clear();
                        subject.clear();
                        start_time.clear();
                        end_time.clear();
                        is_all_day = false;
                        location.clear();
                        organizer_name.clear();
                        body_content.clear();
                        in_organizer = false;
                        in_mailbox = false;
                    }
                    "ItemId" if in_calendar_item => {
                        // ItemId is an empty element with attributes
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Id" {
                                item_id = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    "Subject" if in_calendar_item => current_field = Some("Subject".to_string()),
                    "Start" if in_calendar_item => current_field = Some("Start".to_string()),
                    "End" if in_calendar_item => current_field = Some("End".to_string()),
                    "IsAllDayEvent" if in_calendar_item => {
                        current_field = Some("IsAllDayEvent".to_string())
                    }
                    "Location" if in_calendar_item && !in_organizer => {
                        current_field = Some("Location".to_string())
                    }
                    "Organizer" if in_calendar_item => {
                        in_organizer = true;
                    }
                    "Mailbox" if in_organizer => {
                        in_mailbox = true;
                    }
                    "Name" if in_mailbox => {
                        current_field = Some("OrganizerName".to_string())
                    }
                    "Body" if in_calendar_item => current_field = Some("Body".to_string()),
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_calendar_item {
                    if let Some(ref field) = current_field {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match field.as_str() {
                            "Subject" => subject = text,
                            "Start" => start_time = text,
                            "End" => end_time = text,
                            "IsAllDayEvent" => is_all_day = text == "true",
                            "Location" => location = text,
                            "OrganizerName" => organizer_name = text,
                            "Body" => body_content = text,
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let local_name = std::str::from_utf8(local.as_ref()).unwrap_or("");
                match local_name {
                    "CalendarItem" => {
                        if in_calendar_item {
                            let online_url = detect_meeting_url(
                                Some(&location),
                                Some(&body_content),
                                None,
                                None,
                            );
                            let is_online = online_url.is_some();

                            events.push(CalendarEvent {
                                id: if item_id.is_empty() {
                                    subject.clone()
                                } else {
                                    item_id.clone()
                                },
                                subject: if subject.is_empty() {
                                    "(No subject)".to_string()
                                } else {
                                    subject.clone()
                                },
                                start_time: start_time.clone(),
                                end_time: end_time.clone(),
                                is_all_day,
                                location: if location.is_empty() {
                                    None
                                } else {
                                    Some(location.clone())
                                },
                                organizer: if organizer_name.is_empty() {
                                    None
                                } else {
                                    Some(organizer_name.clone())
                                },
                                is_online,
                                online_url,
                                description: if body_content.is_empty() {
                                    None
                                } else {
                                    Some(body_content.clone())
                                },
                            });
                        }
                        in_calendar_item = false;
                    }
                    "Organizer" => in_organizer = false,
                    "Mailbox" => in_mailbox = false,
                    _ => {
                        current_field = None;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::ExternalApi(format!(
                    "EWS XML parse error: {}",
                    e
                )));
            }
            _ => {}
        }
    }

    // Check for SOAP fault in response
    if events.is_empty() && xml.contains("ResponseClass=\"Error\"") {
        // Try to extract the error message
        if let Some(start) = xml.find("<m:MessageText>") {
            let after = &xml[start + 15..];
            if let Some(end) = after.find("</m:MessageText>") {
                let msg = &after[..end];
                return Err(AppError::ExternalApi(format!("EWS error: {}", msg)));
            }
        }
        if let Some(start) = xml.find("<faultstring>") {
            let after = &xml[start + 13..];
            if let Some(end) = after.find("</faultstring>") {
                let msg = &after[..end];
                return Err(AppError::ExternalApi(format!("EWS SOAP fault: {}", msg)));
            }
        }
    }

    Ok(events)
}

// ─── ICS Parsing ─────────────────────────────────────────────────────────────

/// Parse VEVENT entries from iCalendar text, filtering to a specific date
fn parse_ics_events(ics: &str, date: chrono::NaiveDate) -> Vec<CalendarEvent> {
    let mut events = Vec::new();
    let mut in_event = false;
    let mut props: Vec<(String, String)> = Vec::new();

    for line in unfold_ics_lines(ics) {
        let trimmed = line.trim();
        if trimmed == "BEGIN:VEVENT" {
            in_event = true;
            props.clear();
        } else if trimmed == "END:VEVENT" {
            if in_event {
                if let Some(event) = build_event_from_props(&props, date) {
                    events.push(event);
                }
            }
            in_event = false;
        } else if in_event {
            if let Some(colon_pos) = find_property_colon(trimmed) {
                let key = &trimmed[..colon_pos];
                let value = &trimmed[colon_pos + 1..];
                let base_key = key.split(';').next().unwrap_or(key);
                props.push((base_key.to_uppercase(), value.to_string()));
            }
        }
    }

    events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    events
}

/// Unfold continuation lines in ICS (lines starting with space/tab are continuations)
fn unfold_ics_lines(ics: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in ics.lines() {
        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            if let Some(last) = lines.last_mut() {
                let continued: &mut String = last;
                continued.push_str(raw_line.trim_start());
            }
        } else {
            lines.push(raw_line.to_string());
        }
    }
    lines
}

/// Find the colon that separates property name from value
fn find_property_colon(line: &str) -> Option<usize> {
    let mut in_quotes = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ':' if !in_quotes => return Some(i),
            _ => {}
        }
    }
    None
}

/// Build a CalendarEvent from parsed properties, only if it falls on the given date
fn build_event_from_props(
    props: &[(String, String)],
    date: chrono::NaiveDate,
) -> Option<CalendarEvent> {
    let get = |key: &str| props.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());

    let dtstart = get("DTSTART")?;
    let dtend = get("DTEND").unwrap_or(dtstart);
    let summary = get("SUMMARY").unwrap_or("(No subject)");

    let (start_dt, is_all_day) = parse_ics_datetime(dtstart)?;
    let (end_dt, _) = parse_ics_datetime(dtend)?;

    let start_date = start_dt.date_naive();
    let end_date = end_dt.date_naive();

    if start_date > date || end_date < date {
        return None;
    }

    let location = get("LOCATION").map(|s| unescape_ics(s));
    let description = get("DESCRIPTION").map(|s| unescape_ics(s));
    let online_url = detect_meeting_url(
        location.as_deref(),
        description.as_deref(),
        get("X-MICROSOFT-SKYPETEAMSMEETINGURL"),
        get("URL"),
    );
    let is_online = online_url.is_some();

    let organizer_raw = get("ORGANIZER").unwrap_or("");
    let organizer = parse_organizer(organizer_raw);

    let uid = get("UID").unwrap_or(summary);

    Some(CalendarEvent {
        id: uid.to_string(),
        subject: unescape_ics(summary),
        start_time: start_dt.to_rfc3339(),
        end_time: end_dt.to_rfc3339(),
        is_all_day,
        location: location.filter(|s| !s.is_empty()),
        organizer: if organizer.is_empty() {
            None
        } else {
            Some(organizer)
        },
        is_online,
        online_url,
        description,
    })
}

/// Parse an iCalendar datetime string into a chrono DateTime
fn parse_ics_datetime(s: &str) -> Option<(chrono::DateTime<chrono::Local>, bool)> {
    use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};

    let s = s.trim();

    // All-day: just a date like 20240115
    if s.len() == 8 && !s.contains('T') {
        let nd = NaiveDate::parse_from_str(s, "%Y%m%d").ok()?;
        let dt = nd.and_hms_opt(0, 0, 0)?;
        let local = Local.from_local_datetime(&dt).earliest()?;
        return Some((local, true));
    }

    // UTC: 20240115T100000Z
    if s.ends_with('Z') {
        let naive =
            NaiveDateTime::parse_from_str(s.trim_end_matches('Z'), "%Y%m%dT%H%M%S").ok()?;
        let utc = chrono::Utc.from_utc_datetime(&naive);
        let local: chrono::DateTime<Local> = utc.into();
        return Some((local, false));
    }

    // Local time: 20240115T100000
    let naive = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()?;
    let local = Local.from_local_datetime(&naive).earliest()?;
    Some((local, false))
}

/// Unescape ICS text
fn unescape_ics(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

/// Detect online meeting URL from various sources
fn detect_meeting_url(
    location: Option<&str>,
    description: Option<&str>,
    ms_teams_url: Option<&str>,
    url_prop: Option<&str>,
) -> Option<String> {
    if let Some(url) = ms_teams_url {
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }

    if let Some(url) = url_prop {
        if url.contains("teams.microsoft.com")
            || url.contains("zoom.us")
            || url.contains("meet.google.com")
        {
            return Some(url.to_string());
        }
    }

    if let Some(loc) = location {
        if let Some(url) = extract_meeting_url(loc) {
            return Some(url);
        }
    }

    if let Some(desc) = description {
        if let Some(url) = extract_meeting_url(desc) {
            return Some(url);
        }
    }

    None
}

/// Extract a meeting URL from text
fn extract_meeting_url(text: &str) -> Option<String> {
    let patterns = [
        "https://teams.microsoft.com/l/meetup-join/",
        "https://zoom.us/j/",
        "https://meet.google.com/",
        "https://teams.live.com/meet/",
    ];

    for pattern in &patterns {
        if let Some(start) = text.find(pattern) {
            let url_text = &text[start..];
            let end = url_text
                .find(|c: char| c.is_whitespace() || c == '"' || c == '<' || c == '>')
                .unwrap_or(url_text.len());
            return Some(url_text[..end].to_string());
        }
    }
    None
}

/// Parse organizer from ICS ORGANIZER property
fn parse_organizer(raw: &str) -> String {
    if let Some(cn_start) = raw.find("CN=") {
        let after_cn = &raw[cn_start + 3..];
        let end = after_cn
            .find(|c: char| c == ':' || c == ';')
            .unwrap_or(after_cn.len());
        let name = &after_cn[..end];
        if !name.is_empty() {
            return name.to_string();
        }
    }

    if let Some(mailto_start) = raw.find("mailto:") {
        return raw[mailto_start + 7..].to_string();
    }

    raw.to_string()
}
