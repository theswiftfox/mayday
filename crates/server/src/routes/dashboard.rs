use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_dashboard))
}

/// Fetches all enabled integrations and returns a unified dashboard view
async fn get_dashboard(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let config = state.config.read().await;
    let mut items = Vec::new();
    let mut errors = Vec::new();

    // Fetch GitHub PRs
    if let Some(gh_config) = &config.github {
        match services::github::fetch_prs(&state.http_client, gh_config).await {
            Ok(prs) => {
                for pr in prs {
                    items.push(json!({ "type": "github_pr", "data": pr }));
                }
            }
            Err(e) => errors.push(json!({
                "source": "github",
                "message": e.to_string(),
            })),
        }
    }

    // Fetch JIRA tickets
    if let Some(jira_config) = &config.jira {
        match services::jira::fetch_tickets(&state.http_client, jira_config).await {
            Ok(tickets) => {
                for ticket in tickets {
                    items.push(json!({ "type": "jira_ticket", "data": ticket }));
                }
            }
            Err(e) => errors.push(json!({
                "source": "jira",
                "message": e.to_string(),
            })),
        }
    }

    // Fetch GitLab MRs
    if let Some(gl_config) = &config.gitlab {
        match services::gitlab::fetch_mrs(&state.http_client, gl_config).await {
            Ok(mrs) => {
                for mr in mrs {
                    items.push(json!({ "type": "gitlab_mr", "data": mr }));
                }
            }
            Err(e) => errors.push(json!({
                "source": "gitlab",
                "message": e.to_string(),
            })),
        }

        match services::gitlab::fetch_pipelines(&state.http_client, gl_config).await {
            Ok(pipelines) => {
                for pipeline in pipelines {
                    items.push(json!({ "type": "gitlab_pipeline", "data": pipeline }));
                }
            }
            Err(e) => errors.push(json!({
                "source": "gitlab",
                "message": e.to_string(),
            })),
        }
    }

    // Fetch Calendar events
    if let Some(calendar_config) = &config.calendar {
        match services::calendar::fetch_todays_events(&state.http_client, calendar_config).await {
            Ok(events) => {
                for event in events {
                    items.push(json!({ "type": "calendar_event", "data": event }));
                }
            }
            Err(e) => errors.push(json!({
                "source": "calendar",
                "message": e.to_string(),
            })),
        }
    }

    let timestamp = chrono::Utc::now().to_rfc3339();

    Ok(Json(json!({
        "items": items,
        "errors": errors,
        "last_updated": timestamp,
    })))
}
