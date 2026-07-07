use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;

pub async fn create_pool() -> anyhow::Result<MySqlPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    Ok(pool)
}
