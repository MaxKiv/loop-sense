pub type ChannelName = String;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ChannelType {
    Input,
    Output,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SignalType {
    Analog,
    Digital,
}

#[derive(Debug, Clone)]
/// Generic Nidaq channel
pub struct Channel {
    pub physical_channel: String,
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct ReadChannel(pub Channel);

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct WriteChannel(pub Channel);

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct AnalogOutputChannel {
    pub channel: Channel,
    pub min: f64,
    pub max: f64,
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct AnalogInputChannel {
    pub channel: Channel,
    pub min: f64,
    pub max: f64,
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct DigitalOutputChannel {
    pub channel: Channel,
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct DigitalInputChannel {
    pub channel: Channel,
}
