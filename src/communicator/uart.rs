use love_letter::{Report, Setpoint};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, timeout};
use tokio_serial::{
    DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialPortInfo, SerialPortType,
    SerialStream, StopBits,
};
use tracing::*;

use anyhow::Result;

use crate::communicator::MockloopCommunicator;

const COMMS_TIMEOUT: Duration = Duration::from_millis(2000);
const UART_MANUFACTURER: &str = "Silicon Labs";

pub struct UartCommunicator {
    uart: SerialStream,
}

impl UartCommunicator {
    /// Attempt to construct a new UartCommunicator by configuring and opening a serial port
    pub fn try_new() -> Result<UartCommunicator> {
        // Uart manufacturer to match
        let uart_manufacturer = String::from(UART_MANUFACTURER);

        let ports = tokio_serial::available_ports()
            .map_err(|e| anyhow::anyhow!("No serial ports available: {e}"))?;

        info!("Found serial ports: {:?}", ports);

        // Connect to the first willing serial port that has matching uart manufacturer
        ports
            .into_iter()
            // Filter out devices of different manufacturers
            .filter(|p| match p {
                SerialPortInfo {
                    port_type:
                        SerialPortType::UsbPort(tokio_serial::UsbPortInfo {
                            manufacturer: Some(manufacturer),
                            ..
                        }),
                    ..
                } => *manufacturer == uart_manufacturer,
                _ => false,
            })
            // Attempt to open a serialport using the device
            .filter_map(|p| {
                tokio_serial::new(p.port_name.clone(), love_letter::BAUDRATE)
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
        let mut bytes = Vec::with_capacity(love_letter::REPORT_BYTES);
        loop {
            match timeout(COMMS_TIMEOUT, self.uart.read_u8()).await {
                Ok(Ok(byte)) => {
                    debug!("Received data byte: {}", byte);
                    if byte == 0 {
                        debug!("End of frame byte detected {}", byte);
                        // End of frame: deserialize into Report
                        if !bytes.is_empty() {
                            if let Ok(report) = love_letter::deserialize_report(&mut bytes) {
                                info!("Deserialized received bytes into report: {:?}", report);
                                return report;
                            } else {
                                error!("Failed to deserialize report: {:?}", bytes);
                                bytes.clear();
                            }
                        }
                    } else {
                        // Collect received data bytes
                        bytes.push(byte);
                    }
                }
                _ => {
                    // Idle: Reset buffer
                    bytes.clear();
                }
            }
        }
    }

    async fn send_setpoint(&mut self, setpoint: Setpoint) {
        // Send setpoint over uart
        let mut buf = [0u8; love_letter::SETPOINT_BYTES];
        info!("UART sending setpoint: {:?}", setpoint);

        match love_letter::serialize_setpoint(setpoint, &mut buf) {
            Err(err) => {
                error!("Unable to serialize setpoint: {err}, skipping...");
            }
            Ok(used) => {
                info!("UART sending serialised setpoint: {:?}", used);
                if let Err(err) = self.uart.write(used).await {
                    error!("Unable to write setpoint to UART: {err}");
                }
            }
        }
    }
}
