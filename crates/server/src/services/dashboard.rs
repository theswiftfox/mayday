// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//! Dashboard aggregation service — fetches all integrations concurrently
//! and returns a unified dashboard view as a serialized Value.

use serde_json::Value;

use crate::responses::{DashboardError, DashboardItem, DashboardResponse};
use crate::services;
use crate::state::AppState;

/// Fetch all enabled integrations concurrently and return a serialized dashboard.
/// Results are cached; returns cached value on cache hit.
pub async fn aggregate(state: &AppState) -> Result<Value, serde_json::Error> {
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
                Some(jira_config) => Some(services::jira::fetch_tickets(client, jira_config).await),
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
                    items.push(DashboardItem::GitHubPr(pr));
                }
            }
            Err(e) => errors.push(DashboardError {
                source: "github".to_string(),
                message: e.to_string(),
            }),
        }
    }

    if let Some(result) = jira_result {
        match result {
            Ok(tickets) => {
                for ticket in tickets {
                    items.push(DashboardItem::JiraTicket(ticket));
                }
            }
            Err(e) => errors.push(DashboardError {
                source: "jira".to_string(),
                message: e.to_string(),
            }),
        }
    }

    if let Some(result) = gl_mr_result {
        match result {
            Ok(mrs) => {
                for mr in mrs {
                    items.push(DashboardItem::GitLabMr(mr));
                }
            }
            Err(e) => errors.push(DashboardError {
                source: "gitlab".to_string(),
                message: e.to_string(),
            }),
        }
    }

    if let Some(result) = gl_pipe_result {
        match result {
            Ok(pipelines) => {
                for pipeline in pipelines {
                    items.push(DashboardItem::GitLabPipeline(pipeline));
                }
            }
            Err(e) => errors.push(DashboardError {
                source: "gitlab".to_string(),
                message: e.to_string(),
            }),
        }
    }

    if let Some(result) = cal_result {
        match result {
            Ok(events) => {
                for event in events {
                    items.push(DashboardItem::CalendarEvent(event));
                }
            }
            Err(e) => errors.push(DashboardError {
                source: "calendar".to_string(),
                message: e.to_string(),
            }),
        }
    }

    let dashboard = DashboardResponse {
        items,
        errors,
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    let response = serde_json::to_value(&dashboard)?;

    if dashboard.errors.is_empty() {
        state.api_cache.insert(cache_key, response.clone()).await;
    }

    Ok(response)
}
