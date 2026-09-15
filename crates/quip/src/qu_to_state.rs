use crate::state::MixerState;
use qu::{
    parameters::fader_to_db,
    messages::QuEvent,
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
            state.set_fader(channel, fader_to_db(value));
        }

        QuEvent::Mute { channel, muted } => {
            state.set_muted(channel, muted);
        }

        _ => {}
    }
}
