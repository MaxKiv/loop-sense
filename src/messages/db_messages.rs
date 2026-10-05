use influxdb::{InfluxDbWriteable, Timestamp};
use uom::si::{
    frequency::cycle_per_minute,
    pressure::{bar, millibar, millimeter_of_mercury},
    volume_rate::liter_per_minute,
};

use crate::control::ControllerReport;

#[derive(Debug, Clone, InfluxDbWriteable)]
pub struct DatabaseRecord {
    // Sensor data
    pulmonary_preload_pressure_mmhg: f32,
    systemic_preload_pressure_mmhg: f32,
    pulmonary_afterload_pressure_mmhg: f32,
    systemic_afterload_pressure_mmhg: f32,
    systemic_flow_l_per_min: f32,
    pulmonary_flow_l_per_min: f32,

    // Heart controller
    heart_controller_enable: bool,
    heart_rate: f32,
    pressure_mbar: f32,
    systole_ratio: f32,

    // Mockloop controller
    mockloop_controller_enable: bool,
    systemic_resistance: f32,
    pulmonary_resistance: f32,
    systemic_afterload_compliance_mbar: f32,
    pulmonary_afterload_compliance_mbar: f32,

    // Metadata
    time: Timestamp,
    #[influxdb(tag)]
    experiment_id: String,
    #[influxdb(tag)]
    pub experiment_name: String,
    #[influxdb(tag)]
    experiment_description: String,
}

impl Default for DatabaseRecord {
    fn default() -> Self {
        Self {
            pulmonary_preload_pressure_mmhg: Default::default(),
            systemic_preload_pressure_mmhg: Default::default(),
            pulmonary_afterload_pressure_mmhg: Default::default(),
            systemic_afterload_pressure_mmhg: Default::default(),
            systemic_flow_l_per_min: Default::default(),
            pulmonary_flow_l_per_min: Default::default(),
            heart_controller_enable: Default::default(),
            heart_rate: Default::default(),
            pressure_mbar: Default::default(),
            systole_ratio: Default::default(),
            mockloop_controller_enable: Default::default(),
            systemic_resistance: Default::default(),
            pulmonary_resistance: Default::default(),
            systemic_afterload_compliance_mbar: Default::default(),
            pulmonary_afterload_compliance_mbar: Default::default(),
            time: Timestamp::Microseconds(0),
            experiment_id: Default::default(),
            experiment_name: Default::default(),
            experiment_description: Default::default(),
        }
    }
}

impl From<ControllerReport> for DatabaseRecord {
    fn from(r: ControllerReport) -> Self {
        // Parse uuid into hyphenated string
        let mut buf = [b'0'; 40];
        let uuid = r.experiment.id.as_hyphenated().encode_lower(&mut buf);

        Self {
            // Sensor data
            pulmonary_preload_pressure_mmhg: r
                .measurements
                .pulmonary_preload_pressure
                .get::<millimeter_of_mercury>(),
            systemic_preload_pressure_mmhg: r
                .measurements
                .systemic_preload_pressure
                .get::<millimeter_of_mercury>(),
            pulmonary_afterload_pressure_mmhg: r
                .measurements
                .pulmonary_afterload_pressure
                .get::<millimeter_of_mercury>(),
            systemic_afterload_pressure_mmhg: r
                .measurements
                .systemic_afterload_pressure
                .get::<millimeter_of_mercury>(),
            systemic_flow_l_per_min: r.measurements.systemic_flow.get::<liter_per_minute>(),
            pulmonary_flow_l_per_min: r.measurements.pulmonary_flow.get::<liter_per_minute>(),

            // Heart controller
            heart_controller_enable: r.heart_controller_setpoint.enable,
            heart_rate: r
                .heart_controller_setpoint
                .heart_rate
                .get::<cycle_per_minute>(),
            pressure_mbar: r.heart_controller_setpoint.pressure.get::<millibar>(),
            systole_ratio: r.heart_controller_setpoint.systole_ratio,

            // Mockloop controller
            mockloop_controller_enable: r.mockloop_setpoint.enable,
            systemic_resistance: r.mockloop_setpoint.systemic_resistance,
            pulmonary_resistance: r.mockloop_setpoint.pulmonary_resistance,
            systemic_afterload_compliance_mbar: r
                .mockloop_setpoint
                .systemic_afterload_compliance
                .get::<millibar>(),
            pulmonary_afterload_compliance_mbar: r
                .mockloop_setpoint
                .pulmonary_afterload_compliance
                .get::<millibar>(),

            time: Timestamp::Microseconds(r.time.timestamp_micros() as u128),
            experiment_id: String::from(uuid),
            experiment_name: r.experiment.name,
            experiment_description: r.experiment.description,
        }
    }
}
