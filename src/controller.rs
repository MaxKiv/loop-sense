use love_letter::{Report, Setpoint};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        watch,
    },
    time::{Duration, timeout},
};
use tracing::*;

use crate::{
    axumstate::AxumState,
    experiment::Experiment,
    messages::{db_messages::DatabaseReport, frontend_messages},
};

/// Bounds the maximum duration between consecutive setpoints / reports
const COMMS_TIMEOUT: Duration = Duration::from_millis(5);
const CONTROL_LOOP_PERIOD: Duration = Duration::from_millis(10);

/// High level control loop for the HHH SBC, responsible for:
/// * Parsing received MCU reports
///     - Calculating cardiac output
///     - Sending report to DB task
///     - Populating Appstate::latest_report
/// * Constructing setpoints for the MCU / low level controller
///     -
/// * Reacting to experiment changes
///     - Starting heart controller on new experiment
///     - Stopping heart controller on end/stop of experiment
pub async fn control_loop(
    mut mcu_report_receiver: Receiver<Report>,
    mut mcu_setpoint_sender: Sender<Setpoint>,
    mut experiment_receiver: watch::Receiver<Option<Experiment>>,
    axum_state: AxumState,
    mut db_report_sender: Sender<DatabaseReport>,
) {
    let mut ticker = tokio::time::interval(CONTROL_LOOP_PERIOD);

    let mut current_experiment = None;

    loop {
        // Did the experiment change?
        if experiment_receiver.has_changed().unwrap_or(false) {
            // Ask the experiment manager for the current experiment status
            current_experiment = Some(experiment_receiver.borrow_and_update());

            // TODO: new experiment started, enable loop/heart controller...
        }

        // Check for new setpoint from frontend
        if let Ok(guard) = axum_state.setpoint.lock() {
            if let Some(frontend_setpoint) = *guard {
                // Construct setpoint for MCU
                let mcu_setpoint: love_letter::Setpoint = frontend_setpoint.into();

                // Send setpoint to MCU
                if let Err(err) =
                    timeout(COMMS_TIMEOUT, mcu_setpoint_sender.send(mcu_setpoint)).await
                {
                    error!("timeout sending setpoint to mcu comms task: {err}");
                }
            }
        }

        // Parse MCU report
        match timeout(COMMS_TIMEOUT, mcu_report_receiver.recv()).await {
            Ok(Some(mcu_report)) => {
                let db_report: DatabaseReport = mcu_report.into();

                // Write latest report to db
                db_report_sender.send(db_report);

                // Update /measurement endpoint with latest report
                if let Ok(mut axum_report) = axum_state.report.lock() {
                    *axum_report = Some(frontend_messages::Report::from(mcu_report));
                }
            }
            Err(err) => {
                error!("timeout waiting on report from mcu comms task: {err}");
            }
        }

        ticker.tick().await;
    }
}
