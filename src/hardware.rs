use chrono::{DateTime, Utc};
use rand::{random_bool, random_range};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uom::si::{
    f32::{Pressure, VolumeRate},
    pressure::bar,
    volume_rate::liter_per_minute,
};

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
enum ValveState {
    #[default]
    Closed,

    Open,
}

impl ValveState {
    fn from_bool(b: bool) -> Self {
        match b {
            true => Self::Open,
            false => Self::Closed,
        }
    }
}

/// Sensor data from the micro controlling the Mockloop
#[derive(Debug, Default, Clone, Serialize)]
pub struct SensorData {
    experiment_time: DateTime<Utc>,
    pressure_systemic_preload: Pressure,
    pressure_systemic_afterload: Pressure,
    controller_regulator_actual_pressure: Pressure,
    systemic_flow: VolumeRate,
    pulmonary_flow: VolumeRate,
    controller_valve_left: ValveState,
    controller_valve_right: ValveState,
}

impl SensorData {
    pub fn simulate() -> Self {
        let now = Utc::now();
        SensorData {
            experiment_time: now,
            pressure_systemic_preload: Pressure::new::<bar>(random_range(0.0..=1.0)),
            pressure_systemic_afterload: Pressure::new::<bar>(random_range(0.0..=1.0)),
            controller_regulator_actual_pressure: Pressure::new::<bar>(random_range(0.0..=1.0)),
            systemic_flow: VolumeRate::new::<liter_per_minute>(random_range(0.0..=60.0)),
            pulmonary_flow: VolumeRate::new::<liter_per_minute>(random_range(0.0..=60.0)),
            controller_valve_left: ValveState::from_bool(random_bool(0.5)),
            controller_valve_right: ValveState::from_bool(random_bool(0.5)),
        }
    }
}

/// Actuator setpoints for the micro controlling the Mockloop
#[derive(Debug, Clone, Deserialize)]
pub struct ActuatorSetpoint {
    pub controller_valve_left: ValveState,
    pub controller_valve_right: ValveState,
    pub controller_pressure_regulator: Pressure,
}

impl Default for ActuatorSetpoint {
    fn default() -> Self {
        Self {
            controller_valve_left: Default::default(),
            controller_valve_right: Default::default(),
            controller_pressure_regulator: Pressure::new::<bar>(0.0),
        }
    }
}

impl ActuatorSetpoint {
    fn get_safe() -> Self {
        ActuatorSetpoint::default()
    }
}

#[derive(Error, Debug)]
pub enum MockloopHardwareError {
    #[error("Failed to communicate with the mockloop microcontroller: {0}")]
    CommunicationError(String),
}

pub trait MockloopHardware {
    fn initialize(&mut self) -> Result<(), MockloopHardwareError>;

    fn set_actuators(&mut self, setpoints: ActuatorSetpoint) -> Result<(), MockloopHardwareError>;

    fn set_regulator_pressure(&mut self, pressure: Pressure) -> Result<(), MockloopHardwareError>;

    fn read_sensors(&mut self) -> Result<SensorData, MockloopHardwareError>;

    fn to_safe_state(&mut self) -> Result<(), MockloopHardwareError> {
        self.set_actuators(ActuatorSetpoint::get_safe())
    }
}
