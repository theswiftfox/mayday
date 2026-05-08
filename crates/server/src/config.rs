use serde::{Deserialize, Serialize};

/// Configuration for all integrations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub github: Option<GitHubConfig>,
    pub jira: Option<JiraConfig>,
    pub gitlab: Option<GitLabConfig>,
    pub calendar: Option<CalendarConfig>,
    pub general: GeneralConfig,
    /// Dashboard layout, filters, and importance rules
    #[serde(default)]
    pub dashboard: DashboardConfig,
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
    /// Projects to monitor — stores both numeric ID (for API calls) and path (for display)
    #[serde(default)]
    pub projects: Vec<GitLabProject>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

impl GitLabConfig {
    /// Get numeric project IDs for API calls
    pub fn numeric_project_ids(&self) -> Vec<u64> {
        self.projects.iter().map(|p| p.id).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub path: String,
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
    /// Custom OAuth redirect URI (optional — overrides the default localhost callback)
    pub ms_redirect_uri: Option<String>,
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

/// Dashboard layout and filter preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Section display order (e.g. ["important", "github_pr", "gitlab_mr", ...])
    #[serde(default = "default_section_order")]
    pub section_order: Vec<String>,
    /// Which sections are visible. Empty = all visible (default).
    #[serde(default = "default_visible_sections")]
    pub visible_sections: Vec<String>,
    /// Calendar layout: "sidebar" (right column) or "inline" (regular section)
    #[serde(default = "default_calendar_layout")]
    pub calendar_layout: String,
    /// Rules for auto-populating the Important section
    #[serde(default)]
    pub important_rules: ImportantRules,
    /// Manually pinned items
    #[serde(default)]
    pub pinned_items: Vec<PinnedItem>,
    /// Per-integration filters
    #[serde(default)]
    pub filters: DashboardFilters,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            section_order: default_section_order(),
            visible_sections: default_visible_sections(),
            calendar_layout: default_calendar_layout(),
            important_rules: ImportantRules::default(),
            pinned_items: Vec::new(),
            filters: DashboardFilters::default(),
        }
    }
}

fn default_section_order() -> Vec<String> {
    vec![
        "important".to_string(),
        "github_pr".to_string(),
        "gitlab".to_string(),
        "jira_ticket".to_string(),
        "calendar_event".to_string(),
    ]
}

fn default_visible_sections() -> Vec<String> {
    default_section_order()
}

fn default_calendar_layout() -> String {
    "sidebar".to_string()
}

/// Rules that determine which items automatically appear in the Important section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportantRules {
    #[serde(default)]
    pub github_action_required: bool,
    #[serde(default)]
    pub github_new_comments: bool,
    #[serde(default)]
    pub github_new_commits: bool,
    #[serde(default)]
    pub github_changes_requested: bool,
    #[serde(default)]
    pub gitlab_mr_new_comments: bool,
    #[serde(default)]
    pub gitlab_mr_new_commits: bool,
    #[serde(default)]
    pub gitlab_pipeline_failed: bool,
    #[serde(default)]
    pub jira_high_priority: bool,
    #[serde(default)]
    pub calendar_starting_soon: bool,
}

/// A manually pinned item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedItem {
    /// Item type: "github_pr", "gitlab_mr", "gitlab_pipeline", "jira_ticket", "calendar_event"
    pub item_type: String,
    /// Unique identifier: "org/repo#123" for PRs, "project_id!iid" for MRs, "PROJ-123" for tickets, etc.
    pub item_id: String,
}

/// Per-integration filter settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardFilters {
    #[serde(default)]
    pub github_pr: GitHubPRFilter,
    #[serde(default)]
    pub gitlab_mr: GitLabMRFilter,
    #[serde(default)]
    pub gitlab_pipeline: GitLabPipelineFilter,
    #[serde(default)]
    pub jira_ticket: JiraTicketFilter,
    #[serde(default)]
    pub calendar_event: CalendarEventFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubPRFilter {
    /// Filter by role: ["author", "reviewer", "other"]. Empty = show all.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Hide draft PRs
    #[serde(default)]
    pub hide_drafts: bool,
    /// Only show PRs where action is required
    #[serde(default)]
    pub action_required_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitLabMRFilter {
    /// Filter by role: ["author", "reviewer"]. Empty = show all.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Hide draft MRs
    #[serde(default)]
    pub hide_drafts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitLabPipelineFilter {
    /// Filter by status: ["failed", "running", "pending", ...]. Empty = show all non-success.
    #[serde(default)]
    pub statuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraTicketFilter {
    /// Filter by status category: ["todo", "in_progress", "done"]. Empty = show all.
    #[serde(default)]
    pub status_categories: Vec<String>,
    /// Filter by priority: ["highest", "high", "medium", "low", "lowest"]. Empty = show all.
    #[serde(default)]
    pub priorities: Vec<String>,
    /// Filter by issue type: ["story", "bug", "task", ...]. Empty = show all.
    #[serde(default)]
    pub issue_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarEventFilter {
    /// Hide all-day events
    #[serde(default)]
    pub hide_all_day: bool,
    /// Only show online meetings
    #[serde(default)]
    pub online_only: bool,
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
