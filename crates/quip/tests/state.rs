
use quip::state::{MixerState};
use qu::channels::Channel;

#[test]
fn state_serializes_to_json() {
    let state = MixerState::default();

    let json = serde_json::to_string(&state).unwrap();

    assert!(json.contains("\"inputs\""));
    assert!(json.contains("\"stereo\""));
    assert!(json.contains("\"mixes\""));
    assert!(json.contains("\"fader\""));
    assert!(json.contains("\"muted\""));
}

#[test]
fn state_round_trips_through_json() {
    let mut state = MixerState::default();
    let channel = Channel::input(1).unwrap();

    state.set_fader(channel, -5.0);
    state.set_muted(channel, true);

    let json = serde_json::to_string(&state).unwrap();
    let restored: MixerState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.fader(channel), Some(-5.0));
    assert_eq!(restored.muted(channel), Some(true));
}

#[test]
fn state_defaults_are_zero_db_and_unmuted() {
    let state = MixerState::default();

    assert_eq!(state.fader(Channel::input(1).unwrap()), Some(0.0));
    assert_eq!(state.muted(Channel::input(1).unwrap()), Some(false));

    assert_eq!(state.fader(Channel::stereo(1).unwrap()), Some(0.0));
    assert_eq!(state.muted(Channel::stereo(1).unwrap()), Some(false));

    assert_eq!(state.fader(Channel::mix(1).unwrap()), Some(0.0));
    assert_eq!(state.muted(Channel::mix(1).unwrap()), Some(false));
}

#[test]
fn state_input_fader_can_be_changed() {
    let mut state = MixerState::default();
    let channel = Channel::input(1).unwrap();

    assert!(state.set_fader(channel, -5.0));
    assert_eq!(state.fader(channel), Some(-5.0));
}

#[test]
fn state_stereo_fader_can_be_changed() {
    let mut state = MixerState::default();
    let channel = Channel::stereo(2).unwrap();

    assert!(state.set_fader(channel, -6.0));
    assert_eq!(state.fader(channel), Some(-6.0));
}

#[test]
fn state_mix_fader_can_be_changed() {
    let mut state = MixerState::default();
    let channel = Channel::mix(3).unwrap();

    assert!(state.set_fader(channel, -7.0));
    assert_eq!(state.fader(channel), Some(-7.0));
}

#[test]
fn state_main_lr_is_a_mix_channel() {
    let mut state = MixerState::default();
    let channel = Channel::lr();

    assert!(state.set_fader(channel, -2.0));
    assert_eq!(state.fader(channel), Some(-2.0));
}

#[test]
fn state_mute_can_be_changed() {
    let mut state = MixerState::default();
    let channel = Channel::input(1).unwrap();

    assert!(state.set_muted(channel, true));
    assert_eq!(state.muted(channel), Some(true));

    assert!(state.set_muted(channel, false));
    assert_eq!(state.muted(channel), Some(false));
}

#[test]
fn state_unsupported_channel_returns_false() {
    let mut state = MixerState::default();
    let channel = Channel::fx_send(1).unwrap();

    assert!(!state.set_fader(channel, 2.0));
    assert_eq!(state.fader(channel), None);

    assert!(!state.set_muted(channel, true));
    assert_eq!(state.muted(channel), None);
}
