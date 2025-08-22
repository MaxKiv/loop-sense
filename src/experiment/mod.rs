use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub struct Experiment {
    pub is_running: bool,
    pub experiment_id: Uuid,
    pub experiment_name: String,
    pub description: String,
    pub table_name: String,
    pub start_time: DateTime<Utc>,
    pub duration_seconds: Duration,
}
