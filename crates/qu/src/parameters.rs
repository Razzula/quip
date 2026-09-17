//! Qu mixer parameter identifiers.
//!
//! Defines the parameters exposed by the Qu MIDI protocol, including their
//! numeric protocol IDs and conversion between IDs and typed parameters.

use strum_macros::EnumIter;

/// Identifies a parameter exposed by the Qu MIDI protocol.
///
/// Each variant corresponds to a numeric parameter ID used in the Qu MIDI protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[repr(u8)]
pub enum Parameter {
    LfEqGain = 0x01,
    LfEqFrequency = 0x02,
    LfEqWidth = 0x03,
    LfEqType = 0x04,

    LmEqGain = 0x05,
    LmEqFrequency = 0x06,
    LmEqWidth = 0x07,

    HmEqGain = 0x09,
    HmEqFrequency = 0x0a,
    HmEqWidth = 0x0b,

    HfEqGain = 0x0d,
    HfEqFrequency = 0x0e,
    HfEqWidth = 0x0f,
    HfEqType = 0x10,

    PeqInOut = 0x11,
    UsbSource = 0x12,

    HpfFrequency = 0x13,
    HpfInOut = 0x14,

    Pan = 0x16,
    Fader = 0x17,
    LRAssign = 0x18,
    LocalGain = 0x19,

    SendLevel = 0x20,

    DcaAssignment = 0x40,

    GateAttack = 0x41,
    GateRelease = 0x42,
    GateHold = 0x43,
    GateThreshold = 0x44,
    GateDepth = 0x45,
    GateInOut = 0x46,

    FxDelayTimeMsb = 0x48,
    FxDelayTimeLsb = 0x49,

    DelayTime = 0x4c,
    DelayInOut = 0x4d,

    MixPrePost = 0x50,
    Pafl = 0x51,
    DigitalTrim = 0x52,
    StereoTrim = 0x54,

    MixAssignment = 0x55,

    PreampSource = 0x57,
    DsnakeGain = 0x58,
    DsnakePad = 0x59,
    Dsnake48v = 0x5a,

    MuteGroupAssignment = 0x5c,
    DsnakePatch = 0x5d,
    GroupMixMode = 0x5e,

    RemoteShutdown = 0x5f,

    CompressorType = 0x61,
    CompressorAttack = 0x62,
    CompressorRelease = 0x63,
    CompressorKnee = 0x64,
    CompressorRatio = 0x65,
    CompressorThreshold = 0x66,
    CompressorGain = 0x67,
    CompressorInOut = 0x68,

    Local48v = 0x69,
    Polarity = 0x6a,
    InsertInOut = 0x6b,

    DelayTime2 = 0x6c,
    DelayInOut2 = 0x6d,

    GeqGain = 0x70,
    GeqInOut = 0x71,
}

impl Parameter {
    /// Returns the numeric protocol ID for this parameter.
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// Converts a Qu MIDI parameter ID into its corresponding [`Parameter`].
    pub const fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0x01 => Self::LfEqGain,
            0x02 => Self::LfEqFrequency,
            0x03 => Self::LfEqWidth,
            0x04 => Self::LfEqType,

            0x05 => Self::LmEqGain,
            0x06 => Self::LmEqFrequency,
            0x07 => Self::LmEqWidth,

            0x09 => Self::HmEqGain,
            0x0a => Self::HmEqFrequency,
            0x0b => Self::HmEqWidth,

            0x0d => Self::HfEqGain,
            0x0e => Self::HfEqFrequency,
            0x0f => Self::HfEqWidth,
            0x10 => Self::HfEqType,

            0x11 => Self::PeqInOut,
            0x12 => Self::UsbSource,

            0x13 => Self::HpfFrequency,
            0x14 => Self::HpfInOut,

            0x16 => Self::Pan,
            0x17 => Self::Fader,
            0x18 => Self::LRAssign,
            0x19 => Self::LocalGain,

            0x20 => Self::SendLevel,

            0x40 => Self::DcaAssignment,

            0x41 => Self::GateAttack,
            0x42 => Self::GateRelease,
            0x43 => Self::GateHold,
            0x44 => Self::GateThreshold,
            0x45 => Self::GateDepth,
            0x46 => Self::GateInOut,

            0x48 => Self::FxDelayTimeMsb,
            0x49 => Self::FxDelayTimeLsb,

            0x4c => Self::DelayTime,
            0x4d => Self::DelayInOut,

            0x50 => Self::MixPrePost,
            0x51 => Self::Pafl,
            0x52 => Self::DigitalTrim,
            0x54 => Self::StereoTrim,

            0x55 => Self::MixAssignment,

            0x57 => Self::PreampSource,
            0x58 => Self::DsnakeGain,
            0x59 => Self::DsnakePad,
            0x5a => Self::Dsnake48v,

            0x5c => Self::MuteGroupAssignment,
            0x5d => Self::DsnakePatch,
            0x5e => Self::GroupMixMode,

            0x5f => Self::RemoteShutdown,

            0x61 => Self::CompressorType,
            0x62 => Self::CompressorAttack,
            0x63 => Self::CompressorRelease,
            0x64 => Self::CompressorKnee,
            0x65 => Self::CompressorRatio,
            0x66 => Self::CompressorThreshold,
            0x67 => Self::CompressorGain,
            0x68 => Self::CompressorInOut,

            0x69 => Self::Local48v,
            0x6a => Self::Polarity,
            0x6b => Self::InsertInOut,

            0x6c => Self::DelayTime2,
            0x6d => Self::DelayInOut2,

            0x70 => Self::GeqGain,
            0x71 => Self::GeqInOut,

            _ => return None,
        })
    }
}
