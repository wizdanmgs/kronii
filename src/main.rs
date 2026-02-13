mod config;
mod db;
mod job;
mod metrics;
mod scheduler;
mod state;
mod worker;

use axum::{Router, routing::get};
use clap::Parser;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
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

    // Init metrics
    metrics::init_metrics();

    // Semaphore to limit concurrency
    let semaphore = Arc::new(Semaphore::new(args.max_concurrency));
    scheduler::start_scheduler(jobs, pool.clone(), semaphore.clone()).await;

    tokio::spawn(start_http_server());

    tokio::signal::ctrl_c().await?;
    println!("Shutting down gracefully...");

    Ok(())
}

async fn metrics_handler() -> String {
    metrics::gather_metrics()
}

async fn start_http_server() {
    let app = Router::new().route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    println!("Metrics server running on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
            println!("Shutting down metrics server...");
        })
        .await
        .expect("server error");
}
