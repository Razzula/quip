//! Qu MIDI protocol constants and message encoders.
//!
//! Defines protocol constants and provides functions for constructing valid
//! Qu MIDI, NRPN, and SysEx messages from typed protocol values.

use crate::{
    channels::{Channel, SendDestination},
    parameters::Parameter,
};

// -------------------------------------------------------------------------
// DISCOVERY PROTOCOL
// -------------------------------------------------------------------------
pub const DISCOVERY_PORT: u16 = 51320;
pub const DISCOVERY_MESSAGE: &[u8] = b"QU Find";

// -------------------------------------------------------------------------
// MIDI PROTOCOL
// -------------------------------------------------------------------------
pub const PROTOCOL_VERSION: [u8; 2] = [0x11, 0x01];

pub const TCP_PORT: u16 = 51325;

pub const ACTIVE_SENSE: u8 = 0xfe;

pub const MANUFACTURER_ID: [u8; 3] = [0x00, 0x00, 0x1a];
pub const QU_PRODUCT_ID: u8 = 0x50;
pub const QU16_BOX_ID: u8 = 0x01;

/// MIDI controller used for NRPN parameter MSB.
pub const CC_NRPN_MSB: u8 = 0x63;
/// MIDI controller used for NRPN parameter LSB.
pub const CC_NRPN_LSB: u8 = 0x62;
/// MIDI controller used for NRPN value MSB.
pub const CC_DATA_ENTRY_MSB: u8 = 0x06;
/// MIDI controller used for NRPN value LSB.
pub const CC_DATA_ENTRY_LSB: u8 = 0x26;

/// SysEx all-call channel.
pub const SYSEX_ALL_CALL: u8 = 0x7f;
/// SysEx command: Get System State.
pub const SYSEX_GET_SYSTEM_STATE: u8 = 0x10;
/// SysEx command: End Sync.
pub const SYSEX_END_SYNC: u8 = 0x14;
/// SysEx command: Get Channel Name.
pub const SYSEX_GET_CHANNEL_NAME: u8 = 0x01;
/// SysEx command: Channel Name response.
pub const SYSEX_CHANNEL_NAME: u8 = 0x02;
/// SysEx command: Set Channel Name.
pub const SYSEX_SET_CHANNEL_NAME: u8 = 0x03;

/// Standard Qu SysEx header.
///
/// The final byte is the MIDI channel. `0x7F` is used for
/// all-call messages.
pub const SYSEX_HEADER: [u8; 8] = [
    0xf0,
    MANUFACTURER_ID[0],
    MANUFACTURER_ID[1],
    MANUFACTURER_ID[2],
    QU_PRODUCT_ID,
    PROTOCOL_VERSION[0],
    PROTOCOL_VERSION[1],
    0x00,
];

pub fn sysex(body: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(8 + body.len());
    message.extend_from_slice(&SYSEX_HEADER);
    message.extend_from_slice(body);
    message
}

/// Get System State request for a normal MIDI connection.
pub fn get_system_state() -> Vec<u8> {
    sysex(&[
        SYSEX_ALL_CALL,
        SYSEX_GET_SYSTEM_STATE,
        0x00, // iPad flag
        0xf7,
    ])
}

/// End Sync message.
pub fn end_sync() -> Vec<u8> {
    sysex(&[
        0x00,
        SYSEX_END_SYNC,
        0xf7,
    ])
}

/// Encode a Get System State request.
pub fn get_system_state_with_ipad(ipad: bool) -> Vec<u8> {
    sysex(&[
        SYSEX_ALL_CALL,
        SYSEX_GET_SYSTEM_STATE,
        u8::from(ipad),
        0xf7,
    ])
}

/// Encode a System State response.
pub fn system_state_response(
    channel: u8,
    box_id: u8,
    firmware_major: u8,
    firmware_minor: u8,
) -> Vec<u8> {
    let mut message = Vec::with_capacity(13);

    message.extend_from_slice(&[
        0xf0,
        MANUFACTURER_ID[0],
        MANUFACTURER_ID[1],
        MANUFACTURER_ID[2],
        QU_PRODUCT_ID,
        PROTOCOL_VERSION[0],
        PROTOCOL_VERSION[1],
        0x00,
        channel & 0x0f,
        0x11,
        box_id & 0x7f,
        firmware_major & 0x7f,
        firmware_minor & 0x7f,
        0xf7,
    ]);

    message
}

/// Get Channel Name request.
pub fn get_channel_name(channel: Channel) -> Vec<u8> {
    sysex(&[
        0x00,
        SYSEX_GET_CHANNEL_NAME,
        channel.raw() & 0x7f,
        0xf7,
    ])
}

