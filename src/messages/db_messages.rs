use love_letter::Measurements;
use uom::si::{frequency::hertz, pressure::bar, volume_rate::liter_per_minute};

use crate::{
    experiment::Experiment,
    messages::frontend_messages::{HeartControllerSetpoint, MockloopSetpoint},
};

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
    time: f32,
    experiment_id: f32,          // TODO: make this tag
    experiment_name: f32,        // TODO: make this tag
    experiment_description: f32, // TODO: make this tag
}

pub struct DatabaseReport {
    pub mockloop_setpoint: MockloopSetpoint,
    pub heart_controller_setpoint: HeartControllerSetpoint,
    pub measurements: Measurements,
    pub experiment: Experiment,
    pub time: u64,
}

impl From<DatabaseReport> for DatabaseRecord {
    fn from(r: DatabaseReport) -> Self {
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
            left_afterload_compliance: r.mockloop_setpoint.left_afterload_compliance,
            right_afterload_compliance: r.mockloop_setpoint.right_afterload_compliance,
            simulation_time: 0,
            time: r.measurements.timestamp,
            experiment_id: r.experiment.experiment_id,
            experiment_name: r.experiment.experiment_name,
            experiment_description: r.experiment.description,
        }
    }
}
