use loop_sense::appstate::AppState;
use loop_sense::communicator::mockloop_communicator::MockloopCommunicator;
use loop_sense::controller::backend::mockloop_hardware::SensorData;
use love_letter::{Report, Setpoint};
use tokio::sync::mpsc::Sender;
use tokio::time::{self, Duration, Instant};
use tracing::{error, info, warn};

const COMMS_LOOP_PERIOD: Duration = Duration::from_millis(100);

pub async fn communicate_with_micro<C: MockloopCommunicator>(
    mut communicator: C,
    setpoint_receiver: Receiver<Setpoint>,
    report_sender: Sender<Report>,
) {
    let mut next_tick_time = Instant::now() + COMMS_LOOP_PERIOD;
    loop {
        // Note: Don't use tokio select instead of awaiting communicator sequentially
        // This cause problems when we are halfway through sending and our receiving future
        // completes, causing our sending future to drop.
        // If we really want this sending/receiving should be in seperate tasks

        // Read latest setpoint and forward to hardware
        let setpoint = state
            .setpoint
            .lock()
            .expect("Unable to lock controller_setpoint mutex in communicate_with_micro")
            .clone();
        communicator.send_setpoint(setpoint).await;

        // Receive measurements from hardware and forward to controller
        let data = communicator.receive_data().await;
        if let Ok(mut sensor_data) = state.report.lock() {
            *sensor_data = data.clone()
        }

        // Send sensor data received from the microcontroller to the database task to be logged
        info!("micro comms sending: {:?}", data.clone());
        if let Err(err) = db_sender.send(data).await {
            error!("micro comms send error: {:?}", err);
        }

        next_tick_time += COMMS_LOOP_PERIOD;
        time::sleep_until(next_tick_time).await;
    }
}
