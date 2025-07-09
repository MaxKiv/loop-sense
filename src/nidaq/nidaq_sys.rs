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
const READ_SAMPLES_PER_CHANNEL: u64 = 8;
/// Number of samples per channel to write
const WRITE_SAMPLES_PER_CHANNEL: u64 = 8;
/// Number of samples in the Nidaq output buffer
const SAMPLES_IN_OUTPUT_BUFFER: u32 = 8;

/// Typestate structs for NidaqBuilder
pub struct NotInitialized;
pub struct Initialized;

/// Safety wrapper for the NIDAQ TaskHandle
#[derive(Debug)]
pub struct TaskHandleWrapper {
    inner: TaskHandle,
}

unsafe impl Send for TaskHandleWrapper {}

/// Builder pattern to construct Nidaq
#[derive(Debug)]
pub struct NidaqBuilder<S> {
    analog_input_task: Option<TaskHandleWrapper>,
    analog_output_task: Option<TaskHandleWrapper>,
    digital_input_task: Option<TaskHandleWrapper>,
    digital_output_task: Option<TaskHandleWrapper>,
    analog_input_channels: HashMap<ChannelName, AnalogInputChannel>,
    analog_output_channels: HashMap<ChannelName, AnalogOutputChannel>,
    digital_input_channels: HashMap<ChannelName, DigitalInputChannel>,
    digital_output_channels: HashMap<ChannelName, DigitalOutputChannel>,
    channel_data_idx: HashMap<ChannelName, usize>, // nidaqmx returns each virtual channels' data in the order each was registered, so we track that order here
    current_data_idx: usize,
    state: std::marker::PhantomData<S>, // Typestate marker to force initialization order
}

