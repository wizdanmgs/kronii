mod config;
mod db;
mod job;
mod lock;
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
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "jobs.yaml")]
    config: String,

    #[arg(long, default_value_t = 4)]
    max_concurrency: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    // Load config
    let contents = fs::read_to_string(args.config)?;
    let config: config::Config = serde_yaml_ng::from_str(&contents)?;

    let jobs = config
        .jobs
        .into_iter()
        .map(job::Job::new)
        .collect::<Result<Vec<_>, _>>()?;

    // Init DB
    let pool = db::init_db("sqlite:scheduler.db").await?;

    // Instance ID for distributed locking
    let instance_id = Uuid::new_v4().to_string();
    println!("Instance ID: {}", instance_id);

    // Semaphore to limit concurrency
    let semaphore = Arc::new(Semaphore::new(args.max_concurrency));

    // Init metrics
    metrics::init_metrics();

    // Start scheduler
    scheduler::start_scheduler(jobs, pool.clone(), semaphore.clone(), instance_id.clone()).await;

    // Start metrics server
    tokio::spawn(start_http_server());

    // Graceful shutdown
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
