// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use anyhow::Result;
use moka::future::Cache;
use serde_json::Value;

use crate::config::AppConfig;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub http_client: reqwest::Client,
    /// PKCE code_verifiers keyed by a random session ID.
    /// Each entry expires after 10 minutes (max OAuth flow lifetime).
    pub pkce_verifiers: Cache<String, String>,
    /// Device codes keyed by source name (e.g. "calendar").
    /// Each entry expires after 15 minutes.
    pub device_codes: Cache<String, String>,
    /// TTL cache for API responses (avoids repeated external API calls)
    pub api_cache: Cache<String, Value>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = Self::load_config().await.unwrap_or_default();

        let http_client = reqwest::Client::builder()
            .user_agent("myday/0.1.0")
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        let api_cache = Cache::builder()
            .time_to_live(Duration::from_secs(90))
            .max_capacity(10)
            .build();

        let pkce_verifiers = Cache::builder()
            .time_to_live(Duration::from_secs(600)) // 10 minute expiry
            .max_capacity(16)
            .build();

        let device_codes = Cache::builder()
            .time_to_live(Duration::from_secs(900)) // 15 minute expiry
            .max_capacity(8)
            .build();

        let config_path = Self::config_path();
        let config_existed = config_path.exists();

        let state = Self {
            config: Arc::new(RwLock::new(config)),
            http_client,
            pkce_verifiers,
            device_codes,
            api_cache,
        };

        // Re-save to migrate any legacy snake_case keys to camelCase.
        // The `alias` attributes on config structs accept old names on read,
        // and `rename_all = "camelCase"` writes the canonical names back.
        if config_existed {
            if let Err(e) = state.save_config().await {
                tracing::warn!("Config migration re-save failed: {e}");
            }
        }

        Ok(state)
    }

    async fn load_config() -> Result<AppConfig> {
        // Try to load from config file
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub async fn save_config(&self) -> Result<()> {
        let config = self.config.read().await;
        let config_path = Self::config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&*config)?;

        // Write to temp file first, then atomic rename
        let tmp_path = config_path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, &content).await?;
        tokio::fs::rename(&tmp_path, &config_path).await?;

        // Restrict file permissions to owner-only (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(&config_path, perms).await?;
        }

        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("myday")
            .join("config.json")
    }
}
