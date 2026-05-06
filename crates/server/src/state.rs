use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::config::AppConfig;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub http_client: reqwest::Client,
    /// PKCE code_verifier stored between auth start and callback
    pub pkce_verifier: Arc<RwLock<Option<String>>>,
    /// Device code stored between device-code/start and device-code/poll
    pub device_code: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = Self::load_config().await.unwrap_or_default();

        let http_client = reqwest::Client::builder()
            .user_agent("myday/0.1.0")
            .build()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            http_client,
            pkce_verifier: Arc::new(RwLock::new(None)),
            device_code: Arc::new(RwLock::new(None)),
        })
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
        tokio::fs::write(&config_path, content).await?;
        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("myday")
            .join("config.json")
    }
}
