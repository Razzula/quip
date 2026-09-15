//! Translation between Quip's application state and the Qu MIDI protocol.

use qu::{
    channels::Channel,
    parameters::{db_to_fader},
    protocol,
};

use crate::state::MixerState;

pub fn state(state: &MixerState) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();

    for number in 1..=16 {
        if let Some(channel) = Channel::input(number) {
            if let Some(message) = fader(state, channel) {
                messages.push(message);
            }

            if let Some(message) = mute(state, channel) {
                messages.push(message);
            }
        }
    }

    for number in 1..=3 {
        if let Some(channel) = Channel::stereo(number) {
            if let Some(message) = fader(state, channel) {
                messages.push(message);
            }

            if let Some(message) = mute(state, channel) {
                messages.push(message);
            }
        }
    }

    for number in 1..=7 {
        if let Some(channel) = Channel::mix(number) {
            if let Some(message) = fader(state, channel) {
                messages.push(message);
            }

            if let Some(message) = mute(state, channel) {
                messages.push(message);
            }
        }
    }

    let channel = Channel::lr();

    if let Some(message) = fader(state, channel) {
        messages.push(message);
    }

    if let Some(message) = mute(state, channel) {
        messages.push(message);
    }

    messages
}

pub fn fader(state: &MixerState, channel: Channel) -> Option<Vec<u8>> {
    let value = state.fader(channel)?;

    Some(protocol::fader(
        channel,
        db_to_fader(value),
    ).to_vec())
}

pub fn mute(state: &MixerState, channel: Channel) -> Option<Vec<u8>> {
    let muted = state.muted(channel)?;

    Some(protocol::mute(channel, muted).to_vec())
}