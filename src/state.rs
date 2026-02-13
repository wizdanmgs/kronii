use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub enum JobStatus {
    Idle,
    Running,
    Success,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &str {
        match self {
            JobStatus::Idle => "idle",
            JobStatus::Running => "running",
            JobStatus::Success => "success",
            JobStatus::Failed => "failed",
        }
    }
}

pub async fn upsert_job(
    pool: &PgPool,
    name: &str,
    status: JobStatus,
    last_run: Option<DateTime<Utc>>,
    next_run: Option<DateTime<Utc>>,
    attempts: u32,
    last_error: Option<String>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO job_state (name, status, last_run, next_run, attempts, last_error, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT(name) DO UPDATE SET
            status = excluded.status,
            last_run = excluded.last_run,
            next_run = excluded.next_run,
            attempts = excluded.attempts,
            last_error = excluded.last_error,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(name)
    .bind(status.as_str())
    .bind(last_run.map(|d| d.to_rfc3339()))
    .bind(next_run.map(|d| d.to_rfc3339()))
    .bind(attempts as i64)
    .bind(last_error)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}
