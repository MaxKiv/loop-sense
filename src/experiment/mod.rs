pub mod manage;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
    duration_seconds: i64,
}

impl From<&Experiment> for ExperimentStatus {
    fn from(exp: &Experiment) -> Self {
        // Calculate duration dynamically based on current time
        let duration = if exp.is_running {
            Utc::now().signed_duration_since(exp.start_time)
        } else {
            exp.duration_seconds
        };
        
        Self {
            is_running: exp.is_running,
            experiment_id: exp.id,
            experiment_name: exp.name.clone(),
            description: exp.description.clone(),
            table_name: exp.table_name.clone(),
            start_time: exp.start_time,
            duration_seconds: duration.num_seconds(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperimentStartMessage {
    name: String,
    description: String,
}
