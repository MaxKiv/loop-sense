mod control_loop_task;
mod db_communication_task;
mod micro_communication_task;

use axum::{
    Router,
    routing::{get, post},
};
use control_loop_task::control_loop;
use db_communication_task::communicate_with_db;
use influxdb::Client;
use loop_sense::database::secrets::{DB_ACCESS_TOKEN, DB_NAME, DB_URI};
use loop_sense::{
    appstate::AppState,
    communicator::passthrough::PassThroughCommunicator,
    controller::{backend::mockloop_hardware::SensorData, mockloop_controller::ControllerSetpoint},
    http::{get_data, post_setpoint},
};
use micro_communication_task::communicate_with_micro;
use std::sync::{Arc, Mutex};
use tokio::task;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

/// Application & Tokio executor entrypoint
#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    // Initialize application state
    let initial_setpoint = ControllerSetpoint::default();
    let initial_data = SensorData::default();
    let state = AppState {
        controller_setpoint: Arc::new(Mutex::new(initial_setpoint)),
        sensor_data: Arc::new(Mutex::new(initial_data)),
    };

    let mut handles = Vec::new();

    // Delegate all microcontroller communication to a separate tokio task
    let (db_sender, db_receiver) = tokio::sync::mpsc::channel(100);
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    let handle = task::spawn(communicate_with_micro(
        communicator,
        state.clone(),
        db_sender,
    ));
    handles.push(handle);

    // Start the control loop in a separate task
    let handle = task::spawn(control_loop(receiver));
    handles.push(handle);

    // Start the DB communication task
    let handle = task::spawn(communicate_with_db(db_receiver));
    handles.push(handle);

    // Set up Axum routers
    let app = Router::new()
        // GET endpoint router to access new sensor data over http
        .route("/data", get(get_data))
        // POST endpoint router to update control setpoints over http
        .route("/setpoint", post(post_setpoint))
        // Give the routers access to the application state
        .with_state(state.clone());

    // Start serving webrequests
    info!("Axum Router & communication task initialised");
    info!("Listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // Join all tokio handles, this should never finish
    for handle in handles {
        let _ = handle.await;
        error!("Error, handle finished");
    }
}
