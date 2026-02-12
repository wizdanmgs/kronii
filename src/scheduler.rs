use chrono::Utc;
use tokio::time::{Duration, sleep};
use tracing::info;

use crate::job::Job;
use crate::state::SharedState;
use crate::worker::execute_job;

pub async fn start_scheduler(jobs: Vec<Job>, state: SharedState) {
    for job in jobs {
        let state_clone = state.clone();
        tokio::spawn(schedule_job(job, state_clone));
    }
}

async fn schedule_job(job: Job, state: SharedState) {
    loop {
        if let Some(next) = job.next_run() {
            {
                let mut write = state.write().await;
                if let Some(s) = write.get_mut(&job.config.name) {
                    s.next_run = Some(next);
                }
            }

            let now = Utc::now();
            let duration = next - now;

            let sleep_duration = duration.to_std().unwrap_or(Duration::from_secs(1));

            sleep(sleep_duration).await;

            info!("Triggering job: {}", job.config.name);

            let job_clone = job.clone();
            let state_clone = state.clone();
            
            tokio::spawn(async move {
                execute_job(job_clone, state_clone).await;
            });
        }
    }
}
