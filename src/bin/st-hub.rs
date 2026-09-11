use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "st::hub=info".into()),
        )
        .init();
    st::hub::run(st::hub::HubConfig::parse()).await
}
