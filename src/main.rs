mod config;
mod db;
mod job;
mod scheduler;
mod state;
mod worker;

use clap::Parser;
use std::fs;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "jobs.yaml")]
    config: String,

    #[arg(long, default_value_t = 4)]
    max_concurrency: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let contents = fs::read_to_string(args.config)?;
    let config: config::Config = serde_yaml_ng::from_str(&contents)?;

    let jobs = config
        .jobs
        .into_iter()
        .map(job::Job::new)
        .collect::<Result<Vec<_>, _>>()?;

    // Db as a state store
    let pool = db::init_db("sqlite:scheduler.db").await?;

    // Semaphore to limit concurrency
    let semaphore = Arc::new(Semaphore::new(args.max_concurrency));
    scheduler::start_scheduler(jobs, pool.clone(), semaphore.clone()).await;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down gracefully...");

    Ok(())
}
