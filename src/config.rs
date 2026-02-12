use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub jobs: Vec<JobConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JobConfig {
    pub name: String,
    pub schedule: String,
    pub command: String,
    pub retries: u32,
    pub timeout_seconds: u64,
}
