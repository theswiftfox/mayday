use std::sync::Arc;

use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::config::GitHubConfig;
use crate::error::{AppError, AppResult};

/// Max concurrent enrichment requests to avoid GitHub secondary rate limits
const MAX_CONCURRENT_ENRICHMENTS: usize = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPR {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub repo: String,
    pub author: String,
    pub state: String,
    pub is_draft: bool,
    pub created_at: String,
    pub updated_at: String,
    pub role: String, // "author", "reviewer", or "other"
    pub has_new_comments: bool,
    pub has_new_commits: bool,
    pub action_required: bool,
    pub comment_count: u64,
    pub last_commit_at: Option<String>,
    pub labels: Vec<String>,
    pub review_decision: Option<String>,
    /// CI status: "success", "failure", "pending", "neutral", or null
    pub ci_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPRDetail {
    #[serde(flatten)]
    pub pr: GitHubPR,
    pub body: Option<String>,
    pub comments: Vec<GitHubComment>,
    pub reviews: Vec<GitHubReview>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubComment {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubReview {
    pub id: u64,
    pub author: String,
    pub state: String,
    pub body: Option<String>,
    pub submitted_at: String,
}

/// Fetch PRs that need attention from the authenticated user
pub async fn fetch_prs(client: &Client, config: &GitHubConfig) -> AppResult<Vec<GitHubPR>> {
    let mut all_prs = Vec::new();

    // Fetch authored and review-requested PRs in parallel (they're independent)
    let (authored_result, reviewing_result) = tokio::join!(
        fetch_authored_prs(client, config),
        fetch_review_requested_prs(client, config),
    );

    all_prs.extend(authored_result?);
    all_prs.extend(reviewing_result?);

    // Fetch other open PRs in configured repos (depends on above results for deduplication)
    let other = fetch_other_open_prs(client, config, &all_prs).await?;
    all_prs.extend(other);

    // Enrich PRs with action_required status — only for authored/reviewing
    // Use a semaphore to limit concurrent requests and avoid GitHub rate limits
    let (to_enrich, other_prs): (Vec<_>, Vec<_>) = all_prs
        .into_iter()
        .partition(|pr| pr.role == "author" || pr.role == "reviewer");

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_ENRICHMENTS));
    let mut all_prs: Vec<GitHubPR> = stream::iter(to_enrich)
        .map(|pr| {
            let sem = semaphore.clone();
            let client = client.clone();
            let config = config.clone();
            async move {
                let _permit = sem.acquire().await.unwrap();
                enrich_pr_action_required(&client, &config, pr).await
            }
        })
        .buffer_unordered(MAX_CONCURRENT_ENRICHMENTS)
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    all_prs.extend(other_prs);

    Ok(all_prs)
}

/// Enrich a single PR with action_required by fetching commits and comments
async fn enrich_pr_action_required(
    client: &Client,
    config: &GitHubConfig,
    mut pr: GitHubPR,
) -> AppResult<GitHubPR> {
    let parts: Vec<&str> = pr.repo.split('/').collect();
    if parts.len() != 2 {
        return Ok(pr);
    }
    let (owner, repo) = (parts[0], parts[1]);
    let base_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr.number
    );

    // Fetch commits, comments, reviews, and check-runs ALL in parallel
    let commits_fut = client
        .get(format!("{}/commits?per_page=100", base_url))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send();

    let comments_fut = client
        .get(format!("{}/comments?per_page=100", base_url))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send();

    let reviews_fut = client
        .get(format!("{}/reviews?per_page=100", base_url))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send();

    let (commits_res, comments_res, reviews_res) =
        tokio::join!(commits_fut, comments_fut, reviews_fut);

    // Parse commits - get the latest commit date and head SHA
    let commits: Vec<Value> = match commits_res {
        Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
        _ => vec![],
    };

    let head_sha = commits.last().and_then(|c| c["sha"].as_str()).map(String::from);

    let last_commit_at = commits
        .last()
        .and_then(|c| c["commit"]["committer"]["date"].as_str())
        .map(String::from);

    // For author role: find the last commit by the author
    let my_last_commit_at = commits
        .iter()
        .rev()
        .find(|c| {
            c["author"]["login"].as_str() == Some(&config.username)
                || c["committer"]["login"].as_str() == Some(&config.username)
        })
        .and_then(|c| c["commit"]["committer"]["date"].as_str())
        .map(String::from);

    pr.last_commit_at = last_commit_at.clone();

    // Fetch CI status from check runs for the head commit (fires immediately, awaited later)
    let check_runs_fut = if let Some(ref sha) = head_sha {
        let fut = client
            .get(format!(
                "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
                owner, repo, sha
            ))
            .header("Authorization", format!("Bearer {}", config.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send();
        Some(fut)
    } else {
        None
    };

    // Parse review comments (inline comments on diffs)
    let comments: Vec<Value> = match comments_res {
        Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
        _ => vec![],
    };

    // Parse reviews (approve/request changes/comment reviews)
    let reviews: Vec<Value> = match reviews_res {
        Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
        _ => vec![],
    };

    // Compute action_required based on role
    pr.action_required = match pr.role.as_str() {
        // Never action required for draft PRs
        _ if pr.is_draft => false,
        // Never action required for reviewer when changes are requested (ball is in author's court)
        "reviewer"
            if pr.labels.iter().any(|l| {
                let lower = l.to_ascii_lowercase();
                lower == "changes requested" || lower == "changes-requested"
            }) =>
        {
            false
        }
        "author" => {
            // Action required if there are comments from others newer than my last commit
            if let Some(ref my_commit_date) = my_last_commit_at {
                let has_newer_comment = comments.iter().any(|c| {
                    let is_other = c["user"]["login"].as_str() != Some(&config.username);
                    let comment_date = c["created_at"].as_str().unwrap_or("");
                    is_other && comment_date > my_commit_date.as_str()
                });
                let has_newer_review = reviews.iter().any(|r| {
                    let is_other = r["user"]["login"].as_str() != Some(&config.username);
                    let review_date = r["submitted_at"].as_str().unwrap_or("");
                    let is_substantive = r["state"].as_str() != Some("PENDING");
                    is_other && is_substantive && review_date > my_commit_date.as_str()
                });
                has_newer_comment || has_newer_review
            } else {
                false
            }
        }
        "reviewer" => {
            // Action required if there are new commits since my last comment/review
            let my_last_activity = reviews
                .iter()
                .filter(|r| r["user"]["login"].as_str() == Some(&config.username))
                .filter_map(|r| r["submitted_at"].as_str())
                .chain(
                    comments
                        .iter()
                        .filter(|c| c["user"]["login"].as_str() == Some(&config.username))
                        .filter_map(|c| c["created_at"].as_str()),
                )
                .max()
                .map(String::from);

            if let (Some(ref latest_commit), Some(ref my_activity)) =
                (&last_commit_at, &my_last_activity)
            {
                latest_commit.as_str() > my_activity.as_str()
            } else {
                // If reviewer has never commented, action is required
                my_last_activity.is_none()
            }
        }
        _ => false,
    };

    // Also update has_new_comments/has_new_commits based on the same data
    pr.has_new_comments = match pr.role.as_str() {
        "author" => {
            if let Some(ref my_commit_date) = my_last_commit_at {
                comments.iter().any(|c| {
                    c["user"]["login"].as_str() != Some(&config.username)
                        && c["created_at"].as_str().unwrap_or("") > my_commit_date.as_str()
                }) || reviews.iter().any(|r| {
                    r["user"]["login"].as_str() != Some(&config.username)
                        && r["submitted_at"].as_str().unwrap_or("") > my_commit_date.as_str()
                        && r["state"].as_str() != Some("PENDING")
                })
            } else {
                false
            }
        }
        _ => false,
    };

    pr.has_new_commits = match pr.role.as_str() {
        "reviewer" => {
            let my_last_activity = reviews
                .iter()
                .filter(|r| r["user"]["login"].as_str() == Some(&config.username))
                .filter_map(|r| r["submitted_at"].as_str())
                .chain(
                    comments
                        .iter()
                        .filter(|c| c["user"]["login"].as_str() == Some(&config.username))
                        .filter_map(|c| c["created_at"].as_str()),
                )
                .max();

            if let (Some(latest_commit), Some(my_activity)) = (&last_commit_at, my_last_activity) {
                latest_commit.as_str() > my_activity
            } else {
                false
            }
        }
        _ => false,
    };

    // Await the check-runs response (was fired earlier, concurrently with comment/review parsing)
    if let Some(check_fut) = check_runs_fut {
        if let Ok(resp) = check_fut.await {
            if resp.status().is_success() {
                let body: Value = resp.json().await.unwrap_or_default();
                let check_runs = body["check_runs"].as_array();
                if let Some(runs) = check_runs {
                    if !runs.is_empty() {
                        // Derive overall status:
                        // - any failure/timed_out/cancelled → "failure"
                        // - any in_progress/queued/pending → "pending"
                        // - all success/neutral/skipped → "success"
                        let mut has_failure = false;
                        let mut has_pending = false;

                        for run in runs {
                            let status = run["status"].as_str().unwrap_or("");
                            let conclusion = run["conclusion"].as_str().unwrap_or("");

                            if status == "in_progress" || status == "queued" || status == "pending"
                            {
                                has_pending = true;
                            } else if conclusion == "failure"
                                || conclusion == "timed_out"
                                || conclusion == "cancelled"
                            {
                                has_failure = true;
                            }
                            // neutral/skipped/success all count as passing
                        }

                        pr.ci_status = Some(if has_failure {
                            "failure".to_string()
                        } else if has_pending {
                            "pending".to_string()
                        } else {
                            "success".to_string()
                        });
                    }
                }
            }
        }
    }

    Ok(pr)
}

async fn fetch_authored_prs(client: &Client, config: &GitHubConfig) -> AppResult<Vec<GitHubPR>> {
    let query = format!(
        "is:pr is:open author:{} archived:false",
        config.username
    );

    let resp = client
        .get("https://api.github.com/search/issues")
        .query(&[("q", &query), ("per_page", &"50".to_string())])
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let body: Value = resp.json().await?;
    let empty = vec![];
    let items = body["items"].as_array().unwrap_or(&empty);

    let mut prs = Vec::new();
    for item in items {
        if let Some(pr) = parse_search_result(item, &config.username, "author") {
            // Apply repo filter if configured
            if config.repos.is_empty() || config.repos.iter().any(|r| pr.repo == *r) {
                prs.push(pr);
            }
        }
    }

    Ok(prs)
}

async fn fetch_review_requested_prs(
    client: &Client,
    config: &GitHubConfig,
) -> AppResult<Vec<GitHubPR>> {
    let query = format!(
        "is:pr is:open review-requested:{} archived:false",
        config.username
    );

    let resp = client
        .get("https://api.github.com/search/issues")
        .query(&[("q", &query), ("per_page", &"50".to_string())])
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let body: Value = resp.json().await?;
    let empty = vec![];
    let items = body["items"].as_array().unwrap_or(&empty);

    let mut prs = Vec::new();
    for item in items {
        if let Some(pr) = parse_search_result(item, &config.username, "reviewer") {
            if config.repos.is_empty() || config.repos.iter().any(|r| pr.repo == *r) {
                prs.push(pr);
            }
        }
    }

    Ok(prs)
}

/// Fetch all other open PRs in configured repos that aren't already in the authored/reviewing lists
async fn fetch_other_open_prs(
    client: &Client,
    config: &GitHubConfig,
    existing_prs: &[GitHubPR],
) -> AppResult<Vec<GitHubPR>> {
    // Only fetch if repos are configured
    if config.repos.is_empty() {
        return Ok(vec![]);
    }

    // Build a search query: open PRs in the configured repos
    let repo_filter: Vec<String> = config.repos.iter().map(|r| format!("repo:{}", r)).collect();
    let query = format!("is:pr is:open {}", repo_filter.join(" "));

    let resp = client
        .get("https://api.github.com/search/issues")
        .query(&[("q", &query), ("per_page", &"100".to_string())])
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let body: Value = resp.json().await?;
    let empty = vec![];
    let items = body["items"].as_array().unwrap_or(&empty);

    // Collect IDs of PRs we already have
    let existing_ids: std::collections::HashSet<u64> = existing_prs.iter().map(|pr| pr.id).collect();

    let mut prs = Vec::new();
    for item in items {
        if let Some(pr) = parse_search_result(item, &config.username, "other") {
            // Skip PRs we already have as authored/reviewing
            if !existing_ids.contains(&pr.id) {
                prs.push(pr);
            }
        }
    }

    Ok(prs)
}

/// Fetch detailed PR information including comments and reviews
pub async fn fetch_pr_detail(
    client: &Client,
    config: &GitHubConfig,
    owner: &str,
    repo: &str,
    number: u64,
) -> AppResult<GitHubPRDetail> {
    let base_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, number
    );

    // Fetch PR details
    let pr_resp = client
        .get(&base_url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let pr_data: Value = pr_resp.json().await?;

    // Fetch comments
    let comments_resp = client
        .get(format!("{}/comments", base_url))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let comments_data: Vec<Value> = comments_resp.json().await?;

    // Fetch reviews
    let reviews_resp = client
        .get(format!("{}/reviews", base_url))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let reviews_data: Vec<Value> = reviews_resp.json().await?;

    let repo_full = format!("{}/{}", owner, repo);
    let role = if pr_data["user"]["login"].as_str() == Some(&config.username) {
        "author"
    } else {
        "reviewer"
    };

    let pr = GitHubPR {
        id: pr_data["id"].as_u64().unwrap_or(0),
        number,
        title: pr_data["title"].as_str().unwrap_or("").to_string(),
        url: pr_data["html_url"].as_str().unwrap_or("").to_string(),
        repo: repo_full,
        author: pr_data["user"]["login"].as_str().unwrap_or("").to_string(),
        state: pr_data["state"].as_str().unwrap_or("open").to_string(),
        is_draft: pr_data["draft"].as_bool().unwrap_or(false),
        created_at: pr_data["created_at"].as_str().unwrap_or("").to_string(),
        updated_at: pr_data["updated_at"].as_str().unwrap_or("").to_string(),
        role: role.to_string(),
        has_new_comments: false, // TODO: compute in detail view too
        has_new_commits: false,  // TODO: compute in detail view too
        action_required: false,  // TODO: compute in detail view too
        comment_count: pr_data["comments"].as_u64().unwrap_or(0)
            + pr_data["review_comments"].as_u64().unwrap_or(0),
        last_commit_at: None,
        labels: pr_data["labels"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|v| v["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        review_decision: None,
        ci_status: None,
    };

    let comments = comments_data
        .iter()
        .map(|c| GitHubComment {
            id: c["id"].as_u64().unwrap_or(0),
            author: c["user"]["login"].as_str().unwrap_or("").to_string(),
            body: c["body"].as_str().unwrap_or("").to_string(),
            created_at: c["created_at"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    let reviews = reviews_data
        .iter()
        .map(|r| GitHubReview {
            id: r["id"].as_u64().unwrap_or(0),
            author: r["user"]["login"].as_str().unwrap_or("").to_string(),
            state: r["state"].as_str().unwrap_or("").to_string(),
            body: r["body"].as_str().map(String::from),
            submitted_at: r["submitted_at"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(GitHubPRDetail {
        pr,
        body: pr_data["body"].as_str().map(String::from),
        comments,
        reviews,
    })
}

fn parse_search_result(item: &Value, _username: &str, role: &str) -> Option<GitHubPR> {
    let url = item["html_url"].as_str()?;
    // Extract owner/repo from URL: https://github.com/owner/repo/pull/123
    let parts: Vec<&str> = url.split('/').collect();
    let repo = if parts.len() >= 5 {
        format!("{}/{}", parts[3], parts[4])
    } else {
        return None;
    };

    Some(GitHubPR {
        id: item["id"].as_u64().unwrap_or(0),
        number: item["number"].as_u64().unwrap_or(0),
        title: item["title"].as_str().unwrap_or("").to_string(),
        url: url.to_string(),
        repo,
        author: item["user"]["login"].as_str().unwrap_or("").to_string(),
        state: item["state"].as_str().unwrap_or("open").to_string(),
        is_draft: item["draft"].as_bool().unwrap_or(false),
        created_at: item["created_at"].as_str().unwrap_or("").to_string(),
        updated_at: item["updated_at"].as_str().unwrap_or("").to_string(),
        role: role.to_string(),
        has_new_comments: false,
        has_new_commits: false,
        action_required: false,
        comment_count: item["comments"].as_u64().unwrap_or(0),
        last_commit_at: None,
        labels: item["labels"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|v| v["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        review_decision: None,
        ci_status: None,
    })
}

// --- GitHub Authentication ---

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

/// Try to detect an existing token from the `gh` CLI
pub fn detect_gh_cli_token() -> Option<(String, String)> {
    // First try `gh auth token`
    if let Ok(output) = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
    {
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                // Also get the username
                if let Ok(user_output) = std::process::Command::new("gh")
                    .args(["api", "user", "--jq", ".login"])
                    .output()
                {
                    if user_output.status.success() {
                        let username =
                            String::from_utf8_lossy(&user_output.stdout).trim().to_string();
                        if !username.is_empty() {
                            return Some((token, username));
                        }
                    }
                }
                return Some((token, String::new()));
            }
        }
    }

    // Fallback: try reading ~/.config/gh/hosts.yml
    if let Some(config_dir) = dirs_next::config_dir() {
        let hosts_path = config_dir.join("gh").join("hosts.yml");
        if let Ok(content) = std::fs::read_to_string(&hosts_path) {
            // Simple YAML parsing for the token
            // Format: github.com:\n  oauth_token: <token>\n  user: <username>
            if let Some(token_line) = content.lines().find(|l| l.contains("oauth_token:")) {
                let token = token_line.split(':').nth(1).map(|s| s.trim().to_string());
                let username = content
                    .lines()
                    .find(|l| l.contains("user:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string());

                if let Some(token) = token {
                    return Some((token, username.unwrap_or_default()));
                }
            }
        }
    }

    None
}

/// Start the GitHub device code OAuth flow
/// Requires a registered GitHub OAuth App client_id
pub async fn start_device_code_flow(
    client: &Client,
    client_id: &str,
) -> AppResult<DeviceCodeResponse> {
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("scope", "repo read:org"),
        ])
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub device code: {}", e)))?;

    let body: DeviceCodeResponse = resp.json().await.map_err(|e| {
        AppError::ExternalApi(format!("Failed to parse device code response: {}", e))
    })?;

    Ok(body)
}

/// Poll for the access token after user has authorized the device
pub async fn poll_device_code_token(
    client: &Client,
    client_id: &str,
    device_code: &str,
) -> AppResult<Option<GitHubTokenResponse>> {
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await?;

    let body: Value = resp.json().await?;

    // Check for errors (authorization_pending, slow_down, etc.)
    if let Some(error) = body["error"].as_str() {
        match error {
            "authorization_pending" | "slow_down" => return Ok(None),
            "expired_token" => {
                return Err(AppError::ExternalApi(
                    "Device code expired. Please restart the flow.".to_string(),
                ))
            }
            "access_denied" => {
                return Err(AppError::ExternalApi(
                    "Access was denied by the user.".to_string(),
                ))
            }
            other => {
                return Err(AppError::ExternalApi(format!(
                    "GitHub OAuth error: {}",
                    other
                )))
            }
        }
    }

    if let Some(access_token) = body["access_token"].as_str() {
        Ok(Some(GitHubTokenResponse {
            access_token: access_token.to_string(),
            token_type: body["token_type"].as_str().unwrap_or("bearer").to_string(),
            scope: body["scope"].as_str().unwrap_or("").to_string(),
        }))
    } else {
        Ok(None)
    }
}

/// Fetch the authenticated user's login name
pub async fn fetch_authenticated_user(client: &Client, token: &str) -> AppResult<String> {
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::ExternalApi(format!("GitHub API: {}", e)))?;

    let body: Value = resp.json().await?;
    let login = body["login"]
        .as_str()
        .ok_or_else(|| AppError::ExternalApi("No login in user response".to_string()))?;

    Ok(login.to_string())
}
