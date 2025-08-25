use std::sync::{Arc, Mutex};

use tokio::sync::watch::Sender;

use crate::{
    experiment::{Experiment, ExperimentStartMessage, ExperimentStatus},
    messages::frontend_messages,
};

/// All shared state involved in http (DB & frontend) communication
#[derive(Debug, Clone)]
pub struct AxumState {
    // Latest setpoint received from the frontend
    pub setpoint: Arc<Mutex<frontend_messages::Setpoint>>,

    /// Latest report to expose to http
    pub report: Arc<Mutex<Option<frontend_messages::Report>>>,

    /// Notifies changes to the current experiment from the frontend experiment manager
    pub experiment_watch: Sender<Option<ExperimentStartMessage>>,

    /// Status of the currently running experiment
    pub experiment_status: Arc<Mutex<Option<ExperimentStatus>>>,

    /// List of all experiments
    pub experiments: Arc<Mutex<Vec<Experiment>>>,
}