/// Channel Name response.
pub fn channel_name(channel: Channel, name: &[u8]) -> Vec<u8> {
    let mut message = sysex(&[
        0x00,
        SYSEX_CHANNEL_NAME,
        channel.raw() & 0x7f,
    ]);

    message.extend_from_slice(name);
    message.push(0xf7);
    message
}

/// Set Channel Name request.
pub fn set_channel_name(channel: Channel, name: &[u8]) -> Vec<u8> {
    let mut message = sysex(&[
        0x00, // MIDI channel
        SYSEX_SET_CHANNEL_NAME,
        channel.raw() & 0x7f,
    ]);

    message.extend_from_slice(name);
    message.push(0xf7);
    message
}

/// Encode a MIDI Control Change message.
pub const fn encode_cc(
    channel: u8,
    controller: u8,
    value: u8,
) -> [u8; 3] {
    [
        0xb0 | (channel & 0x0f),
        controller & 0x7f,
        value & 0x7f,
    ]
}

/// Encode a Qu NRPN message.
///
/// Qu's format is:
///
/// Bn 63 CH
/// Bn 62 ID
/// Bn 06 VA
/// Bn 26 VX
pub const fn encode_nrpn(
    midi_channel: u8,
    channel: Channel,
    parameter: Parameter,
    value: u8,
    index: u8,
) -> [u8; 12] {
    [
        0xb0 | (midi_channel & 0x0f),
        CC_NRPN_MSB,
        channel.raw() & 0x7f,

        0xb0 | (midi_channel & 0x0f),
        CC_NRPN_LSB,
        parameter.id() & 0x7f,

        0xb0 | (midi_channel & 0x0f),
        CC_DATA_ENTRY_MSB,
        value & 0x7f,

        0xb0 | (midi_channel & 0x0f),
        CC_DATA_ENTRY_LSB,
        index & 0x7f,
    ]
}

/// Encode a fader value.
pub const fn fader(
    channel: Channel,
    value: u8,
) -> [u8; 12] {
    encode_nrpn(
        0,
        channel,
        Parameter::Fader,
        value,
        0,
    )
}

/// Encode a send-level value.
pub const fn send_level(
    channel: Channel,
    destination: SendDestination,
    value: u8,
) -> [u8; 12] {
    encode_nrpn(
        0,
        channel,
        Parameter::SendLevel,
        value,
        destination.index(),
    )
}

/// Encode a pan value.
pub const fn pan(
    channel: Channel,
    destination: SendDestination,
    value: u8,
) -> [u8; 12] {
    encode_nrpn(
        0,
        channel,
        Parameter::Pan,
        value,
        destination.index(),
    )
}

/// Encode a generic Qu parameter.
pub const fn parameter(
    channel: Channel,
    parameter: Parameter,
    value: u8,
    index: u8,
) -> [u8; 12] {
    encode_nrpn(
        0,
        channel,
        parameter,
        value,
        index,
    )
}

/// Encode a mute message.
///
/// Qu uses Note On messages for mute:
/// `velocity >= 0x40` = on,
/// `velocity < 0x40` = off.
///
/// The corresponding Note Off message should normally follow.
pub const fn mute(channel: Channel, muted: bool) -> [u8; 6] {
    let velocity = if muted { 0x7f } else { 0x3f };
    let note = channel.raw();

    [0x90, note, velocity, 0x90, note, 0x00]
}

/// Encode a Note Off following a Qu mute message.
pub const fn mute_note_off(channel: Channel) -> [u8; 3] {
    [
        0x80,
        channel.raw() & 0x7f,
        0x00,
    ]
}

/// Encode a mute-group assignment.
pub const fn mute_group_assignment(
    channel: Channel,
    group: Channel,
    assigned: bool,
) -> [u8; 12] {
    let group_number = group.raw() & 0x03;

    let value = if assigned {
        0x40 | group_number
    } else {
        group_number
    };
    // let value = 0x40 | group_number; // DEBUG

    encode_nrpn(
        0,
        channel,
        Parameter::MuteGroupAssignment,
        value,
        0x07,
    )
}

/// Encode a Program Change message.
pub const fn program_change(
    channel: u8,
    program: u8,
) -> [u8; 2] {
    [
        0xc0 | (channel & 0x0f),
        program & 0x7f,
    ]
}

/// Encode Bank Select + Program Change for a Qu scene.
///
/// Bank 1 is:
///
/// Bn 00 00
/// Bn 20 00
/// Cn SS
///
/// where `scene` is 0..=99 for scenes 1..=100.
pub const fn scene_recall(scene: u8) -> [u8; 8] {
    [
        0xb0,
        0x00,
        0x00,

        0xb0,
        0x20,
        0x00,

        0xc0,
        scene & 0x7f,
    ]
}
