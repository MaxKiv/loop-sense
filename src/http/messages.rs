use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::experiment::Experiment;

#[derive(Serialize)]
pub struct HeartbeatMessage {
    status: &'static str,
    timestamp: DateTime<Utc>,
}

impl HeartbeatMessage {
    pub fn new() -> Self {
        Self {
            status: "alive",
            timestamp: Utc::now(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ExperimentList {
    experiments: Vec<Experiment>,
}
impl ExperimentList {
    pub fn new() -> Self {
        Self {
            experiments: Vec::new(),
        }
    }
}
