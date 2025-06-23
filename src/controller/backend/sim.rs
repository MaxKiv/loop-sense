use anyhow::Result;

use crate::controller::backend::mockloop_hardware::{
    MockloopHardware, SensorData, Valve, ValveState,
};

#[derive(Debug)]
pub struct Sim;

impl MockloopHardware for Sim {
    fn set_regulator_pressure(&mut self, pressure: uom::si::f64::Pressure) -> Result<()> {
        tracing::info!("Setting regulator pressure to {:?}", pressure);
        Ok(())
    }

    fn read_sensors(&mut self) -> Result<SensorData> {
        Ok(SensorData::simulate())
    }

    fn set_valve(&mut self, valve: Valve, setpoint: ValveState) -> Result<()> {
        tracing::info!("Setting valve: {:?} to {:?}", valve, setpoint);
        Ok(())
    }
}
