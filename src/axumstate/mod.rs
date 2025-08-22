use std::sync::{Arc, Mutex};

use crate::messages::frontend_messages;

/// All shared state involved in http (DB & frontend) communication
#[derive(Debug, Clone)]
pub struct AxumState {
    // Latest setpoint received from the frontend
    pub setpoint: Arc<Mutex<Option<frontend_messages::Setpoint>>>,

    /// Latest report to expose to http
    pub report: Arc<Mutex<Option<frontend_messages::Report>>>,

    ///
    pub experiment: Arc<Mutex<Option<Experiment>>>,
}
