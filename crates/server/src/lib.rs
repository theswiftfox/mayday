pub mod config;
pub mod error;
pub mod services;
pub mod state;

#[cfg(feature = "http-server")]
pub mod routes;

#[cfg(feature = "http-server")]
pub use run::run_server;

#[cfg(feature = "http-server")]
mod run {
    use anyhow::Result;
    use axum::{routing::get, Router};
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::services::ServeDir;
    use tower_http::trace::TraceLayer;

    use crate::state::AppState;

    /// Start the myday API server. This function runs until the server is shut down.
    pub async fn run_server() -> Result<()> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Initialize application state
        let state = AppState::new().await?;

        // Build API router
        let api_router = Router::new()
            .route("/health", get(crate::routes::health::health_check))
            .nest("/github", crate::routes::github::router())
            .nest("/jira", crate::routes::jira::router())
            .nest("/gitlab", crate::routes::gitlab::router())
            .nest("/calendar", crate::routes::calendar::router())
            .nest("/config", crate::routes::config::router())
            .nest("/dashboard", crate::routes::dashboard::router());

        let mut app = Router::new()
            .nest("/api", api_router)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        // In production, serve static frontend files
        if let Ok(static_dir) = std::env::var("MYDAY_STATIC_DIR") {
            tracing::info!("Serving static files from: {}", static_dir);
            app = app.fallback_service(ServeDir::new(static_dir));
        }

        // Start server
        let port = std::env::var("MYDAY_PORT").unwrap_or_else(|_| "3001".to_string());
        let addr = format!("0.0.0.0:{}", port);
        tracing::info!("Starting server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
