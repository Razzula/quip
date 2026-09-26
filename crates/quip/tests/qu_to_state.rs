use quip::{
    state::{MixerState, MeterState},
    qu_to_state::handle_event,
};
use qu::{
    channels::Channel,
    messages::QuEvent,
    faders::fader_to_db,
};

#[test]
fn handles_fader() {
    let mut state = MixerState::default();
    let mut meters = MeterState::default();
    let channel = Channel::input(1).unwrap();

    handle_event(
        &mut state,
        &mut meters,
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
    let mut meters = MeterState::default();
    let channel = Channel::input(1).unwrap();

    handle_event(
        &mut state,
        &mut meters ,
        QuEvent::Mute {
            channel,
            muted: true,
        },
    );

    assert_eq!(state.muted(channel), Some(true));
}
