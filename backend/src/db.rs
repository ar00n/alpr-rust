use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init_db() -> Result<sqlx::SqlitePool, Box<dyn std::error::Error>> {
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://anpr.db?mode=rwc".to_string());

    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let db = SqlitePoolOptions::new().connect_with(options).await?;

    sqlx::migrate!().run(&db).await?;

    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&db)
        .await?;

    if admin_count.0 == 0 {
        let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST)?;
        sqlx::query("INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)")
            .bind("admin")
            .bind(hash)
            .bind(true)
            .execute(&db)
            .await?;
        tracing::info!("✅ Created default admin user (username: admin, password: admin)");
    }

    Ok(db)
}
