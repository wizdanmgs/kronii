use chrono::Utc;
use sqlx::SqlitePool;
use tokio::time::{Duration, sleep};

use crate::job::Job;
use crate::state;
use crate::worker::execute_job;

pub async fn start_scheduler(jobs: Vec<Job>, pool: SqlitePool) {
    for job in jobs {
        let pool_clone = pool.clone();
        tokio::spawn(schedule_job(job, pool_clone));
    }
}

async fn schedule_job(job: Job, pool: SqlitePool) {
    loop {
        if let Some(next) = job.next_run() {
            state::upsert_job(
                &pool,
                &job.config.name,
                state::JobStatus::Idle,
                None,
                Some(next),
                0,
                None,
            )
            .await
            .ok();

            let now = Utc::now();
            let duration = next - now;

            let sleep_duration = duration.to_std().unwrap_or(Duration::from_secs(1));

            sleep(sleep_duration).await;

            let job_clone = job.clone();
            let pool_clone = pool.clone();

            tokio::spawn(async move {
                execute_job(job_clone, pool_clone).await;
            });
        }
    }
}
