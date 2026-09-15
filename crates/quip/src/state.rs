use qu::channels::Channel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    pub inputs: [ChannelState; 16],
    pub stereo: [ChannelState; 3],
    pub mixes: [ChannelState; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub name: String,
    pub fader: f32,
    pub muted: bool,
}

impl ChannelState {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            fader: 0.0,
            muted: false,
        }
    }
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            inputs: [
                ChannelState::new("CH1"),
                ChannelState::new("CH2"),
                ChannelState::new("CH3"),
                ChannelState::new("CH4"),
                ChannelState::new("CH5"),
                ChannelState::new("CH6"),
                ChannelState::new("CH7"),
                ChannelState::new("CH8"),
                ChannelState::new("CH9"),
                ChannelState::new("CH10"),
                ChannelState::new("CH11"),
                ChannelState::new("CH12"),
                ChannelState::new("CH13"),
                ChannelState::new("CH14"),
                ChannelState::new("CH15"),
                ChannelState::new("CH16"),
            ],
            stereo: [
                ChannelState::new("ST1"),
                ChannelState::new("ST2"),
                ChannelState::new("ST3"),
            ],
            mixes: [
                ChannelState::new("MIX1"),
                ChannelState::new("MIX2"),
                ChannelState::new("MIX3"),
                ChannelState::new("MIX4"),
                ChannelState::new("MIX5-6"),
                ChannelState::new("MIX7-8"),
                ChannelState::new("MIX9-10"),
                ChannelState::new("LR"),
            ],
        }
    }
}

impl MixerState {
    // -------------------------------------------------------------------------
    // Fader
    // -------------------------------------------------------------------------

    pub fn fader(&self, channel: Channel) -> Option<f32> {
        self.channel(channel).map(|state| state.fader)
    }

    pub fn set_fader(&mut self, channel: Channel, value: f32) -> bool {
        let Some(state) = self.channel_mut(channel) else {
            return false;
        };

        state.fader = value;
        true
    }

    // -------------------------------------------------------------------------
    // Mute
    // -------------------------------------------------------------------------

    pub fn muted(&self, channel: Channel) -> Option<bool> {
        self.channel(channel).map(|state| state.muted)
    }

    pub fn set_muted(&mut self, channel: Channel, muted: bool) -> bool {
        let Some(state) = self.channel_mut(channel) else {
            return false;
        };

        state.muted = muted;
        true
    }

    // -------------------------------------------------------------------------
    // Channel routing
    // -------------------------------------------------------------------------

    fn channel(&self, channel: Channel) -> Option<&ChannelState> {
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(&self.inputs[(number - 1) as usize]);
            }
        }

        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(&self.stereo[(number - 1) as usize]);
            }
        }

        for number in 1..=7 {
            if Channel::mix(number) == Some(channel) {
                return Some(&self.mixes[(number - 1) as usize]);
            }
        }

        if channel == Channel::lr() {
            return Some(&self.mixes[7]);
        }

        None
    }

    fn channel_mut(&mut self, channel: Channel) -> Option<&mut ChannelState> {
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(&mut self.inputs[(number - 1) as usize]);
            }
        }

        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(&mut self.stereo[(number - 1) as usize]);
            }
        }

        for number in 1..=7 {
            if Channel::mix(number) == Some(channel) {
                return Some(&mut self.mixes[(number - 1) as usize]);
            }
        }

        if channel == Channel::lr() {
            return Some(&mut self.mixes[7]);
        }

        None
    }
}
