use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{error, info};

use crate::job::Job;
use crate::{lock, state, worker};

pub async fn start_scheduler(jobs: Vec<Job>, pool: PgPool, semaphore: Arc<Semaphore>) {
    for job in jobs {
        let pool_clone = pool.clone();
        let sem_clone = semaphore.clone();

        tokio::spawn(schedule_job(job, pool_clone, sem_clone));
    }
}

async fn schedule_job(job: Job, pool: PgPool, semaphore: Arc<Semaphore>) {
    loop {
        if let Some(next) = job.next_run() {
            // Persist next_run state
            if let Err(e) = state::upsert_job(
                &pool,
                &job.config.name,
                state::JobStatus::Idle,
                None,
                Some(next),
                0,
                None,
            )
            .await
            {
                error!("Failed to persist next_run: {}", e);
            }

            let now = Utc::now();
            let duration = next - now;

            let sleep_duration = duration.to_std().unwrap_or(Duration::from_secs(1));

            sleep(sleep_duration).await;

            // Try distributed lock
            let lock_tx = match lock::try_acquire_lock(&pool, &job.config.name).await {
                Ok(Some(tx)) => tx,
                Ok(None) => {
                    info!("Another instance holds lock for {}", job.config.name);
                    continue;
                }
                Err(e) => {
                    error!("Lock error: {}", e);
                    continue;
                }
            };

            // Acquire worker pool permit BEFORE spawning
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => continue,
            };

            let job_clone = job.clone();
            let pool_clone = pool.clone();

            tokio::spawn(async move {
                let _permit = permit; // auto-release on drop

                // keep transaction alive while job runs
                let mut tx = lock_tx;

                worker::execute_job(job_clone.clone(), pool_clone.clone()).await;

                // Release lock explicitly
                let key = lock::lock_key(&job_clone.config.name);
                let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
                    .bind(key)
                    .execute(&mut *tx)
                    .await;

                let _ = tx.commit().await;
            });
        }
    }
}
