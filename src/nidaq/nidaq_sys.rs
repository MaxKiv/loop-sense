use anyhow::{Result, bail};
use std::{ffi::CString, ptr};
use tracing::{error, info};

use chrono::Duration;
use ndarray::Array2;
use std::collections::HashMap;

use super::{bindings::nidaqmx_bindings::*, channel::ChannelName};
use crate::nidaq::channel::{
    AnalogInputChannel, AnalogOutputChannel, Channel, DigitalInputChannel, DigitalOutputChannel,
};

/// Combined sample rate for all nidaq
/// NOTE: maximum is 20kS/s, but we do not require this resolution
const COMBINED_TASK_SAMPLE_RATE: f64 = 2000.0;
/// Timeout for nidaq communications
const COMMS_WAIT_TIME: Duration = Duration::milliseconds(50);
/// NIDAQ Device name prefix
const DEVICE_PREFIX: &str = "dev1";
/// Number of samples per channel to read
const READ_SAMPLES_PER_CHANNEL: u64 = 100;
/// Number of samples per channel to write
const WRITE_SAMPLES_PER_CHANNEL: u64 = 1;

/// Typestate structs for NidaqBuilder
pub struct NotInitialized;
pub struct Initialized;

/// Safety wrapper for the NIDAQ TaskHandle
#[derive(Debug)]
pub struct TaskHandleWrapper {
    inner: TaskHandle,
}

unsafe impl Send for TaskHandleWrapper {}

#[derive(Debug)]
pub struct NidaqBuilder<S> {
    read_analog_task_handle: Option<TaskHandleWrapper>,
    write_analog_task_handle: Option<TaskHandleWrapper>,
    read_digital_task_handle: Option<TaskHandleWrapper>,
    write_digital_task_handle: Option<TaskHandleWrapper>,
    analog_input_channels: HashMap<ChannelName, AnalogInputChannel>,
    analog_output_channels: HashMap<ChannelName, AnalogOutputChannel>,
    digital_input_channels: HashMap<ChannelName, DigitalInputChannel>,
    digital_output_channels: HashMap<ChannelName, DigitalOutputChannel>,
    channel_data_idx: HashMap<ChannelName, usize>, // nidaqmx returns each virtual channels' data in the order each was registered, so we track that order here
    current_data_idx: usize,
    state: std::marker::PhantomData<S>, // Typestate marker
}

impl NidaqBuilder<NotInitialized> {
    pub fn new() -> Self {
        Self {
            read_analog_task_handle: None,
            write_analog_task_handle: None,
            read_digital_task_handle: None,
            write_digital_task_handle: None,
            analog_input_channels: HashMap::new(),
            analog_output_channels: HashMap::new(),
            digital_input_channels: HashMap::new(),
            digital_output_channels: HashMap::new(),
            channel_data_idx: HashMap::new(),
            state: std::marker::PhantomData::<NotInitialized>,
            current_data_idx: 0,
        }
    }

    pub fn initialize(self) -> Result<NidaqBuilder<Initialized>> {
        let mut read_analog_task_handle = std::ptr::null_mut();
        let mut write_analog_task_handle = std::ptr::null_mut();
        let mut read_digital_task_handle = std::ptr::null_mut();
        let mut write_digital_task_handle = std::ptr::null_mut();

        unsafe {
            check_err(DAQmxCreateTask(ptr::null(), &mut read_analog_task_handle))?;
            check_err(DAQmxCreateTask(ptr::null(), &mut write_analog_task_handle))?;
            check_err(DAQmxCreateTask(ptr::null(), &mut read_digital_task_handle))?;
            check_err(DAQmxCreateTask(ptr::null(), &mut write_digital_task_handle))?;
        }

        Ok(NidaqBuilder::<Initialized> {
            read_analog_task_handle: Some(TaskHandleWrapper {
                inner: read_analog_task_handle,
            }),
            write_analog_task_handle: Some(TaskHandleWrapper {
                inner: write_analog_task_handle,
            }),
            read_digital_task_handle: Some(TaskHandleWrapper {
                inner: read_digital_task_handle,
            }),
            write_digital_task_handle: Some(TaskHandleWrapper {
                inner: write_digital_task_handle,
            }),
            analog_input_channels: self.analog_input_channels.clone(),
            analog_output_channels: self.analog_output_channels.clone(),
            digital_input_channels: self.digital_input_channels.clone(),
            digital_output_channels: self.digital_output_channels.clone(),
            channel_data_idx: self.channel_data_idx.clone(),
            state: std::marker::PhantomData::<Initialized>,
            current_data_idx: 0,
        })
    }
}

