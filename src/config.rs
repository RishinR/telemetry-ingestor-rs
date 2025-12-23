use anyhow::{anyhow, Context, Result};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub api_token: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;
        let api_token =
            env::var("API_TOKEN").context("API_TOKEN environment variable is required")?;
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        if api_token.trim().is_empty() {
            return Err(anyhow!("API_TOKEN must not be empty"));
        }

        Ok(Self {
            database_url,
            api_token,
            port,
        })
    }
}
