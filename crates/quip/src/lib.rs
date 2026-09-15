//! Qu Mixer LAN (TCP) Communication Library.
//!
//! Provides reusable client and server components for communicating with Qu
//! mixers over their TCP-based MIDI connection.

mod client;
pub mod server;

pub use client::{Quip, QuipError};
pub use server::Server;