impl NidaqBuilder<Initialized> {
    pub fn with_analog_input_channel(
        mut self,
        name: &'static str,
        channel: AnalogInputChannel,
    ) -> Result<Self> {
        self.analog_input_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    pub fn with_analog_output_channel(
        mut self,
        name: &'static str,
        channel: AnalogOutputChannel,
    ) -> Result<Self> {
        self.analog_output_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    pub fn with_digital_input_channel(
        mut self,
        name: &'static str,
        channel: DigitalInputChannel,
    ) -> Result<Self> {
        self.digital_input_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    pub fn with_digital_output_channel(
        mut self,
        name: &'static str,
        channel: DigitalOutputChannel,
    ) -> Result<Self> {
        self.digital_output_channels
            .insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    /// Set sampling configuration for the nidaq tasks
    /// NOTE: this must only be called once, after the virtual channels have been assigned
    fn cfg_samp_clk_timing(&self) -> Result<()> {
        let handles = vec![
            self.read_analog_task_handle
                .as_ref()
                .expect("read analog handle should exist after build step"),
            self.write_analog_task_handle
                .as_ref()
                .expect("write analog handle should exist after build step"),
            self.read_digital_task_handle
                .as_ref()
                .expect("write digital handle should exist after build step"),
            self.write_digital_task_handle
                .as_ref()
                .expect("write digital handle should exist after build step"),
        ];

        for handle in handles {
            unsafe {
                let err = DAQmxCfgSampClkTiming(
                    handle.inner,
                    ptr::null(),
                    COMBINED_TASK_SAMPLE_RATE
                        / (self.analog_output_channels.len()
                            + self.digital_output_channels.len()
                            + self.analog_output_channels.len()
                            + self.digital_output_channels.len()) as f64,
                    DAQmx_Val_Rising as i32,
                    DAQmx_Val_ContSamps as i32,
                    READ_SAMPLES_PER_CHANNEL,
                );
                check_err(err)?;
            }
        }

        Ok(())
    }

    pub fn start(self) -> Result<Nidaq> {
        // Register all channels with the Nidaq
        for AnalogInputChannel {
            channel: Channel { physical_channel },
            min,
            max,
        } in self.analog_input_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!("nidaqmx-sys: adding channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateAIVoltageChan(
                    self.read_analog_task_handle
                        .as_ref()
                        .expect("read task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    DAQmx_Val_Cfg_Default,
                    *min,
                    *max,
                    DAQmx_Val_Volts as i32,
                    ptr::null(),
                );
                check_err(err)?;
            }
        }

        for AnalogOutputChannel {
            channel: Channel { physical_channel },
            min,
            max,
        } in self.analog_output_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!("nidaqmx-sys: adding channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateAOVoltageChan(
                    self.write_analog_task_handle
                        .as_ref()
                        .expect("write task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    *min,
                    *max,
                    DAQmx_Val_Volts as i32,
                    ptr::null(),
                );
                check_err(err)?;
            }
        }

        for DigitalInputChannel {
            channel: Channel { physical_channel },
        } in self.digital_input_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!("nidaqmx-sys: adding channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateDIChan(
                    self.read_digital_task_handle
                        .as_ref()
                        .expect("read task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    DAQmx_Val_ChanForAllLines as i32,
                );
                check_err(err)?;
            }
        }

        for DigitalInputChannel {
            channel: Channel { physical_channel },
        } in self.digital_input_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!("nidaqmx-sys: adding channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateDIChan(
                    self.write_digital_task_handle
                        .as_ref()
                        .expect("read task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    DAQmx_Val_ChanForAllLines as i32,
                );
                check_err(err)?;
            }
        }

        // Set sampling config
        self.cfg_samp_clk_timing()?;

        // Start Nidaq tasks
        let handles = vec![
            self.read_analog_task_handle
                .as_ref()
                .expect("read analog handle should exist after build step"),
            self.write_analog_task_handle
                .as_ref()
                .expect("write analog handle should exist after build step"),
            self.read_digital_task_handle
                .as_ref()
                .expect("write digital handle should exist after build step"),
            self.write_digital_task_handle
                .as_ref()
                .expect("write digital handle should exist after build step"),
        ];
        for handle in handles {
            unsafe {
                check_err(DAQmxStartTask(handle.inner))?;
            }
        }

        Ok(Nidaq {
            read_analog_task_handle: TaskHandleWrapper {
                inner: self
                    .read_analog_task_handle
                    .as_ref()
                    .expect("read_analog_task_handle should be initialized during build step")
                    .inner,
            },
            write_analog_task_handle: TaskHandleWrapper {
                inner: self
                    .write_analog_task_handle
                    .as_ref()
                    .expect("write_analog_task_handle should be initialized during build step")
                    .inner,
            },
            read_digital_task_handle: TaskHandleWrapper {
                inner: self
                    .read_digital_task_handle
                    .as_ref()
                    .expect("write_digital_task_handle should be initialized during build step")
                    .inner,
            },
            write_digital_task_handle: TaskHandleWrapper {
                inner: self
                    .write_digital_task_handle
                    .as_ref()
                    .expect("read_digital_task_handle should be initialized during build step")
                    .inner,
            },
            analog_input_channels: self.analog_input_channels.clone(),
            analog_output_channels: self.analog_output_channels.clone(),
            digital_input_channels: self.digital_input_channels.clone(),
            digital_output_channels: self.digital_output_channels.clone(),
            channel_data_idx: self.channel_data_idx.clone(),

            current_setpoint: NidaqWriteData {
                analog: Array2::zeros((
                    self.analog_input_channels.len(),
                    WRITE_SAMPLES_PER_CHANNEL as usize,
                )),
                digital: Array2::zeros((
                    self.digital_input_channels.len(),
                    WRITE_SAMPLES_PER_CHANNEL as usize,
                )),
            },
            current_data: NidaqReading {
                analog: Array2::zeros((
                    self.analog_output_channels.len(),
                    READ_SAMPLES_PER_CHANNEL as usize,
                )),
                digital: Array2::zeros((
                    self.digital_output_channels.len(),
                    READ_SAMPLES_PER_CHANNEL as usize,
                )),
            },
        })
    }
}

