use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use love_letter::Measurements;
use uom::si::{frequency::hertz, pressure::bar, volume_rate::liter_per_minute};

use crate::{
    experiment::Experiment,
    messages::frontend_messages::{HeartControllerSetpoint, MockloopSetpoint},
};

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct DatabaseRecord {
    right_preload_pressure_mmhg: f32,
    left_preload_pressure_mmhg: f32,
    right_afterload_pressure_mmhg: f32,
    left_afterload_pressure_mmhg: f32,
    systemic_flow_l_per_min: f32,
    pulmonary_flow_l_per_min: f32,
    heart_rate: f32,
    pressure: f32,
    systole_ratio: f32,
    systemic_resistance: f32,
    pulmonary_resistance: f32,
    left_afterload_compliance: f32,
    right_afterload_compliance: f32,
    simulation_time: f32,
    time: DateTime<Utc>,
    experiment_id: String,
    experiment_name: String,
    experiment_description: String,
}

pub struct DatabaseReport {
    pub mockloop_setpoint: MockloopSetpoint,
    pub heart_controller_setpoint: HeartControllerSetpoint,
    pub measurements: Measurements,
    pub experiment: Experiment,
    pub time: DateTime<Utc>,
}

impl From<DatabaseReport> for DatabaseRecord {
    fn from(r: DatabaseReport) -> Self {
        // Parse uuid into hyphenated string
        let mut buf = [b'0'; 40];
        let uuid = r.experiment.id.as_hyphenated().encode_lower(&mut buf);

        Self {
            right_preload_pressure_mmhg: r.measurements.pulmonary_preload_pressure.get::<bar>(),
            left_preload_pressure_mmhg: r.measurements.systemic_preload_pressure.get::<bar>(),
            right_afterload_pressure_mmhg: r.measurements.pulmonary_afterload_pressure.get::<bar>(),
            left_afterload_pressure_mmhg: r.measurements.systemic_afterload_pressure.get::<bar>(),
            systemic_flow_l_per_min: r.measurements.systemic_flow.get::<liter_per_minute>(),
            pulmonary_flow_l_per_min: r.measurements.pulmonary_flow.get::<liter_per_minute>(),
            heart_rate: r.heart_controller_setpoint.heart_rate.get::<hertz>(),
            pressure: r.heart_controller_setpoint.pressure.get::<bar>(),
            systole_ratio: r.heart_controller_setpoint.systole_ratio,
            systemic_resistance: r.mockloop_setpoint.systemic_resistance,
            pulmonary_resistance: r.mockloop_setpoint.pulmonary_resistance,
            left_afterload_compliance: r.mockloop_setpoint.systemic_afterload_compliance,
            right_afterload_compliance: r.mockloop_setpoint.pulmonary_afterload_compliance,
            simulation_time: 0.0,
            time: r.time,
            experiment_id: String::from(uuid),
            experiment_name: r.experiment.name,
            experiment_description: r.experiment.description,
        }
    }
}
