use std::sync::{Arc, Mutex};

use crate::hardware::{ActuatorSetpoint, SensorData};

/// All state involved in http communication
/// NOTE: cloning a bunch of Arc is cheap
#[derive(Debug, Default, Clone)]
pub struct AppState {
    /// Latest sensor data from the mockloop
    pub sensor_data: Arc<Mutex<SensorData>>,
    /// Latest setpoint for the micro controlling the mockloop
    pub setpoint: Arc<Mutex<ActuatorSetpoint>>,
    /// Whether the high level mockloop controller should be running
    pub enable_controller: Arc<Mutex<bool>>,
}
