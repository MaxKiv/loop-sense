use std::sync::{Arc, Mutex};

/// All shared state involved in http (DB & frontend) communication
#[derive(Debug, Clone)]
pub struct AppState {
    // Setpoint received from the frontend
    pub setpoint: Arc<Mutex<Setpoint>>,

    ///
    pub report: Arc<Mutex<SensorData>>,
}
