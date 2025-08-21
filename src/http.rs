use axum::Json;
use tracing::{error, info, warn};

use crate::{
    appstate::AppState,
    controller::{backend::mockloop_hardware::SensorData, mockloop_controller::Setpoint},
};

/// Allow GET requests to fetch latest sensor data over http
pub async fn get_data(state: axum::extract::State<AppState>) -> Json<SensorData> {
    if let Ok(sensor_data) = state.sensor_data.lock() {
        Json(sensor_data.clone())
    } else {
        // TODO: proper error handling, for now we just spit back garbage
        error!("shit hit the fan @ sensordata router");
        Json(SensorData::simulate())
    }
}

/// Allow POST requests to update the mockloop controller setpoints over http
#[axum::debug_handler]
pub async fn post_setpoint(
    state: axum::extract::State<AppState>,
    Json(payload): Json<Setpoint>,
) -> &'static str {
    let test = Json(payload.clone());
    let serialized = serde_json::to_string(&test.0).unwrap();
    info!("JSON setpoint: {:?}", serialized);

    if let Ok(mut state) = state.controller_setpoint.lock() {
        *state = payload.clone();
    }

    if payload.enable {
        warn!("ENABLE control");
    } else {
        warn!("DISABLE control");
    }
    "OK"
}
