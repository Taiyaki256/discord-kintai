use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub admin_role_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable is required"))?;

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:attendance.db".to_string());

        let admin_role_id = env::var("ADMIN_ROLE_ID").ok();

        Ok(Config {
            discord_token,
            database_url,
            admin_role_id,
        })
    }
}
