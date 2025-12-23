use axum::Router;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
mod app;
mod config;
mod db;
mod middleware;
mod models;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    // Load configuration
    dotenvy::dotenv().ok();
    let cfg = config::Config::from_env()?;
    info!(port = cfg.port, "Starting telemetry ingestor");

    // Create DB pool
    let pool = db::postgres::create_pool(&cfg.database_url).await?;

    // Preload signal registry into memory for fast validation
    let signal_registry = db::postgres::load_signal_registry(&pool).await?;
    info!(count = signal_registry.len(), "Loaded signal registry");

    // Build app router (in-process caching only)
    let app: Router = app::build_router(cfg.clone(), pool.clone(), signal_registry);

    // Bind address
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!(%addr, "Listening on");

    // Server with graceful shutdown
    let server = axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        error!(error = %e, "Server error");
    }

    info!("Shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
