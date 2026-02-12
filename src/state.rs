use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum JobStatus {
    Idle,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct JobState {
    pub name: String,
    pub status: JobStatus,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub last_error: Option<String>,
}

impl JobState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            status: JobStatus::Idle,
            last_run: None,
            next_run: None,
            attempts: 0,
            last_error: None,
        }
    }
}

pub type SharedState = Arc<RwLock<HashMap<String, JobState>>>;
