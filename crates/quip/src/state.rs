use qu::channels::Channel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    pub inputs: [ChannelState; 16],
    pub stereo: [ChannelState; 3],
    pub mixes: [ChannelState; 8],
    pub mute_groups: [MuteGroupState; 4],
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuteGroupState {
    pub name: String,
    pub muted: bool,
    pub channels: Vec<ChannelRef>,
}

impl MuteGroupState {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            muted: false,
            channels: Vec::new(),
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
            mute_groups: [
                MuteGroupState::new("MG1"),
                MuteGroupState::new("MG2"),
                MuteGroupState::new("MG3"),
                MuteGroupState::new("MG4"),
            ],
        }
    }
}

impl MixerState {

    // -------------------------------------------------------------------------
    // CHANNEL
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
        if let Some(state) = self.channel(channel) {
            return Some(state.muted);
        }

        if let Some(channel) = ChannelRef::from_channel(channel) {
            if let Some(state) = self.mute_group(channel) {
                return Some(state.muted);
            }
        }

        None
    }

    pub fn set_muted(&mut self, channel: Channel, muted: bool) -> bool {
        if let Some(state) = self.channel_mut(channel) {
            state.muted = muted;
            return true;
        }

        if let Some(channel) = ChannelRef::from_channel(channel) {
            if let Some(state) = self.mute_group_mut(channel) {
                state.muted = muted;
                return true;
            }
        }

        false
    }

    // -------------------------------------------------------------------------
    // Name
    // -------------------------------------------------------------------------

    pub fn name(&self, channel: Channel) -> Option<String> {
        if let Some(state) = self.channel(channel) {
            return Some(state.name.clone());
        }

        if let Some(channel) = ChannelRef::from_channel(channel) {
            if let Some(state) = self.mute_group(channel) {
                return Some(state.name.clone());
            }
        }

        None
    }

    pub fn set_name(&mut self, channel: Channel, name: String) -> bool {
        if let Some(state) = self.channel_mut(channel) {
            state.name = name;
            return true;
        }

        if let Some(channel) = ChannelRef::from_channel(channel) {
            if let Some(state) = self.mute_group_mut(channel) {
                state.name = name;
                return true;
            }
        }

        false
    }

    // -------------------------------------------------------------------------
    // MUTE GROUPS
    // -------------------------------------------------------------------------

    pub fn mute_group(&self, channel: ChannelRef) -> Option<&MuteGroupState> {
        match channel {
            ChannelRef::MuteGroup(number) => {
                self.mute_groups.get((number - 1) as usize)
            }
            _ => None,
        }
    }

    pub fn mute_group_mut(
        &mut self,
        channel: ChannelRef,
    ) -> Option<&mut MuteGroupState> {
        match channel {
            ChannelRef::MuteGroup(number) => {
                self.mute_groups.get_mut((number - 1) as usize)
            }
            _ => None,
        }
    }

    pub fn mute_group_muted(&self, channel: ChannelRef) -> Option<bool> {
        self.mute_group(channel).map(|group| group.muted)
    }

    pub fn set_mute_group_muted(
        &mut self,
        channel: ChannelRef,
        muted: bool,
    ) -> bool {
        let Some(group) = self.mute_group_mut(channel) else {
            return false;
        };

        group.muted = muted;
        true
    }

    pub fn mute_group_assignments(
        &self,
        channel: ChannelRef,
    ) -> Option<&[ChannelRef]> {
        self.mute_group(channel)
            .map(|group| group.channels.as_slice())
    }

    pub fn set_mute_group_assignment(
        &mut self,
        group: ChannelRef,
        channel: ChannelRef,
        assigned: bool,
    ) -> bool {
        let Some(group) = self.mute_group_mut(group) else {
            return false;
        };

        if assigned {
            if !group.channels.contains(&channel) {
                group.channels.push(channel);
            }
        } else {
            group.channels.retain(|current| *current != channel);
        }

        true
    }
}

// TODO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "number")]
pub enum ChannelRef {
    Input(u8),
    Stereo(u8),
    Mix(u8),
    Lr,
    MuteGroup(u8),
}

impl ChannelRef {
    pub fn channel(self) -> Option<qu::channels::Channel> {
        use qu::channels::Channel;

        match self {
            Self::Input(number) => Channel::input(number),
            Self::Stereo(number) => Channel::stereo(number),
            Self::Mix(number) => Channel::mix(number),
            Self::Lr => Some(Channel::lr()),
            Self::MuteGroup(number) => Channel::mute_group(number),
        }
    }

    pub fn from_channel(channel: qu::channels::Channel) -> Option<Self> {
        use qu::channels::Channel;

        // CH
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(Self::Input(number));
            }
        }

        // ST
        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(Self::Stereo(number));
            }
        }

        // MIX
        for number in 1..=8 {
            if Channel::mix(number) == Some(channel) {
                return Some(Self::Mix(number));
            }
        }
        if Channel::lr() == channel {
            return Some(Self::Lr);
        }

        // MG
        for number in 1..=4 {
            if Channel::mute_group(number) == Some(channel) {
                return Some(Self::MuteGroup(number));
            }
        }

        None
    }
}
