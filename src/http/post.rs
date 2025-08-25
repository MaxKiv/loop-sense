use crate::axumstate::AxumState;
use crate::experiment::{self};
use crate::messages::frontend_messages::{HeartControllerSetpoint, MockloopSetpoint};
use axum::Json;
use axum::http::StatusCode;
use tracing::*;

/// POST request handler to update the mockloop setpoints (hemodynamic resistance/compliance)
#[axum::debug_handler]
pub async fn post_loop_setpoint(
    state: axum::extract::State<AxumState>,
    Json(new_setpoint): Json<MockloopSetpoint>,
) -> StatusCode {
    // Attempt to lock mutex guarding the latest setpoint
    if let Ok(mut setpoint) = state.setpoint.lock() {
        // Update the latest setpoint to the one received
        info!(
            "POST updated mockloop controller setpoint to: {:?}",
            new_setpoint
        );

        setpoint.mockloop_setpoint = new_setpoint;
        return StatusCode::OK;
    }
    // Unable to lock mutex, or mutex was poisoned
    error!(
        "unable to update mockloop controller setpoint in post_loop_setpoint, mutex poisoned or unable to lock - Returning INTERNAL_SERVER_ERROR"
    );
    StatusCode::INTERNAL_SERVER_ERROR
}

/// POST request handler to update the mockloop setpoints (hemodynamic resistance/compliance)
#[axum::debug_handler]
pub async fn post_heart_setpoint(
    state: axum::extract::State<AxumState>,
    Json(new_setpoint): Json<HeartControllerSetpoint>,
) -> StatusCode {
    // Attempt to lock mutex guarding the latest setpoint
    if let Ok(mut setpoint) = state.setpoint.lock() {
        // Update the latest setpoint to the one received
        info!(
            "POST updated heart controller setpoint to: {:?}",
            &new_setpoint
        );

        setpoint.heart_controller_setpoint = new_setpoint;
        return StatusCode::OK;
    }
    // Unable to lock mutex, or mutex was poisoned
    error!(
        "unable to update heart controller setpoint in post_heart_setpoint, mutex poisoned or unable to lock - Returning INTERNAL_SERVER_ERROR"
    );
    StatusCode::INTERNAL_SERVER_ERROR
}

#[axum::debug_handler]
pub async fn post_start_experiment(
    state: axum::extract::State<AxumState>,
    Json(start_message): Json<experiment::ExperimentStartMessage>,
) -> StatusCode {
    state.experiment_watch.send(Some(start_message));
    StatusCode::OK
}

#[axum::debug_handler]
pub async fn post_stop_experiment(state: axum::extract::State<AxumState>) -> StatusCode {
    state.experiment_watch.send(None);
    StatusCode::OK
}
