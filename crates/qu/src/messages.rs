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
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },

    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    ProgramChange {
        channel: u8,
        program: u8,
    },

    SysEx(Vec<u8>),

    ActiveSense,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuEvent {
    ActiveSense,

    Mute {
        channel: Channel,
        muted: bool,
    },

    Fader {
        channel: Channel,
        value: u8,
    },

    Pan {
        channel: Channel,
        destination: SendDestination,
        value: u8,
    },

    SendLevel {
        channel: Channel,
        destination: SendDestination,
        value: u8,
    },

    Pafl {
        channel: Channel,
        enabled: bool,
    },

    PhantomPower {
        channel: Channel,
        enabled: bool,
    },

    LRAssign {
        channel: Channel,
        enabled: bool,
    },

    Polarity {
        channel: Channel,
        reversed: bool,
    },

    Parameter {
        channel: Channel,
        parameter: Parameter,
        value: u8,
        index: u8,
    },

    SysEx(Vec<u8>),

    ProgramChange {
        channel: u8,
        program: u8,
    },

    Midi(MidiMessage),

    Unknown(Vec<u8>),
}

impl QuEvent {
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

pub fn hex(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn db_value(value: u8) -> String {
    let db = fader_to_db(value);

    if db.is_infinite() {
        "-inf dB".into()
    } else {
        format!("{db:.1} dB")
    }
}
