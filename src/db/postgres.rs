use crate::app::SignalKind;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn load_signal_registry(pool: &PgPool) -> Result<HashMap<String, SignalKind>> {
    let rows = sqlx::query(
        r#"
        SELECT signal_name, signal_type
        FROM signal_register_table
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::new();
    for r in rows {
        let signal_name: String = r.try_get("signal_name")?;
        let signal_type: String = r.try_get("signal_type")?;
        let kind = match signal_type.as_str() {
            "digital" => SignalKind::Digital,
            "analog" => SignalKind::Analog,
            _ => SignalKind::Analog,
        };
        map.insert(signal_name, kind);
    }
    Ok(map)
}

pub async fn vessel_exists(pool: &PgPool, vessel_id: &str) -> Result<bool> {
    let row = sqlx::query(
        r#"SELECT EXISTS(SELECT 1 FROM vessel_register_table WHERE vessel_id = $1 AND is_active = TRUE) as exists"#,
    )
    .bind(vessel_id)
    .fetch_one(pool)
    .await?;
    let exists: bool = row.try_get("exists")?;
    Ok(exists)
}

pub async fn insert_raw(
    pool: &PgPool,
    vessel_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    signal_name: &str,
    signal_value: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO main_raw (vessel_id, timestamp_utc, signal_name, signal_value)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(vessel_id)
    .bind(timestamp)
    .bind(signal_name)
    .bind(signal_value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_filtered(
    pool: &PgPool,
    vessel_id: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    signal_name: &str,
    signal_value: f64,
    reason: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO filtered_raw (vessel_id, timestamp_utc, signal_name, signal_value, reason)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(vessel_id)
    .bind(timestamp)
    .bind(signal_name)
    .bind(signal_value)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_metrics(
    pool: &PgPool,
    vessel_id: &str,
    validation_ms: i64,
    ingestion_ms: i64,
    total_ms: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO server_metrics (vessel_id, validation_ms, ingestion_ms, total_ms)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(vessel_id)
    .bind(validation_ms)
    .bind(ingestion_ms)
    .bind(total_ms)
    .execute(pool)
    .await?;
    Ok(())
}
