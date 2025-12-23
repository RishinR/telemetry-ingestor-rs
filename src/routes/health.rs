use crate::app::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

pub async fn healthz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();

    if db_ok {
        (StatusCode::OK, Json(json!({ "status": "ok", "db": "up" })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "degraded", "db": "down" })),
        )
    }
}
