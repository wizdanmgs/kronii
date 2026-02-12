use chrono::Utc;
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use tracing::{error, info};

use crate::job::Job;
use crate::state::{JobStatus, SharedState};

pub async fn execute_job(job: Job, state: SharedState) {
    let mut attempts = 0;

    {
        let mut write = state.write().await;
        if let Some(s) = write.get_mut(&job.config.name) {
            s.status = JobStatus::Running;
            s.last_run = Some(Utc::now());
            s.attempts = 0;
            s.last_error = None;
        }
    }

    while attempts <= job.config.retries {
        attempts += 1;

        {
            let mut write = state.write().await;
            if let Some(s) = write.get_mut(&job.config.name) {
                s.attempts = attempts;
            }
        }

        info!("Running job: {} (attempt {})", job.config.name, attempts);

        let result = timeout(
            Duration::from_secs(job.config.timeout_seconds),
            run_command(&job.config.command),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                let mut write = state.write().await;
                if let Some(s) = write.get_mut(&job.config.name) {
                    s.status = JobStatus::Success;
                    info!("Job {} completed successfully", job.config.name);
                }
                return;
            }
            Ok(Err(e)) => {
                let mut write = state.write().await;
                if let Some(s) = write.get_mut(&job.config.name) {
                    s.last_error = Some(e.to_string());
                    error!("Job {} failed: {}", job.config.name, e);
                }
            }
            Err(_) => {
                let mut write = state.write().await;
                if let Some(s) = write.get_mut(&job.config.name) {
                    s.last_error = Some("Timeout".into());
                    error!("Job {} timed out", job.config.name);
                }
            }
        }
    }

    let mut write = state.write().await;
    if let Some(s) = write.get_mut(&job.config.name) {
        s.status = JobStatus::Failed;
        error!("Job {} exhausted retries", job.config.name);
    }
}

async fn run_command(cmd: &str) -> anyhow::Result<()> {
    let mut child = Command::new("sh").arg("-c").arg(cmd).spawn()?;

    let status = child.wait().await?;

    if !status.success() {
        anyhow::bail!("Command exited with failure");
    }

    Ok(())
}
