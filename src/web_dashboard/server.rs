//! Axum HTTP server for the web dashboard

use super::{api, assets, websocket, DashboardState, SharedState};
use anyhow::Result;
use axum::{
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Start the web dashboard server
pub async fn start_server(port: u16, open_browser: bool) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let state: SharedState = Arc::new(RwLock::new(DashboardState::new(cwd)));

    let app = Router::new()
        // Static assets
        .route("/", get(serve_index))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .route("/xterm.min.js", get(serve_xterm_js))
        .route("/xterm.css", get(serve_xterm_css))
        .route("/xterm-addon-fit.min.js", get(serve_xterm_fit_js))
        .route("/marked.min.js", get(serve_marked_js))
        // API endpoints
        .route("/api/health", get(api::health))
        .route("/api/files", get(api::list_files))
        .route("/api/file", get(api::read_file))
        .route("/api/file", post(api::write_file))
        .route("/api/tree", get(api::get_tree))
        .route("/api/markdown", get(api::render_markdown))
        // WebSocket endpoints
        .route("/ws/terminal", get(websocket::terminal_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("\x1b[32m");
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║     Smart Tree Web Dashboard                 ║");
    println!("  ╠══════════════════════════════════════════════╣");
    println!("  ║  http://127.0.0.1:{}                      ║", port);
    println!("  ║                                              ║");
    println!("  ║  Terminal: Real PTY with bash/zsh            ║");
    println!("  ║  Files: Browse and edit                      ║");
    println!("  ║  Preview: Markdown rendering                 ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!("\x1b[0m");

    if open_browser {
        let url = format!("http://127.0.0.1:{}", port);
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Static asset handlers
async fn serve_index() -> Html<&'static str> {
    Html(assets::INDEX_HTML)
}

async fn serve_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        assets::STYLE_CSS,
    )
}

async fn serve_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        assets::APP_JS,
    )
}

async fn serve_xterm_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        assets::XTERM_JS,
    )
}

async fn serve_xterm_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        assets::XTERM_CSS,
    )
}

async fn serve_xterm_fit_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        assets::XTERM_FIT_JS,
    )
}

async fn serve_marked_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        assets::MARKED_JS,
    )
}
