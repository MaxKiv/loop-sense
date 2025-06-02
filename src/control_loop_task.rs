use loop_sense::appstate::AppState;
use tokio::time::{self, Duration};

const CONTROL_LOOP_PERIOD: Duration = Duration::from_millis(500);

pub async fn control_loop(state: AppState) {
    let mut interval = time::interval(CONTROL_LOOP_PERIOD);
    loop {
        interval.tick().await;
        println!("control looping")
    }
}
