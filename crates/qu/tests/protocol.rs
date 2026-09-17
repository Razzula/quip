use qu::{
    channels::{Channel, SendDestination},
    parameters::Parameter,
    protocol,
};

#[test]
fn protocol_constants_are_correct() {
    assert_eq!(protocol::TCP_PORT, 51325);
    assert_eq!(protocol::ACTIVE_SENSE, 0xfe);
    assert_eq!(protocol::MANUFACTURER_ID, [0x00, 0x00, 0x1a]);
    assert_eq!(protocol::QU_PRODUCT_ID, 0x50);
    assert_eq!(protocol::QU16_BOX_ID, 0x01);
}

#[test]
fn encode_cc_is_correct() {
    assert_eq!(
        protocol::encode_cc(0, 0x63, 0x20),
        [0xb0, 0x63, 0x20]
    );

    assert_eq!(
        protocol::encode_cc(3, 0x06, 0x62),
        [0xb3, 0x06, 0x62]
    );
}

#[test]
fn encode_cc_masks_midi_values() {
    assert_eq!(
        protocol::encode_cc(0xff, 0xff, 0xff),
        [0xbf, 0x7f, 0x7f]
    );
}

#[test]
fn encode_nrpn_is_correct() {
    let bytes = protocol::encode_nrpn(
        0,
        Channel::input(1).unwrap(),
        Parameter::Fader,
        0x62,
        0,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x17,
            0xb0, 0x06, 0x62,
            0xb0, 0x26, 0x00,
        ]
    );
}

#[test]
fn encode_nrpn_supports_nonzero_midi_channels() {
    let bytes = protocol::encode_nrpn(
        3,
        Channel::input(1).unwrap(),
        Parameter::Fader,
        0x62,
        0x12,
    );

    assert_eq!(
        bytes,
        [
            0xb3, 0x63, 0x20,
            0xb3, 0x62, 0x17,
            0xb3, 0x06, 0x62,
            0xb3, 0x26, 0x12,
        ]
    );
}

#[test]
fn fader_encoding_is_correct() {
    let bytes = protocol::fader(
        Channel::input(1).unwrap(),
        0x62,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x17,
            0xb0, 0x06, 0x62,
            0xb0, 0x26, 0x00,
        ]
    );
}

#[test]
fn send_level_encoding_is_correct() {
    let bytes = protocol::send_level(
        Channel::input(1).unwrap(),
        SendDestination::Mix1,
        0x62,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x20,
            0xb0, 0x06, 0x62,
            0xb0, 0x26, 0x00,
        ]
    );
}

#[test]
fn send_level_uses_destination_as_index() {
    let bytes = protocol::send_level(
        Channel::input(1).unwrap(),
        SendDestination::Mix5_6,
        0x62,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x20,
            0xb0, 0x06, 0x62,
            0xb0, 0x26, 0x04,
        ]
    );
}

#[test]
fn pan_encoding_is_correct() {
    let bytes = protocol::pan(
        Channel::input(1).unwrap(),
        SendDestination::Lr,
        0x25,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x16,
            0xb0, 0x06, 0x25,
            0xb0, 0x26, 0x07,
        ]
    );
}

#[test]
fn parameter_encoding_is_correct() {
    let bytes = protocol::parameter(
        Channel::input(1).unwrap(),
        Parameter::GateThreshold,
        0x40,
        0,
    );

    assert_eq!(
        bytes,
        [
            0xb0, 0x63, 0x20,
            0xb0, 0x62, 0x44,
            0xb0, 0x06, 0x40,
            0xb0, 0x26, 0x00,
        ]
    );
}

#[test]
fn mute_on_encoding_is_correct() {
    assert_eq!(
        protocol::mute(Channel::input(1).unwrap(), true),
        [0x90, 0x20, 0x7F, 0x90, 0x20, 0x00]
    );
}

#[test]
fn mute_off_encoding_is_correct() {
    assert_eq!(
        protocol::mute(Channel::input(1).unwrap(), false),
        [0x90, 0x20, 0x3F, 0x90, 0x20, 0x00]
    );
}

#[test]
fn mute_note_off_is_correct() {
    assert_eq!(
        protocol::mute_note_off(Channel::input(1).unwrap()),
        [0x80, 0x20, 0x00]
    );
}

#[test]
fn program_change_is_correct() {
    assert_eq!(
        protocol::program_change(0, 5),
        [0xc0, 0x05]
    );

    assert_eq!(
        protocol::program_change(3, 99),
        [0xc3, 0x63]
    );
}

#[test]
fn get_system_state_is_correct() {
    assert_eq!(
        protocol::get_system_state(),
        [
            0xf0,
            0x00, 0x00, 0x1a,
            0x50,
            0x11, 0x01,
            0x00,
            0x7f,
            0x10,
            0x00,
            0xf7,
        ]
    );
}

#[test]
fn get_system_state_ipad_flag_is_encoded() {
    assert_eq!(
        protocol::get_system_state_with_ipad(false),
        protocol::get_system_state()
    );

    assert_eq!(
        protocol::get_system_state_with_ipad(true),
        [
            0xf0,
            0x00, 0x00, 0x1a,
            0x50,
            0x11, 0x01,
            0x00,
            0x7f,
            0x10,
            0x01,
            0xf7,
        ]
    );
}

#[test]
fn system_state_response_is_correct() {
    let response = protocol::system_state_response(
        0,
        protocol::QU16_BOX_ID,
        1,
        99,
    );

    assert_eq!(
        response,
        [
            0xf0,
            0x00, 0x00, 0x1a,
            0x50,
            0x11, 0x01,
            0x00,
            0x00,
            0x11,
            0x01, // Box ID
            0x01, // Firmware major
            0x63, // Firmware minor
            0xf7,
        ]
    );
}

#[test]
fn system_state_response_masks_7_bit_fields() {
    let response = protocol::system_state_response(
        0xff,
        0xff,
        0xff,
        0xff,
    );

    assert_eq!(
        response,
        [
            0xf0,
            0x00, 0x00, 0x1a,
            0x50,
            0x11, 0x01,
            0x00,
            0x0f,
            0x11,
            0x7f, // Box ID
            0x7f, // Firmware major
            0x7f, // Firmware minor
            0xf7,
        ]
    );
}

#[test]
fn end_sync_is_correct() {
    assert_eq!(
        protocol::END_SYNC,
        [
            0xf0,
            0x00, 0x00, 0x1a,
            0x50,
            0x11, 0x01,
            0x00,
            0x00,
            0x14,
            0xf7,
        ]
    );
}

#[test]
fn scene_recall_is_correct() {
    assert_eq!(
        protocol::scene_recall(0),
        [
            0xb0, 0x00, 0x00,
            0xb0, 0x20, 0x00,
            0xc0, 0x00,
        ]
    );

    assert_eq!(
        protocol::scene_recall(99),
        [
            0xb0, 0x00, 0x00,
            0xb0, 0x20, 0x00,
            0xc0, 0x63,
        ]
    );
}
