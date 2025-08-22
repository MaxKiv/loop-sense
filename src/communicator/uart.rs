use love_letter::{Report, Setpoint};
use tokio::io::AsyncWriteExt;
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialStream, StopBits};
use tracing::*;

use anyhow::Result;

use crate::communicator::MockloopCommunicator;

const BAUD_RATE: u32 = 115200;

pub struct UartCommunicator {
    uart: SerialStream,
}

impl UartCommunicator {
    /// Attempt to construct a new UartCommunicator by opening the first available serial port
    pub fn try_new() -> Result<UartCommunicator> {
        let ports = tokio_serial::available_ports()
            .map_err(|e| anyhow::anyhow!("No serial ports available: {e}"))?;

        info!("Found serial ports: {:?}", ports);

        // Connect to the first willing serial port
        ports
            .into_iter()
            .filter_map(|p| {
                tokio_serial::new(p.port_name.clone(), BAUD_RATE)
                    .flow_control(FlowControl::None)
                    .data_bits(DataBits::Eight)
                    .parity(Parity::None)
                    .stop_bits(StopBits::One)
                    .open_native_async()
                    .map(|mut uart| {
                        if let Err(e) = uart.set_exclusive(false) {
                            error!("Cannot set serial port un-exclusive: {e}");
                        }
                        UartCommunicator { uart }
                    })
                    .map_err(|err| {
                        error!("Cannot open serial port {:?}: {err}", p.port_name);
                        err
                    })
                    .ok()
            })
            .next()
            .ok_or_else(|| anyhow::anyhow!("No usable serial ports found"))
    }
}

#[async_trait::async_trait]
impl MockloopCommunicator for UartCommunicator {
    async fn receive_report(&mut self) -> Report {
        // Receive data over uart
        let mut buf = [0u8; 32];
        loop {
            match tokio::io::AsyncReadExt::read(&mut self.uart, &mut buf).await {
                Ok(data) => {
                    info!("Received {} data-bytes: {:?}", data, buf);

                    match love_letter::deserialize_report(&mut buf) {
                        Ok(_report) => {
                            // parse received data into SensorData
                            let out = todo!();
                            return out;
                        }
                        Err(err) => {
                            error!("Failed to deserialize report: {err} - report: {}", data);
                        }
                    }
                }
                Err(err) => {
                    error!("UART communication error: {err}");
                }
            };
        }
    }

    async fn send_setpoint(&mut self, setpoint: Setpoint) {
        // Send setpoint over uart
        let mut buf = [0u8; 32];
        info!("UART sending setpoint: {:?}", setpoint);

        love_letter::serialize_setpoint(setpoint, &mut buf);

        self.uart.write(&buf).await;
    }
}
