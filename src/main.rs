mod control_loop_task;
mod db_communication_task;
mod micro_communication_task;

use axum::{
    Router,
    routing::{get, post},
};
use control_loop_task::high_lvl_control_loop;
use db_communication_task::communicate_with_db;
use loop_sense::{
    appstate::AppState,
    communicator::{passthrough::PassThroughCommunicator, uart::UartCommunicator},
    controller::{backend::mockloop_hardware::SensorData, mockloop_controller::Setpoint},
    http::{get_data, post_setpoint},
};
use loop_sense::{
    controller::mockloop_controller::MockloopController,
    database::secrets::{DB_ACCESS_TOKEN, DB_NAME, DB_URI},
};
use micro_communication_task::communicate_with_micro;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use tokio::{task, time};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

/// Application & Tokio executor entrypoint
#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    // Initialize application state
    let initial_setpoint = Setpoint::default();
    let initial_data = SensorData::default();
    let state = AppState {
        setpoint: Arc::new(Mutex::new(initial_setpoint)),
        report: Arc::new(Mutex::new(initial_data)),
    };

    // Track async task handles to join them later
    let mut handles = Vec::new();

    #[cfg(not(feature = "stm32g4"))]
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    #[cfg(feature = "stm32g4")]
    // Spin until uart connection is established
    let communicator = loop {
        match UartCommunicator::try_new() {
            Ok(c) => break c,
            Err(err) => {
                error!("Unable to open uart communicator, spinning...");
                time::sleep(Duration::from_millis(500)).await;
            }
        }
    };

    // Create communication channels between tasks
    let (db_sender, db_receiver) = tokio::sync::mpsc::channel(100);

    // Delegate all microcontroller communication to a separate tokio task
    let handle = task::spawn(communicate_with_micro(
        communicator,
        state.clone(),
        db_sender,
    ));
    handles.push(handle);

    // Start the high level control loop in a separate task
    #[cfg(not(feature = "stm32g4"))]
    let handle = task::spawn(high_lvl_control_loop(receiver));
    #[cfg(feature = "stm32g4")]
    let handle = task::spawn(high_lvl_control_loop(state.clone()));

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
