mod communication_task;
mod control_loop_task;

use axum::{
    Router,
    routing::{get, post},
};
use communication_task::communicate_with_micro;
use control_loop_task::control_loop;
use loop_sense::{
    appstate::AppState,
    communicator::passthrough::PassThroughCommunicator,
    controller::{backend::mockloop_hardware::SensorData, mockloop_controller::ControllerSetpoint},
    http::{get_data, post_setpoint},
};
use std::sync::{Arc, Mutex};
use tokio::task;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

/// Application & Tokio executor entrypoint
#[tokio::main]
async fn main() {
    // Initialize application state
    // We are using tokio channels/watches to communicate between tasks
    let initial_setpoint = ControllerSetpoint::default();
    let initial_data = SensorData::default();

    let state = AppState {
        controller_setpoint: Arc::new(Mutex::new(initial_setpoint)),
        sensor_data: Arc::new(Mutex::new(initial_data)),
    };

    // Set up tracing logs
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    // Set up Axum routers
    let app = Router::new()
        // GET router to access new sensor data over http
        .route("/data", get(get_data))
        // POST router to update control setpoints over http
        .route("/setpoint", post(post_setpoint))
        // Give the routers access to the application state
        .with_state(state.clone());

    // Delegate all microcontroller communication to a separate tokio task
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    task::spawn(communicate_with_micro(communicator, state.clone()));

    // Start the control loop in a separate task
    task::spawn(control_loop(receiver));

    // Start serving requests
    info!("Axum Router & communication task initialised");
    info!("Listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
