use crate::state::{ChannelRef, MixerState};
use qu::{
    faders::fader_to_db,
    messages::QuEvent,
    parameters::Parameter,
    channels::Channel,
};

pub fn handle_event(state: &mut MixerState, event: QuEvent) {
    match event {
        // Qu supports MIDI Active Sensing over its TCP/IP Ethernet connection to detect connection
        // status. Qu will send an initial Active Sense byte (FE) once an Ethernet connection is established,
        // and then once every 300ms or so during any period of inactivity.
        QuEvent::ActiveSense => {
            // Qu also responds to Active Sense. If it receives an Active Sense byte it will expect to receive
            // regular MIDI data from that point onwards (either valid control data, or more Active Sense bytes
            // during any period of inactivity). If it does not receive any data for 12 seconds, it will close the
            // Ethernet connection.
        }

        QuEvent::Fader { channel, value } => {
            state.set_fader(
                channel,
                fader_to_db(value),
            );
        }

        QuEvent::Mute { channel, muted } => {
            state.set_muted(channel, muted);
        }

        QuEvent::Name { channel, name } => {
            state.set_name(channel, name);
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

            state.set_mute_group_assignment(
                mute_group,
                channel,
                assigned,
            );
        }

        _ => {}
    }
}
