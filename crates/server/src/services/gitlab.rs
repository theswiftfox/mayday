use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::GitLabConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabMR {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub url: String,
    pub project: String,
    pub project_id: u64,
    pub author: String,
    pub state: String,
    pub is_draft: bool,
    pub created_at: String,
    pub updated_at: String,
    pub role: String, // "author" or "reviewer"
    pub has_new_comments: bool,
    pub has_new_commits: bool,
    pub comment_count: u64,
    pub labels: Vec<String>,
    pub merge_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabMRDetail {
    #[serde(flatten)]
    pub mr: GitLabMR,
    pub description: Option<String>,
    pub notes: Vec<GitLabNote>,
    pub pipelines: Vec<GitLabPipeline>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabNote {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub created_at: String,
    pub system: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabPipeline {
    pub id: u64,
    pub status: String,
    pub ref_name: String,
    pub url: String,
    pub project: String,
    pub project_id: u64,
    pub created_at: String,
    pub updated_at: String,
    pub duration: Option<u64>,
}

/// Fetch MRs that need attention
pub async fn fetch_mrs(client: &Client, config: &GitLabConfig) -> AppResult<Vec<GitLabMR>> {
    let base_url = format!("https://{}/api/v4", sanitize_host(&config.host));
    let mut all_mrs = Vec::new();

    // Fetch MRs authored by user
    let authored_resp = client
        .get(format!("{}/merge_requests", base_url))
        .query(&[
            ("state", "opened"),
            ("scope", "created_by_me"),
            ("per_page", "50"),
        ])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitLab API: {}", e)))?;

    let authored: Vec<Value> = authored_resp.json().await?;
    for mr in &authored {
        if let Some(parsed) = parse_gitlab_mr(mr, "author", config) {
            all_mrs.push(parsed);
        }
    }

    // Fetch MRs where user is reviewer
    let review_resp = client
        .get(format!("{}/merge_requests", base_url))
        .query(&[
            ("state", "opened"),
            ("scope", "all"),
            ("reviewer_username", config.username.as_str()),
            ("per_page", "50"),
        ])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitLab API: {}", e)))?;

    let reviewing: Vec<Value> = review_resp.json().await?;
    for mr in &reviewing {
        if let Some(parsed) = parse_gitlab_mr(mr, "reviewer", config) {
            // Avoid duplicates
            if !all_mrs.iter().any(|m| m.id == parsed.id) {
                all_mrs.push(parsed);
            }
        }
    }

    Ok(all_mrs)
}

/// Fetch recent pipelines for the user's projects
pub async fn fetch_pipelines(
    client: &Client,
    config: &GitLabConfig,
) -> AppResult<Vec<GitLabPipeline>> {
    let base_url = format!("https://{}/api/v4", sanitize_host(&config.host));
    let mut all_pipelines = Vec::new();

    // Determine project IDs: use configured list, or derive from user's MRs
    let project_ids = if !config.project_ids.is_empty() {
        config.project_ids.clone()
    } else {
        // Auto-discover from MRs the user is involved in
        discover_project_ids(client, config).await?
    };

    for project_id in &project_ids {
        let resp = client
            .get(format!("{}/projects/{}/pipelines", base_url, project_id))
            .query(&[("per_page", "10"), ("order_by", "updated_at")])
            .header("PRIVATE-TOKEN", &config.token)
            .send()
            .await?;

        if resp.status().is_success() {
            let pipelines: Vec<Value> = resp.json().await?;
            for p in &pipelines {
                if let Some(pipeline) = parse_gitlab_pipeline(p, *project_id, config) {
                    all_pipelines.push(pipeline);
                }
            }
        }
    }

    // Sort by created_at descending (most recent first)
    all_pipelines.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(all_pipelines)
}

/// Discover project IDs from MRs the user is involved in
async fn discover_project_ids(client: &Client, config: &GitLabConfig) -> AppResult<Vec<u64>> {
    let base_url = format!("https://{}/api/v4", sanitize_host(&config.host));
    let mut project_ids = Vec::new();

    // Get project IDs from authored MRs
    let resp = client
        .get(format!("{}/merge_requests", base_url))
        .query(&[
            ("state", "opened"),
            ("scope", "created_by_me"),
            ("per_page", "50"),
        ])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?;

    if resp.status().is_success() {
        let mrs: Vec<Value> = resp.json().await?;
        for mr in &mrs {
            if let Some(pid) = mr["project_id"].as_u64() {
                if !project_ids.contains(&pid) {
                    project_ids.push(pid);
                }
            }
        }
    }

    // Also from reviewer MRs
    let resp = client
        .get(format!("{}/merge_requests", base_url))
        .query(&[
            ("state", "opened"),
            ("scope", "all"),
            ("reviewer_username", config.username.as_str()),
            ("per_page", "50"),
        ])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?;

    if resp.status().is_success() {
        let mrs: Vec<Value> = resp.json().await?;
        for mr in &mrs {
            if let Some(pid) = mr["project_id"].as_u64() {
                if !project_ids.contains(&pid) {
                    project_ids.push(pid);
                }
            }
        }
    }

    Ok(project_ids)
}

/// Fetch detailed MR info
pub async fn fetch_mr_detail(
    client: &Client,
    config: &GitLabConfig,
    project_id: u64,
    iid: u64,
) -> AppResult<GitLabMRDetail> {
    let base_url = format!("https://{}/api/v4", sanitize_host(&config.host));

    // Fetch MR details
    let mr_resp = client
        .get(format!(
            "{}/projects/{}/merge_requests/{}",
            base_url, project_id, iid
        ))
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitLab API: {}", e)))?;

    let mr_data: Value = mr_resp.json().await?;

    let mr = parse_gitlab_mr(&mr_data, "unknown", config)
        .ok_or_else(|| AppError::ExternalApi("Failed to parse GitLab MR".to_string()))?;

    // Fetch notes (comments)
    let notes_resp = client
        .get(format!(
            "{}/projects/{}/merge_requests/{}/notes",
            base_url, project_id, iid
        ))
        .query(&[("per_page", "50")])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitLab API: {}", e)))?;

    let notes_data: Vec<Value> = notes_resp.json().await?;

    let notes = notes_data
        .iter()
        .map(|n| GitLabNote {
            id: n["id"].as_u64().unwrap_or(0),
            author: n["author"]["username"].as_str().unwrap_or("").to_string(),
            body: n["body"].as_str().unwrap_or("").to_string(),
            created_at: n["created_at"].as_str().unwrap_or("").to_string(),
            system: n["system"].as_bool().unwrap_or(false),
        })
        .collect();

    // Fetch pipelines for this MR
    let pipelines_resp = client
        .get(format!(
            "{}/projects/{}/merge_requests/{}/pipelines",
            base_url, project_id, iid
        ))
        .query(&[("per_page", "5")])
        .header("PRIVATE-TOKEN", &config.token)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitLab API: {}", e)))?;

    let pipelines_data: Vec<Value> = pipelines_resp.json().await?;
    let pipelines = pipelines_data
        .iter()
        .filter_map(|p| parse_gitlab_pipeline(p, project_id, config))
        .collect();

    Ok(GitLabMRDetail {
        mr,
        description: mr_data["description"].as_str().map(String::from),
        notes,
        pipelines,
    })
}

fn parse_gitlab_mr(mr: &Value, role: &str, config: &GitLabConfig) -> Option<GitLabMR> {
    let id = mr["id"].as_u64()?;
    let project_id = mr["project_id"].as_u64().unwrap_or(0);

    // Apply project filter
    if !config.project_ids.is_empty() && !config.project_ids.contains(&project_id) {
        return None;
    }

    Some(GitLabMR {
        id,
        iid: mr["iid"].as_u64().unwrap_or(0),
        title: mr["title"].as_str().unwrap_or("").to_string(),
        url: mr["web_url"].as_str().unwrap_or("").to_string(),
        project: mr["references"]["full"]
            .as_str()
            .unwrap_or("")
            .split('!')
            .next()
            .unwrap_or("")
            .to_string(),
        project_id,
        author: mr["author"]["username"].as_str().unwrap_or("").to_string(),
        state: mr["state"].as_str().unwrap_or("opened").to_string(),
        is_draft: mr["draft"].as_bool().unwrap_or(false)
            || mr["work_in_progress"].as_bool().unwrap_or(false),
        created_at: mr["created_at"].as_str().unwrap_or("").to_string(),
        updated_at: mr["updated_at"].as_str().unwrap_or("").to_string(),
        role: role.to_string(),
        has_new_comments: false, // TODO: track based on last viewed
        has_new_commits: false,  // TODO: track based on last viewed
        comment_count: mr["user_notes_count"].as_u64().unwrap_or(0),
        labels: mr["labels"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        merge_status: mr["merge_status"].as_str().map(String::from),
    })
}

fn parse_gitlab_pipeline(p: &Value, project_id: u64, _config: &GitLabConfig) -> Option<GitLabPipeline> {
    Some(GitLabPipeline {
        id: p["id"].as_u64()?,
        status: p["status"].as_str().unwrap_or("unknown").to_string(),
        ref_name: p["ref"].as_str().unwrap_or("").to_string(),
        url: p["web_url"].as_str().unwrap_or("").to_string(),
        project: p["project"]["path_with_namespace"]
            .as_str()
            .unwrap_or(&format!("project/{}", project_id))
            .to_string(),
        project_id,
        created_at: p["created_at"].as_str().unwrap_or("").to_string(),
        updated_at: p["updated_at"].as_str().unwrap_or("").to_string(),
        duration: p["duration"].as_u64(),
    })
}

/// Strip protocol prefix and trailing slashes from a host string
fn sanitize_host(host: &str) -> String {
    host.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}
