use loop_sense::{appstate::AppState, controller::MockloopController};

pub async fn control_loop(state: AppState) {
    let controller = NidaqController();

    controller.run();
}
