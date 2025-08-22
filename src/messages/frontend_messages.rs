use serde::{Deserialize, Serialize};
use uom::si::{
    f32::{Frequency, Pressure},
    frequency::hertz,
    pressure::bar,
};

#[derive(Debug, Clone, Copy)]
pub struct Report {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Setpoint {
    // pub current_time: DateTimeWrapper,
    /// Should the mockloop controller be enabled?
    pub enable: bool,
    pub mockloop_setpoint: MockloopSetpoint,
    pub heart_controller_setpoint: HeartControllerSetpoint,
}

impl Into<love_letter::Setpoint> for Setpoint {
    fn into(self) -> love_letter::Setpoint {
        love_letter::Setpoint {
            enable: self.enable,
            mockloop_setpoint: self.mockloop_setpoint.into(),
            heart_controller_setpoint: self.heart_controller_setpoint.into(),
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
    /// NOTE: usually 3/7
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
