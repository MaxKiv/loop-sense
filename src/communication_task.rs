use loop_sense::appstate::AppState;
use loop_sense::communicator::mockloop_communicator::MockloopCommunicator;
use tokio::time::{self, Duration, Instant};

const COMMS_LOOP_PERIOD_MS: u64 = 100;

pub async fn communicate_with_micro<C: MockloopCommunicator>(mut communicator: C, state: AppState) {
    let mut next_tick_time = Instant::now() + Duration::from_millis(COMMS_LOOP_PERIOD_MS);
    loop {
        // TODO: use tokio select instead of awaiting communicator sequentially

        // read latest setpoint and forward to hardware
        let setpoint = state
            .controller_setpoint
            .lock()
            .expect("Unable to lock controller_setpoint mutex in communicate_with_micro")
            .clone();
        communicator.send_setpoint(setpoint).await;

        // receive from hardware and forward to channel
        let data = communicator.receive_data().await;
        if let Ok(mut sensor_data) = state.sensor_data.lock() {
            *sensor_data = data
        }

        next_tick_time += Duration::from_millis(COMMS_LOOP_PERIOD_MS);
        time::sleep_until(next_tick_time).await;
    }
}
