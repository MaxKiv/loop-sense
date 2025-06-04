use std::sync::{Arc, Mutex};

use crate::hardware::SensorData;

/// All shared state involved in http and microcontroller communication
#[derive(Debug, Default, Clone)]
pub struct AppState {
    /// Latest sensor data received from the mockloop
    pub sensor_data: Arc<Mutex<SensorData>>,
    /// Should the mockloop pneumatic controller be enabled?
    pub enable_controller: Arc<Mutex<bool>>,
}
