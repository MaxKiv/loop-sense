use loop_sense::appstate::{AppState, SensorData};
use tokio::time::{self, Duration};

const COMMUNICATION_PERIOD: Duration = Duration::from_millis(100);

pub async fn communicate_with_micro(state: AppState) {
    let mut interval = time::interval(COMMUNICATION_PERIOD);
    loop {
        interval.tick().await;
        send_setpoint(state.clone()).await;
        receive_data(state.clone()).await;
    }
}

async fn send_setpoint(state: AppState) {
    let setpoint = state.setpoint.lock().unwrap();
    println!("send state to micro: {:?}", setpoint);
}

async fn receive_data(state: AppState) {
    let mut data = state.sensor_data.lock().unwrap();
    *data = SensorData::simulate();
    println!("received data from micro: {:?}", data);
}
