use crate::axumstate::AxumState;
use crate::control::controller::control_loop;
use crate::database::db_communication_task::communicate_with_db;
use crate::experiment::manage::manage_experiments;
use crate::http::CONVEX_URI;
use crate::http::get::*;
use crate::http::messages::ExperimentList;
use crate::http::post::*;
use crate::http::ws::handle_websocket_request;
use crate::messages::frontend_messages;
use crate::micro_communication_task::communicate_with_micro;
use axum::Router;
use axum::routing::{any, get, post};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::*;
use tracing_subscriber::FmtSubscriber;

pub mod axumstate;
pub mod communicator;
pub mod control;
pub mod database;
pub mod experiment;
pub mod http;
pub mod messages;
pub mod micro_communication_task;

const PI_IP: &str = "192.168.0.4";
const FRONTEND_PORT_PROD: usize = 80;
const FRONTEND_PORT_DEV: usize = 5173;

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
    let (mcu_setpoint_sender, mcu_setpoint_receiver) =
        tokio::sync::watch::channel(love_letter::Setpoint::default());
    let (mcu_report_sender, mcu_report_receiver): (
        mpsc::Sender<love_letter::Report>,
        mpsc::Receiver<love_letter::Report>,
    ) = tokio::sync::mpsc::channel(10);
    let (experiment_sender, experiment_receiver) = tokio::sync::watch::channel(None);
    let (experiment_started_sender, experiment_started_receiver) =
        tokio::sync::watch::channel(None);

    // Initialize application state
    let initial_setpoint: frontend_messages::FrontendSetpoint =
        love_letter::Setpoint::default().into();

    let initial_report = None;
    let initial_experiment = None;
    let state = AxumState {
        setpoint: Arc::new(Mutex::new(initial_setpoint)),
        report: Arc::new(Mutex::new(initial_report)),
        current_experiment: Arc::new(Mutex::new(initial_experiment)),
        experiment_watch: experiment_started_sender,
        experiments: Arc::new(Mutex::new(ExperimentList::new())),
        start_time: Arc::new(Utc::now()),
    };

    // Delegate all microcontroller communication to a separate tokio task
    task::spawn(communicate_with_micro(
        mcu_setpoint_receiver,
        mcu_report_sender,
    ));

    // Start the high level control loop, probably the most important routine of this application
    #[cfg(feature = "sim-mcu")]
    let handle = task::spawn(high_lvl_control_loop(state.clone()));
    #[cfg(not(feature = "sim-mcu"))]
    task::spawn(control_loop(
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

    // Define CORS rules
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any);

    // Set up Axum routers
    let app = Router::new()
        // GET endpoints
        .route("/heartbeat", get(get_heartbeat))
        .route("/measurements", get(get_measurements))
        .route("/ws/measurements", any(handle_websocket_request))
        .route("/experiment/status", get(get_experiment_status))
        .route("/experiment/list", get(get_list_experiments_from_db))
        .route(
            "/experiment/download/{table_name}",
            get(download_experiment_csv),
        )
        // POST endpoints
        .route("/control/loop", post(post_loop_setpoint))
        .route("/control/heart", post(post_heart_setpoint))
        .route("/experiment/start", post(post_start_experiment))
        .route("/experiment/stop", post(post_stop_experiment))
        .layer(cors.clone()) // Attach CORS middleware
        .with_state(state.clone()); // Give the routers access to the application state

    // Start serving webrequests
    info!("Axum Router & communication task initialised");
    info!("Listening on http://localhost:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
