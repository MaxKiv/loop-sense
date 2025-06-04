use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;

use crate::controller::ControllerSetpoint;
use crate::hardware::SensorData;

/// All shared state involved in http and microcontroller communication
#[derive(Debug)]
pub struct AppState {
    // Note: no mutex is required because the setpoint sender is only used in the setpoint POST router
    /// Sends the controller setpoints received over http to the microcontroller
    pub controller_setpoint_sender: Arc<Sender<ControllerSetpoint>>,

    /// Receives the sensor data from the microcontroller and sends it over http
    pub sensor_data_receiver: Receiver<SensorData>,
}
