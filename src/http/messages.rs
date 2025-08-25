use chrono::{DateTime, Utc};
use serde::Serialize;

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