impl NidaqBuilder<NotInitialized> {
    pub fn new() -> Self {
        Self {
            analog_input_task: None,
            analog_output_task: None,
            digital_input_task: None,
            digital_output_task: None,
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
        info!("creating nidaq tasks");

        let analog_input_name = CString::new("analog_input_task")?;
        let analog_output_name = CString::new("analog_output_task")?;
        let digital_input_name = CString::new("digital_input_task")?;
        let digital_output_name = CString::new("digital_output_task")?;

        let mut input_analog_task_handle = std::ptr::null_mut();
        let mut output_analog_task_handle = std::ptr::null_mut();
        let mut input_digital_task_handle = std::ptr::null_mut();
        let mut output_digital_task_handle = std::ptr::null_mut();

        unsafe {
            check_err(DAQmxCreateTask(
                analog_input_name.as_ptr(),
                &mut input_analog_task_handle,
            ))?;
            check_err(DAQmxCreateTask(
                analog_output_name.as_ptr(),
                &mut output_analog_task_handle,
            ))?;
            check_err(DAQmxCreateTask(
                digital_input_name.as_ptr(),
                &mut input_digital_task_handle,
            ))?;
            check_err(DAQmxCreateTask(
                digital_output_name.as_ptr(),
                &mut output_digital_task_handle,
            ))?;
        }

        Ok(NidaqBuilder::<Initialized> {
            analog_input_task: Some(TaskHandleWrapper {
                inner: input_analog_task_handle,
            }),
            analog_output_task: Some(TaskHandleWrapper {
                inner: output_analog_task_handle,
            }),
            digital_input_task: Some(TaskHandleWrapper {
                inner: input_digital_task_handle,
            }),
            digital_output_task: Some(TaskHandleWrapper {
                inner: output_digital_task_handle,
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
    /// Add an analog input channel
    pub fn with_analog_input_channel(
        mut self,
        name: &'static str,
        channel: AnalogInputChannel,
    ) -> Result<Self> {
        info!("Adding analog input channel {}: {:?}", name, channel);

        self.analog_input_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    /// Add an analog output channel
    pub fn with_analog_output_channel(
        mut self,
        name: &'static str,
        channel: AnalogOutputChannel,
    ) -> Result<Self> {
        info!("Adding analog output channel {}: {:?}", name, channel);

        self.analog_output_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    /// Add a digital input channel
    pub fn with_digital_input_channel(
        mut self,
        name: &'static str,
        channel: DigitalInputChannel,
    ) -> Result<Self> {
        info!("Adding digital input channel {}: {:?}", name, channel);

        self.digital_input_channels.insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    /// Add a digital output channel
    pub fn with_digital_output_channel(
        mut self,
        name: &'static str,
        channel: DigitalOutputChannel,
    ) -> Result<Self> {
        info!("Adding digital output channel {}: {:?}", name, channel);

        self.digital_output_channels
            .insert(name.to_owned(), channel);
        self.channel_data_idx
            .insert(name.to_owned(), self.current_data_idx);
        self.current_data_idx += 1;

        Ok(self)
    }

    /// Set sampling configuration for the nidaq tasks
    /// NOTE: this must only be called once, after the virtual channels have been assigned
    fn cfg_samp_clk_timing(task_handle: &TaskHandleWrapper, rate: f64) -> Result<()> {
        info!("cfg samp clk timing: {:?}", task_handle);
        unsafe {
            let err = DAQmxCfgSampClkTiming(
                task_handle.inner,
                ptr::null(),
                rate,
                DAQmx_Val_Rising as i32,
                // DAQmx_Val_FiniteSamps as i32,
                DAQmx_Val_ContSamps as i32,
                READ_SAMPLES_PER_CHANNEL,
            );
            check_err(err)?;
        }

        Ok(())
    }

    fn cfg_output_buffer(task_handle: &TaskHandleWrapper) -> Result<()> {
        info!("Setting output buffer of task {:?}", task_handle);
        unsafe {
            let err = DAQmxCfgOutputBuffer(task_handle.inner, SAMPLES_IN_OUTPUT_BUFFER);
            check_err(err)?;
        }

        Ok(())
    }

    /// Start the Nidaq, finishing its construction
    pub fn start(self) -> Result<Nidaq> {
        info!("registering all channels with the nidaq");
        // Register all channels with the Nidaq
        for AnalogInputChannel {
            channel: Channel { physical_channel },
            min,
            max,
        } in self.analog_input_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!("nidaqmx-sys: adding analog input channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateAIVoltageChan(
                    self.analog_input_task
                        .as_ref()
                        .expect("analog input task handle should exist after build step")
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
            info!("nidaqmx-sys: adding analog output channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateAOVoltageChan(
                    self.analog_output_task
                        .as_ref()
                        .expect("analog output task handle should exist after build step")
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
            info!("nidaqmx-sys: adding digital input channel: {:?}", c_channel);

            unsafe {
                let err = DAQmxCreateDIChan(
                    self.digital_input_task
                        .as_ref()
                        .expect("digital read task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    DAQmx_Val_ChanForAllLines as i32,
                );
                check_err(err)?;
            }
        }

        for DigitalOutputChannel {
            channel: Channel { physical_channel },
        } in self.digital_output_channels.values()
        {
            let physical_channel = format!("{DEVICE_PREFIX}/{physical_channel}");
            let c_channel = CString::new(physical_channel.clone())?;
            info!(
                "nidaqmx-sys: adding digital output channel: {:?}",
                c_channel
            );

            unsafe {
                let err = DAQmxCreateDOChan(
                    self.digital_output_task
                        .as_ref()
                        .expect("digital output task handle should exist after build step")
                        .inner,
                    c_channel.as_ptr(),
                    ptr::null(),
                    DAQmx_Val_ChanForAllLines as i32,
                );
                check_err(err)?;
            }
        }

        // Set sampling config
        let rate = COMBINED_TASK_SAMPLE_RATE
            / (self.analog_output_channels.len()
                + self.digital_output_channels.len()
                + self.analog_output_channels.len()
                + self.digital_output_channels.len()) as f64;
        info!("Setting nidaq task sampling rate to {:?}", rate);

        if !self.analog_input_channels.is_empty() {
            Self::cfg_samp_clk_timing(
                self.analog_input_task
                    .as_ref()
                    .expect("read analog handle should exist after build step"),
                rate,
            )?;
        }
        if !self.analog_output_channels.is_empty() {
            Self::cfg_samp_clk_timing(
                self.analog_output_task
                    .as_ref()
                    .expect("write analog handle should exist after build step"),
                rate,
            )?;
            Self::cfg_output_buffer(
                self.analog_output_task
                    .as_ref()
                    .expect("read digital handle should exist after build step"),
            )?;
        }

        if !self.digital_input_channels.is_empty() {
            // NOTE: this break the nidaq, so we just skip it
            // Self::cfg_samp_clk_timing(
            //     self.digital_input_task
            //         .as_ref()
            //         .expect("read digital handle should exist after build step"),
            //     rate,
            // )?;
        }
        if !self.digital_output_channels.is_empty() {
            // NOTE: this break the nidaq, so we just skip it
            // Self::cfg_samp_clk_timing(
            //     self.digital_output_task
            //         .as_ref()
            //         .expect("write digital handle should exist after build step"),
            //     rate,
            // )?;

            // NOTE: this is apparently not required for digital tasks?
            // Self::cfg_output_buffer(
            //     self.digital_output_task
            //         .as_ref()
            //         .expect("read digital handle should exist after build step"),
            // )?;
        }

        // Write analog output before starting the nidaq tasks to fix Error 200462, 2 is the magic number!
        // I love this device
        let NidaqWriteData { analog, digital } = NidaqWriteData {
            analog: Array2::zeros((self.analog_input_channels.len(), 2)),
            digital: Array2::zeros((self.digital_input_channels.len(), 2)),
        };

        info!(
            "Writing initial setpoints to nidaq task: {:?}",
            self.analog_output_task.as_ref().unwrap().inner
        );
        let mut written_samples_per_channel: i32 = 0;
        unsafe {
            let err = DAQmxWriteAnalogF64(
                self.analog_output_task.as_ref().unwrap().inner,
                2,
                0,
                COMMS_WAIT_TIME.as_seconds_f64(),
                DAQmx_Val_GroupByChannel,
                analog.as_ptr(),
                &mut written_samples_per_channel,
                ptr::null_mut(),
            );
            check_err(err)?;
        }

        // Verify tasks
        let err = unsafe {
            DAQmxTaskControl(
                self.analog_output_task.as_ref().unwrap().inner,
                DAQmx_Val_Task_Verify.try_into().unwrap(),
            )
        };
        info!("verify: analog output task returns {err}");
        check_err(err)?;

        // Start Nidaq tasks
        info!("Starting nidaq tasks");
        if !self.analog_input_channels.is_empty() {
            unsafe {
                check_err(DAQmxStartTask(
                    self.analog_input_task
                        .as_ref()
                        .expect("task handle should exist")
                        .inner,
                ))?;
            }
        }
        if !self.analog_output_channels.is_empty() {
            unsafe {
                check_err(DAQmxStartTask(
                    self.analog_output_task
                        .as_ref()
                        .expect("task handle should exist")
                        .inner,
                ))?;
            }
        }
        if !self.digital_input_channels.is_empty() {
            unsafe {
                check_err(DAQmxStartTask(
                    self.digital_input_task
                        .as_ref()
                        .expect("task handle should exist")
                        .inner,
                ))?;
            }
        }
        if !self.digital_output_channels.is_empty() {
            unsafe {
                check_err(DAQmxStartTask(
                    self.digital_output_task
                        .as_ref()
                        .expect("task handle should exist")
                        .inner,
                ))?;
            }
        }

        Ok(Nidaq {
            analog_input_task: TaskHandleWrapper {
                inner: self
                    .analog_input_task
                    .as_ref()
                    .expect("read_analog_task_handle should be initialized during build step")
                    .inner,
            },
            analog_output_task: TaskHandleWrapper {
                inner: self
                    .analog_output_task
                    .as_ref()
                    .expect("write_analog_task_handle should be initialized during build step")
                    .inner,
            },
            digital_input_task: TaskHandleWrapper {
                inner: self
                    .digital_input_task
                    .as_ref()
                    .expect("write_digital_task_handle should be initialized during build step")
                    .inner,
            },
            digital_output_task: TaskHandleWrapper {
                inner: self
                    .digital_output_task
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

#[derive(Debug)]
pub struct Nidaq {
    analog_input_task: TaskHandleWrapper,
    analog_output_task: TaskHandleWrapper,
    digital_input_task: TaskHandleWrapper,
    digital_output_task: TaskHandleWrapper,
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
            DAQmxClearTask(self.analog_input_task.inner);
        }
        unsafe {
            DAQmxClearTask(self.analog_output_task.inner);
        }
        unsafe {
            DAQmxClearTask(self.digital_input_task.inner);
        }
        unsafe {
            DAQmxClearTask(self.digital_output_task.inner);
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
                self.analog_input_task.inner,
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
                self.analog_input_task.inner,
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

        info!(
            "Writing analog setpoint to nidaq: {:?} to task {:?}",
            analog, self.analog_output_task.inner
        );

        let mut written_samples_per_channel: i32 = 0;
        unsafe {
            let err = DAQmxWriteAnalogF64(
                self.analog_output_task.inner,
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

        info!("Writing digital setpoint to nidaq: {:?}", digital);
        unsafe {
            let err = DAQmxWriteDigitalU8(
                self.digital_output_task.inner,
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
            tracing::warn!(
                "nidaq returns warning code: {err} -> {}",
                c_str.to_string_lossy()
            );
        }
    }
    Ok(())
}

#[derive(Debug)]
/// Data read from the Nidaq
pub struct NidaqReading {
    pub analog: Array2<f64>,
    pub digital: Array2<u8>,
}

#[derive(Debug)]
/// Data written to the Nidaq
pub struct NidaqWriteData {
    pub analog: Array2<f64>,
    pub digital: Array2<u8>,
}
