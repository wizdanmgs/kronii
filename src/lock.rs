use sqlx::{PgPool, Postgres, Transaction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Convert job name into i64 lock key
pub fn lock_key(job_name: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    job_name.hash(&mut hasher);
    hasher.finish() as i64
}

pub async fn try_acquire_lock<'c>(
    pool: &PgPool,
    job_name: &str,
) -> anyhow::Result<Option<Transaction<'c, Postgres>>> {
    let mut tx = pool.begin().await?;

    let key = lock_key(job_name);

    let row: (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
        .bind(key)
        .fetch_one(&mut *tx)
        .await?;

    if row.0 {
        Ok(Some(tx)) // lock acquired, keep transaction alive
    } else {
        tx.rollback().await?;
        Ok(None)
    }
}
