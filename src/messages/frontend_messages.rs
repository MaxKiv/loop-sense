use serde::{Deserialize, Serialize};
use uom::si::{
    f32::{Frequency, Pressure},
    frequency::hertz,
    pressure::bar,
};

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
    pub pneumatic_controller_setpoint: HeartControllerSetpoint,
}

/// Setpoint for the mockloop hemodynamics controller
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct MockloopSetpoint {
    pub systemic_resistance: f32,
    pub pulmonary_resistance: f32,
    pub left_afterload_compliance: f32,
    pub right_afterload_compliance: f32,
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
            left_afterload_compliance: frontend.left_afterload_compliance_l_per_mmhg,
            right_afterload_compliance: frontend.right_afterload_compliance_l_per_mmhg,
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
