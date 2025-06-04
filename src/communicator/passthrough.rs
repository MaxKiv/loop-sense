use std::time::Duration;

use tokio::{
    sync::watch::{self, Receiver, Sender},
    time::sleep,
};
use tracing::info;

use crate::{controller::ControllerSetpoint, hardware::SensorData};

use super::mockloop_communicator::MockloopCommunicator;

pub struct PassThroughCommunicator {
    sender: Sender<ControllerSetpoint>,
}

#[async_trait::async_trait]
impl MockloopCommunicator for PassThroughCommunicator {
    async fn receive_data(&mut self) -> SensorData {
        let out = SensorData::simulate();
        info!("Simulated communicator received data: {:?}", out);
        out
    }

    async fn send_setpoint(&mut self, setpoint: ControllerSetpoint) {
        info!("Passthrough communicator received setpoint: {:?}", setpoint);
        self.sender
            .send(setpoint)
            .expect("unable to send received setpoint to onboard controller");
    }
}

impl PassThroughCommunicator {
    fn new_with_receiver() -> (Self, Receiver<ControllerSetpoint>) {
        let initial_setpoint = ControllerSetpoint::default();

        let (tx_setpoint, mut rx_setpoint) = watch::channel(initial_setpoint);
        (
            PassThroughCommunicator {
                sender: tx_setpoint,
            },
            rx_setpoint,
        )
    }
}
