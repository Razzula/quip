use qu::channels::Channel;
use qu::parameters::db_to_fader;

#[derive(Debug, Clone)]
pub struct MixerState {
    inputs: [ChannelState; 16],
    stereo: [ChannelState; 3],
    mixes: [ChannelState; 8],
}

#[derive(Debug, Clone, Copy)]
struct ChannelState {
    fader: u8,
    muted: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            fader: db_to_fader(0.0), // 0 dB
            muted: false,
        }
    }
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            inputs: [ChannelState::default(); 16],
            stereo: [ChannelState::default(); 3],
            mixes: [ChannelState::default(); 8],
        }
    }
}

impl MixerState {
    // -------------------------------------------------------------------------
    // Fader
    // -------------------------------------------------------------------------

    pub fn fader(&self, channel: Channel) -> Option<u8> {
        self.channel(channel).map(|state| state.fader)
    }

    pub fn set_fader(&mut self, channel: Channel, value: u8) -> bool {
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
    //
    // The storage layout is deliberately hidden from callers. Channel
    // identifiers are resolved through the constructors provided by `qu`.
    // -------------------------------------------------------------------------

    fn channel(&self, channel: Channel) -> Option<&ChannelState> {
        // Input 1-16
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(&self.inputs[(number - 1) as usize]);
            }
        }

        // Stereo 1-3
        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(&self.stereo[(number - 1) as usize]);
            }
        }

        // Mix 1, 2, 3, 4, 5-6, 7-8, 9-10
        for number in 1..=7 {
            if Channel::mix(number) == Some(channel) {
                return Some(&self.mixes[(number - 1) as usize]);
            }
        }

        // Main LR
        if channel == Channel::lr() {
            return Some(&self.mixes[7]);
        }

        None
    }

    fn channel_mut(&mut self, channel: Channel) -> Option<&mut ChannelState> {
        // Input 1-16
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(&mut self.inputs[(number - 1) as usize]);
            }
        }

        // Stereo 1-3
        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(&mut self.stereo[(number - 1) as usize]);
            }
        }

        // Mix 1, 2, 3, 4, 5-6, 7-8, 9-10
        for number in 1..=7 {
            if Channel::mix(number) == Some(channel) {
                return Some(&mut self.mixes[(number - 1) as usize]);
            }
        }

        // Main LR
        if channel == Channel::lr() {
            return Some(&mut self.mixes[7]);
        }

        None
    }
}
