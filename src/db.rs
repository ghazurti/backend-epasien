use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::env;
use std::time::Duration;

pub async fn init_pool() -> Pool<MySql> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(3)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}
