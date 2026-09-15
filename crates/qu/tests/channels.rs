use qu::channels::{Channel, SendDestination};

#[test]
fn input_channels_have_correct_ids() {
    for number in 1..=32 {
        assert_eq!(
            Channel::input(number).unwrap().raw(),
            0x20 + number - 1
        );
    }
}

#[test]
fn stereo_channels_have_correct_ids() {
    for number in 1..=3 {
        assert_eq!(
            Channel::stereo(number).unwrap().raw(),
            0x40 + number - 1
        );
    }
}

#[test]
fn mix_channels_have_correct_ids() {
    for number in 1..=10 {
        assert_eq!(
            Channel::mix(number).unwrap().raw(),
            0x60 + number - 1
        );
    }
}

#[test]
fn dca_channels_have_correct_ids() {
    for number in 1..=4 {
        assert_eq!(
            Channel::dca(number).unwrap().raw(),
            0x10 + number - 1
        );
    }
}

#[test]
fn mute_group_channels_have_correct_ids() {
    for number in 1..=4 {
        assert_eq!(
            Channel::mute_group(number).unwrap().raw(),
            0x50 + number - 1
        );
    }
}

#[test]
fn group_channels_have_correct_ids() {
    for number in 1..=4 {
        assert_eq!(
            Channel::group(number).unwrap().raw(),
            0x68 + number - 1
        );
    }
}

#[test]
fn matrix_channels_have_correct_ids() {
    for number in 1..=2 {
        assert_eq!(
            Channel::matrix(number).unwrap().raw(),
            0x6c + number - 1
        );
    }
}

#[test]
fn fx_return_channels_have_correct_ids() {
    for number in 1..=4 {
        assert_eq!(
            Channel::fx_return(number).unwrap().raw(),
            0x08 + number - 1
        );
    }
}

#[test]
fn fx_send_channels_have_correct_ids() {
    for number in 1..=4 {
        assert_eq!(
            Channel::fx_send(number).unwrap().raw(),
            number - 1
        );
    }
}

#[test]
fn lr_has_correct_id() {
    assert_eq!(Channel::lr().raw(), 0x67);
}

#[test]
fn channel_constructors_reject_invalid_numbers() {
    assert!(Channel::input(0).is_none());
    assert!(Channel::input(33).is_none());

    assert!(Channel::stereo(0).is_none());
    assert!(Channel::stereo(4).is_none());

    assert!(Channel::mix(0).is_none());
    assert!(Channel::mix(11).is_none());

    assert!(Channel::dca(0).is_none());
    assert!(Channel::dca(5).is_none());

    assert!(Channel::mute_group(0).is_none());
    assert!(Channel::mute_group(5).is_none());

    assert!(Channel::group(0).is_none());
    assert!(Channel::group(5).is_none());

    assert!(Channel::matrix(0).is_none());
    assert!(Channel::matrix(3).is_none());

    assert!(Channel::fx_return(0).is_none());
    assert!(Channel::fx_return(5).is_none());

    assert!(Channel::fx_send(0).is_none());
    assert!(Channel::fx_send(5).is_none());
}

#[test]
fn send_destinations_have_correct_indices() {
    let destinations = [
        (SendDestination::Mix1, 0x00),
        (SendDestination::Mix2, 0x01),
        (SendDestination::Mix3, 0x02),
        (SendDestination::Mix4, 0x03),
        (SendDestination::Mix5_6, 0x04),
        (SendDestination::Mix7_8, 0x05),
        (SendDestination::Mix9_10, 0x06),
        (SendDestination::Lr, 0x07),
        (SendDestination::Group1_2, 0x08),
        (SendDestination::Group3_4, 0x09),
        (SendDestination::Group5_6, 0x0a),
        (SendDestination::Group7_8, 0x0b),
        (SendDestination::Matrix1_2, 0x0c),
        (SendDestination::Matrix3_4, 0x0d),
        (SendDestination::FxSend1, 0x10),
        (SendDestination::FxSend2, 0x11),
        (SendDestination::FxSend3, 0x12),
        (SendDestination::FxSend4, 0x13),
    ];

    for (destination, expected) in destinations {
        assert_eq!(destination.index(), expected);
    }
}

#[test]
fn send_destination_indices_round_trip() {
    for index in 0..=0x13 {
        let destination = SendDestination::from_index(index);

        assert_eq!(destination.index(), index);
    }
}

#[test]
fn unknown_send_destination_round_trips() {
    let destination = SendDestination::from_index(0x7f);

    assert_eq!(destination, SendDestination::Unknown(0x7f));
    assert_eq!(destination.index(), 0x7f);
}
