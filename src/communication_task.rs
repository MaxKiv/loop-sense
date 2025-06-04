use loop_sense::communicator::mockloop_communicator::MockloopCommunicator;
use loop_sense::controller::ControllerSetpoint;
use loop_sense::{appstate::AppState, hardware::SensorData};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tokio::time::{Duration, Instant};
use tracing::info;

pub async fn communicate_with_micro<C: MockloopCommunicator>(
    communicator: C,
    data_sender: Sender<SensorData>,
    setpoint_receiver: Receiver<ControllerSetpoint>,
) {
    let mut next_tick_time = Instant::now() + Duration::from_secs_f32(period);
    loop {
        // TODO: use tokio select instead of polling here

        // read latest setpoint and forward to hardware
        let setpoint = *setpoint_receiver.borrow();
        communicator.send_setpoint(setpoint).await;

        // receive from hardware and forward to channel
        let sensor_data = communicator.receive_data().await;
        data_sender.send(sensor_data).await.unwrap();

        next_tick_time += Duration::from_secs_f32(period);
        time::sleep_until(next_tick_time).await;
    }
}
