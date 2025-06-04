use super::mockloop_hardware::{MockloopHardware, SensorData};

#[derive(Debug)]
pub struct Sim;

impl MockloopHardware for Sim {
    fn initialize(&mut self) -> Result<(), super::mockloop_hardware::MockloopHardwareError> {
        tracing::info!("initializing Simulated MockloopHardware");
        Ok(())
    }

    fn set_regulator_pressure(
        &mut self,
        pressure: uom::si::f64::Pressure,
    ) -> Result<(), super::mockloop_hardware::MockloopHardwareError> {
        tracing::info!("Setting regulator pressure to {:?}", pressure);
        Ok(())
    }

    fn read_sensors(
        &mut self,
    ) -> Result<super::mockloop_hardware::SensorData, super::mockloop_hardware::MockloopHardwareError>
    {
        Ok(SensorData::simulate())
    }

    fn set_valve(
        &mut self,
        valve: super::mockloop_hardware::Valve,
        setpoint: super::mockloop_hardware::ValveState,
    ) -> Result<(), super::mockloop_hardware::MockloopHardwareError> {
        tracing::info!("Setting valve: {:?} to {:?}", valve, setpoint);
        Ok(())
    }
}
