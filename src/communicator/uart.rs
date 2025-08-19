use tracing::*;

use crate::controller::{
    backend::mockloop_hardware::SensorData, mockloop_controller::ControllerSetpoint,
};

use super::mockloop_communicator::MockloopCommunicator;

pub struct UartCommunicator {}

impl UartCommunicator {
    pub fn new() -> Self {
        UartCommunicator {}
    }
}

#[async_trait::async_trait]
impl MockloopCommunicator for UartCommunicator {
    async fn receive_data(&mut self) -> SensorData {
        // Receive data over uart
        let out = SensorData::simulate();
        info!("Simulated communicator received data: {:?}", out);
        out
    }

    async fn send_setpoint(&mut self, setpoint: ControllerSetpoint) {
        // Send setpoint over uart
        info!("Simulated communicator received setpoint: {:?}", setpoint);
    }
}
