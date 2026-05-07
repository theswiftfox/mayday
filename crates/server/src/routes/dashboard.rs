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

/// Fetches all enabled integrations and returns a unified dashboard view.
/// All external API calls run concurrently via tokio::join! for minimal latency.
/// Results are cached for 90 seconds to avoid redundant external API calls.
async fn get_dashboard(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let cache_key = "dashboard".to_string();

    // Check cache first (Fix 2)
    if let Some(cached) = state.api_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    // Clone config and release the read lock immediately (Fix 3)
    let config = state.config.read().await.clone();

    let client = &state.http_client;

    // Launch all fetches concurrently (Fix 1)
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
                Some(gl_config) => Some(services::gitlab::fetch_pipelines(client, gl_config).await),
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

    // Collect GitHub PRs
    if let Some(result) = gh_result {
        match result {
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

    // Collect JIRA tickets
    if let Some(result) = jira_result {
        match result {
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

    // Collect GitLab MRs
    if let Some(result) = gl_mr_result {
        match result {
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
    }

    // Collect GitLab Pipelines
    if let Some(result) = gl_pipe_result {
        match result {
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

    // Collect Calendar events
    if let Some(result) = cal_result {
        match result {
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

    let response = serde_json::json!({
        "items": items,
        "errors": errors,
        "last_updated": timestamp,
    });

    // Store in cache for subsequent requests (Fix 2)
    state.api_cache.insert(cache_key, response.clone()).await;

    Ok(Json(response))
}
