// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::JiraConfig;
use crate::error::{AppError, AppResult};

use super::sanitize_host;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTicket {
    pub id: String,
    pub key: String,
    pub title: String,
    pub url: String,
    pub status: String,
    pub status_category: String, // "todo", "in_progress", "done"
    pub priority: String,
    pub assignee: Option<String>,
    pub issue_type: String,
    pub updated_at: String,
    pub created_at: String,
    pub labels: Vec<String>,
    pub sprint_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraTicketDetail {
    #[serde(flatten)]
    pub ticket: JiraTicket,
    pub description: Option<String>,
    pub comments: Vec<JiraComment>,
    pub subtasks: Vec<JiraSubtask>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSubtask {
    pub key: String,
    pub title: String,
    pub status: String,
}

/// Fetch tickets assigned to the current user
pub async fn fetch_tickets(client: &Client, config: &JiraConfig) -> AppResult<Vec<JiraTicket>> {
    let host = sanitize_host(&config.host);
    let base_url = format!("https://{host}/rest/api/3");

    // JQL: assigned to current user, not done
    let mut jql = "assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC".to_string();

    if !config.project_keys.is_empty() {
        let projects = config
            .project_keys
            .iter()
            .map(|k| {
                // Only allow alphanumeric and underscore characters in project keys
                let sanitized: String = k.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
                format!("\"{sanitized}\"")
            })
            .collect::<Vec<_>>()
            .join(", ");
        jql = format!(
            "assignee = currentUser() AND project IN ({projects}) AND statusCategory != Done ORDER BY updated DESC"
        );
    }

    let fields = vec![
        "summary", "status", "priority", "assignee",
        "issuetype", "updated", "created", "labels", "sprint",
    ];

    // Use the newer POST-based search endpoint (GET /search is deprecated / 410 Gone)
    let resp = client
        .post(format!("{base_url}/search/jql"))
        .basic_auth(&config.email, Some(&config.api_token))
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "jql": jql,
            "maxResults": 50,
            "fields": fields,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("JIRA API: {e}")))?;

    let body: Value = resp.json().await?;
    let empty = vec![];
    let issues = body["issues"].as_array().unwrap_or(&empty);

    let tickets = issues
        .iter()
        .filter_map(|issue| parse_jira_issue(issue, &host))
        .collect();

    Ok(tickets)
}

/// Fetch detailed ticket information
pub async fn fetch_ticket_detail(
    client: &Client,
    config: &JiraConfig,
    key: &str,
) -> AppResult<JiraTicketDetail> {
    let host = sanitize_host(&config.host);
    let base_url = format!("https://{host}/rest/api/3");

    // Fetch issue with all fields
    let resp = client
        .get(format!("{base_url}/issue/{key}"))
        .query(&[("expand", "renderedFields")])
        .basic_auth(&config.email, Some(&config.api_token))
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("JIRA API: {e}")))?;

    let issue: Value = resp.json().await?;

    let ticket = parse_jira_issue(&issue, &host)
        .ok_or_else(|| AppError::ExternalApi("Failed to parse JIRA issue".to_string()))?;

    // Parse comments
    let empty_comments = vec![];
    let comments = issue["fields"]["comment"]["comments"]
        .as_array()
        .unwrap_or(&empty_comments)
        .iter()
        .map(|c| JiraComment {
            id: c["id"].as_str().unwrap_or("").to_string(),
            author: c["author"]["displayName"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            body: c["body"]["content"]
                .as_array()
                .and_then(|arr| {
                    arr.first()
                        .and_then(|p| p["content"].as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|t| t["text"].as_str())
                        .map(String::from)
                })
                .unwrap_or_default(),
            created_at: c["created"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    // Parse subtasks
    let empty_subtasks = vec![];
    let subtasks = issue["fields"]["subtasks"]
        .as_array()
        .unwrap_or(&empty_subtasks)
        .iter()
        .map(|s| JiraSubtask {
            key: s["key"].as_str().unwrap_or("").to_string(),
            title: s["fields"]["summary"].as_str().unwrap_or("").to_string(),
            status: s["fields"]["status"]["name"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    // Get rendered description (HTML)
    let description = issue["renderedFields"]["description"]
        .as_str()
        .map(String::from);

    Ok(JiraTicketDetail {
        ticket,
        description,
        comments,
        subtasks,
    })
}

fn parse_jira_issue(issue: &Value, host: &str) -> Option<JiraTicket> {
    let key = issue["key"].as_str()?;
    let fields = &issue["fields"];

    let status_category = match fields["status"]["statusCategory"]["key"].as_str() {
        Some("new") => "todo",
        Some("indeterminate") => "in_progress",
        Some("done") => "done",
        _ => "todo",
    };

    // Try to get sprint name from the sprint field
    let sprint_name = fields["sprint"]["name"].as_str().map(String::from);

    let clean_host = sanitize_host(host);

    Some(JiraTicket {
        id: issue["id"].as_str().unwrap_or("").to_string(),
        key: key.to_string(),
        title: fields["summary"].as_str().unwrap_or("").to_string(),
        url: format!("https://{clean_host}/browse/{key}"),
        status: fields["status"]["name"].as_str().unwrap_or("").to_string(),
        status_category: status_category.to_string(),
        priority: fields["priority"]["name"]
            .as_str()
            .unwrap_or("Medium")
            .to_string(),
        assignee: fields["assignee"]["displayName"]
            .as_str()
            .map(String::from),
        issue_type: fields["issuetype"]["name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        updated_at: fields["updated"].as_str().unwrap_or("").to_string(),
        created_at: fields["created"].as_str().unwrap_or("").to_string(),
        labels: fields["labels"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        sprint_name,
    })
}
