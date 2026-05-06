use serde::{Deserialize, Serialize};

/// Configuration for all integrations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub github: Option<GitHubConfig>,
    pub jira: Option<JiraConfig>,
    pub gitlab: Option<GitLabConfig>,
    pub calendar: Option<CalendarConfig>,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    #[serde(default)]
    pub token: String,
    pub username: String,
    #[serde(default)]
    pub repos: Vec<String>, // optional filter: "org/repo" format
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    /// OAuth client ID for device code flow (optional - if not set, uses token directly)
    pub oauth_client_id: Option<String>,
    /// How the token was obtained: "manual", "gh_cli", or "device_code"
    #[serde(default = "default_token_source")]
    pub token_source: String,
}

fn default_token_source() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub host: String, // e.g., yourcompany.atlassian.net
    pub email: String,
    pub api_token: String,
    #[serde(default)]
    pub project_keys: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    pub host: String, // e.g., gitlab.com
    pub token: String,
    pub username: String,
    #[serde(default)]
    pub project_ids: Vec<u64>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    /// "ics", "microsoft", or "ews"
    #[serde(default = "default_calendar_source")]
    pub source: String,
    /// ICS feed URL (used when source = "ics")
    pub ics_url: Option<String>,
    /// Microsoft OAuth client ID (defaults to Azure CLI public client)
    pub ms_client_id: Option<String>,
    /// Microsoft tenant ID (defaults to "common")
    pub ms_tenant_id: Option<String>,
    /// OAuth refresh token (persisted after auth flow)
    pub ms_refresh_token: Option<String>,
    /// EWS endpoint URL (defaults to Exchange Online)
    pub ews_url: Option<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_calendar_source() -> String {
    "ics".to_string()
}

/// Azure CLI public client ID — first-party Microsoft app, pre-approved in all tenants
pub const DEFAULT_MS_CLIENT_ID: &str = "04b07795-8ddb-461a-bbcf-e7f2f0e4a92c";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub refresh_on_focus: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_on_focus: true,
            theme: "system".to_string(),
        }
    }
}

fn default_poll_interval() -> u64 {
    300 // 5 minutes
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "system".to_string()
}
