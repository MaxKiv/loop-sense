use anyhow::{bail, Result};
use chrono::Utc;
use tracing::{debug, info};
use uom::si::{
    f64::{Pressure, VolumeRate},
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::{controller::backend::mockloop_hardware::SensorData, nidaq::nidaq_sys::{Nidaq}};

use super::mockloop_hardware::{
    MockloopHardware, REGULATOR_MIN_PRESSURE_BAR, Valve, ValveState,
};

impl MockloopHardware for Nidaq {
    fn set_regulator_pressure(&mut self, pressure: Pressure) -> Result<()> {
        debug!("setting regulator pressure {:?}", pressure);
        let pressure = pressure_to_voltage(
            pressure,
            Pressure::new::<bar>(REGULATOR_MIN_PRESSURE_BAR),
            Pressure::new::<bar>(2.0),
            0.0,
            10.0,
        )?;
        self.write

        // info!("Nidaq setting regulator pressure to: {:?}", pressure);
        Ok(self.regulator_set_pressure_task.write_analog(pressure)?)
    }

    fn read_sensors(&mut self) -> Result<SensorData> {
        // info!("Nidaq reading sensors");
        let pressure_systemic_preload = self
            .pressure_systemic_preload_task
            .read_analog()?
            .data
            .mean()
            .unwrap();

        let pressure_systemic_afterload = self
            .pressure_systemic_afterload_task
            .read_analog()?
            .data
            .mean()
            .unwrap();

        let regulator_actual_pressure = self
            .regulator_set_pressure_task
            .read_analog()?
            .data
            .mean()
            .unwrap();

        let systemic_flow = self.systemic_flow_task.read_analog()?.data.mean().unwrap();

        let pulmonary_flow = self.pulmonary_flow_task.read_analog()?.data.mean().unwrap();

        Ok(SensorData {
            experiment_time: Utc::now(),
            pressure_systemic_preload: voltage_to_pressure(
                pressure_systemic_preload,
                0.0,
                5.0,
                Pressure::new::<bar>(0.0),
                Pressure::new::<bar>(1.0),
            )?,
            pressure_systemic_afterload: voltage_to_pressure(
                pressure_systemic_afterload,
                0.0,
                5.0,
                Pressure::new::<bar>(0.0),
                Pressure::new::<bar>(1.0),
            )?,
            controller_regulator_actual_pressure: voltage_to_pressure(
                regulator_actual_pressure,
                0.0,
                10.0,
                Pressure::new::<bar>(0.0),
                Pressure::new::<bar>(2.0),
            )?,
            systemic_flow: voltage_to_flow(
                systemic_flow,
                0.0,
                10.0,
                VolumeRate::new::<liter_per_minute>(0.0),
                VolumeRate::new::<liter_per_minute>(60.0),
            )?,
            pulmonary_flow: voltage_to_flow(
                pulmonary_flow,
                0.0,
                10.0,
                VolumeRate::new::<liter_per_minute>(0.0),
                VolumeRate::new::<liter_per_minute>(60.0),
            )?,
        })
    }

    fn set_valve(&mut self, valve: Valve, setpoint: ValveState) -> Result<()> {
        // info!("Nidaq setting valve {:?}: {:?}", valve, setpoint);
        let task = match valve {
            Valve::Left => &self.set_valve_left_task,
            Valve::Right => &self.set_valve_right_task,
        };

        match setpoint {
            ValveState::Closed => task.write_digital(false)?,
            ValveState::Open => task.write_digital(true)?,
        };

        Ok(())
    }
}

fn pressure_to_voltage(
    pressure: Pressure,
    min_pressure: Pressure,
    max_pressure: Pressure,
    vmin: f64,
    vmax: f64,
) -> Result<f64> {
    if pressure < min_pressure {
        bail!(
            "Attempting to set pressure lower than min pressure: {:?} < {:?}",
            pressure, min_pressure
        );
    } else if pressure > max_pressure {
        bail!(
            "Attempting to set pressure larger than max pressure: {:?} > {:?}",
            pressure, max_pressure
        );
    }

    // Scale pressure setpoint to voltage
    Ok(
        (pressure.get::<bar>() - min_pressure.get::<bar>()) / max_pressure.get::<bar>()
            * (vmax - vmin),
    )
}

fn voltage_to_pressure(
    voltage: f64,
    vmin: f64,
    vmax: f64,
    min_pressure: Pressure,
    max_pressure: Pressure,
) -> Result<Pressure> {
    if voltage < vmin || voltage > vmax {
        bail!(
            "Voltage out of range in pressure conversion of: {:?} - Range [{:?},{:?}]",
            voltage, vmin, vmax
        );
    }

    let pressure = (voltage - vmin) / (vmax - vmin) * (max_pressure - min_pressure).get::<bar>()
        + min_pressure.get::<bar>();

    Ok(Pressure::new::<bar>(pressure))
}

fn voltage_to_flow(
    voltage: f64,
    vmin: f64,
    vmax: f64,
    min_flow: VolumeRate,
    max_flow: VolumeRate,
) -> Result<VolumeRate> {
    if voltage < vmin || voltage > vmax {
        bail!(
            "Voltage out of range in flowrate conversion of: {:?} - Range [{:?},{:?}]",
            voltage, vmin, vmax
        );
    }

    let flow = (voltage - vmin) / (vmax - vmin) * (max_flow - min_flow).get::<liter_per_minute>()
        + min_flow.get::<liter_per_minute>();

    Ok(VolumeRate::new::<liter_per_minute>(flow))
}
