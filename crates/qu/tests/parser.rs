use qu::{
    channels::{Channel, SendDestination},
    messages::{QuEvent},
    parameters::{Parameter, MeterBlock},
    parser::Parser,
};

#[test]
fn parses_active_sense() {
    let mut parser = Parser::new();

    let events = parser.push(&[0xfe]);

    assert_eq!(events, vec![QuEvent::ActiveSense]);
}

#[test]
fn parses_fader() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x17,
        0xb0, 0x06, 0x62,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Fader {
                channel: Channel::input(1).unwrap(),
                value: 0x62,
            }
        ]
    );
}

#[test]
fn parses_send_level() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x20,
        0xb0, 0x06, 0x62,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::SendLevel {
                channel: Channel::input(1).unwrap(),
                destination: SendDestination::Mix1,
                value: 0x62,
            }
        ]
    );
}

#[test]
fn parses_pan() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x16,
        0xb0, 0x06, 0x25,
        0xb0, 0x26, 0x07,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Pan {
                channel: Channel::input(1).unwrap(),
                destination: SendDestination::Lr,
                value: 0x25,
            }
        ]
    );
}

#[test]
fn parses_generic_parameter() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x44,
        0xb0, 0x06, 0x40,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Parameter {
                channel: Channel::input(1).unwrap(),
                parameter: Parameter::GateThreshold,
                value: 0x40,
                index: 0,
            }
        ]
    );
}

#[test]
fn parses_fader_with_index() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x17,
        0xb0, 0x06, 0x62,
        0xb0, 0x26, 0x05,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Fader {
                channel: Channel::input(1).unwrap(),
                value: 0x62,
            }
        ]
    );
}

#[test]
fn parses_mute_on() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0x90, 0x20, 0x7f,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Mute {
                channel: Channel::input(1).unwrap(),
                muted: true,
            }
        ]
    );
}

#[test]
fn parses_mute_off() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0x90, 0x20, 0x01,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Mute {
                channel: Channel::input(1).unwrap(),
                muted: false,
            }
        ]
    );
}

#[test]
fn ignores_mute_note_off() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0x80, 0x20, 0x00,
    ]);

    assert!(events.is_empty());
}

#[test]
fn ignores_zero_velocity_note_on_for_mute() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0x90, 0x20, 0x00,
    ]);

    assert!(events.is_empty());
}

#[test]
fn parses_pafl() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x51,
        0xb0, 0x06, 0x01,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Pafl {
                channel: Channel::input(1).unwrap(),
                enabled: true,
            }
        ]
    );
}

#[test]
fn parses_phantom_power() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x69,
        0xb0, 0x06, 0x01,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::PhantomPower {
                channel: Channel::input(1).unwrap(),
                enabled: true,
            }
        ]
    );
}

#[test]
fn parses_lr_assign() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x18,
        0xb0, 0x06, 0x01,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::LRAssign {
                channel: Channel::input(1).unwrap(),
                enabled: true,
            }
        ]
    );
}

#[test]
fn parses_polarity() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x6a,
        0xb0, 0x06, 0x01,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Polarity {
                channel: Channel::input(1).unwrap(),
                reversed: true,
            }
        ]
    );
}

#[test]
fn parses_program_change() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xc0, 0x05,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::ProgramChange {
                channel: 0,
                program: 5,
            }
        ]
    );
}

#[test]
fn parses_sysex() {
    let mut parser = Parser::new();

    let message = [
        0xf0,
        0x00, 0x00, 0x1a,
        0x50,
        0x11, 0x01,
        0x00,
        0x7f,
        0x14,
        0xf7,
    ];

    let events = parser.push(&message);

    assert_eq!(
        events,
        vec![QuEvent::SysEx(message.to_vec())]
    );
}

#[test]
fn parser_handles_fragmented_nrpn() {
    let mut parser = Parser::new();
    let mut events = Vec::new();

    for byte in [
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x17,
        0xb0, 0x06, 0x62,
        0xb0, 0x26, 0x00,
    ] {
        events.extend(parser.push(&[byte]));
    }

    assert_eq!(
        events,
        vec![
            QuEvent::Fader {
                channel: Channel::input(1).unwrap(),
                value: 0x62,
            }
        ]
    );
}

