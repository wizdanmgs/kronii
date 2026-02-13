use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, sleep};

use crate::job::Job;
use crate::worker::execute_job;
use crate::{lock, state};

pub async fn start_scheduler(
    jobs: Vec<Job>,
    pool: SqlitePool,
    semaphore: Arc<Semaphore>,
    instance_id: String,
) {
    for job in jobs {
        tokio::spawn(schedule_job(
            job,
            pool.clone(),
            semaphore.clone(),
            instance_id.clone(),
        ));
    }
}

async fn schedule_job(job: Job, pool: SqlitePool, semaphore: Arc<Semaphore>, instance_id: String) {
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

            // Attempt distributed lock
            let acquired = lock::try_acquire_lock(&pool, &job.config.name, &instance_id, 60)
                .await
                .unwrap_or(false);

            if !acquired {
                // Another instance holds the lock
                continue;
            }

            // Acquire worker permit before spawning
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let job_clone = job.clone();
            let pool_clone = pool.clone();
            let instance_clone = instance_id.clone();

            tokio::spawn(async move {
                let _permit = permit; // dropped automatically after spawn

                // Execute job
                execute_job(job_clone.clone(), pool_clone.clone()).await;

                // Early lock release
                lock::release_lock(&pool_clone, &job_clone.config.name, &instance_clone)
                    .await
                    .unwrap();
            });
        }
    }
}
