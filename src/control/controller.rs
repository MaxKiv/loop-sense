use crate::control::ControllerReport;
use chrono::TimeDelta;
use love_letter::{Report, Setpoint};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        watch,
    },
    time::{Duration, timeout},
};
use tracing::*;

use crate::{axumstate::AxumState, experiment::Experiment};

/// Bounds the maximum duration between consecutive setpoints / reports
const COMMS_TIMEOUT: Duration = Duration::from_millis(2000);
const CONTROL_LOOP_PERIOD: Duration = Duration::from_millis(10);

/// High level control loop for the HHH SBC, responsible for:
/// * Parsing received MCU reports
///     - Calculating cardiac output
///     - Sending report to DB task
///     - Populating Appstate::latest_report
/// * Constructing setpoints for the MCU / low level controller
///     -
pub async fn control_loop(
    mut mcu_report_receiver: mpsc::Receiver<Report>,
    mcu_setpoint_sender: watch::Sender<Setpoint>,
    mut experiment_receiver: watch::Receiver<Option<Experiment>>,
    axum_state: AxumState,
    db_report_sender: mpsc::Sender<ControllerReport>,
) {
    let mut ticker = tokio::time::interval(CONTROL_LOOP_PERIOD);

    let mut current_experiment: Option<Experiment> = None;

    loop {
        // Did the experiment change?
        if experiment_receiver.has_changed().unwrap_or(false) {
            // Ask the experiment manager for the current experiment status
            info!("Experiment change detected");
            current_experiment = (*experiment_receiver.borrow_and_update()).clone();
            
            // Update the experiment status in AxumState for the GET /experiment/status endpoint
            if let Ok(mut status) = axum_state.experiment_status.lock() {
                *status = current_experiment.as_ref().map(|exp| exp.into());
            } else {
                error!("Failed to update experiment status in AxumState");
            }
        }

        // Check for new setpoint from frontend
        if let Ok(frontend_setpoint) = axum_state.setpoint.lock() {
            // Construct setpoint for MCU
            let mcu_setpoint: love_letter::Setpoint = (*frontend_setpoint).clone().into();

            // Notify mcu communication task of the new mcu setpoint
            if let Err(err) = mcu_setpoint_sender.send(mcu_setpoint) {
                error!("unable to notify mcu communication task of new setpoint: {err}");
            }
        }

        // Parse MCU report received from the mcu communcation task
        match timeout(COMMS_TIMEOUT, mcu_report_receiver.recv()).await {
            Ok(Some(mcu_report)) => {
                info!("Received MCU report: {:?}", mcu_report.clone());

                // The MCU provides only an offset in micros since firwmare start:
                // Compute the right report timestamp
                let report_time = *axum_state.start_time
                    + TimeDelta::microseconds(
                        mcu_report
                            .measurements
                            .timestamp
                            .try_into()
                            .unwrap_or(i64::MAX), // Breaks after 17598506CE, should be ok :)
                    );
                let report = ControllerReport::from_mcu_report(
                    mcu_report,
                    report_time,
                    current_experiment.clone(),
                );
                info!("Exposing Controller Report to axum: {:?}", report.clone());

                // Update /measurement endpoint with latest report
                if let Ok(mut axum_report) = axum_state.report.lock() {
                    *axum_report = Some(report.clone().into());
                }

                // If an experiment is currently running: Update the DB
                if let Some(Experiment {
                    is_running,
                    ref id,
                    ref name,
                    ref duration_seconds,
                    ..
                }) = current_experiment
                {
                    info!("An Experiment is running: writing to DB");
                    if is_running {
                        info!(
                            "Experiment {id} - {name} is running for {}s, writing report to DB: {:?}",
                            duration_seconds.as_seconds_f32(),
                            report.clone()
                        );
                        // Write latest report to db
                        if let Err(err) = db_report_sender.send(report).await {
                            error!("Unable to send latest report to database task: {err}");
                        }
                    }
                } else {
                    info!("No experiment currently running, skipping DB write...");
                }
            }
            Ok(_) => {
                warn!("Received empty report from mcu communication task, skipping...");
            }
            Err(err) => {
                error!("timeout waiting on report from mcu comms task: {err}");
            }
        }

        ticker.tick().await;
    }
}
