use crate::state::{ChannelRef, MeterState, MixerState};
use qu::{
    channels::Channel,
    faders::fader_to_db,
    messages::QuEvent,
    parameters::Parameter,
};

pub fn handle_event(
    mixer: &mut MixerState,
    meters: &mut MeterState,
    event: QuEvent,
) {
    match event {
        QuEvent::ActiveSense => {
            // Qu also responds to Active Sense. If it receives an Active Sense byte it will expect to receive
            // regular MIDI data from that point onwards (either valid control data, or more Active Sense bytes
            // during any period of inactivity). If it does not receive any data for 12 seconds, it will close the
            // Ethernet connection.
        }

        QuEvent::Fader { channel, value } => {
            mixer.set_fader(
                channel,
                fader_to_db(value),
            );
        }

        QuEvent::Mute { channel, muted } => {
            mixer.set_muted(channel, muted);
        }

        QuEvent::Name { channel, name } => {
            mixer.set_name(channel, name);
        }

        QuEvent::Meters { values } => {
            meters.update(&values);
        }

        QuEvent::Parameter {
            channel,
            parameter: Parameter::MuteGroupAssignment,
            value,
            ..
        } => {
            let mute_group_number = (value & 0x03) + 1;

            let Some(mute_group) = Channel::mute_group(mute_group_number) else {
                return;
            };

            let Some(mute_group) = ChannelRef::from_channel(mute_group) else {
                return;
            };

            let Some(channel) = ChannelRef::from_channel(channel) else {
                return;
            };

            let assigned = value & 0x40 != 0;

            mixer.set_mute_group_assignment(
                mute_group,
                channel,
                assigned,
            );
        }

        _ => {}
    }
}
