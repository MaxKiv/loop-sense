use std::{ffi::CString, ptr};

use chrono::Duration;
use ndarray::Array2;

use super::bindings::nidaqmx_bindings::*;

#[derive(Debug, Eq, PartialEq)]
enum ChannelType {
    Input,
    Output,
}

#[derive(Debug, Eq, PartialEq)]
enum SignalType {
    Analog,
    Digital,
}

/// Combined sample rate for all nidaq
/// NOTE: maximum is 20kS/s, but we do not require this resolution
const COMBINED_TASK_SAMPLE_RATE: f64 = 2000.0;
const COMMS_WAIT_TIME: Duration = Duration::milliseconds(50);
/// Read buffer size in samples
const N_SAMPLES_READ: usize = 1024;

#[derive(Debug)]
pub struct NidaqError(pub String);

#[derive(Debug)]
/// Nidaqmx task wrapper
/// Note: currently only a single channel per task is supported
pub struct Task {
    handle: TaskHandle,
    channel: Option<Channel>,
}

#[derive(Debug)]
/// Nidaq in/output channel
pub struct Channel {
    name: &'static str,
    signal_type: SignalType,
    channel_type: ChannelType,
    physical_channel: String,
}

#[derive(Debug)]
pub struct NidaqAnalogReading {
    pub data: Array2<f64>,
}

#[derive(Debug)]
pub struct NidaqDigitalReading {
    pub data: u32,
}

// TODO: find a better method, Send should not be impl generally as people will think its
// threadsafe, which its not
/// SAFETY: Send is only used by tracing task (by #[instrument]), which reads it.
/// Invariant: the task handle is never changed after construction, and only accessed through the
/// public Task methods
unsafe impl Send for Task {}

impl Task {
    pub fn new() -> Result<Self, NidaqError> {
        let mut handle = std::ptr::null_mut();
        unsafe {
            let err = DAQmxCreateTask(ptr::null(), &mut handle);
            check_err(err)?;
        }
        Ok(Task {
            handle,
            channel: None,
        })
    }

    pub fn start(&self) -> Result<(), NidaqError> {
        unsafe {
            let err = DAQmxStartTask(self.handle);
            check_err(err)?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), NidaqError> {
        unsafe {
            let err = DAQmxStopTask(self.handle);
            check_err(err)?;
        }
        Ok(())
    }

    pub fn add_ai_voltage_chan(
        &mut self,
        channel: &'static str,
        name: &'static str,
        min: f64,
        max: f64,
    ) -> Result<(), NidaqError> {
        let c_channel = CString::new(channel)
            .map_err(|_| NidaqError("Null char in channel name".to_owned()))?;

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
        let c_channel = CString::new(channel)
            .map_err(|_| NidaqError("Null char in channel name".to_owned()))?;

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
        let c_channel = CString::new(channel)
            .map_err(|_| NidaqError("Null char in lines descriptor".to_owned()))?;

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
        let c_channel = CString::new(channel)
            .map_err(|_| NidaqError("Null char in lines descriptor".to_owned()))?;

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

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            DAQmxClearTask(self.handle);
        }
    }
}

fn check_err(err: i32) -> Result<(), NidaqError> {
    if err < 0 {
        // Allocate the error buffer
        let mut buf = [0i8; 2048];
        // Fetch latest error information
        unsafe {
            DAQmxGetExtendedErrorInfo(buf.as_mut_ptr(), buf.len() as u32);
        }
        // Return that as NidaqError
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        return Err(NidaqError(c_str.to_string_lossy().into_owned()));
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
