pub mod appstate;
pub mod controller;
pub mod hardware;

use appstate::{AppState, SensorData, Setpoint};
use axum::Json;

/// Allow GET requests to fetch latest sensor data over http
pub async fn get_data(state: axum::extract::State<AppState>) -> Json<SensorData> {
    let data = state.sensor_data.lock().unwrap().clone();
    Json(data)
}

/// Allow POST requests to update the mockloop controller setpoints over http
pub async fn update_control(
    state: axum::extract::State<AppState>,
    Json(payload): Json<Setpoint>,
) -> &'static str {
    let mut setpoint = state.setpoint.lock().unwrap();
    *setpoint = payload;
    "OK"
}
