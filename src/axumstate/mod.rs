use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use tokio::sync::watch::Sender;

use crate::{
    experiment::{Experiment, ExperimentStartMessage},
    http::messages::ExperimentList,
    messages::frontend_messages,
};

/// All shared state involved in http (DB & frontend) communication
#[derive(Debug, Clone)]
pub struct AxumState {
    // Latest setpoint received from the frontend
    pub setpoint: Arc<Mutex<frontend_messages::FrontendSetpoint>>,

    /// Latest report to expose to http
    pub report: Arc<Mutex<Option<frontend_messages::Report>>>,

    /// Notifies changes to the current experiment from the frontend experiment manager
    pub experiment_watch: Sender<Option<ExperimentStartMessage>>,

    /// Currently running experiment (stored to calculate duration dynamically)
    pub current_experiment: Arc<Mutex<Option<Experiment>>>,

    /// List of all experiments
    pub experiments: Arc<Mutex<ExperimentList>>,

    /// Time at which this application was started
    pub start_time: Arc<DateTime<Utc>>,
}
