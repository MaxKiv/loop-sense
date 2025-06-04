use appstate::AppState;
use axum::Json;
use controller::ControllerSetpoint;
use hardware::SensorData;
use tracing::{error, warn};

/// Allow GET requests to fetch latest sensor data over http
pub async fn get_data(mut state: axum::extract::State<AppState>) -> Json<SensorData> {
    if let Some(sensor_data) = state.sensor_data_receiver.recv().await {
        Json(sensor_data)
    } else {
        // TODO: proper error handling, for now we just spit back garbage
        error!("shit hit the fan @ sensordata router");
        Json(SensorData::simulate())
    }
}

/// Allow POST requests to update the mockloop controller setpoints over http
pub async fn post_setpoint(
    state: axum::extract::State<AppState>,
    Json(payload): Json<ControllerSetpoint>,
) -> &'static str {
    state
        .controller_setpoint_sender
        .send(payload.clone())
        .expect("unable to send controller setpoint received from http through channel");

    if payload.enable {
        warn!("ENABLE control");
    } else {
        warn!("DISABLE control");
    }
    "OK"
}
