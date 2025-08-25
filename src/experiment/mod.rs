pub mod manage;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Experiment {
    pub is_running: bool,
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub table_name: String,
    pub start_time: DateTime<Utc>,
    pub duration_seconds: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentStatus {
    is_running: bool,
    experiment_id: Uuid,
    experiment_name: String,
    description: String,
    table_name: String,
    start_time: DateTime<Utc>,
    duration_seconds: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperimentStartMessage {
    name: String,
    description: String,
}
