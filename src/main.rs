mod config;
mod db;
mod job;
mod lock;
mod metrics;
mod scheduler;
mod state;
mod worker;

use clap::Parser;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tracing_subscriber::FmtSubscriber;

use axum::{Router, routing::get};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "jobs.yaml")]
    config: String,

    #[arg(
        long,
        default_value = "postgres://postgres:postgres@localhost/scheduler"
    )]
    database_url: String,

    #[arg(long, default_value_t = 4)]
    max_concurrency: usize,

    #[arg(long, default_value_t = 3000)]
    metrics_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    // Init DB
    let pool = db::init_db(&args.database_url).await?;

    // Init metrics
    metrics::init_metrics();

    // Load config
    let contents = fs::read_to_string(&args.config)?;
    let config: config::Config = serde_yaml_ng::from_str(&contents)?;

    let jobs = config
        .jobs
        .into_iter()
        .map(job::Job::new)
        .collect::<Result<Vec<_>, _>>()?;

    // Worker pool
    let semaphore = Arc::new(Semaphore::new(args.max_concurrency));

    // Start scheduler
    scheduler::start_scheduler(jobs, pool.clone(), semaphore.clone()).await;

    // Start metrics server
    let metrics_port = args.metrics_port;
    tokio::spawn(async move {
        start_http_server(metrics_port).await;
    });

    println!("Scheduler running. Press CTRL+C to stop.");

    // Graceful shutdown
    tokio::signal::ctrl_c().await?;
    println!("Shutdown signal received. Exiting.");

    Ok(())
}

async fn metrics_handler() -> String {
    metrics::gather_metrics()
}

async fn start_http_server(port: u16) {
    let app = Router::new().route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind metrics port");

    println!("Metrics server running on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .unwrap();
}
