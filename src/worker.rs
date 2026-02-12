use chrono::Utc;
use sqlx::SqlitePool;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use crate::job::Job;
use crate::state;

pub async fn execute_job(job: Job, pool: SqlitePool) {
    let mut attempts = 0;

    state::upsert_job(
        &pool,
        &job.config.name,
        state::JobStatus::Idle,
        Some(Utc::now()),
        None,
        0,
        None,
    )
    .await
    .ok();

    while attempts <= job.config.retries {
        attempts += 1;

        let result = timeout(
            Duration::from_secs(job.config.timeout_seconds),
            run_command(&job.config.command),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                state::upsert_job(
                    &pool,
                    &job.config.name,
                    state::JobStatus::Success,
                    Some(Utc::now()),
                    None,
                    attempts,
                    None,
                )
                .await
                .ok();

                return;
            }
            Ok(Err(e)) => {
                state::upsert_job(
                    &pool,
                    &job.config.name,
                    state::JobStatus::Running,
                    Some(Utc::now()),
                    None,
                    attempts,
                    Some(e.to_string()),
                )
                .await
                .ok();
            }
            Err(_) => {
                state::upsert_job(
                    &pool,
                    &job.config.name,
                    state::JobStatus::Running,
                    Some(Utc::now()),
                    None,
                    attempts,
                    Some("Timeout".into()),
                )
                .await
                .ok();
            }
        }
    }

    state::upsert_job(
        &pool,
        &job.config.name,
        state::JobStatus::Failed,
        Some(Utc::now()),
        None,
        attempts,
        Some("Retries exhausted".into()),
    )
    .await
    .ok();
}

async fn run_command(cmd: &str) -> anyhow::Result<()> {
    let mut child = Command::new("sh").arg("-c").arg(cmd).spawn()?;

    let status = child.wait().await?;

    if !status.success() {
        anyhow::bail!("Command exited with failure");
    }

    Ok(())
}
