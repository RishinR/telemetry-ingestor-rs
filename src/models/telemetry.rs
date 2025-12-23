use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct TelemetryRequest {
    pub vesselId: String,
    pub timestampUTC: String,
    #[serde(default)]
    pub epochUTC: Option<i64>,
    pub signals: HashMap<String, serde_json::Value>,
}

impl TelemetryRequest {
    pub fn parse_timestamp(&self) -> anyhow::Result<DateTime<Utc>> {
        let ts = DateTime::parse_from_rfc3339(&self.timestampUTC)?.with_timezone(&Utc);
        Ok(ts)
    }
}

#[derive(Debug)]
pub struct ParsedSignal<'a> {
    pub name: &'a str,
    pub value: f64,
}