impl<S> Drop for NidaqBuilder<S> {
    fn drop(&mut self) {
        if let Some(handle) = &self.read_analog_task_handle {
            unsafe {
                DAQmxClearTask(handle.inner);
            }
        }
        if let Some(handle) = &self.write_analog_task_handle {
            unsafe {
                DAQmxClearTask(handle.inner);
            }
        }
    }
}

#[derive(Debug)]
pub struct Nidaq {
    read_analog_task_handle: TaskHandleWrapper,
    write_analog_task_handle: TaskHandleWrapper,
    read_digital_task_handle: TaskHandleWrapper,
    write_digital_task_handle: TaskHandleWrapper,
    /// Nidaqmx expects a full set of setpoints on every write, this tracks the latest setpoints.
    /// Setpoints are mutated through the public methods of this struct
    /// NOTE: This means we write zeros to everything before setpoints are received, this is fine.
    current_setpoint: NidaqWriteData,
    /// Nidaqmx reads all sensordata at once, this tracks the latest reading for further processing in the public methods
    current_data: NidaqReading,
    analog_input_channels: HashMap<ChannelName, AnalogInputChannel>,
    analog_output_channels: HashMap<ChannelName, AnalogOutputChannel>,
    digital_input_channels: HashMap<ChannelName, DigitalInputChannel>,
    digital_output_channels: HashMap<ChannelName, DigitalOutputChannel>,
    channel_data_idx: HashMap<ChannelName, usize>, // nidaqmx returns each virtual channels' data in the order each was registered, so we track that order here
}

impl Drop for Nidaq {
    fn drop(&mut self) {
        unsafe {
            DAQmxClearTask(self.read_analog_task_handle.inner);
        }
        unsafe {
            DAQmxClearTask(self.write_analog_task_handle.inner);
        }
        unsafe {
            DAQmxClearTask(self.read_digital_task_handle.inner);
        }
        unsafe {
            DAQmxClearTask(self.write_digital_task_handle.inner);
        }
    }
}

