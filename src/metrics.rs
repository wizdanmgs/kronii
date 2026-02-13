use once_cell::sync::Lazy;
use prometheus::{Encoder, HistogramVec, IntCounterVec, IntGauge, Registry, TextEncoder};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static JOB_EXECUTIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        prometheus::Opts::new("job_executions_total", "Total job executions"),
        &["job_name"][..],
    )
    .unwrap()
});

pub static JOB_FAILURES: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        prometheus::Opts::new("job_failures_total", "Total job failures"),
        &["job_name"][..],
    )
    .unwrap()
});

pub static JOB_RUNNING: Lazy<IntGauge> =
    Lazy::new(|| IntGauge::new("jobs_running", "Number of currently running jobs").unwrap());

pub static JOB_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        prometheus::HistogramOpts::new("job_duration_seconds", "Job execution duration in seconds"),
        &["job_name"][..],
    )
    .unwrap()
});

pub fn init_metrics() {
    REGISTRY.register(Box::new(JOB_EXECUTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(JOB_FAILURES.clone())).unwrap();
    REGISTRY.register(Box::new(JOB_RUNNING.clone())).unwrap();
    REGISTRY.register(Box::new(JOB_DURATION.clone())).unwrap();
}

pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
