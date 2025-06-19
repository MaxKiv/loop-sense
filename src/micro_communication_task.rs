use loop_sense::appstate::AppState;
use loop_sense::communicator::mockloop_communicator::MockloopCommunicator;
use loop_sense::controller::backend::mockloop_hardware::SensorData;
use tokio::sync::mpsc::Sender;
use tokio::time::{self, Duration, Instant};
use tracing::{error, info, warn};

const COMMS_LOOP_PERIOD: Duration = Duration::from_millis(100);

pub async fn communicate_with_micro<C: MockloopCommunicator>(
    mut communicator: C,
    state: AppState,
    db_sender: Sender<SensorData>,
) {
    let mut next_tick_time = Instant::now() + COMMS_LOOP_PERIOD;
    loop {
        // TODO: use tokio select instead of awaiting communicator sequentially?

        // read latest setpoint and forward to hardware
        let setpoint = state
            .controller_setpoint
            .lock()
            .expect("Unable to lock controller_setpoint mutex in communicate_with_micro")
            .clone();
        communicator.send_setpoint(setpoint).await;

        // receive from hardware and forward to controller
        let data = communicator.receive_data().await;
        if let Ok(mut sensor_data) = state.sensor_data.lock() {
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
