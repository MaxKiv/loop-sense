use chrono::{DateTime, Utc};
use love_letter::Measurements;

use crate::{
    experiment::Experiment,
    messages::frontend_messages::{HeartControllerSetpoint, MockloopSetpoint},
};

pub mod controller;

#[derive(Clone, Debug)]
pub struct ControllerReport {
    pub mockloop_setpoint: Option<MockloopSetpoint>,
    pub heart_controller_setpoint: Option<HeartControllerSetpoint>,
    pub measurements: Measurements,
    pub experiment: Experiment,
    pub time: DateTime<Utc>,
}
impl ControllerReport {
    fn from_mcu_report(
        mcu_report: love_letter::Report,
        report_time: DateTime<Utc>,
        current_experiment: Option<Experiment>,
    ) -> Self {
        Self {
            mockloop_setpoint: mcu_report.setpoint.mockloop_setpoint.map(Into::into),
            heart_controller_setpoint: mcu_report
                .setpoint
                .heart_controller_setpoint
                .map(Into::into),
            measurements: mcu_report.measurements,
            experiment: current_experiment.unwrap_or_default(),
            time: report_time,
        }
    }
}
