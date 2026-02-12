mod config;
mod job;
mod scheduler;
mod state;
mod worker;

use clap::Parser;
use std::fs;
use tracing_subscriber::FmtSubscriber;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use state::{SharedState, JobState};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "jobs.yaml")]
    config: String,
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

    let state: SharedState = Arc::new(RwLock::new(HashMap::new()));
    {
        let mut write = state.write().await;
        for job in &jobs {
            write.insert(job.config.name.clone(), JobState::new(job.config.name.clone()));
        }

    }

    scheduler::start_scheduler(jobs, state.clone()).await;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down gracefully...");

    Ok(())
}
