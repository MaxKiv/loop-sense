use loop_sense::controller::mockloop_controller::{ControllerSetpoint, MockloopController};
use tokio::sync::watch::Receiver;
use tracing::info;

#[cfg(feature = "sim")]
use loop_sense::controller::backend::sim::Sim;

#[cfg(feature = "nidaq")]
use loop_sense::controller::backend::nidaq::Nidaq;

pub async fn high_lvl_control_loop(setpoint_receiver: Receiver<ControllerSetpoint>) {
    #[cfg(feature = "sim")]
    let controller = MockloopController::new(Sim, setpoint_receiver);

    #[cfg(feature = "nidaq")]
    let controller = MockloopController::new(
        Nidaq::try_new().expect("unable to create new NIDAQ controller"),
        setpoint_receiver,
    );

    info!("Running controller");
    controller.run().await;
}
