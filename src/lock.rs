use chrono::{Duration, Utc};
use sqlx::SqlitePool;

pub async fn try_acquire_lock(
    pool: &SqlitePool,
    job_name: &str,
    instance_id: &str,
    lease_seconds: i64,
) -> anyhow::Result<bool> {
    let now = Utc::now();
    let lock_until = now + Duration::seconds(lease_seconds);

    let result = sqlx::query(
        r#"
        INSERT INTO job_lock (job_name, locked_until, locked_by)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(job_name) DO UPDATE SET
            locked_until = excluded.locked_until,
            locked_by = excluded.locked_by
        WHERE job_lock.locked_until <= ?4
        "#,
    )
    .bind(job_name)
    .bind(lock_until.to_rfc3339())
    .bind(instance_id)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn release_lock(
    pool: &SqlitePool,
    job_name: &str,
    instance_id: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        DELETE FROM job_lock
        WHERE job_name = ?1 AND locked_by = ?2
        "#,
    )
    .bind(job_name)
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}
