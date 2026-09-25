use love_letter::{Report, Setpoint};
use tokio::sync::{mpsc, watch};
use tokio::time::Duration;
use tokio::time::{self, timeout};
use tracing::*;

use crate::communicator::MockloopCommunicator;

/// Bounds the maximum duration between consecutive setpoints / reports
const COMMS_TIMEOUT: Duration = Duration::from_millis(2000);

pub async fn send_setpoints_to_mcu(mut setpoint_receiver: watch::Receiver<Setpoint>) {
    #[cfg(feature = "sim-mcu")]
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    #[cfg(not(feature = "sim-mcu"))]
    // Spin until uart connection is established
    let mut mcu_communicator = loop {
        use crate::communicator::uart::UartCommunicator;
        const UART_RETRY_DURATION: Duration = Duration::from_millis(500);

        match UartCommunicator::try_new() {
            Ok(c) => break c,
            Err(err) => {
                error!(
                    "Unable to open uart communicator: {err}, retrying in {}ms",
                    UART_RETRY_DURATION.as_millis()
                );
                time::sleep(UART_RETRY_DURATION).await;
            }
        }
    };

    loop {
        // Send latest setpoint to the mcu on change
        if let Ok(_) = setpoint_receiver.changed().await {
            let setpoint = setpoint_receiver.borrow().clone();
            if let Err(err) = timeout(COMMS_TIMEOUT, mcu_communicator.send_setpoint(setpoint)).await
            {
                error!("timeout sending setpoint to mcu: {err}");
            }
        } else {
            error!(
                "receive_reports_from_mcu: setpoint receiver watch closed,
                this indicates a fatal issue"
            );
            return;
        }
    }
}

pub async fn receive_reports_from_mcu(report_sender: mpsc::Sender<Report>) {
    #[cfg(feature = "sim-mcu")]
    let (communicator, receiver) = PassThroughCommunicator::new_with_receiver();
    #[cfg(not(feature = "sim-mcu"))]
    // Spin until uart connection is established
    let mut mcu_communicator = loop {
        use crate::communicator::uart::UartCommunicator;
        const UART_RETRY_DURATION: Duration = Duration::from_millis(500);

        match UartCommunicator::try_new() {
            Ok(c) => break c,
            Err(err) => {
                error!(
                    "Unable to open uart communicator: {err}, retrying in {}ms",
                    UART_RETRY_DURATION.as_millis()
                );
                time::sleep(UART_RETRY_DURATION).await;
            }
        }
    };

    loop {
        // Receive latest report from mcu and forward to controller task
        match timeout(COMMS_TIMEOUT, mcu_communicator.receive_report()).await {
            Ok(mcu_report) => {
                if let Err(err) = timeout(COMMS_TIMEOUT, report_sender.send(mcu_report)).await {
                    error!("timeout sending report to controller task: {err}");
                }
            }
            Err(err) => {
                error!("timeout receiving report from mcu: {err}");
            }
        }
    }
}
