mod config;
mod job;
mod scheduler;
mod worker;

use clap::Parser;
use tracing_subscriber::FmtSubscriber;
use std::fs;

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

    let jobs = config.jobs
        .into_iter()
        .map(job::Job::new)
        .collect::<Result<Vec<_>, _>>()?;

    scheduler::start_scheduler(jobs).await;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down gracefully...");

    Ok(())
}
