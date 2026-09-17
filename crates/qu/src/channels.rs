//! Qu mixer channel and routing identifiers.
//!
//! Defines the channel, strip, and send-destination identifiers used by the
//! Qu MIDI protocol, providing typed representations of the protocol's
//! numeric channel IDs.

use std::fmt;

/// A Qu MIDI channel/strip identifier.
/// These values are the CH values used by the Qu MIDI protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel(pub u8);

/// Identifies a channel, strip, group, or other controllable signal path (CH)
/// in the Qu MIDI protocol.
///
/// The wrapped value is the protocol's numeric channel identifier.
impl Channel {
    // Inputs (CH)
    // 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, ...
    pub fn input(number: u8) -> Option<Self> {
        if (1..=32).contains(&number) {
            Some(Self(0x20 + number - 1))
        }
        else {
            None
        }
    }

    // Steroes (ST)
    // 1, 2, 3
    pub fn stereo(number: u8) -> Option<Self> {
        if (1..=3).contains(&number) {
            Some(Self(0x40 + number - 1))
        }
        else {
            None
        }
    }

    // Mute Groups (MG)
    // 1, 2, 3, 4
    pub fn mute_group(number: u8) -> Option<Self> {
        if (1..=4).contains(&number) {
            Some(Self(0x50 + number - 1))
        }
        else {
            None
        }
    }

    // Mixes (MIX)
    // 1, 2, 3, 4, 5-6, 7-8, 9-10, LR
    pub fn mix(number: u8) -> Option<Self> {
        if (1..=8).contains(&number) {
            Some(Self(0x60 + number - 1))
        }
        else {
            None
        }
    }

    // Main LR
    // NB. This is actually just the 8th MIX,
    // but is treated as a special case.
    pub fn lr() -> Self {
        Self(0x67)
    }

    // Groups (MIX) [GRP]
    // 1-2, 3-4, 5-6, 7-8
    // NB. these are technically also mixes
    pub fn group(number: u8) -> Option<Self> {
        if (1..=4).contains(&number) {
            Some(Self(0x68 + number - 1))
        }
        else {
            None
        }
    }

    // MATRIX (MIX) [MT]
    // 1-2, 3-4
    // NB. these are technically also mixes
    // NB. NOT QU-16
    pub fn matrix(number: u8) -> Option<Self> {
        match number {
            1 => Some(Self(0x6c)),
            2 => Some(Self(0x6d)),
            _ => None,
        }
    }

    // DCA Groups (DG)
    // 1, 2, 3, 4
    pub fn dca(number: u8) -> Option<Self> {
        if (1..=4).contains(&number) {
            Some(Self(0x10 + number - 1))
        }
        else {
            None
        }
    }

    // FX Send (CH)
    // 1, 2, 3, 4
    pub fn fx_send(number: u8) -> Option<Self> {
        if (1..=4).contains(&number) {
            Some(Self(0x00 + number - 1))
        }
        else {
            None
        }
    }

    // FX Return (CH)
    // 1, 2, 3, 4
    pub fn fx_return(number: u8) -> Option<Self> {
        if (1..=4).contains(&number) {
            Some(Self(0x08 + number - 1))
        }
        else {
            None
        }
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub fn from_raw(value: u8) -> Self {
        Self(value & 0x7f)
    }
}

/// Formats the channel using its human-readable Qu designation.
///
/// Known protocol ranges are rendered using their corresponding names,
/// such as `CH1`, `ST1`, `Mix 1`, ...
/// Unrecognised channel type: raw hexadecimal identifier.
impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0x00..=0x03 => write!(f, "FX Send {}", self.0 + 1),
            0x08..=0x0b => write!(f, "FX Return {}", self.0 - 0x08 + 1),
            0x10..=0x13 => write!(f, "DCA {}", self.0 - 0x10 + 1),
            0x20..=0x3f => write!(f, "CH{}", self.0 - 0x20 + 1),
            0x40..=0x42 => write!(f, "ST{}", self.0 - 0x40 + 1),
            0x50..=0x53 => write!(f, "Mute Group {}", self.0 - 0x50 + 1),
            0x60..=0x66 => write!(f, "Mix {}", self.0 - 0x60 + 1),
            0x67 => write!(f, "LR"),
            0x68..=0x6b => write!(f, "Group {}", self.0 - 0x68 + 1),
            0x6c => write!(f, "Matrix 1-2"),
            0x6d => write!(f, "Matrix 3-4"),
            _ => write!(f, "Channel 0x{:02X}", self.0),
        }
    }
}

