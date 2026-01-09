use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use uom::si::{
    frequency::{cycle_per_minute, hertz},
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::control::ControllerReport;

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct DatabaseRecord {
    pulmonary_preload_pressure_mmhg: f32,
    systemic_preload_pressure_mmhg: f32,
    pulmonary_afterload_pressure_mmhg: f32,
    systemic_afterload_pressure_mmhg: f32,
    systemic_flow_l_per_min: f32,
    pulmonary_flow_l_per_min: f32,
    heart_rate: Option<f32>,
    pressure: Option<f32>,
    systole_ratio: Option<f32>,
    systemic_resistance: Option<f32>,
    pulmonary_resistance: Option<f32>,
    systemic_afterload_compliance: Option<f32>,
    pulmonary_afterload_compliance: Option<f32>,
    simulation_time: f32,
    time: DateTime<Utc>,
    #[influxdb(tag)]
    experiment_id: String,
    #[influxdb(tag)]
    experiment_name: String,
    #[influxdb(tag)]
    experiment_description: String,
}

impl From<ControllerReport> for DatabaseRecord {
    fn from(r: ControllerReport) -> Self {
        // Parse uuid into hyphenated string
        let mut buf = [b'0'; 40];
        let uuid = r.experiment.id.as_hyphenated().encode_lower(&mut buf);

        Self {
            pulmonary_preload_pressure_mmhg: r.measurements.pulmonary_preload_pressure.get::<bar>(),
            systemic_preload_pressure_mmhg: r.measurements.systemic_preload_pressure.get::<bar>(),
            pulmonary_afterload_pressure_mmhg: r
                .measurements
                .pulmonary_afterload_pressure
                .get::<bar>(),
            systemic_afterload_pressure_mmhg: r
                .measurements
                .systemic_afterload_pressure
                .get::<bar>(),
            systemic_flow_l_per_min: r.measurements.systemic_flow.get::<liter_per_minute>(),
            pulmonary_flow_l_per_min: r.measurements.pulmonary_flow.get::<liter_per_minute>(),
            heart_rate: r
                .heart_controller_setpoint
                .as_ref()
                .map(|s| s.heart_rate.get::<cycle_per_minute>()),
            pressure: r
                .heart_controller_setpoint
                .as_ref()
                .map(|s| s.pressure.get::<bar>()),
            systole_ratio: r
                .heart_controller_setpoint
                .as_ref()
                .map(|s| s.systole_ratio),
            systemic_resistance: r.mockloop_setpoint.as_ref().map(|s| s.systemic_resistance),
            pulmonary_resistance: r.mockloop_setpoint.as_ref().map(|s| s.pulmonary_resistance),
            systemic_afterload_compliance: r
                .mockloop_setpoint
                .as_ref()
                .map(|s| s.systemic_afterload_compliance),
            pulmonary_afterload_compliance: r
                .mockloop_setpoint
                .as_ref()
                .map(|s| s.pulmonary_afterload_compliance),
            simulation_time: 0.0,
            time: r.time,
            experiment_id: String::from(uuid),
            experiment_name: r.experiment.name,
            experiment_description: r.experiment.description,
        }
    }
}
