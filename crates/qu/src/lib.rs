//! Qu Mixer MIDI Protocol (Firmware V1.9+) Library.
//!
//! This crate is transport-agnostic and does not perform any networking.

pub mod channels;
pub mod messages;
pub mod parameters;
pub mod parser;
pub mod protocol;

pub use channels::{Channel, SendDestination};
pub use messages::{MidiMessage, QuEvent};
pub use parameters::Parameter;
pub use parser::Parser;
pub use protocol::{
    get_system_state,
    encode_cc,
    encode_nrpn,
};
