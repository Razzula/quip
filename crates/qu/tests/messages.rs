use qu::{
    channels::{Channel, SendDestination},
    messages::{MidiMessage, QuEvent},
    parameters::Parameter,
};

#[test]
fn midi_message_variants_can_represent_protocol_messages() {
    let messages = [
        MidiMessage::ControlChange {
            channel: 0,
            controller: 0x63,
            value: 0x20,
        },
        MidiMessage::NoteOn {
            channel: 0,
            note: 0x20,
            velocity: 0x7f,
        },
        MidiMessage::NoteOff {
            channel: 0,
            note: 0x20,
            velocity: 0,
        },
        MidiMessage::ProgramChange {
            channel: 0,
            program: 1,
        },
        MidiMessage::SysEx(vec![0xf0, 0xf7]),
        MidiMessage::ActiveSense,
    ];

    assert_eq!(messages.len(), 6);
}

#[test]
fn qu_event_variants_can_represent_common_mixer_state() {
    let input = Channel::input(1).unwrap();

    let events = [
        QuEvent::ActiveSense,
        QuEvent::Fader {
            channel: input,
            value: 0x62,
        },
        QuEvent::SendLevel {
            channel: input,
            destination: SendDestination::Mix1,
            value: 0x62,
        },
        QuEvent::Pan {
            channel: input,
            destination: SendDestination::Lr,
            value: 0x25,
        },
        QuEvent::Mute {
            channel: input,
            muted: true,
        },
        QuEvent::Pafl {
            channel: input,
            enabled: true,
        },
        QuEvent::PhantomPower {
            channel: input,
            enabled: true,
        },
        QuEvent::LRAssign {
            channel: input,
            enabled: true,
        },
        QuEvent::Polarity {
            channel: input,
            reversed: true,
        },
        QuEvent::Parameter {
            channel: input,
            parameter: Parameter::GateThreshold,
            value: 0x40,
            index: 0,
        },
        QuEvent::SysEx(vec![0xf0, 0xf7]),
        QuEvent::ProgramChange {
            channel: 0,
            program: 1,
        },
        QuEvent::Midi(MidiMessage::ActiveSense),
        QuEvent::Unknown(vec![0xff]),
    ];

    assert_eq!(events.len(), 14);
}

#[test]
fn events_are_cloneable_and_comparable() {
    let event = QuEvent::Fader {
        channel: Channel::input(1).unwrap(),
        value: 0x62,
    };

    assert_eq!(event.clone(), event);
}

#[test]
fn midi_messages_are_cloneable_and_comparable() {
    let message = MidiMessage::ControlChange {
        channel: 0,
        controller: 0x63,
        value: 0x20,
    };

    assert_eq!(message.clone(), message);
}
