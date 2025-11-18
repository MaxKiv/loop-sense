use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

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

/// Response format for listing experiments from the database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentListFromDB {
    pub experiments: Vec<ExperimentFromDB>,
}

/// Individual experiment details from the database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentFromDB {
    pub table_name: String,
    pub experiment_id: String,
    pub experiment_name: String,
    pub description: String,
    pub start_time: Option<String>,
    pub duration_seconds: f64,
}
