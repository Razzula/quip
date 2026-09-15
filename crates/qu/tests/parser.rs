use qu::{
    channels::{Channel, SendDestination},
    messages::{QuEvent},
    parameters::Parameter,
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
