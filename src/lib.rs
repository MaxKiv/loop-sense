pub mod appstate;

use appstate::{AppState, SensorData, Setpoint};
use axum::Json;

/// Fetch latest micro sensor data
pub async fn get_data(state: axum::extract::State<AppState>) -> Json<SensorData> {
    let data = state.sensor_data.lock().unwrap().clone();
    Json(data)
}

/// Update the micro control setpoints
pub async fn update_control(
    state: axum::extract::State<AppState>,
    Json(payload): Json<Setpoint>,
) -> &'static str {
    let mut setpoint = state.setpoint.lock().unwrap();
    *setpoint = payload;
    "OK"
}
