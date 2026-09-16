//! Qu Mixer LAN (TCP) Communication Library.
//!
//! Provides reusable client and server components for communicating with Qu
//! mixers over their TCP-based MIDI connection.

pub mod client;
pub mod server;
pub mod state;
pub mod qu_from_state;
pub mod qu_to_state;

pub use client::tcp::{Quip, QuipError};
pub use server::tcp;
