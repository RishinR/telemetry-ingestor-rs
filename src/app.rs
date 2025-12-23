use crate::middleware::auth::auth_middleware;
use crate::{config::Config, routes::health::healthz, routes::telemetry::ingest_telemetry};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
    pub pool: PgPool,
    pub signal_registry: HashMap<String, SignalKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    Digital,
    Analog,
}

pub fn build_router(
    cfg: Config,
    pool: PgPool,
    signal_registry: HashMap<String, SignalKind>,
) -> Router {
    let state = AppState {
        cfg: cfg.clone(),
        pool,
        signal_registry,
    };

    // Public routes (no auth)
    let public = Router::new().route("/healthz", get(healthz));

    // Protected routes (with auth)
    let protected = Router::new()
        .route("/api/v1/telemetry", post(ingest_telemetry))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    public.merge(protected).with_state(state)
}
