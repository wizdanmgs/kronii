use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

use crate::config::JobConfig;

#[derive(Clone)]
pub struct Job {
    pub config: JobConfig,
    pub schedule: Schedule,
}

impl Job {
    pub fn new(config: JobConfig) -> anyhow::Result<Self> {
        let schedule = Schedule::from_str(&config.schedule)?;
        Ok(Self { config, schedule })
    }

    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        self.schedule.upcoming(Utc).next()
    }
}
