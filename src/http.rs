use axum::Json;
use hyper::StatusCode;
use tracing::{error, info, warn};

use crate::{
    axumstate::AxumState,
    experiment::{self},
};

/// Allow GET requests to fetch latest sensor data over http
pub async fn get_data(state: axum::extract::State<AxumState>) -> Json<SensorData> {
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
    state: axum::extract::State<AxumState>,
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

#[axum::debug_handler]
pub async fn get_experiment_status(
    state: axum::extract::State<AxumState>,
) -> Result<&'static str, StatusCode> {
    if let Ok(status) = state.0.experiment_status.lock() {
        Json(status.clone());
        Ok("OK")
    } else {
        error!("Unable to fetch the current experiment status");
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[axum::debug_handler]
pub async fn start_experiment(
    state: axum::extract::State<AxumState>,
    Json(start_message): Json<experiment::ExperimentStartMessage>,
) -> &'static str {
    state.experiment_watch.send(Some(start_message.clone()));
    "OK"
}

#[axum::debug_handler]
pub async fn stop_experiment(
    state: axum::extract::State<AxumState>,
    Json(start_message): Json<experiment::ExperimentStartMessage>,
) -> &'static str {
    state.experiment_watch.send(Some(start_message.clone()));
    "OK"
}