/// Identifies a possible destination for a Qu channel send.
///
/// Each variant corresponds to the destination index used by the Qu MIDI
/// protocol. [`SendDestination::Unknown`] preserves unrecognised protocol
/// values so that they can be handled without losing the original index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendDestination {
    Mix1,
    Mix2,
    Mix3,
    Mix4,
    Mix5_6,
    Mix7_8,
    Mix9_10,
    Lr,
    Group1_2,
    Group3_4,
    Group5_6,
    Group7_8,
    Matrix1_2,
    Matrix3_4,
    FxSend1,
    FxSend2,
    FxSend3,
    FxSend4,
    Unknown(u8),
}

impl SendDestination {
    /// Converts a Qu send-destination (VX) index into its typed representation.
    ///
    /// Known protocol indices are mapped to their corresponding destination.
    /// Unrecognised indices are preserved as [`SendDestination::Unknown`].
    pub fn from_index(index: u8) -> Self {
        match index {
            0x00 => Self::Mix1,
            0x01 => Self::Mix2,
            0x02 => Self::Mix3,
            0x03 => Self::Mix4,
            0x04 => Self::Mix5_6,
            0x05 => Self::Mix7_8,
            0x06 => Self::Mix9_10,
            0x07 => Self::Lr,
            0x08 => Self::Group1_2,
            0x09 => Self::Group3_4,
            0x0a => Self::Group5_6,
            0x0b => Self::Group7_8,
            0x0c => Self::Matrix1_2,
            0x0d => Self::Matrix3_4,
            0x10 => Self::FxSend1,
            0x11 => Self::FxSend2,
            0x12 => Self::FxSend3,
            0x13 => Self::FxSend4,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw protocol index for the send destination.
    ///
    /// For [`SendDestination::Unknown`], the original unrecognised index is
    /// returned unchanged.
    pub const fn index(self) -> u8 {
        match self {
            Self::Mix1 => 0x00,
            Self::Mix2 => 0x01,
            Self::Mix3 => 0x02,
            Self::Mix4 => 0x03,
            Self::Mix5_6 => 0x04,
            Self::Mix7_8 => 0x05,
            Self::Mix9_10 => 0x06,
            Self::Lr => 0x07,
            Self::Group1_2 => 0x08,
            Self::Group3_4 => 0x09,
            Self::Group5_6 => 0x0a,
            Self::Group7_8 => 0x0b,
            Self::Matrix1_2 => 0x0c,
            Self::Matrix3_4 => 0x0d,
            Self::FxSend1 => 0x10,
            Self::FxSend2 => 0x11,
            Self::FxSend3 => 0x12,
            Self::FxSend4 => 0x13,
            Self::Unknown(value) => value,
        }
    }
}

/// Formats the send destination using its human-readable Qu designation.
impl fmt::Display for SendDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Mix1 => "Mix 1",
            Self::Mix2 => "Mix 2",
            Self::Mix3 => "Mix 3",
            Self::Mix4 => "Mix 4",
            Self::Mix5_6 => "Mix 5-6",
            Self::Mix7_8 => "Mix 7-8",
            Self::Mix9_10 => "Mix 9-10",
            Self::Lr => "LR",
            Self::Group1_2 => "Group 1-2",
            Self::Group3_4 => "Group 3-4",
            Self::Group5_6 => "Group 5-6",
            Self::Group7_8 => "Group 7-8",
            Self::Matrix1_2 => "Matrix 1-2",
            Self::Matrix3_4 => "Matrix 3-4",
            Self::FxSend1 => "FX Send 1",
            Self::FxSend2 => "FX Send 2",
            Self::FxSend3 => "FX Send 3",
            Self::FxSend4 => "FX Send 4",
            Self::Unknown(value) => return write!(f, "Index 0x{value:02X}"),
        };

        write!(f, "{text}")
    }
}
