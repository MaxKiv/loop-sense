use serde::{Deserialize, Serialize};
use uom::si::{
    f32::{Frequency, Pressure},
    frequency::hertz,
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::control::ControllerReport;

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    right_preload_pressure_mmhg: f32,
    left_preload_pressure_mmhg: f32,
    right_afterload_pressure_mmhg: f32,
    left_afterload_pressure_mmhg: f32,
    systemic_flow_l_per_min: f32,
    pulmonary_flow_l_per_min: f32,
    heart_rate: Option<f32>,
    pressure: Option<f32>,
    systole_ratio: Option<f32>,
    systemic_resistance: Option<f32>,
    pulmonary_resistance: Option<f32>,
    left_afterload_compliance: Option<f32>,
    right_afterload_compliance: Option<f32>,
    time: i64,
    experiment_id: String,          // TODO: make this tag
    experiment_name: String,        // TODO: make this tag
    experiment_description: String, // TODO: make this tag
}

impl From<ControllerReport> for Report {
    fn from(r: ControllerReport) -> Self {
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
            heart_rate: r
                .heart_controller_setpoint
                .as_ref()
                .map(|s| s.heart_rate.get::<hertz>()),
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
            left_afterload_compliance: r
                .mockloop_setpoint
                .as_ref()
                .map(|s| s.systemic_afterload_compliance),
            right_afterload_compliance: r
                .mockloop_setpoint
                .as_ref()
                .map(|s| s.pulmonary_afterload_compliance),
            time: r.time.timestamp_nanos_opt().unwrap_or(0i64),
            experiment_id: uuid.to_string(),
            experiment_name: r.experiment.name,
            experiment_description: r.experiment.description,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FrontendSetpoint {
    /// Should the mockloop controller be enabled?
    pub mockloop_setpoint: Option<MockloopSetpoint>,
    pub heart_controller_setpoint: Option<HeartControllerSetpoint>,
}

impl From<love_letter::Setpoint> for FrontendSetpoint {
    fn from(setpoint: love_letter::Setpoint) -> Self {
        Self {
            mockloop_setpoint: setpoint.mockloop_setpoint.map(Into::into),
            heart_controller_setpoint: setpoint.heart_controller_setpoint.map(Into::into),
        }
    }
}

impl From<FrontendSetpoint> for love_letter::Setpoint {
    fn from(setpoint: FrontendSetpoint) -> Self {
        Self {
            mockloop_setpoint: setpoint.mockloop_setpoint.map(Into::into),
            heart_controller_setpoint: setpoint.heart_controller_setpoint.map(Into::into),
        }
    }
}

/// Setpoint for the mockloop hemodynamics controller
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct MockloopSetpoint {
    pub systemic_resistance: f32,
    pub pulmonary_resistance: f32,
    pub systemic_afterload_compliance: f32,
    pub pulmonary_afterload_compliance: f32,
}

/// Setpoint for the pneumatic heart prototype controller
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeartControllerSetpoint {
    /// Desired heart rate
    pub heart_rate: Frequency,
    /// Desired regulator pressure
    pub pressure: Pressure,
    /// Ratio of systole duration to total cardiac phase duration
    pub systole_ratio: f32,
}

pub struct FrontendMockloopSetpoint {
    systemic_mmhg_s_per_l: f32,
    pulmonary_mmhg_s_per_l: f32,
    left_afterload_compliance_l_per_mmhg: f32,
    right_afterload_compliance_l_per_mmhg: f32,
}

pub struct FrontendHeartControllerSetpoint {
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
            systemic_resistance: frontend.systemic_mmhg_s_per_l,
            pulmonary_resistance: frontend.pulmonary_mmhg_s_per_l,
            systemic_afterload_compliance: frontend.left_afterload_compliance_l_per_mmhg,
            pulmonary_afterload_compliance: frontend.right_afterload_compliance_l_per_mmhg,
        }
    }
}

impl From<love_letter::MockloopSetpoint> for MockloopSetpoint {
    fn from(mcu: love_letter::MockloopSetpoint) -> Self {
        Self {
            systemic_resistance: mcu.systemic_resistance,
            pulmonary_resistance: mcu.pulmonary_resistance,
            systemic_afterload_compliance: mcu.systemic_afterload_compliance,
            pulmonary_afterload_compliance: mcu.pulmonary_afterload_compliance,
        }
    }
}

impl Into<love_letter::MockloopSetpoint> for MockloopSetpoint {
    fn into(self) -> love_letter::MockloopSetpoint {
        love_letter::MockloopSetpoint {
            systemic_resistance: self.systemic_resistance,
            pulmonary_resistance: self.pulmonary_resistance,
            systemic_afterload_compliance: self.systemic_afterload_compliance,
            pulmonary_afterload_compliance: self.pulmonary_afterload_compliance,
        }
    }
}

impl From<FrontendHeartControllerSetpoint> for HeartControllerSetpoint {
    fn from(frontend: FrontendHeartControllerSetpoint) -> Self {
        HeartControllerSetpoint {
            heart_rate: Frequency::new::<hertz>(frontend.heart_rate),
            pressure: Pressure::new::<bar>(frontend.pressure),
            systole_ratio: frontend.systole_ratio,
        }
    }
}

impl Into<love_letter::HeartControllerSetpoint> for HeartControllerSetpoint {
    fn into(self) -> love_letter::HeartControllerSetpoint {
        love_letter::HeartControllerSetpoint {
            heart_rate: self.heart_rate,
            pressure: self.pressure,
            systole_ratio: self.systole_ratio,
        }
    }
}

impl From<love_letter::HeartControllerSetpoint> for HeartControllerSetpoint {
    fn from(mcu: love_letter::HeartControllerSetpoint) -> Self {
        Self {
            heart_rate: mcu.heart_rate,
            pressure: mcu.pressure,
            systole_ratio: mcu.systole_ratio,
        }
    }
}
