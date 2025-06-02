use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rand::{random_bool, random_range};
use serde::{Deserialize, Serialize};

/// Sensor data from the micro controlling the Mockloop
#[derive(Debug, Default, Clone, Serialize)]
pub struct SensorData {
    experiment_time: DateTime<Utc>,
    pressure_systemic_preload: f64,
    pressure_systemic_afterload: f64,
    controller_regulator_actual_pressure: f64,
    systemic_flow: f64,
    pulmonary_flow: f64,
    controller_valve_left: bool,
    controller_valve_right: bool,
}

/// Setpoints for the micro controlling the Mockloop
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Setpoint {
    controller_valve_left: bool,
    controller_valve_right: bool,
    controller_pressure_regulator: f64,
}

/// Application state, contains latest sensor data from the micro and setpoint for the control loop
/// running on the micro
#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub sensor_data: Arc<Mutex<SensorData>>,
    pub setpoint: Arc<Mutex<Setpoint>>,
}

impl SensorData {
    pub fn simulate() -> Self {
        let now = Utc::now();
        SensorData {
            experiment_time: now,
            pressure_systemic_preload: random_range(0.0..=1.0),
            pressure_systemic_afterload: random_range(0.0..=1.0),
            controller_regulator_actual_pressure: random_range(0.0..=1.0),
            systemic_flow: random_range(0.0..=60.0),
            pulmonary_flow: random_range(0.0..=60.0),
            controller_valve_left: random_bool(0.5),
            controller_valve_right: random_bool(0.5),
        }
    }
}
