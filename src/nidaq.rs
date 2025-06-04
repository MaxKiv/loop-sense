use tracing::info;
use uom::si::f32::Pressure;

use crate::hardware::{MockloopHardware, MockloopHardwareError, SensorData, Valve, ValveState};

#[derive(Debug)]
pub struct Nidaq {}

impl MockloopHardware for Nidaq {
    fn initialize(&mut self) -> Result<(), MockloopHardwareError> {
        // info!("Nidaq initializing");
        Ok(())
    }

    fn set_regulator_pressure(&mut self, pressure: Pressure) -> Result<(), MockloopHardwareError> {
        // info!("Nidaq setting regulator pressure to: {:?}", pressure);
        Ok(())
    }

    fn read_sensors(&mut self) -> Result<SensorData, MockloopHardwareError> {
        // info!("Nidaq reading sensors");
        Ok(SensorData::simulate())
    }

    fn set_valve(
        &mut self,
        valve: Valve,
        setpoint: ValveState,
    ) -> Result<(), MockloopHardwareError> {
        // info!("Nidaq setting valve {:?}: {:?}", valve, setpoint);
        Ok(())
    }
}
