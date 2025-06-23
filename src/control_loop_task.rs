use anyhow::Result;
use loop_sense::controller::mockloop_controller::{ControllerSetpoint, MockloopController};
use tokio::sync::watch::Receiver;
use tracing::info;

#[cfg(feature = "sim")]
use loop_sense::controller::backend::sim::Sim;

#[cfg(feature = "nidaq")]
use loop_sense::controller::backend::nidaq::{
    PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL, PRESSURE_SYSTEMIC_PRELOAD_CHANNEL, PULMONARY_FLOW_CHANNEL,
    REGULATOR_ACTUAL_PRESSURE_CHANNEL, SYSTEMIC_FLOW_CHANNEL, VALVE_LEFT_CHANNEL,
    VALVE_RIGHT_CHANNEL,
};

#[cfg(feature = "nidaq")]
use loop_sense::nidaq::nidaq_sys::Nidaq;

pub async fn high_lvl_control_loop(setpoint_receiver: Receiver<ControllerSetpoint>) {
    #[cfg(feature = "sim")]
    let controller = initialize_simulated_controller(setpoint_receiver)
        .expect("should be able to initialize simulated controller");

    #[cfg(feature = "nidaq")]
    let controller = initialize_nidaq_controller(setpoint_receiver)
        .expect("should be able to initialize nidaq controller");

    info!("Running controller");
    controller.run().await;
}

#[cfg(feature = "sim")]
fn initialize_simulated_controller(
    setpoint_receiver: Receiver<ControllerSetpoint>,
) -> Result<MockloopController<Sim>> {
    Ok(MockloopController::new(Sim {}, setpoint_receiver))
}

#[cfg(feature = "nidaq")]
fn initialize_nidaq_controller(
    setpoint_receiver: Receiver<ControllerSetpoint>,
) -> Result<MockloopController<Nidaq>> {
    Ok(MockloopController::new(
        initialize_nidaq().expect("error initializing nidaq"),
        setpoint_receiver,
    ))
}

#[cfg(feature = "nidaq")]
fn initialize_nidaq() -> Result<Nidaq> {
    use loop_sense::nidaq::{
        channel::{AnalogInputChannel, AnalogOutputChannel, Channel, DigitalOutputChannel},
        nidaq_sys::NidaqBuilder,
    };

    let out = NidaqBuilder::new()
        .initialize()?
        .with_analog_input_channel(
            PRESSURE_SYSTEMIC_PRELOAD_CHANNEL,
            AnalogInputChannel {
                channel: Channel {
                    physical_channel: "ai0".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL,
            AnalogInputChannel {
                channel: Channel {
                    physical_channel: "ai1".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            REGULATOR_ACTUAL_PRESSURE_CHANNEL,
            AnalogInputChannel {
                channel: Channel {
                    physical_channel: "ai4".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            SYSTEMIC_FLOW_CHANNEL,
            AnalogInputChannel {
                channel: Channel {
                    physical_channel: "ai6".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_input_channel(
            PULMONARY_FLOW_CHANNEL,
            AnalogInputChannel {
                channel: Channel {
                    physical_channel: "ai7".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_analog_output_channel(
            REGULATOR_ACTUAL_PRESSURE_CHANNEL,
            AnalogOutputChannel {
                channel: Channel {
                    physical_channel: "ao0".to_string(),
                },
                min: 0.0,
                max: 10.0,
            },
        )?
        .with_digital_output_channel(
            VALVE_LEFT_CHANNEL,
            DigitalOutputChannel {
                channel: Channel {
                    physical_channel: "port0/line0".to_string(),
                },
            },
        )?
        .with_digital_output_channel(
            VALVE_RIGHT_CHANNEL,
            DigitalOutputChannel {
                channel: Channel {
                    physical_channel: "port0/line1".to_string(),
                },
            },
        )?
        .start()?;

    Ok(out)
}
