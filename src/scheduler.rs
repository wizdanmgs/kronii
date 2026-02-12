use chrono::Utc;
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::job::Job;
use crate::worker::execute_job;

pub async fn start_scheduler(jobs: Vec<Job>) {
    for job in jobs {
        tokio::spawn(schedule_job(job));
    }
}

async fn schedule_job(job: Job) {
    loop {
        if let Some(next) = job.next_run() {
            let now = Utc::now();
            let duration = next - now;

            let sleep_duration = duration
                .to_std()
                .unwrap_or(Duration::from_secs(1));

            sleep(sleep_duration).await;

            info!("Triggering job: {}", job.config.name);

            let job_clone = job.clone();
            tokio::spawn(async move {
                execute_job(job_clone).await;
            });
        }
    }
}

