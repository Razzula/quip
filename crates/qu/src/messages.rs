//! MIDI messages and high-level Qu events.
//!
//! Defines representations of raw MIDI messages and semantic events produced
//! by or sent to a Qu mixer, providing a convenient abstraction over the
//! protocol's byte-level representation.

use crate::{
    channels::{Channel, SendDestination},
    parameters::Parameter,
    faders::fader_to_db,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiMessage {
    /// Represents a MIDI Control Change message.
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },

    /// Represents a MIDI Note On message.
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    /// Represents a MIDI Note Off message.
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    /// Represents a MIDI Program Change message.
    ProgramChange {
        channel: u8,
        program: u8,
    },

    /// Represents a MIDI System Exclusive message.
    SysEx(Vec<u8>),

    /// Represents a MIDI Active Sense message.
    ActiveSense,
}

/// Represents a semantic event produced by or sent to a Qu mixer.
///
/// `QuEvent` provides a higher-level representation of Qu's MIDI protocol by
/// interpreting raw MIDI messages as mixer operations where possible.
/// Messages or operations that do not have a dedicated representation are
/// retained as generic MIDI, SysEx, or unknown events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuEvent {
    /// Represents a MIDI Active Sense message.
    ActiveSense,

    /// Represents a mixer channel mute state change.
    Mute {
        channel: Channel,
        muted: bool,
    },

    /// Represents a mixer channel fader position change.
    Fader {
        channel: Channel,
        value: u8, // raw 7-bit fader value
    },

    /// Represents a channel pan position for a specific send destination.
    Pan {
        channel: Channel,
        destination: SendDestination,
        value: u8, // raw pan position
    },

    /// Represents a channel's send level to a specific destination.
    SendLevel {
        channel: Channel,
        destination: SendDestination,
        value: u8, // raw 7-bit level
    },

    /// Represents a channel's PAFL (Pre/Post Fade Listen) state.
    Pafl {
        channel: Channel,
        enabled: bool,
    },

    /// Represents a channel's phantom power state.
    PhantomPower {
        channel: Channel,
        enabled: bool,
    },

    /// Represents whether a channel is assigned to the main LR bus.
    LRAssign {
        channel: Channel,
        enabled: bool,
    },

    /// Represents a channel's polarity state.
    Polarity {
        channel: Channel,
        reversed: bool, // is inverted?
    },

    /// Represents a generic Qu mixer parameter change.
    ///
    /// The parameter is identified separately from the channel, while the value
    /// and index preserve the raw protocol values used by the corresponding
    /// parameter.
    Parameter {
        channel: Channel,
        parameter: Parameter,
        value: u8,
        index: u8,
    },

    /// Represents a raw MIDI System Exclusive event from the Qu mixer.
    SysEx(Vec<u8>),

    /// Represents a MIDI Program Change event.
    ProgramChange {
        channel: u8,
        program: u8,
    },

    /// Represents a MIDI message that has not been interpreted as a Qu-specific event.
    Midi(MidiMessage),

    /// Represents an unrecognised sequence of MIDI or protocol bytes.
    ///
    /// The original bytes are preserved so that unsupported or unknown messages
    /// can still be inspected and handled by higher layers.
    Unknown(Vec<u8>),
}

impl QuEvent {
    /// Returns a human-readable description of the event.
    ///
    /// The description is intended for logging and diagnostics rather than for
    /// machine-readable protocol communication.
    pub fn describe(&self) -> String {
        match self {
            Self::ActiveSense => "Active Sense".to_string(),

            Self::Fader { channel, value } => {
                format!(
                    "{channel} Fader: {} (MIDI {value:02X})",
                    db_value(*value)
                )
            }

            Self::SendLevel {
                channel,
                destination,
                value,
            } => {
                format!(
                    "{channel} {destination} send: {} (MIDI {value:02X})",
                    db_value(*value)
                )
            }

            Self::Pan {
                channel,
                destination,
                value,
            } => {
                let position = match value {
                    0x00 => "full left".to_string(),
                    0x25 => "centre".to_string(),
                    0x4a => "full right".to_string(),
                    value => format!("raw {value:02X}"),
                };

                format!("{channel} Pan ({destination}): {position}")
            }

            Self::Mute { channel, muted } => {
                format!(
                    "{channel} MUTE: {}",
                    if *muted { "ON" } else { "OFF" }
                )
            }

            Self::Pafl { channel, enabled } => {
                format!(
                    "{channel} PAFL: {}",
                    if *enabled { "ON" } else { "OFF" }
                )
            }

            Self::PhantomPower { channel, enabled } => {
                format!(
                    "{channel} Phantom Power: {}",
                    if *enabled { "ON" } else { "OFF" }
                )
            }

            Self::LRAssign { channel, enabled } => {
                format!(
                    "{channel} LR Assign: {}",
                    if *enabled { "ON" } else { "OFF" }
                )
            }

            Self::Polarity { channel, reversed } => {
                format!(
                    "{channel} Polarity: {}",
                    if *reversed {
                        "REVERSED"
                    } else {
                        "normal"
                    }
                )
            }

            Self::Parameter {
                channel,
                parameter,
                value,
                index,
            } => {
                format!(
                    "{channel} {parameter:?}: value={value:02X} index={index:02X}"
                )
            }

            Self::SysEx(data) => {
                format!("SysEx: {}", hex(data))
            }

            Self::ProgramChange { channel, program } => {
                format!(
                    "Program Change: channel={} program={program}",
                    channel + 1
                )
            }

            Self::Midi(message) => {
                format!("{message:?}")
            }

            Self::Unknown(data) => {
                format!("Unknown: {}", hex(data))
            }
        }
    }
}

/// Formats a byte slice as a space-separated hexadecimal string.
pub fn hex(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Converts a raw Qu fader value into a human-readable dB representation.
///
/// The special Qu value representing negative infinity is displayed as
/// `-inf dB`; finite values are formatted to one decimal place.
fn db_value(value: u8) -> String {
    let db = fader_to_db(value);

    if db.is_infinite() {
        "-inf dB".into()
    } else {
        format!("{db:.1} dB")
    }
}
