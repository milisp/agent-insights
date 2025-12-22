mod domain;
mod scanner;
mod agents;
mod services;
mod api;
mod cache;
mod websocket;

use axum::{
    routing::{get, get_service},
    Router,
    response::Html,
};
use tower_http::services::ServeDir;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Agent Insights API server...");

    let service = services::CollectionService::new()
        .expect("Failed to initialize collection service");

    match service.collect_all().await {
        Ok(records) => {
            tracing::info!("Startup scan complete: {} total records found", records.len());
        }
        Err(e) => {
            tracing::warn!("Startup scan warning: {}", e);
        }
    }

    if let Ok(stats) = service.cache_stats() {
        tracing::info!("Cache stats: {} total entries", stats.total_entries);
        for (agent, count) in stats.entries_by_agent {
            tracing::info!("  {}: {} cached entries", agent, count);
        }
    }

    // Set up WebSocket broadcast channel
    let (tx, _rx) = broadcast::channel::<websocket::UpdateMessage>(100);
    let ws_state = Arc::new(websocket::WsState { tx: tx.clone() });

    // Start file watcher
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let watch_paths = vec![
        ("Claude".to_string(), PathBuf::from(&home_dir).join(".claude").join("projects")),
        ("Gemini".to_string(), PathBuf::from(&home_dir).join(".gemini").join("tmp")),
        ("Codex".to_string(), PathBuf::from(&home_dir).join(".codex").join("sessions")),
    ];

    let mut watcher = websocket::FileWatcher::new(tx);
    if let Err(e) = watcher.start(watch_paths) {
        tracing::warn!("Failed to start file watcher: {}", e);
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve index.html at the root
    async fn index() -> Html<String> {
        let html = tokio::fs::read_to_string("dist/index.html")
            .await
            .unwrap_or_else(|_| "<h1>Index not found</h1>".to_string());
        Html(html)
    }

    let app = Router::new()
        .route("/", get(index))
        .nest_service(
            "/assets",
            get_service(ServeDir::new("dist/assets")).handle_error(|_| async {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Static file error")
            }),
        )
        .route("/api/heatmaps", get(api::get_all_heatmaps))
        .route("/api/heatmap/:agent", get(api::get_agent_heatmap))
        .route("/ws", get(websocket::ws_handler))
        .layer(cors)
        .with_state(ws_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("Server running on http://127.0.0.1:3001");
    tracing::info!("Available endpoints:");
    tracing::info!("  GET /api/heatmaps - Get all agent heatmaps");
    tracing::info!("  GET /api/heatmap/:agent - Get specific agent heatmap (claude, gemini, codex)");
    tracing::info!("  WS  /ws - WebSocket for real-time updates");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
