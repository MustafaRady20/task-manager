use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;

pub async fn create_pool() -> anyhow::Result<MySqlPool> {
    dotenvy::dotenv().ok();

    if std::env::var("DATABASE_URL").is_err() {
        if let Some(home) = dirs::home_dir() {
            let _ = dotenvy::from_path(home.join(".config/task-manager/.env"));
        }
    }

    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        anyhow::anyhow!(
            "DATABASE_URL not set. Set it in your shell profile or in ~/.config/task-manager/.env"
        )
    })?;
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    Ok(pool)
}