#[test]
fn parser_handles_fragmented_sysex() {
    let mut parser = Parser::new();

    let message = [
        0xf0,
        0x00, 0x00, 0x1a,
        0x50,
        0x11, 0x01,
        0x00,
        0x7f,
        0x14,
        0xf7,
    ];

    let mut events = Vec::new();

    for byte in message {
        events.extend(parser.push(&[byte]));
    }

    assert_eq!(
        events,
        vec![QuEvent::SysEx(message.to_vec())]
    );
}

#[test]
fn parser_handles_multiple_messages_in_one_chunk() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xfe,
        0x90, 0x20, 0x7f,
        0x90, 0x20, 0x01,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::ActiveSense,
            QuEvent::Mute {
                channel: Channel::input(1).unwrap(),
                muted: true,
            },
            QuEvent::Mute {
                channel: Channel::input(1).unwrap(),
                muted: false,
            },
        ]
    );
}

#[test]
fn parser_preserves_running_status() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0x90, 0x20, 0x7f,
        0x21, 0x7f,
        0x22, 0x01,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Mute {
                channel: Channel::input(1).unwrap(),
                muted: true,
            },
            QuEvent::Mute {
                channel: Channel::input(2).unwrap(),
                muted: true,
            },
            QuEvent::Mute {
                channel: Channel::input(3).unwrap(),
                muted: false,
            },
        ]
    );
}

#[test]
fn parser_can_parse_nonzero_midi_channel() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb3, 0x63, 0x20,
        0xb3, 0x62, 0x17,
        0xb3, 0x06, 0x62,
        0xb3, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Fader {
                channel: Channel::input(1).unwrap(),
                value: 0x62,
            }
        ]
    );
}

#[test]
fn parser_can_parse_multiple_nrpn_messages() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x17,
        0xb0, 0x06, 0x62,
        0xb0, 0x26, 0x00,

        0xb0, 0x63, 0x21,
        0xb0, 0x62, 0x17,
        0xb0, 0x06, 0x72,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Fader {
                channel: Channel::input(1).unwrap(),
                value: 0x62,
            },
            QuEvent::Fader {
                channel: Channel::input(2).unwrap(),
                value: 0x72,
            },
        ]
    );
}

#[test]
fn parser_handles_unknown_parameter() {
    let mut parser = Parser::new();

    let events = parser.push(&[
        0xb0, 0x63, 0x20,
        0xb0, 0x62, 0x7e,
        0xb0, 0x06, 0x40,
        0xb0, 0x26, 0x00,
    ]);

    assert_eq!(
        events,
        vec![
            QuEvent::Unknown(vec![
                0xb0, 0x63, 0x20,
                0xb0, 0x62, 0x7e,
                0xb0, 0x06, 0x40,
                0xb0, 0x26, 0x00,
            ])
        ]
    );
}

#[test]
fn parses_meter_data() {
    let mut parser = Parser::new();

    // This is the actual beginning of a Qu-16 meter response:
    //
    // 54 17 4B 17 4B 17 4B
    //
    // which decodes to:
    //
    //   97 4B 97 4B 97 4B
    //
    // i.e. three meter values, each representing -52.41015625 dB.

    let message = [
        0xf0,
        0x00, 0x00, 0x1a,
        0x50,
        0x11, 0x01,
        0x00,
        0x00,
        0x13,

        0x54,
        0x17, 0x4b,
        0x17, 0x4b,
        0x17, 0x4b,

        0xf7,
    ];

    let events = parser.push(&message);

    assert_eq!(events.len(), 1);

    match &events[0] {
        QuEvent::Meters { values } => {
            assert_eq!(values.len(), 3);

            for value in values {
                assert!((value.db + 52.41015625).abs() < 0.0001);
            }
        }

        event => panic!("expected meter event, got {event:?}"),
    }
}

fn hex_bytes(input: &str) -> Vec<u8> {
    input
        .split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).unwrap())
        .collect()
}

