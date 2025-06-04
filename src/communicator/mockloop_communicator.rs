use crate::controller::{
    backend::mockloop_hardware::SensorData, mockloop_controller::ControllerSetpoint,
};

#[async_trait::async_trait]
pub trait MockloopCommunicator: Send + Sync {
    async fn receive_data(&mut self) -> SensorData;
    async fn send_setpoint(&mut self, setpoint: ControllerSetpoint);
}
