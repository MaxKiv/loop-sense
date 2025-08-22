use love_letter::{Report, Setpoint};
use tokio::sync::watch::{self, Receiver, Sender};

use crate::communicator::MockloopCommunicator;

pub struct PassThroughCommunicator {
    sender: Sender<love_letter::Setpoint>,
}

#[async_trait::async_trait]
impl MockloopCommunicator for PassThroughCommunicator {
    async fn receive_report(&mut self) -> love_letter::Report {
        // info!("Simulated communicator received data: {:?}", out);
        simulate_report()
    }

    async fn send_setpoint(&mut self, setpoint: Setpoint) {
        // info!("Passthrough communicator received setpoint: {:?}", setpoint);
        self.sender
            .send(setpoint)
            .expect("unable to send received setpoint to onboard controller");
    }
}

impl PassThroughCommunicator {
    pub fn new_with_receiver() -> (Self, Receiver<Setpoint>) {
        let initial_setpoint = Setpoint::default();

        let (tx_setpoint, rx_setpoint) = watch::channel(initial_setpoint);
        (
            PassThroughCommunicator {
                sender: tx_setpoint,
            },
            rx_setpoint,
        )
    }
}

pub fn simulate_report() -> Report {
    Report {
        app_state: (),
        measurements: (),
    }
}
