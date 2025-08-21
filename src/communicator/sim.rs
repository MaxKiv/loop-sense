use std::time::Duration;

use tokio::time::sleep;
use tracing::info;

use crate::controller::{backend::mockloop_hardware::SensorData, mockloop_controller::Setpoint};

use super::mockloop_communicator::MockloopCommunicator;

pub struct SimulatedCommunicator;

#[async_trait::async_trait]
impl MockloopCommunicator for SimulatedCommunicator {
    async fn receive_data(&mut self) -> SensorData {
        sleep(Duration::from_millis(10)).await;
        let out = SensorData::simulate();
        info!("Simulated communicator received data: {:?}", out);
        out
    }

    async fn send_setpoint(&mut self, setpoint: Setpoint) {
        sleep(Duration::from_millis(10)).await;
        info!("Simulated communicator received setpoint: {:?}", setpoint);
    }
}
