//! A self-hosted repository archive and retrieval hub.
//!
//! Git remains the archive format. T8R stores exact source passages and feedback;
//! native MEM8 stores recall vectors. SQLite provides a durable token index and
//! transactions for visibility, consent, and publication of index generations.

mod api;
mod indexing;
mod storage;

use anyhow::{ensure, Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Debug, Parser)]
#[command(
    name = "st-hub",
    about = "Smart Tree repository archive and recall hub"
)]
pub struct HubConfig {
    #[arg(long, env = "ST_HUB_BIND", default_value = "127.0.0.1:8428")]
    pub bind: SocketAddr,
    #[arg(long, env = "ST_HUB_STATE", default_value = "/var/lib/smart-tree-hub")]
    pub state_dir: PathBuf,
    #[arg(
        long,
        env = "ST_HUB_ARCHIVES",
        default_value = "/data/git/smart-tree-hub"
    )]
    pub archive_dir: PathBuf,
    /// Existing collections the administrator may import; never exposed as paths.
    #[arg(long, env = "ST_HUB_IMPORT_ROOTS", value_delimiter = ',')]
    pub import_roots: Vec<PathBuf>,
    /// File containing a private, randomly generated administrator token.
    #[arg(long, env = "ST_HUB_ADMIN_TOKEN_FILE")]
    pub admin_token_file: PathBuf,
    /// Optional local embedding endpoint; source text never goes to a cloud provider.
    #[arg(long, env = "ST_HUB_EMBED_URL")]
    pub embed_url: Option<String>,
    #[arg(long, env = "ST_HUB_ORIGIN", default_value = "https://8s.is")]
    pub origin: String,
    #[arg(long, env = "ST_HUB_MAX_FILES", default_value_t = 20000)]
    pub max_files: usize,
    #[arg(long, env = "ST_HUB_MAX_CHUNKS", default_value_t = 40000)]
    pub max_chunks: usize,
    /// Trusted reverse proxy only. The listener must remain on loopback.
    #[arg(long, env = "ST_HUB_TRUST_PROXY", default_value_t = false)]
    pub trust_proxy: bool,
}

pub(super) struct Hub {
    config: HubConfig,
    store: Mutex<storage::Store>,
    admin_hash: String,
    http: reqwest::Client,
    reads: Arc<tokio::sync::Semaphore>,
    git_reads: Arc<tokio::sync::Semaphore>,
    limits: Mutex<api::RateLimits>,
    stopping: std::sync::atomic::AtomicBool,
    latest_release: tokio::sync::Mutex<Option<(std::time::Instant, serde_json::Value)>>,
}

impl Hub {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, storage::Store>> {
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("Hub storage lock poisoned"))
    }
}

pub async fn run(config: HubConfig) -> Result<()> {
    ensure!(
        config.bind.ip().is_loopback(),
        "The hub must listen on loopback behind a TLS proxy"
    );
    ensure!(
        config.max_files > 0 && config.max_chunks > 0,
        "Index limits must be positive"
    );
    let token = std::fs::read_to_string(&config.admin_token_file)
        .context("Read administrator token file")?;
    ensure!(
        token.trim().len() >= 32,
        "Administrator token must contain at least 32 characters"
    );
    if let Some(url) = &config.embed_url {
        let url = reqwest::Url::parse(url)?;
        ensure!(
            url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost")),
            "Embedding service must be local"
        );
    }
    std::fs::create_dir_all(&config.archive_dir)?;
    let store = storage::Store::open(&config.state_dir)?;
    let hub = Arc::new(Hub {
        config: config.clone(),
        store: Mutex::new(store),
        admin_hash: storage::hash_token(token.trim()),
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?,
        reads: Arc::new(tokio::sync::Semaphore::new(8)),
        git_reads: Arc::new(tokio::sync::Semaphore::new(4)),
        limits: Mutex::new(api::RateLimits::default()),
        stopping: std::sync::atomic::AtomicBool::new(false),
        latest_release: tokio::sync::Mutex::new(None),
    });
    let worker = tokio::spawn(indexing::worker(hub.clone()));
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "Smart Tree hub listening");
    let shutdown_hub = hub.clone();
    axum::serve(
        listener,
        api::router(hub).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        #[cfg(unix)]
        {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut signal) => {
                    tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = signal.recv() => {} }
                }
                Err(error) => {
                    tracing::error!(%error,"Install termination handler");
                    let _ = tokio::signal::ctrl_c().await;
                }
            }
        }
        #[cfg(not(unix))]
        let _ = tokio::signal::ctrl_c().await;
        shutdown_hub
            .stopping
            .store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .await?;
    worker.abort();
    Ok(())
}
