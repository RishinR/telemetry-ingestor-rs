use crate::app::{AppState, SignalKind};
use crate::db::postgres::{insert_filtered, insert_metrics, insert_raw, vessel_exists};
use crate::models::telemetry::{ParsedSignal, TelemetryRequest};
use axum::http::HeaderMap;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

pub async fn ingest_telemetry(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<TelemetryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let total_start = Instant::now();

    // Extract vessel_id
    let vessel_id: String = payload.vesselId.clone();

    // Parse timestamp
    let ts = payload
        .parse_timestamp()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid timestampUTC".to_string()))?;

    // Validate vessel exists
    let validation_start = Instant::now();
    let exists = vessel_exists(&state.pool, &vessel_id)
        .await
        .map_err(internal_err)?;
    if !exists {
        return Err((
            StatusCode::FORBIDDEN,
            "Unknown or inactive vessel".to_string(),
        ));
    }

    // Validate and categorize signals
    let mut valid_signals: Vec<ParsedSignal> = Vec::new();

    for (name, value) in payload.signals.iter() {
        let kind_opt = state.signal_registry.get(name).copied();

        match kind_opt {
            Some(kind) => {
                match (kind, value) {
                    // Digital signals: integers 0 or 1 only
                    (SignalKind::Digital, serde_json::Value::Number(n)) => {
                        // Accept signed or unsigned integers only
                        let is_int = n.is_i64() || n.is_u64();
                        if is_int {
                            let v = if let Some(i) = n.as_i64() {
                                i
                            } else if let Some(u) = n.as_u64() {
                                u as i64
                            } else {
                                -1
                            };
                            if v == 0 || v == 1 {
                                let val_f = if v == 1 { 1.0 } else { 0.0 };
                                valid_signals.push(ParsedSignal { name, value: val_f });
                            } else {
                                let _ = insert_filtered(
                                    &state.pool,
                                    &vessel_id,
                                    ts,
                                    name,
                                    v as f64,
                                    "out_of_range",
                                )
                                .await
                                .map_err(internal_err)?;
                            }
                        } else {
                            let _ = insert_filtered(
                                &state.pool,
                                &vessel_id,
                                ts,
                                name,
                                f64::NAN,
                                "type_mismatch",
                            )
                            .await
                            .map_err(internal_err)?;
                        }
                    }
                    // Analog signals: floats 1.0..=65535.0 only
                    (SignalKind::Analog, serde_json::Value::Number(n)) if n.is_f64() => {
                        let val_f = n.as_f64().unwrap_or(f64::NAN);
                        if (val_f >= 1.0) && (val_f <= 65535.0) {
                            valid_signals.push(ParsedSignal { name, value: val_f });
                        } else {
                            let _ = insert_filtered(
                                &state.pool,
                                &vessel_id,
                                ts,
                                name,
                                val_f,
                                "out_of_range",
                            )
                            .await
                            .map_err(internal_err)?;
                        }
                    }
                    // Anything else is a type mismatch (e.g., strings, bools, or integer for analog)
                    _ => {
                        let _ = insert_filtered(
                            &state.pool,
                            &vessel_id,
                            ts,
                            name,
                            f64::NAN,
                            "type_mismatch",
                        )
                        .await
                        .map_err(internal_err)?;
                    }
                }
            }
            None => {
                // Unknown signal name; accept only numeric values for storage, else NaN
                let val_f = match value {
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
                    _ => f64::NAN,
                };
                let _ = insert_filtered(&state.pool, &vessel_id, ts, name, val_f, "unknown_signal")
                    .await
                    .map_err(internal_err)?;
            }
        }
    }

    let validation_ms = validation_start.elapsed().as_millis() as i64;

    // Insert valid signals into main_raw
    let ingestion_start = Instant::now();
    for sig in &valid_signals {
        insert_raw(&state.pool, &vessel_id, ts, sig.name, sig.value)
            .await
            .map_err(internal_err)?;
    }
    let ingestion_ms = ingestion_start.elapsed().as_millis() as i64;
    let total_ms = total_start.elapsed().as_millis() as i64;

    // Record metrics
    insert_metrics(
        &state.pool,
        &vessel_id,
        validation_ms,
        ingestion_ms,
        total_ms,
    )
    .await
    .map_err(internal_err)?;

    info!(
        vessel_id,
        validation_ms,
        ingestion_ms,
        total_ms,
        valid_count = valid_signals.len(),
        "Telemetry ingested"
    );

    Ok(Json(json!({
        "ok": true,
        "vesselId": vessel_id,
        "validSignals": valid_signals.len(),
        "validationMs": validation_ms,
        "ingestionMs": ingestion_ms,
        "totalMs": total_ms,
    })))
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    warn!(error=%e, "Internal error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    )
}
