use crate::axumstate::AxumState;
use crate::db_communication_task::communicate_with_db;
use crate::experiment::manage::manage_experiments;
use crate::http::get::*;
use crate::http::post::*;
use crate::messages::frontend_messages;
use crate::micro_communication_task::communicate_with_micro;
use axum::Router;
use axum::routing::get;
use axum::routing::post;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time;
use tokio::time::Duration;
use tracing::*;
use tracing_subscriber::FmtSubscriber;

pub mod axumstate;
pub mod communicator;
pub mod controller;
pub mod database;
pub mod db_communication_task;
pub mod experiment;
pub mod http;
pub mod messages;
pub mod micro_communication_task;

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

    // Create communication channels between tasks
    let (db_report_sender, db_report_receiver) = tokio::sync::mpsc::channel(100);
    let (mcu_setpoint_sender, mcu_setpoint_receiver) = tokio::sync::mpsc::channel(100);
    let (mcu_report_sender, mcu_report_receiver) = tokio::sync::mpsc::channel(100);
    let (experiment_sender, experiment_receiver) = tokio::sync::watch::channel(None);
    let (experiment_started_sender, experiment_started_receiver) =
        tokio::sync::watch::channel(None);

    // Initialize application state
    let initial_setpoint: frontend_messages::Setpoint = love_letter::Setpoint::default().into();
    let initial_report = None;
    let initial_experiment_status = None;
    let state = AxumState {
        setpoint: Arc::new(Mutex::new(initial_setpoint)),
        report: Arc::new(Mutex::new(initial_report)),
        experiment_status: Arc::new(Mutex::new(initial_experiment_status)),
        experiment_watch: experiment_started_sender,
        experiments: Arc::new(Mutex::new(Vec::new())),
    };

    #[cfg(feature = "sim-mcu")]
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    #[cfg(not(feature = "sim-mcu"))]
    // Spin until uart connection is established
    let mcu_communicator = loop {
        use crate::communicator::uart::UartCommunicator;

        match UartCommunicator::try_new() {
            Ok(c) => break c,
            Err(err) => {
                error!("Unable to open uart communicator, spinning...");
                time::sleep(Duration::from_millis(500)).await;
            }
        }
    };

    // Delegate all microcontroller communication to a separate tokio task
    task::spawn(communicate_with_micro(
        mcu_communicator,
        mcu_setpoint_receiver,
        mcu_report_sender,
    ));

    // Start the high level control loop, probably the most important routine of this application
    #[cfg(feature = "sim-mcu")]
    let handle = task::spawn(high_lvl_control_loop(state.clone()));
    #[cfg(not(feature = "sim-mcu"))]
    task::spawn(controller::control_loop(
        mcu_report_receiver,
        mcu_setpoint_sender,
        experiment_receiver,
        state.clone(),
        db_report_sender,
    ));

    // Start the experiment manager task
    task::spawn(manage_experiments(
        experiment_started_receiver,
        experiment_sender,
    ));

    // Start the DB communication task
    task::spawn(communicate_with_db(db_report_receiver));

    // Set up Axum routers
    let app = Router::new()
        // GET endpoints
        .route("/hearbeat", get(get_heartbeat))
        .route("/measurements", get(get_measurements))
        .route("/experiment/status", get(get_experiment_status))
        // POST endpoints
        .route("/control/loop", post(post_loop_setpoint))
        .route("/control/heart", post(post_heart_setpoint))
        .route("/experiment/list", post(get_list_experiment))
        .route("/experiment/start", post(post_start_experiment))
        .route("/experiment/stop", post(post_stop_experiment))
        // Give the routers access to the application state
        .with_state(state.clone());

    // Start serving webrequests
    info!("Axum Router & communication task initialised");
    info!("Listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
