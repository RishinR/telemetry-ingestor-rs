use crate::app::AppState;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tracing::warn;

pub async fn auth_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Expect format: "Bearer <API_TOKEN>"
    let token = auth_header.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        warn!("Unauthorized request: missing Bearer token");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    if token != state.cfg.api_token {
        warn!("Unauthorized request: invalid API token");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    next.run(req).await
}
