use anyhow::Result;
use std::{ffi::CString, ptr};
use thiserror::Error;
use tracing::error;

use chrono::Duration;
use ndarray::Array2;
use std::collections::HashMap;

use super::{
    bindings::nidaqmx_bindings::*,
    channel::{ChannelName, ReadChannel, WriteChannel},
};

/// Combined sample rate for all nidaq
/// NOTE: maximum is 20kS/s, but we do not require this resolution
const COMBINED_TASK_SAMPLE_RATE: f64 = 2000.0;
/// Timeout for nidaq communications
const COMMS_WAIT_TIME: Duration = Duration::milliseconds(50);
/// Read buffer size in samples
const N_SAMPLES_READ: usize = 1024;
/// NIDAQ Device name prefix
const DEVICE_PREFIX: &str = "dev1";

#[derive(Debug, Error)]
pub enum NidaqError {
    #[error("Nidaq Error: {0}")]
    Error(String),
}

/// Typestate structs for NidaqBuilder
pub struct NotInitialized;
pub struct Initialized;

#[derive(Debug)]
pub struct NidaqBuilder<S> {
    read_task_handle: Option<TaskHandle>,
    write_task_handle: Option<TaskHandle>,
    read_channels: HashMap<ChannelName, ReadChannel>,
    write_channels: HashMap<ChannelName, WriteChannel>,
    state: std::marker::PhantomData<S>,
}

impl NidaqBuilder<NotInitialized> {
    pub fn new() -> Self {
        Self {
            read_task_handle: None,
            write_task_handle: None,
            read_channels: HashMap::new(),
            write_channels: HashMap::new(),
            state: std::marker::PhantomData::<NotInitialized>, // Typestate marker
        }
    }

    pub fn with_read_channel(mut self, channel_name: ChannelName, channel: ReadChannel) -> Self {
        self.read_channels.insert(channel_name, channel);
        self
    }

    pub fn with_write_channel(
        mut self,
        channel_name: ChannelName,
        channel: WriteChannel,
    ) -> Result<Self> {
        let c_channel = CString::new(format!("{}/{}", DEVICE_PREFIX, channel.0.physical_channel))?;
        error!("nidaqmx-sys: adding channel: {:?}", c_channel);

        unsafe {
            let err = DAQmxCreateAOVoltageChan(
                self.read_task_handle
                    .expect("No task handle when trying to add channel"),
                c_channel.as_ptr(),
                ptr::null(),
                channel.0.min,
                channel.0.max,
                DAQmx_Val_Volts as i32,
                ptr::null(),
            );
            check_err(err)?;
        }

        self.write_channels.insert(channel_name, channel);
        Ok(self)
    }

    pub fn build(self) -> Result<NidaqBuilder<Initialized>> {
        let mut read_task_handle = std::ptr::null_mut();
        let mut write_task_handle = std::ptr::null_mut();

        unsafe {
            check_err(DAQmxCreateTask(ptr::null(), &mut read_task_handle))?;
            check_err(DAQmxCreateTask(ptr::null(), &mut write_task_handle))?;
        }

        Ok(NidaqBuilder::<Initialized> {
            read_task_handle: Some(read_task_handle),
            write_task_handle: Some(write_task_handle),
            write_channels: self.write_channels.clone(),
            read_channels: self.read_channels.clone(),
            state: std::marker::PhantomData::<Initialized>,
        })
    }
}

impl<S> Drop for NidaqBuilder<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.read_task_handle {
            unsafe {
                DAQmxClearTask(handle);
            }
        }
        if let Some(handle) = self.write_task_handle {
            unsafe {
                DAQmxClearTask(handle);
            }
        }
    }
}

