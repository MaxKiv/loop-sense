use crate::nidaq::nidaq_sys::{NidaqError, Task};
use chrono::Utc;
use tracing::info;
use uom::si::{
    f64::{Pressure, VolumeRate},
    pressure::bar,
    volume_rate::liter_per_minute,
};

use crate::controller::backend::mockloop_hardware::SensorData;

use super::mockloop_hardware::{MockloopHardware, MockloopHardwareError, Valve, ValveState};

#[derive(Debug)]
pub struct Nidaq {
    pressure_systemic_preload_task: Task,
    pressure_systemic_afterload_task: Task,
    // pressure_pulmonary_preload_task: Task,
    // pressure_pulmonary_afterload_task: Task,
    regulator_actual_pressure_task: Task,
    systemic_flow_task: Task,
    pulmonary_flow_task: Task,
    set_valve_left_task: Task,
    set_valve_right_task: Task,
    regulator_set_pressure_task: Task,
}

impl Nidaq {
    pub fn try_new() -> Result<Self, NidaqError> {
        Ok(Nidaq {
            pressure_systemic_preload_task: Task::new()?,
            pressure_systemic_afterload_task: Task::new()?,
            // pressure_pulmonary_preload_task: Task::new()?,
            // pressure_pulmonary_afterload_task: Task::new()?,
            regulator_actual_pressure_task: Task::new()?,
            systemic_flow_task: Task::new()?,
            pulmonary_flow_task: Task::new()?,
            set_valve_left_task: Task::new()?,
            set_valve_right_task: Task::new()?,
            regulator_set_pressure_task: Task::new()?,
        })
    }
}

impl MockloopHardware for Nidaq {
    fn initialize(&mut self) -> Result<(), MockloopHardwareError> {
        info!("Nidaq initializing");

        self.pressure_systemic_preload_task.add_ai_voltage_chan(
            "ai0",
            "pressure_systemic_preload",
            0.0,
            10.0,
        )?;
        self.pressure_systemic_afterload_task.add_ai_voltage_chan(
            "ai1",
            "pressure_systemic_afterload",
            0.0,
            10.0,
        )?;
        // self.pressure_pulmonary_preload_task.add_ai_voltage_chan(channel, min, max)?;
        // self.pressure_pulmonary_afterload_task.add_ai_voltage_chan(channel, min, max)?;
        self.regulator_actual_pressure_task.add_ai_voltage_chan(
            "ai4",
            "regulator_actual_pressure",
            0.0,
            10.0,
        )?;
        self.systemic_flow_task
            .add_ai_voltage_chan("ai6", "systemic_flow", 0.0, 10.0)?;
        self.pulmonary_flow_task
            .add_ai_voltage_chan("ai7", "pulmonary_flow", 0.0, 10.0)?;

        self.pressure_systemic_preload_task.cfg_samp_clk_timing(5)?;
        self.pressure_systemic_afterload_task
            .cfg_samp_clk_timing(5)?;
        // self.pressure_pulmonary_preload_task.cfg_samp_clk_timing(5)?;
        // self.pressure_pulmonary_afterload_task
        //     .cfg_samp_clk_timing(5)?;
        self.regulator_actual_pressure_task.cfg_samp_clk_timing(5)?;
        self.systemic_flow_task.cfg_samp_clk_timing(5)?;
        self.pulmonary_flow_task.cfg_samp_clk_timing(5)?;

        self.set_valve_left_task
            .add_do_chan("port0/line0", "valve_left")?;
        self.set_valve_right_task
            .add_do_chan("port0/line1", "valve_right")?;
        self.regulator_set_pressure_task.add_ao_voltage_chan(
            "ai3",
            "regulator_set_pressure",
            0.0,
            10.0,
        )?;

        Ok(())
    }

    fn set_regulator_pressure(&mut self, pressure: Pressure) -> Result<(), MockloopHardwareError> {
        let pressure = pressure_to_voltage(
            pressure,
            Pressure::new::<bar>(0.02),
            Pressure::new::<bar>(2.0),
            0.0,
            10.0,
        )?;
        // info!("Nidaq setting regulator pressure to: {:?}", pressure);
        Ok(self.regulator_set_pressure_task.write_analog(pressure)?)
    }

    fn read_sensors(&mut self) -> Result<SensorData, MockloopHardwareError> {
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

    fn set_valve(
        &mut self,
        valve: Valve,
        setpoint: ValveState,
    ) -> Result<(), MockloopHardwareError> {
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

impl From<NidaqError> for MockloopHardwareError {
    fn from(value: NidaqError) -> Self {
        MockloopHardwareError(value.0)
    }
}

impl From<ConversionError> for MockloopHardwareError {
    fn from(value: ConversionError) -> Self {
        MockloopHardwareError(value.0)
    }
}

#[derive(Debug)]
pub struct ConversionError(String);

fn pressure_to_voltage(
    pressure: Pressure,
    min_pressure: Pressure,
    max_pressure: Pressure,
    vmin: f64,
    vmax: f64,
) -> Result<f64, ConversionError> {
    if pressure < min_pressure {
        return Err(ConversionError(format!(
            "Attempting to set pressure lower than min pressure: {:?} < {:?}",
            pressure, min_pressure
        )));
    } else if pressure > max_pressure {
        return Err(ConversionError(format!(
            "Attempting to set pressure larger than max pressure: {:?} > {:?}",
            pressure, max_pressure
        )));
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
) -> Result<Pressure, ConversionError> {
    if voltage < vmin || voltage > vmax {
        return Err(ConversionError(format!(
            "Voltage out of range in pressure conversion of: {:?} - Range [{:?},{:?}]",
            voltage, vmin, vmax
        )));
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
) -> Result<VolumeRate, ConversionError> {
    if voltage < vmin || voltage > vmax {
        return Err(ConversionError(format!(
            "Voltage out of range in flowrate conversion of: {:?} - Range [{:?},{:?}]",
            voltage, vmin, vmax
        )));
    }

    let flow = (voltage - vmin) / (vmax - vmin) * (max_flow - min_flow).get::<liter_per_minute>()
        + min_flow.get::<liter_per_minute>();

    Ok(VolumeRate::new::<liter_per_minute>(flow))
}
