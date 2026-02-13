use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_db(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS job_state (
            name TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            last_run TEXT,
            next_run TEXT,
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
