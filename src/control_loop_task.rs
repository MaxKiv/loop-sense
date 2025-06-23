use anyhow::Result;
use loop_sense::controller::mockloop_controller::{ControllerSetpoint, MockloopController};
use tokio::sync::watch::Receiver;
use tracing::info;

#[cfg(feature = "sim")]
use loop_sense::controller::backend::sim::Sim;

#[cfg(feature = "nidaq")]
use loop_sense::controller::backend::nidaq::{
    channel::{AnalogInputChannel, AnalogOutputChannel, DigitalOutputChannel},
    nidaq_sys::{Nidaq, NidaqBuilder},
};

#[cfg(feature = "nidaq")]
use loop_sense::controller::backend::nidaq::Nidaq;

pub async fn high_lvl_control_loop(setpoint_receiver: Receiver<ControllerSetpoint>) {
    #[cfg(feature = "sim")]
    let controller = MockloopController::new(Sim, setpoint_receiver);

    #[cfg(feature = "nidaq")]
    let controller = MockloopController::new(
        initialize_nidaq().expect("error initializing nidaq"),
        setpoint_receiver,
    );

    info!("Running controller");
    controller.run().await;
}

#[cfg(feature = "nidaq")]
fn initialize_nidaq() -> Result<Nidaq> {
    let out = NidaqBuilder::new()
        .initialize()?
        .with_analog_input_channel(
            "pressure_systemic_preload",
            AnalogInputChannel {
                channel: "ai0",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            "pressure_systemic_afterload",
            AnalogInputChannel {
                channel: "ai0",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            "regulator_actual_pressure",
            AnalogInputChannel {
                channel: "ai4",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            "systemic_flow",
            AnalogInputChannel {
                channel: "ai6",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            "pulmonary_flow",
            AnalogInputChannel {
                channel: "ai7",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_output_channel(
            "regulator_set_pressure",
            AnalogOutputChannel {
                channel: "ao0",
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_digital_output_channel(
            "valve_left",
            DigitalOutputChannel {
                channel: "port0/line0",
            },
        )?
        .with_digital_output_channel(
            "valve_right",
            DigitalOutputChannel {
                channel: "port0/line1",
            },
        )?
        .start()?;

    Ok(out)
}
