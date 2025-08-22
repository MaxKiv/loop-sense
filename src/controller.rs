use love_letter::{Report, Setpoint};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::*;

use crate::{axumstate::AxumState, experiment::Experiment, messages::db_messages::DatabaseReport};

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
    mcu_report_receiver: Receiver<Report>,
    mcu_setpoint_sender: Sender<Setpoint>,
    experiment_receiver: Receiver<Experiment>,
    axum_state: AxumState,
    db_report_sender: Sender<DatabaseReport>,
) {
    // Send sensor data received from the microcontroller to the database task to be logged
    info!("micro comms sending: {:?}", data.clone());
    if let Err(err) = db_sender.send(data).await {
        error!("micro comms send error: {:?}", err);
    }
}
