use chrono::Duration;
use love_letter::{Report, Setpoint};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{self, timeout},
};
use tracing::*;

use crate::communicator::MockloopCommunicator;

/// Defines the frequency at which mcu communication takes place
const COMMS_LOOP_PERIOD: Duration = Duration::from_millis(10);
/// Bounds the maximum duration between consecutive setpoints / reports
const COMMS_TIMEOUT: Duration = Duration::from_millis(5);

pub async fn communicate_with_micro<C: MockloopCommunicator>(
    mut communicator: C,
    setpoint_receiver: Receiver<Setpoint>,
    report_sender: Sender<Report>,
) {
    let mut ticker = time::interval(COMMS_LOOP_PERIOD);

    loop {
        // Note: Don't use tokio select instead of awaiting communicator sequentially
        // This cause problems when we are halfway through sending and our receiving future
        // completes, causing our sending future to drop.
        // If we really want this sending/receiving should be in seperate tasks

        // Receive latest mcu setpoint from the controller task and send to the mcu
        match timeout(COMMS_TIMEOUT, setpoint_receiver.recv()).await {
            Ok(setpoint) => {
                if let Err(err) = timeout(COMMS_TIMEOUT, communicator.send_setpoint(setpoint)).await
                {
                    error!("timeout sending setpoint to mcu: {err}");
                }
            }
            Err(err) => {
                error!("timeout receiving setpoint from controller task: {err}");
            }
        }

        // Receive latest report from mcu and forward to controller task
        match timeout(COMMS_TIMEOUT, communicator.receive_report()).await {
            Ok(report) => {
                if let Err(err) = timeout(COMMS_TIMEOUT, report_sender.send(report)).await {
                    error!("timeout sending report to controller task: {err}");
                }
            }
            Err(err) => {
                error!("timeout receiving report from mcu: {err}");
            }
        }

        ticker.tick().await;
    }
}
