use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

use crate::job::Job;

pub async fn execute_job(job: Job) {
    let mut attempts = 0;

    while attempts <= job.config.retries {
        attempts += 1;

        info!("Running job: {} (attempt {})", job.config.name, attempts);

        let result = timeout(
            Duration::from_secs(job.config.timeout_seconds),
            run_command(&job.config.command),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                info!("Job {} completed successfully", job.config.name);
                return;
            }
            Ok(Err(e)) => {
                error!("Job {} failed: {}", job.config.name, e);
            }
            Err(_) => {
                error!("Job {} timed out", job.config.name);
            }
        }
    }

    error!("Job {} exhausted retries", job.config.name);
}

async fn run_command(cmd: &str) -> anyhow::Result<()> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()?;

    let status = child.wait().await?;

    if !status.success() {
        anyhow::bail!("Command exited with failure");
    }

    Ok(())
}

