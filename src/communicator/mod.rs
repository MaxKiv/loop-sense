use love_letter::{Report, Setpoint};

pub mod passthrough;
pub mod uart;

#[async_trait::async_trait]
pub trait MockloopCommunicator: Send + Sync {
    async fn receive_report(&mut self) -> Report;
    async fn send_setpoint(&mut self, setpoint: Setpoint);
}
