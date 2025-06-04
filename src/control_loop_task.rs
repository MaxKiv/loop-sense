use loop_sense::{
    appstate::AppState,
    controller::{ControllerSetpoint, MockloopController},
};
use tokio::sync::watch::Receiver;
use tracing::info;

pub async fn control_loop(setpoint_receiver: Receiver<ControllerSetpoint>) {
    let controller = MockloopController::new(Nidaq {}, setpoint_receiver);

    info!("Running controller");
    controller.run().await;
}
