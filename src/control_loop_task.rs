use loop_sense::{appstate::AppState, controller::MockloopController, nidaq::Nidaq};
use tracing::info;

pub async fn control_loop(state: AppState) {
    let controller = MockloopController::new(Nidaq {});

    info!("Running controller");
    controller.run(state).await;
}
