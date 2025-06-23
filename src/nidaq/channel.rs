pub type ChannelName = String;

#[derive(Debug, Eq, PartialEq, Clone)]
enum ChannelType {
    Input,
    Output,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum SignalType {
    Analog,
    Digital,
}

#[derive(Debug, Clone)]
/// Generic Nidaq channel
pub struct Channel {
    pub name: ChannelName,
    pub physical_channel: String,
    pub signal_type: SignalType,
    pub min: f64,
    pub max: f64,
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct ReadChannel(Channel);

impl ReadChannel {
    fn try_new(
        name: ChannelName,
        physical_channel: String,
        signal_type: SignalType,
        channel_type: ChannelType,
    ) -> Option<Self> {
        if channel_type == ChannelType::Input {
            Some(Self(Channel {
                name,
                physical_channel,
                signal_type,
                channel_type,
            }))
        } else {
            None
        }
    }
}

/// Nidaq read channel
#[derive(Debug, Clone)]
pub struct WriteChannel(pub Channel);

impl WriteChannel {
    fn try_new(
        name: ChannelName,
        physical_channel: String,
        signal_type: SignalType,
        channel_type: ChannelType,
    ) -> Option<Self> {
        if channel_type == ChannelType::Output {
            Some(Self(Channel {
                name,
                physical_channel,
                signal_type,
                channel_type,
            }))
        } else {
            None
        }
    }
}
