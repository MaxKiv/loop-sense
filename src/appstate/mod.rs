use std::sync::{Arc, Mutex};

use crate::controller::backend::mockloop_hardware::SensorData;
use crate::controller::mockloop_controller::ControllerSetpoint;

/// All shared state involved in http and microcontroller communication
#[derive(Debug, Clone)]
pub struct AppState {
    // Note: no mutex is required because the setpoint sender is only used in the setpoint POST router
    /// Sends the controller setpoints received over http to the microcontroller
    pub controller_setpoint: Arc<Mutex<ControllerSetpoint>>,

    /// Receives the sensor data from the microcontroller and sends it over http
    pub sensor_data: Arc<Mutex<SensorData>>,
}
