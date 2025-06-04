use loop_sense::{appstate::AppState, hardware::SensorData};
use tokio::time::{self, Duration};
use tracing::info;

const COMMUNICATION_PERIOD: Duration = Duration::from_millis(1000);
const SIMULATE: bool = true;

pub async fn communicate_with_micro(state: AppState) {
    let mut interval = time::interval(COMMUNICATION_PERIOD);
    loop {
        interval.tick().await;
        receive_data(state.clone()).await;
    }
}

async fn receive_data(state: AppState) {
    let mut data = state.sensor_data.lock().unwrap();
    if SIMULATE {
        *data = SensorData::simulate();
    } else {
        // *data =
    }
    info!("received data from micro: {:?}", data);
}
