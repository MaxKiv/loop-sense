mod communication_task;
mod control_loop_task;

use axum::{
    Router,
    routing::{get, post},
};
use communication_task::communicate_with_micro;
use control_loop_task::control_loop;
use loop_sense::{appstate::AppState, get_data, update_control};
use tokio::task;

/// Application & Tokio executor entrypoint
#[tokio::main]
async fn main() {
    let state = AppState::default();

    // Set up Axum routers
    let app = Router::new()
        // GET router to access new sensor data over http
        .route("/data", get(get_data))
        // POST router to update control setpoints over http
        .route("/control", post(update_control))
        // Give the routers access to the application state
        .with_state(state.clone());

    // Delegate all microcontroller communication to a separate tokio task
    task::spawn(communicate_with_micro(state.clone()));
    // Start the control loop
    task::spawn(control_loop(state.clone()));

    // Start serving requests
    println!("Axum Router & communication task initialised");
    println!("Listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
