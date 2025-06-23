use anyhow::{Context, Result, bail};
use chrono::Utc;
use tracing::{debug, info};
use uom::si::{
    f64::{Pressure, VolumeRate},
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::{
    controller::backend::mockloop_hardware::SensorData,
    nidaq::nidaq_sys::{Nidaq, NidaqWriteData},
};

use super::mockloop_hardware::{MockloopHardware, REGULATOR_MIN_PRESSURE_BAR, Valve, ValveState};

pub const PRESSURE_SYSTEMIC_PRELOAD_CHANNEL: &str = "pressure_systemic_preload";
pub const PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL: &str = "pressure_systemic_afterload";
pub const PRESSURE_PULMONARY_PRELOAD_CHANNEL: &str = "pressure_pulmonary_preload";
pub const PRESSURE_PULMONARY_AFTERLOAD_CHANNEL: &str = "pressure_pulmonary_afterload";
pub const REGULATOR_ACTUAL_PRESSURE_CHANNEL: &str = "regulator_actual_pressure";
pub const SET_REGULATOR_PRESSURE_CHANNEL: &str = "regulator_set_pressure";
pub const SYSTEMIC_FLOW_CHANNEL: &str = "systemic_flow";
pub const PULMONARY_FLOW_CHANNEL: &str = "pulmonary_flow";
pub const VALVE_LEFT_CHANNEL: &str = "valve_left";
pub const VALVE_RIGHT_CHANNEL: &str = "valve_rigth";

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

        let idx = self
            .get_data_idx(PRESSURE_SYSTEMIC_PRELOAD_CHANNEL)
            .expect("PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL should exist");
        self.update_analog_setpoint(*idx, pressure);

        // Update nidaq setpoints
        self.write()?;

        Ok(())
    }

    fn set_valve(&mut self, valve: Valve, setpoint: ValveState) -> Result<()> {
        debug!("setting valve {:?} to {:?}", valve, setpoint);
        let channel_name = match valve {
            Valve::Left => &VALVE_LEFT_CHANNEL,
            Valve::Right => &VALVE_RIGHT_CHANNEL,
        };

        let idx = self
            .get_data_idx(channel_name)
            .with_context(|| format!("channel {} should exists", channel_name))?;

        let value = match setpoint {
            ValveState::Closed => 0,
            ValveState::Open => 1,
        };

        self.update_digital_setpoint(*idx, value);

        Ok(())
    }

    fn read_sensors(&mut self) -> Result<SensorData> {
        info!("Nidaq reading sensors");

        let data = self.read()?;

        let pressure_systemic_preload = data
            .analog
            .row(
                *(self
                    .get_data_idx(PRESSURE_SYSTEMIC_PRELOAD_CHANNEL)
                    .expect("PRESSURE_SYSTEMIC_PRELOAD_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let pressure_systemic_afterload = data
            .analog
            .row(
                *(self
                    .get_data_idx(PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL)
                    .expect("PRESSURE_SYSTEMIC_AFTERLOAD_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let pressure_pulmonary_preload = data
            .analog
            .row(
                *(self
                    .get_data_idx(PRESSURE_PULMONARY_PRELOAD_CHANNEL)
                    .expect("PRESSURE_PULMONARY_PRELOAD_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let pressure_pulmonary_afterload = data
            .analog
            .row(
                *(self
                    .get_data_idx(PRESSURE_PULMONARY_AFTERLOAD_CHANNEL)
                    .expect("PRESSURE_PULMONARY_AFTERLOAD_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let regulator_actual_pressure = data
            .analog
            .row(
                *(self
                    .get_data_idx(REGULATOR_ACTUAL_PRESSURE_CHANNEL)
                    .expect("REGULATOR_ACTUAL_PRESSURE_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let systemic_flow = data
            .analog
            .row(
                *(self
                    .get_data_idx(SYSTEMIC_FLOW_CHANNEL)
                    .expect("SYSTEMIC_FLOW_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

        let pulmonary_flow = data
            .analog
            .row(
                *(self
                    .get_data_idx(PULMONARY_FLOW_CHANNEL)
                    .expect("PULMONARY_FLOW_CHANNEL should exist")),
            )
            .mean()
            .unwrap();

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
            pressure,
            min_pressure
        );
    } else if pressure > max_pressure {
        bail!(
            "Attempting to set pressure larger than max pressure: {:?} > {:?}",
            pressure,
            max_pressure
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
            voltage,
            vmin,
            vmax
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
            voltage,
            vmin,
            vmax
        );
    }

    let flow = (voltage - vmin) / (vmax - vmin) * (max_flow - min_flow).get::<liter_per_minute>()
        + min_flow.get::<liter_per_minute>();

    Ok(VolumeRate::new::<liter_per_minute>(flow))
}
