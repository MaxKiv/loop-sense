use crate::experiment::{Experiment, ExperimentStatus};
use crate::{AxumState, http::messages::HeartbeatMessage, messages::frontend_messages::Report};
use axum::Json;
use axum::http::StatusCode;
use tracing::*;

/// Returns the latest measurement report from the mcu
#[axum::debug_handler]
pub async fn get_measurements(
    state: axum::extract::State<AxumState>,
) -> Result<Json<Report>, StatusCode> {
    if let Ok(guard) = state.report.lock() {
        if let Some(report) = *guard {
            // Return json-serialised report
            info!("GET measurements returning report: {:?}", report);
            return Ok(Json(report.clone()));
        } else {
            // No report was generated yet
            warn!("GET measurements attempted but no report in axum state");
            return Err(StatusCode::NO_CONTENT);
        }
    }

    // Unable to lock the axum state mutex_guard, shit definitely hit the fan
    error!(
        "Unable to lock the report mutex guard during GET measurements, returning INTERNAL_SERVER_ERROR and moving on with life..."
    );
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Return a heartbeat message
#[axum::debug_handler]
pub async fn get_heartbeat(_state: axum::extract::State<AxumState>) -> Json<HeartbeatMessage> {
    Json(HeartbeatMessage::new())
}

/// Return status of the currently running experiment
#[axum::debug_handler]
pub async fn get_experiment_status(
    state: axum::extract::State<AxumState>,
) -> Result<Json<ExperimentStatus>, StatusCode> {
    if let Ok(status) = state.0.experiment_status.lock() {
        if let Some(status) = status.clone() {
            Ok(Json(status.clone()))
        } else {
            return Err(StatusCode::NO_CONTENT);
        }
    } else {
        error!("Unable to fetch the current experiment status");
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Return all experiments
#[axum::debug_handler]
pub async fn get_list_experiment(
    state: axum::extract::State<AxumState>,
) -> Result<Json<Vec<Experiment>>, StatusCode> {
    let experiments = state.experiments.lock();

    if let Ok(experiments) = experiments {
        let experiments = experiments.clone();
        info!("Returning experiment list: {:?}", experiments);
        return Ok(Json(experiments));
    }

    warn!("GET experiment list returned nothing");
    return Err(StatusCode::NO_CONTENT);
}