impl NidaqBuilder<Initialized> {
    pub fn initialize(self) -> Result<Nidaq> {
        // Start Nidaq tasks
        unsafe {
            check_err(DAQmxStartTask(
                self.read_task_handle
                    .expect("read task handle should exist after build step"),
            ))?;
            check_err(DAQmxStartTask(
                self.write_task_handle
                    .expect("write task handle should exist after build step"),
            ))?;
        }

        Ok(Nidaq {
            read_task_handle: self
                .read_task_handle
                .expect("read_task_handle should be initialized during build step"),
            write_task_handle: self
                .write_task_handle
                .expect("write_task_handle should be initialized during build step"),
            read_channels: self.read_channels.clone(),
            write_channels: self.write_channels.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Nidaq {
    read_task_handle: TaskHandle,
    write_task_handle: TaskHandle,
    read_channels: HashMap<String, ReadChannel>,
    write_channels: HashMap<String, WriteChannel>,
}

impl Drop for Nidaq {
    fn drop(&mut self) {
        unsafe {
            DAQmxClearTask(self.read_task_handle);
        }
        unsafe {
            DAQmxClearTask(self.write_task_handle);
        }
    }
}

// pressure_systemic_preload_task: Task,
// pressure_systemic_afterload_task: Task,
// // pressure_pulmonary_preload_task: Task,
// // pressure_pulmonary_afterload_task: Task,
// regulator_actual_pressure_task: Task,
// systemic_flow_task: Task,
// pulmonary_flow_task: Task,
// set_valve_left_task: Task,
// set_valve_right_task: Task,
// regulator_set_pressure_task: Task,

// TODO: find a better method, Send should not be impl generally as people will think its
// threadsafe, which its not
/// SAFETY: Send is only used by tracing task (by #[instrument]), which reads it.
/// Invariant: the task handle is never changed after construction, and only accessed through the
/// public Task methods
// unsafe impl Send for Task {}

impl Nidaq {
    pub fn add_ai_voltage_chan(
        &mut self,
        channel: &'static str,
        name: &'static str,
        min: f64,
        max: f64,
    ) -> Result<(), NidaqError> {
        let c_channel = CString::new(format!("{}/{}", DEVICE_PREFIX, channel))
            .map_err(|_| NidaqError::Error("Null char in channel name".to_owned()))?;
        error!("nidaqmx-sys: adding channel: {:?}", c_channel);

        unsafe {
            let err = DAQmxCreateAIVoltageChan(
                self.handle,
                c_channel.as_ptr(),
                ptr::null(),
                DAQmx_Val_Cfg_Default,
                min,
                max,
                DAQmx_Val_Volts as i32,
                ptr::null(),
            );
            check_err(err)?;
        }

        self.channel = Some(Channel {
            name,
            signal_type: SignalType::Analog,
            channel_type: ChannelType::Input,
            physical_channel: channel.to_owned(),
        });

        Ok(())
    }

    pub fn add_ao_voltage_chan(
        &mut self,
        channel: &'static str,
        name: &'static str,
        min: f64,
        max: f64,
    ) -> Result<(), NidaqError> {
        let c_channel = CString::new(format!("{}/{}", self.device, channel))
            .map_err(|_| NidaqError("Null char in channel name".to_owned()))?;
        error!("nidaqmx-sys: adding channel: {:?}", c_channel);

        unsafe {
            let err = DAQmxCreateAOVoltageChan(
                self.handle,
                c_channel.as_ptr(),
                ptr::null(),
                min,
                max,
                DAQmx_Val_Volts as i32,
                ptr::null(),
            );
            check_err(err)?;
        }

        self.channel = Some(Channel {
            name,
            signal_type: SignalType::Analog,
            channel_type: ChannelType::Output,
            physical_channel: channel.to_owned(),
        });

        Ok(())
    }

    pub fn add_di_chan(
        &mut self,
        channel: &'static str,
        name: &'static str,
    ) -> Result<(), NidaqError> {
        let c_channel = CString::new(format!("{}/{}", self.device, channel))
            .map_err(|_| NidaqError("Null char in channel name".to_owned()))?;
        error!("nidaqmx-sys: adding channel: {:?}", c_channel);

        unsafe {
            let err = DAQmxCreateDIChan(
                self.handle,
                c_channel.as_ptr(),
                ptr::null(),
                DAQmx_Val_ChanForAllLines as i32,
            );
            check_err(err)?;
        }

        self.channel = Some(Channel {
            name,
            signal_type: SignalType::Digital,
            channel_type: ChannelType::Input,
            physical_channel: channel.to_owned(),
        });

        Ok(())
    }

    pub fn add_do_chan(
        &mut self,
        channel: &'static str,
        name: &'static str,
    ) -> Result<(), NidaqError> {
        let c_channel = CString::new(format!("{}/{}", self.device, channel))
            .map_err(|_| NidaqError("Null char in channel name".to_owned()))?;
        error!("nidaqmx-sys: adding channel: {:?}", c_channel);

        unsafe {
            let err = DAQmxCreateDOChan(
                self.handle,
                c_channel.as_ptr(),
                ptr::null(),
                DAQmx_Val_ChanForAllLines as i32,
            );
            check_err(err)?;
        }

        self.channel = Some(Channel {
            name,
            signal_type: SignalType::Digital,
            channel_type: ChannelType::Output,
            physical_channel: channel.to_owned(),
        });

        Ok(())
    }

    pub fn cfg_samp_clk_timing(&self, num_analog_input_tasks: usize) -> Result<(), NidaqError> {
        unsafe {
            let err = DAQmxCfgSampClkTiming(
                self.handle,
                ptr::null(),
                COMBINED_TASK_SAMPLE_RATE / (num_analog_input_tasks as f64),
                DAQmx_Val_Rising as i32,
                DAQmx_Val_ContSamps as i32,
                1000,
            );
            check_err(err)?;
        }
        Ok(())
    }

    pub fn read_analog(&self) -> Result<NidaqAnalogReading, NidaqError> {
        self.check_channel(SignalType::Analog, ChannelType::Input)?;

        let mut data = Array2::zeros((1, N_SAMPLES_READ));

        let mut samps_per_channel = 0;
        unsafe {
            let err = DAQmxReadAnalogF64(
                self.handle,
                DAQmx_Val_Auto,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                data.as_mut_ptr(),
                N_SAMPLES_READ as u32,
                &mut samps_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        Ok(NidaqAnalogReading { data })
    }

    pub fn write_analog(&self, val: f64) -> Result<(), NidaqError> {
        self.check_channel(SignalType::Analog, ChannelType::Output)?;

        unsafe {
            let err = DAQmxWriteAnalogScalarF64(
                self.handle,
                0,
                COMMS_WAIT_TIME.as_seconds_f64(),
                val,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        Ok(())
    }

    pub fn read_digital(&self) -> Result<NidaqDigitalReading, NidaqError> {
        self.check_channel(SignalType::Digital, ChannelType::Input)?;

        let mut data = 0u32;
        unsafe {
            let err = DAQmxReadDigitalScalarU32(
                self.handle,
                COMMS_WAIT_TIME.as_seconds_f64(),
                &mut data,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        Ok(NidaqDigitalReading { data })
    }

    pub fn write_digital(&self, val: bool) -> Result<(), NidaqError> {
        self.check_channel(SignalType::Digital, ChannelType::Output)?;

        let mut buf = [0u8; 1];
        buf[0] = val as u8;

        let mut samps_per_channel = 0;

        unsafe {
            let err = DAQmxWriteDigitalLines(
                self.handle,
                1,
                0,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                buf.as_ptr(),
                &mut samps_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        Ok(())
    }

    fn check_channel(
        &self,
        correct_signal_type: SignalType,
        correct_channel_type: ChannelType,
    ) -> Result<(), NidaqError> {
        if self.channel.is_none() {
            return Err(NidaqError(format!(
                "Tried analog reading a non-initialized channel: {:?}",
                self.channel
            )));
        }
        if let Some(Channel {
            signal_type,
            channel_type,
            ..
        }) = &self.channel
        {
            if *signal_type != correct_signal_type {
                return Err(NidaqError(format!(
                    "Tried analog reading a non-analog channel: {:?}",
                    self.channel
                )));
            }
            if *channel_type != correct_channel_type {
                return Err(NidaqError(format!(
                    "Tried analog reading a non-input channel: {:?}",
                    self.channel
                )));
            }
        }
        Ok(())
    }
}

pub fn check_err(err: i32) -> Result<(), NidaqError> {
    if err < 0 {
        // Allocate the error buffer
        let mut buf = [0i8; 2048];
        // Fetch latest error information
        unsafe {
            DAQmxGetExtendedErrorInfo(buf.as_mut_ptr(), buf.len() as u32);
        }
        // Return that as NidaqError
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        return Err(NidaqError::Error((c_str.to_string_lossy().into_owned())));
    } else if err > 0 {
        // Trace warning
        let mut buf = [0i8; 2048];
        unsafe {
            DAQmxGetExtendedErrorInfo(buf.as_mut_ptr(), buf.len() as u32);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        tracing::warn!("{}", c_str.to_string_lossy());
    }
    Ok(())
}

#[derive(Debug)]
pub struct NidaqAnalogReading {
    pub data: Array2<f64>,
}

#[derive(Debug)]
pub struct NidaqDigitalReading {
    pub data: u32,
}
