pub mod error;
pub mod routes;
pub mod state;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

/// Build the Axum application with all routes.
pub fn build_app(templates_dir: PathBuf, static_dir: Option<PathBuf>) -> Router {
    let state = Arc::new(AppState::new(templates_dir));

    // Start session sweeper
    let sweep_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            sweep_state.sweep_idle_sessions();
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = routes::api_router()
        .layer(cors)
        .with_state(state);

    // Serve static files if a directory is provided
    if let Some(static_path) = static_dir {
        let serve_dir = tower_http::services::ServeDir::new(&static_path)
            .fallback(tower_http::services::ServeFile::new(static_path.join("index.html")));
        app = app.fallback_service(serve_dir);
    }

    app
}