#[test]
fn parses_real_qu16_meter_block() {
    let data = hex_bytes(
        r#"
        F0 00 00 1A 50 11 01 00 00 13
        54 17 4B 17 4B 17 4B 3E 00 4D 3E 4D 3E 4D 01 12
        13 17 6C 7B 7F 0D 00 24 2A 51 49 50 49 50 49 50
        50 24 51 49 50 01 12 17 24 6C 7B 7F 0D 00
        01 12 00 01 12 01 12 01 12 01 00 12 01 12 01 12
        17 6C 2D 0D 00 0D 00 7B 3A 51 2A 3D 51 3D 51 3D
        51 3D 40 51 3D 01 12 16 6C 0D 50 00 0D 00
        01 12 01 12 00 01 12 01 12 01 12 01 01 12 01 12
        17 6C 0D 00 20 0D 00 01 12 01 12 01 00 12 01 12
        01 12 01 12 02 01 12 17 6C 0D 00 0D 40 00 01 12
        01 12 01 12 00 01 12 01 12 01 12 01 05 12 17 6C
        0D 00 0D 00 00 01 12 01 12 01 12 01 00 12 01 12
        01 12 01 12 0A 17 6C 0D 00 0D 00 01 00 12 01 12
        01 12 01 12 00 01 12 01 12 01 12 17 14 6C 0D 00
        0D 00 01 12 00 01 12 01 12 01 12 01 00 12 01 12
        01 12 17 6C 29 0D 00 0D 00 70 65 01 2A 64 01 64
        01 64 4B 64 42 01 64 01 12 0E 00 0D 5A 00 0D 00
        53 61 7F 5C 51 7F 5C 7F 5C 70 5D 7F 05 5C 01 12
        0E 00 0D 00 35 0D 00 51 3D 3A 43 3A 2A 43 64 40
        51 3D 64 40 02 01 12 16 6C 0D 00 0D 40 00 01 12
        01 12 01 12 00 01 12 01 12 01 12 01 05 12 16 6C
        0D 00 0D 00 55 6A 4E 6A 4E 6A 4E 6A 28 4E 6A 4E
        6A 4E 01 12 0A 17 6C 0D 00 0D 00 01 00 12 01 12
        01 12 01 12 00 01 12 01 12 01 12 17 14 6C 0D 00
        0D 00 01 12 00 01 12 01 12 01 12 01 00 12 01 12
        01 12 01 00 20 0D 00 01 00 01 12 01 00 12 01 12
        01 12 01 12 00 01 12 01 12 01 00 0D 40 00 01 00
        01 12 01 12 00 01 12 01 12 01 12 01 01 12 01 12
        01 00 0D 00 00 01 00 01 12 01 12 01 00 12 01 12
        01 12 01 12 02 01 12 01 00 0D 00 01 00 00 01 12
        01 12 01 12 00 01 12 01 12 01 12 01 04 12 01 00
        0D 00 01 00 00 01 12 01 12 01 12 01 00 12 01 12
        01 12 01 12 08 01 00 0D 00 01 00 01 00 12 01 12
        01 12 01 12 00 01 12 01 12 01 12 01 10 00 0D 00
        01 00 01 12 00 01 12 01 12 01 12 01 00 12 01 12
        01 12 01 00 25 0D 00 01 00 2B 79 30 28 79 2D 79
        2D 79 12 7A 13 12 7A 77 78 0E 00 7B 10 7F 0D 00
        71 7A 75 7A 00 73 7A 73 7A 12 7A 12 26 7A 40 79
        0E 00 7B 7F 20 0D 00 01 12 01 12 01 00 12 01 12
        01 12 01 12 02 01 12 16 6C 0D 00 0D 40 00 01 12
        01 12 01 12 00 01 12 01 12 01 12 01 05 12 16 6C
        0D 00 0D 00 00 01 12 01 12 01 00 12 01 12 01 12
        0A 16 6C 0D 00 0D 00 01 00 12 01 12 01 12 00 01
        12 01 12 01 12 01 12 16 14 6C 0D 00 0D 00 01 12
        00 01 12 01 12 01 12 00 50 12 00 12 01 12 01 00
        20 0D 00 01 00 01 12 01 01 12 01 12 01 12 00 12
        20 00 12 01 12 01 00 0D 42 00 01 00 01 12 34 73
        55 34 73 31 73 31 73 05 2A 73 31 73 31 73 7B 7F
        00 01 00 01 12 01 12 01 00 12 01 12 01 12 01 12
        02 01 12 01 12 0D 00 01 00 00 01 12 01 12 01 12
        00 01 12 01 12 01 04 12 01 12 0D 00 01 00 00 01
        12 01 12 01 12 01 00 12 01 12 01 12 01 12 08 01
        12 0D 00 01 00 01 00 12 01 12 01 12 01 12 00 01
        12 01 12 01 12 01 10 12 0D 00 01 00 01 12 00 01
        12 01 12 01 00 12 01 12 01 12 01 12 20 0D 00 01
        00 01 12 01 00 12 01 12 01 12 01 12 00 01 12 01
        12 01 12 0D 40 00 01 00 01 12 01 12 00 01 12 01
        12 01 12 01 12 01 01 12 01 12 01 12 0D 00 00 01
        00 01 12 01 12 01 00 12 01 12 01 12 01 12 02 01
        12 01 12 0D 00 01 00 00 01 12 01 12 01 12 00 01
        12 01 12 01 12 04 12 01 12 0D 00 01 00 15 01 12
        63 78 63 78 63 22 78 60 78 59 78 60 78 10 44 79
        7B 7F 01 00 01 2A 12 26 79 26 79 26 79 44 26 79
        1D 79 26 79 44 20 79 7B 7F 01 00 59 78 11 1D 79
        49 78 01 18 5E 28 5D 6F 78 37 79 66 78 04 2B 79
        55 79 49 78 01 00 12 01 12 66 78 2B 79 00 66 78
        01 12 01 12 01 00 12 01 12 01 12 75 1E 54 51 3D
        51 3D 3A 43 5B 28 47 3F 49 6A 4E 32 53 05 0D 59
        49 5E 07 65 29 08 6F 51 78 29 6F 42 65 45 67 5E
        0D 59 68 53 6A 22 4E 17 4B 5B 47 3A 43 55 51 3D
        51 3D 51 3D 51 00 3D 01 12 01 12 01 12 01 01 12
        01 12 75 1E 51 28 3D 51 3D 3A 43 5B 47 55 17 4B
        61 4F 68 53 45 00 59 1B 5F 39 66 6D 70 05 1A 79
        6D 70 78 65 00 2A 5F 45 59 12 54 61 4F 45 17 4B
        5B 47 3A 43 51 2A 3D 51 3D 51 3D 51 3D 00 01 12
        01 12 01 12 01 0A 12 01 12 4F 49 4F 49 00 01 12
        01 12 01 12 01 12 00 01 12 01 12 01 12 01 12 00
        01 12 01 12 01 12 01 12 00 01 12 01 12 01 12 01
        00 01 00 00 01 12 01 12 01 12 00 01 12 01 12 01
        12 00 01 12 01 12 01 12 00 01 12 01 12 01 12 01
        12 00 01 12 01 12 01 00 00 01 00 01 12 01 12 01
        00 12 01 12 01 12 01 12 00 01 12 01 12 01 12 01
        12 00 01 12 01 12 01 12 00 00 01 00 01 12 01 12
        00 01 12 01 12 01 12 01 00 12 01 12 01 12 01 12
        00 01 12 01 12 01 00 12 01 12 01 12 01 00 00 01
        00 01 12 01 12 01 00 01 12 01 12 01 12 01 00 12
        01 12 01 12 01 12 01 12 01 12 01 12
        F7
        "#,
    );

    let mut parser = Parser::new();
    let events = parser.push(&data);

    let values = events
        .iter()
        .find_map(|event| match event {
            QuEvent::Meters { values } => Some(values),
            _ => None,
        })
        .expect("expected meter event");

    let get = |block: MeterBlock, index: usize| {
        values
            .iter()
            .find(|meter| meter.block == block && meter.index == index)
            .unwrap()
            .db
    };

    assert!((get(MeterBlock::MonoInput(1), 0) + 52.41).abs() < 0.01);
    assert!((get(MeterBlock::MonoInput(11), 0) + 26.56).abs() < 0.01);

    assert!((get(MeterBlock::StereoInput(1), 0) + 6.33).abs() < 0.01);
    assert!((get(MeterBlock::StereoInput(1), 10) + 5.56).abs() < 0.01);

    assert!((get(MeterBlock::MonoMix(1), 5) + 12.48).abs() < 0.01);

    assert!((get(MeterBlock::StereoMix(4), 5) + 7.65).abs() < 0.01);
    assert!((get(MeterBlock::StereoMix(4), 15) + 6.89).abs() < 0.01);
}
