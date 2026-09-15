use quip::{
    state::MixerState,
    qu_to_state::handle_event,
};
use qu::{
    channels::Channel,
    messages::QuEvent,
    parameters::fader_to_db,
};

#[test]
fn handles_fader() {
    let mut state = MixerState::default();
    let channel = Channel::input(1).unwrap();

    handle_event(
        &mut state,
        QuEvent::Fader {
            channel,
            value: 0x40,
        },
    );

    assert_eq!(state.fader(channel), Some(fader_to_db(0x40)));
}

#[test]
fn handles_mute() {
    let mut state = MixerState::default();
    let channel = Channel::input(1).unwrap();

    handle_event(
        &mut state,
        QuEvent::Mute {
            channel,
            muted: true,
        },
    );

    assert_eq!(state.muted(channel), Some(true));
}
