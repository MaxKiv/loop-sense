use love_letter::{HeartControllerSetpoint, MockloopSetpoint};
use serde::{Deserialize, Serialize};
use uom::si::pressure::millibar;
use uom::si::{
    f32::{Frequency, Pressure},
    frequency::cycle_per_minute,
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::control::ControllerReport;

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    // Sensor data
    pulmonary_preload_pressure_mmhg: f32,
    systemic_preload_pressure_mmhg: f32,
    pulmonary_afterload_pressure_mmhg: f32,
    systemic_afterload_pressure_mmhg: f32,
    systemic_flow_l_per_min: f32,
    pulmonary_flow_l_per_min: f32,
    heart_actual_pressure_mbar: f32,
    systemic_compliance_actual_pressure_mbar: f32,
    pulmonary_compliance_actual_pressure_mbar: f32,

    // Heart controller
    heart_controller_enable: bool,
    heart_rate: Option<f32>,
    pressure_mbar: Option<f32>,
    systole_ratio: Option<f32>,

    // Mockloop controller
    mockloop_controller_enable: bool,
    systemic_resistance: Option<f32>,
    pulmonary_resistance: Option<f32>,
    systemic_compliance_mbar: Option<f32>,
    pulmonary_compliance_mbar: Option<f32>,

    // Metadata
    time: i64,
    experiment_id: String,
    experiment_name: String,
    experiment_description: String,
}

impl From<ControllerReport> for Report {
    fn from(r: ControllerReport) -> Self {
        // Parse uuid into hyphenated string
        let mut buf = [b'0'; 40];
        let uuid = r.experiment.id.as_hyphenated().encode_lower(&mut buf);

        Self {
            // Sensor data
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

            heart_actual_pressure_mbar: r.measurements.heart_actual_pressure.get::<millibar>(),
            systemic_compliance_actual_pressure_mbar: r
                .measurements
                .systemic_compliance_actual_pressure
                .get::<millibar>(),
            pulmonary_compliance_actual_pressure_mbar: r
                .measurements
                .pulmonary_compliance_actual_pressure
                .get::<millibar>(),

            // Heart controller
            heart_controller_enable: r.heart_controller_setpoint.enable,
            heart_rate: r.heart_controller_setpoint.enable.then_some(
                r.heart_controller_setpoint
                    .heart_rate
                    .get::<cycle_per_minute>(),
            ),
            pressure_mbar: r
                .heart_controller_setpoint
                .enable
                .then_some(r.heart_controller_setpoint.pressure.get::<bar>()),
            systole_ratio: r
                .heart_controller_setpoint
                .enable
                .then_some(r.heart_controller_setpoint.systole_ratio),

            // Mockloop controller
            mockloop_controller_enable: r.mockloop_setpoint.enable,
            systemic_resistance: r
                .mockloop_setpoint
                .enable
                .then_some(r.mockloop_setpoint.systemic_resistance),
            pulmonary_resistance: r
                .mockloop_setpoint
                .enable
                .then_some(r.mockloop_setpoint.pulmonary_resistance),
            systemic_compliance_mbar: r.mockloop_setpoint.enable.then_some(
                r.mockloop_setpoint
                    .systemic_afterload_compliance
                    .get::<millibar>(),
            ),
            pulmonary_compliance_mbar: r.mockloop_setpoint.enable.then_some(
                r.mockloop_setpoint
                    .pulmonary_afterload_compliance
                    .get::<millibar>(),
            ),

            // Metadata
            time: r.time.timestamp_nanos_opt().unwrap_or(0i64),
            experiment_id: uuid.to_string(),
            experiment_name: r.experiment.name,
            experiment_description: r.experiment.description,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FrontendSetpoint {
    /// Should the mockloop controller be enabled?
    pub mockloop_setpoint: MockloopSetpoint,
    /// Should the heart controller be enabled?
    pub heart_controller_setpoint: HeartControllerSetpoint,
}

impl From<love_letter::Setpoint> for FrontendSetpoint {
    fn from(setpoint: love_letter::Setpoint) -> Self {
        Self {
            mockloop_setpoint: setpoint.mockloop_setpoint.into(),
            heart_controller_setpoint: setpoint.heart_controller_setpoint.into(),
        }
    }
}

impl From<FrontendSetpoint> for love_letter::Setpoint {
    fn from(setpoint: FrontendSetpoint) -> Self {
        Self {
            mockloop_setpoint: setpoint.mockloop_setpoint.into(),
            heart_controller_setpoint: setpoint.heart_controller_setpoint.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FrontendMockloopSetpoint {
    enable: bool,
    systemic_resistance_mmhg_s_per_l: f32,
    pulmonary_resistance_mmhg_s_per_l: f32,
    systemic_afterload_compliance_mbar: f32,
    pulmonary_afterload_compliance_mbar: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FrontendHeartControllerSetpoint {
    enable: bool,
    heart_rate: f32,
    pressure: f32,
    systole_ratio: f32,
}

pub struct FrontendExperimentSetpoint {
    name: String,
    description: String,
}

impl From<FrontendMockloopSetpoint> for MockloopSetpoint {
    fn from(frontend: FrontendMockloopSetpoint) -> Self {
        MockloopSetpoint {
            enable: frontend.enable,
            systemic_resistance: frontend.systemic_resistance_mmhg_s_per_l,
            pulmonary_resistance: frontend.pulmonary_resistance_mmhg_s_per_l,
            systemic_afterload_compliance: Pressure::new::<millibar>(
                frontend.systemic_afterload_compliance_mbar,
            ),
            pulmonary_afterload_compliance: Pressure::new::<millibar>(
                frontend.pulmonary_afterload_compliance_mbar,
            ),
        }
    }
}

impl From<FrontendHeartControllerSetpoint> for HeartControllerSetpoint {
    fn from(frontend: FrontendHeartControllerSetpoint) -> Self {
        HeartControllerSetpoint {
            enable: frontend.enable,
            heart_rate: Frequency::new::<cycle_per_minute>(frontend.heart_rate),
            pressure: Pressure::new::<millibar>(frontend.pressure),
            systole_ratio: frontend.systole_ratio,
        }
    }
}
