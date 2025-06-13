use chrono::{DateTime, Utc};
use rand::random_range;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uom::si::{
    f64::{Pressure, VolumeRate},
    pressure::bar,
    volume_rate::liter_per_minute,
};

#[derive(Debug)]
pub enum Valve {
    Left,
    Right,
}

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum ValveState {
    #[default]
    Closed,

    Open,
}

/// Sensor data from the micro controlling the Mockloop
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorData {
    pub experiment_time: DateTime<Utc>,
    pub pressure_systemic_preload: Pressure,
    pub pressure_systemic_afterload: Pressure,
    pub controller_regulator_actual_pressure: Pressure,
    pub systemic_flow: VolumeRate,
    pub pulmonary_flow: VolumeRate,
    // controller_valve_left: ValveState,
    // controller_valve_right: ValveState,
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
            // controller_valve_left: ValveState::from_bool(random_bool(0.5)),
            // controller_valve_right: ValveState::from_bool(random_bool(0.5)),
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

#[derive(Debug)]
pub struct MockloopHardwareError(pub String);

pub trait MockloopHardware: Debug {
    fn initialize(&mut self) -> Result<(), MockloopHardwareError>;

    fn set_regulator_pressure(&mut self, pressure: Pressure) -> Result<(), MockloopHardwareError>;

    fn read_sensors(&mut self) -> Result<SensorData, MockloopHardwareError>;

    fn to_safe_state(&mut self) -> Result<(), MockloopHardwareError> {
        let safe_setpoint = ActuatorSetpoint::get_safe();
        self.set_regulator_pressure(safe_setpoint.controller_pressure_regulator)
            .unwrap();
        self.set_valve(Valve::Left, safe_setpoint.controller_valve_left)
            .unwrap();
        self.set_valve(Valve::Right, safe_setpoint.controller_valve_right)
            .unwrap();
        Ok(())
    }

    fn set_valve(
        &mut self,
        valve: Valve,
        setpoint: ValveState,
    ) -> Result<(), MockloopHardwareError>;
}
