use qu::parameters::Parameter;
use strum::IntoEnumIterator;

#[test]
fn parameter_ids_are_bidirectional() {
    for parameter in Parameter::iter() {
        let id = parameter.id();

        assert_eq!(
            Parameter::from_id(id),
            Some(parameter),
            "{parameter:?} does not round-trip through ID {id:#04x}"
        );
    }
}

#[test]
fn parameter_ids_are_unique() {
    let parameters: Vec<_> = Parameter::iter().collect();

    for (index, parameter) in parameters.iter().enumerate() {
        for other in parameters.iter().skip(index + 1) {
            assert_ne!(
                parameter.id(),
                other.id(),
                "{parameter:?} and {other:?} have the same ID {:#04x}",
                parameter.id()
            );
        }
    }
}

#[test]
fn parameter_ids_match_protocol_values() {
    let expected = [
        (Parameter::LfEqGain, 0x01),
        (Parameter::LfEqFrequency, 0x02),
        (Parameter::LfEqWidth, 0x03),
        (Parameter::LfEqType, 0x04),
        (Parameter::LmEqGain, 0x05),
        (Parameter::LmEqFrequency, 0x06),
        (Parameter::LmEqWidth, 0x07),
        (Parameter::HmEqGain, 0x09),
        (Parameter::HmEqFrequency, 0x0a),
        (Parameter::HmEqWidth, 0x0b),
        (Parameter::HfEqGain, 0x0d),
        (Parameter::HfEqFrequency, 0x0e),
        (Parameter::HfEqWidth, 0x0f),
        (Parameter::HfEqType, 0x10),
        (Parameter::PeqInOut, 0x11),
        (Parameter::UsbSource, 0x12),
        (Parameter::HpfFrequency, 0x13),
        (Parameter::HpfInOut, 0x14),
        (Parameter::Pan, 0x16),
        (Parameter::Fader, 0x17),
        (Parameter::LRAssign, 0x18),
        (Parameter::LocalGain, 0x19),
        (Parameter::SendLevel, 0x20),
        (Parameter::DcaAssignment, 0x40),
        (Parameter::GateAttack, 0x41),
        (Parameter::GateRelease, 0x42),
        (Parameter::GateHold, 0x43),
        (Parameter::GateThreshold, 0x44),
        (Parameter::GateDepth, 0x45),
        (Parameter::GateInOut, 0x46),
        (Parameter::FxDelayTimeMsb, 0x48),
        (Parameter::FxDelayTimeLsb, 0x49),
        (Parameter::DelayTime, 0x4c),
        (Parameter::DelayInOut, 0x4d),
        (Parameter::MixPrePost, 0x50),
        (Parameter::Pafl, 0x51),
        (Parameter::DigitalTrim, 0x52),
        (Parameter::StereoTrim, 0x54),
        (Parameter::MixAssignment, 0x55),
        (Parameter::PreampSource, 0x57),
        (Parameter::DsnakeGain, 0x58),
        (Parameter::DsnakePad, 0x59),
        (Parameter::Dsnake48v, 0x5a),
        (Parameter::MuteGroupAssignment, 0x5c),
        (Parameter::DsnakePatch, 0x5d),
        (Parameter::GroupMixMode, 0x5e),
        (Parameter::RemoteShutdown, 0x5f),
        (Parameter::CompressorType, 0x61),
        (Parameter::CompressorAttack, 0x62),
        (Parameter::CompressorRelease, 0x63),
        (Parameter::CompressorKnee, 0x64),
        (Parameter::CompressorRatio, 0x65),
        (Parameter::CompressorThreshold, 0x66),
        (Parameter::CompressorGain, 0x67),
        (Parameter::CompressorInOut, 0x68),
        (Parameter::Local48v, 0x69),
        (Parameter::Polarity, 0x6a),
        (Parameter::InsertInOut, 0x6b),
        (Parameter::DelayTime2, 0x6c),
        (Parameter::DelayInOut2, 0x6d),
        (Parameter::GeqGain, 0x70),
        (Parameter::GeqInOut, 0x71),
    ];

    assert_eq!(
        expected.len(),
        Parameter::iter().count(),
        "protocol mapping test does not cover every Parameter variant"
    );

    for (parameter, expected_id) in expected {
        assert_eq!(
            parameter.id(),
            expected_id,
            "{parameter:?} has incorrect ID"
        );

        assert_eq!(
            Parameter::from_id(expected_id),
            Some(parameter),
            "ID {expected_id:#04x} does not map back to {parameter:?}"
        );
    }
}

#[test]
fn parameter_invalid_ids_return_none() {
    for id in 0..=0x7f {
        if Parameter::from_id(id).is_some() {
            continue;
        }

        assert_eq!(
            Parameter::from_id(id),
            None,
            "unexpected Parameter for invalid ID {id:#04x}"
        );
    }
}
