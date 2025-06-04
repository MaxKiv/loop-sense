pub mod appstate;
pub mod controller;
pub mod hardware;
pub mod nidaq;

use appstate::AppState;
use axum::Json;
use hardware::{ActuatorSetpoint, SensorData};
use tracing::warn;

/// Allow GET requests to fetch latest sensor data over http
pub async fn get_data(state: axum::extract::State<AppState>) -> Json<SensorData> {
    let data = state.sensor_data.lock().unwrap().clone();
    Json(data)
}

/// Allow POST requests to update the mockloop controller setpoints over http
pub async fn enable_control(
    state: axum::extract::State<AppState>,
    Json(payload): Json<bool>,
) -> &'static str {
    let mut enable = state.enable_controller.lock().unwrap();
    *enable = payload;
    if *enable {
        warn!("ENABLE control");
    } else {
        warn!("DISABLE control");
    }
    "OK"
}
