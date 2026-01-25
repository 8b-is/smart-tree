//! Axum HTTP server for the web dashboard

use super::{api, assets, websocket, DashboardState, SharedState};
use crate::in_memory_logger::InMemoryLogStore;
use anyhow::Result;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use ipnet::IpNet;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Allowed networks for connection filtering
#[derive(Clone)]
struct AllowedNetworks {
    networks: Vec<IpNet>,
    allow_all: bool,
}

impl AllowedNetworks {
    fn new(cidrs: Vec<String>) -> Self {
        if cidrs.is_empty() {
            // Default: localhost only
            return Self {
                networks: vec!["127.0.0.0/8".parse().unwrap(), "::1/128".parse().unwrap()],
                allow_all: false,
            };
        }

        let mut networks = Vec::new();
        let mut allow_all = false;

        for cidr in cidrs {
            if cidr == "0.0.0.0/0" || cidr == "::/0" || cidr == "any" {
                allow_all = true;
                break;
            }
            if let Ok(net) = cidr.parse::<IpNet>() {
                networks.push(net);
            } else {
                eprintln!("Warning: Invalid CIDR '{}', ignoring", cidr);
            }
        }

        Self {
            networks,
            allow_all,
        }
    }

    fn is_allowed(&self, ip: IpAddr) -> bool {
        if self.allow_all {
            return true;
        }
        // Always allow localhost
        if ip.is_loopback() {
            return true;
        }
        self.networks.iter().any(|net| net.contains(&ip))
    }
}

/// Middleware to check if client IP is allowed
async fn check_allowed_network(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let allowed = req
        .extensions()
        .get::<AllowedNetworks>()
        .map(|nets| nets.is_allowed(addr.ip()))
        .unwrap_or(true);

    if allowed {
        Ok(next.run(req).await)
    } else {
        eprintln!("Rejected connection from {}", addr.ip());
        Err(StatusCode::FORBIDDEN)
    }
}

/// Start the web dashboard server
pub async fn start_server(
    port: u16,
    open_browser: bool,
    allow_networks: Vec<String>,
    log_store: InMemoryLogStore,
) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let state: SharedState = Arc::new(RwLock::new(DashboardState::new(cwd, log_store)));

    let has_explicit_networks = !allow_networks.is_empty();
    let allowed = AllowedNetworks::new(allow_networks.clone());
    let bind_all = has_explicit_networks || allowed.allow_all;

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
        .route("/api/logs", get(api::get_logs))
        // Config endpoints
        .route(
            "/api/config/layout",
            get(api::get_layout_config).post(api::save_layout_config),
        )
        .route(
            "/api/config/theme",
            get(api::get_theme_config).post(api::save_theme_config),
        )
        // WebSocket endpoints
        .route("/ws/terminal", get(websocket::terminal_handler))
        .layer(axum::Extension(allowed.clone()))
        .with_state(state);

    // Bind to all interfaces if networks specified, localhost otherwise
    let bind_addr: IpAddr = if bind_all {
        [0, 0, 0, 0].into()
    } else {
        [127, 0, 0, 1].into()
    };
    let addr = SocketAddr::from((bind_addr, port));

    println!("\x1b[32m");
    println!("  ╔══════════════════════════════════════════════════════╗");
    println!("  ║        Smart Tree Web Dashboard                      ║");
    println!("  ╠══════════════════════════════════════════════════════╣");
    if bind_all {
        println!(
            "  ║  http://0.0.0.0:{}                                ║",
            port
        );
        println!("  ║                                                      ║");
        println!("  ║  Allowed networks:                                   ║");
        if allowed.allow_all {
            println!("  ║    ANY (0.0.0.0/0)                                   ║");
        } else {
            for net in &allowed.networks {
                if !net.addr().is_loopback() {
                    println!("  ║    {}                                       ║", net);
                }
            }
        }
    } else {
        println!(
            "  ║  http://127.0.0.1:{}                              ║",
            port
        );
        println!("  ║                                                      ║");
        println!("  ║  Localhost only (use --allow for network access)     ║");
    }
    println!("  ║                                                      ║");
    println!("  ║  Terminal: Real PTY with bash/zsh                    ║");
    println!("  ║  Files: Browse and edit                              ║");
    println!("  ║  Preview: Markdown rendering                         ║");
    println!("  ╚══════════════════════════════════════════════════════╝");
    println!("\x1b[0m");

    if open_browser {
        let url = format!("http://127.0.0.1:{}", port);
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// Static asset handlers
async fn serve_index() -> Html<&'static str> {
    Html(assets::INDEX_HTML)
}

async fn serve_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], assets::STYLE_CSS)
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
    ([(header::CONTENT_TYPE, "text/css")], assets::XTERM_CSS)
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
