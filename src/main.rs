use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL was not found.");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_uri)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let mut conn = pool.acquire().await?;
    let _ = sqlx::query!("SELECT id FROM uploads")
        .fetch_all(&mut conn)
        .await?;

    Ok(())
}