impl Nidaq {
    /// Read all data from the nidaq
    pub fn read(&self) -> Result<NidaqReading> {
        let mut analog = Array2::zeros((
            self.analog_input_channels.len(),
            READ_SAMPLES_PER_CHANNEL as usize,
        ));
        let mut read_samples_per_channel: i32 = 0;
        unsafe {
            let err = DAQmxReadAnalogF64(
                self.read_analog_task_handle.inner,
                DAQmx_Val_Auto,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                analog.as_mut_ptr(),
                (READ_SAMPLES_PER_CHANNEL * self.analog_output_channels.len() as u64)
                    .try_into()
                    .unwrap(),
                &mut read_samples_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        if read_samples_per_channel as u64 != READ_SAMPLES_PER_CHANNEL {
            bail!(
                "Analog read should read {} samples but only read {} samples",
                READ_SAMPLES_PER_CHANNEL,
                read_samples_per_channel
            );
        }

        let mut digital = Array2::zeros((
            self.digital_input_channels.len(),
            READ_SAMPLES_PER_CHANNEL as usize,
        ));
        unsafe {
            let err = DAQmxReadDigitalU8(
                self.read_analog_task_handle.inner,
                DAQmx_Val_Auto,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                digital.as_mut_ptr(),
                READ_SAMPLES_PER_CHANNEL as u32 * self.analog_input_channels.len() as u32,
                &mut read_samples_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        if read_samples_per_channel as u64 != READ_SAMPLES_PER_CHANNEL {
            bail!(
                "Digital read should read {} samples but only read {} samples",
                READ_SAMPLES_PER_CHANNEL,
                read_samples_per_channel
            );
        }

        Ok(NidaqReading { analog, digital })
    }

    pub fn get_data_idx(&self, channel_name: &str) -> Option<&usize> {
        self.channel_data_idx.get(channel_name)
    }

    pub fn update_analog_setpoint(&mut self, row: usize, value: f64) {
        self.current_setpoint.analog[[row, 0]] = value;
    }

    pub fn update_digital_setpoint(&mut self, row: usize, value: u8) {
        self.current_setpoint.digital[[row, 0]] = value;
    }

    /// Send latest setpoints to the nidaq
    pub fn write(&self) -> Result<()> {
        let NidaqWriteData { analog, digital } = &self.current_setpoint;

        info!("Writing setpoints to nidaq: {:?} - {:?}", analog, digital);

        let mut written_samples_per_channel: i32 = 0;
        unsafe {
            let err = DAQmxWriteAnalogF64(
                self.write_analog_task_handle.inner,
                WRITE_SAMPLES_PER_CHANNEL as i32,
                0,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                analog.as_ptr(),
                &mut written_samples_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        if written_samples_per_channel as u64 != READ_SAMPLES_PER_CHANNEL {
            bail!(
                "Analog write should write {} samples but only read {} samples",
                READ_SAMPLES_PER_CHANNEL,
                written_samples_per_channel
            );
        }

        unsafe {
            let err = DAQmxWriteDigitalU8(
                self.write_analog_task_handle.inner,
                WRITE_SAMPLES_PER_CHANNEL as i32,
                0,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                digital.as_ptr(),
                &mut written_samples_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        if written_samples_per_channel as u64 != READ_SAMPLES_PER_CHANNEL {
            bail!(
                "Digital write should write {} samples but only read {} samples",
                READ_SAMPLES_PER_CHANNEL,
                written_samples_per_channel
            );
        }

        Ok(())
    }
}

fn check_err(err: i32) -> Result<()> {
    if err != 0 {
        // Allocate the error buffer
        let mut buf = [0i8; 2048];
        // Fetch latest error information
        unsafe {
            DAQmxGetExtendedErrorInfo(buf.as_mut_ptr(), buf.len() as u32);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        if err < 0 {
            // Log error and return it
            error!("{}", c_str.to_string_lossy());
            bail!("{}", c_str.to_string_lossy().into_owned());
        } else if err > 0 {
            // Log warning
            tracing::warn!("{}", c_str.to_string_lossy());
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct NidaqReading {
    pub analog: Array2<f64>,
    pub digital: Array2<u8>,
}

#[derive(Debug)]
pub struct NidaqWriteData {
    pub analog: Array2<f64>,
    pub digital: Array2<u8>,
}
