//! Qu mixer emulator.
//!
//! Implements the behaviour of a virtual Qu mixer, responding to incoming
//! Qu MIDI protocol messages and providing simulated mixer state over TCP.

use qu::messages::hex;

use std::io;

use qu::{
    channels::Channel,
    parser::Parser,
    protocol::{
        self,
        ACTIVE_SENSE,
        END_SYNC,
        GET_SYSTEM_STATE,
        QU16_BOX_ID,
    },
};
use quip::{
    state::MixerState,
    qu_to_state::handle_event,
    qu_from_state,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{self, Duration},
};

/// Firmware version reported by the Squib emulator.
const FIRMWARE_MAJOR: u8 = 1;
const FIRMWARE_MINOR: u8 = 99;

pub async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut parser = Parser::new();
    let mut state = MixerState::default();

    // TCP is a byte stream, not a message stream.
    // Keep bytes here so a SysEx message split across TCP reads
    // can still be detected.
    let mut rx_buffer = Vec::<u8>::new();

    // Qu sends Active Sense approximately every 300 ms.
    let mut active_sense_interval = time::interval(Duration::from_millis(300));

    let mut buffer = [0u8; 4096];

    loop {
        tokio::select! {
            result = stream.read(&mut buffer) => {
                let count = result?;
                if count == 0 {
                    return Ok(());
                }

                let data = &buffer[..count];

                println!("[qu-16 ] RX {} bytes: {}", count, hex(data));

                // Parse incoming MIDI and update the simulated mixer state.
                for event in parser.push(data) {
                    println!("[qu-16 ] RX: {}", event.describe());

                    handle_event(&mut state, event);

                    // echo
                    // XXX: needs to actually store the changes for multi-device
                    write_all(&mut stream, data).await?;
                }

                // Keep a copy for protocol-level SysEx detection.
                rx_buffer.extend_from_slice(data);

                while let Some(message) = next_sysex(&mut rx_buffer) {
                    handle_sysex(
                        &mut stream,
                        &state,
                        &message,
                    ).await?;
                }

                // Don't allow an endlessly growing buffer if the peer
                // sends ordinary MIDI without any SysEx.
                if rx_buffer.len() > 64 * 1024 {
                    rx_buffer.clear();
                }
            }

            _ = active_sense_interval.tick() => {
                stream.write_all(&[ACTIVE_SENSE]).await?;
            }
        }
    }
}

async fn handle_sysex(
    stream: &mut TcpStream,
    state: &MixerState,
    message: &[u8],
) -> io::Result<()> {
    if message == GET_SYSTEM_STATE {
        println!("[qu-16 ] RX: Get System State");
        send_system_state(stream, state).await?;
    }

    Ok(())
}

async fn send_system_state(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    // SYSTEM STATE RESPONSE

    // The request uses the all-call SysEx channel (0x7F), but
    // the response is sent on MIDI channel 1 (0x00).
    let response = protocol::system_state_response(
        0x00,
        QU16_BOX_ID,
        FIRMWARE_MAJOR,
        FIRMWARE_MINOR,
    );
    write_all(stream, &response).await?;

    // CURRENT STATE
    send_current_state(stream, state).await?;

    // END SYNC
    write_all(stream, &END_SYNC).await?;

    Ok(())
}

async fn send_current_state(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    // CHANNEL STATE [CH 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST 1,2,3]
    send_channel_state(stream, state).await?;

    // BASIC GROUP STATE [GRP1,2,3,4]
        // fader
        // lr assignment
        // lr pan
        // mixes [1-9/10]
            // mix send
            // mix assignment
            // mix pre/post
        // fxes [1-2]
            // fx send
            // fx assignment
            // fx pre/post
        // mute group assignments (4)
        // dca assignments (4)

    // BASIC MIX STATE [MIX1,2,3,4,5-6,7-8,9-10,LR]
        // fader
        // mute
        // mute group assignments (4)
        // dca assignments (4)

    // BASIC ??? STATE [0x00,0x01]
        // fader
        // mute group assignments (8)
        // dca assignments (4)

    // mute group state [1,2,3,4]
    // DCA fader [1,2,3,4]

    // CHANNEL PEQ [CH1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST1,2,3, GRP1,2,3,4, MIX1,2,3,4,5-6,7-8,9-10,LR]
        // lf eq gain
        // lf eq freq
        // lf eq width
        // lf eq type
        // lm eq gain
        // lm eq freq
        // lm eq width
        // lm eq type
        // hm eq gain
        // hm eq freq
        // hm eq width
        // hm eq type
        // hf eq gain
        // hf eq freq
        // hf eq width
        // hf eq type
        // peq in/out

    // CHANNEL ??? [CH1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST1,2,3]
        // insert in/out
        // delay
        // delay in/out
        // 48v phantom power
        // polarity
        // usb source
        // hpf freq
        // hpf in/out
        // digital trim
        // preamp source
        // dSNAKE gain
        // dSNAKE phantom power
        // dSNAKE pad
        // local gain (or stereo trim for STs)

    // GROUP ??? [GRP1,2,3,4]
        // insert in/out
        // delay
        // delay in/out
        // 48v phantom power
        // polarity
        // usb source
        // hpf freq
        // hpf in/out
        // digital trim
        // preamp source

    // MIX ??? [MIX1,2,3,4,5-6,7-8,9-10,LR, 0x00,0x01]
        // insert in/out
        // delay
        // delay in/out

    // CHANNEL GATE [CH1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST1,2,3]
        // gate attack
        // gate release
        // gate hold
        // gate threshold
        // gate depth
        // gate in/out

    // CHANNEL COMP [CH1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST1,2,3, MIX1,2,3,4,5-6,7-8,9-10,LR]
        // comp type
        // comp attack
        // comp release
        // comp knee
        // comp ratio
        // comp threshold
        // comp gain
        // comp in/out

    // MIX GEQ [MIX1,2,3,4,5-6,7-8,9-10,LR]
        // qeq in/out
        // geq band [1-28]

    // ??? FX [0x00, 0x01, 0x02, 0x03]
        // Unknown parameter 0x47: value=15 index=00 [B0 63 00 62 47 06 15 26 00]
        // fx delay time lsb
        // fx delay time msb

    // CHANNEL NAMES [CH1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, GRP1,2,3,4, MIX1,2,3,4,5-6,7-8,9-10,LR, 0x00,0x01, DCA1,2,3,4, MUTEGRP1,2,3,4]
        // F0 00 00 1A 50 11 01 00 00 02 CH NAME... F7

    Ok(())
}

async fn send_channel_state(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    // BASIC CHANNEL STATE
    for number in 1..=16 {
        // CH 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16
        let channel = Channel::input(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::fader(&state, channel).unwrap(),
        )
        .await?;

        // lr assignment
        // lr pan

        // mute
        write_all(
            stream,
            &qu_from_state::mute(&state, channel).unwrap(),
        )
        .await?;

        // mixes [MIX 1,2,3,4,5-6,7-8,9-10,LR, FX 1,2,3,4]
            // mix send
            // mix assignment
            // mix pre/post
        // mute group assignments
        // dca assignments
    }

    // -------------------------------------------------------------------------
    // Stereo 1-3
    // -------------------------------------------------------------------------

    for number in 1..=3 {
        let channel = Channel::stereo(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::fader(&state, channel).unwrap(),
        )
        .await?;

        // mute
        write_all(
            stream,
            &qu_from_state::mute(&state, channel).unwrap(),
        )
        .await?;
    }

    // -------------------------------------------------------------------------
    // Mix 1-10 + Main LR
    // -------------------------------------------------------------------------

    for number in 1..=8 {
        let channel = Channel::mix(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::fader(&state, channel).unwrap(),
        )
        .await?;

        // mute
        write_all(
            stream,
            &qu_from_state::mute(&state, channel).unwrap(),
        )
        .await?;
    }

    Ok(())
}

fn next_sysex(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let start = buffer.iter().position(|&byte| byte == 0xF0)?;

    // Discard anything before the SysEx start.
    if start > 0 {
        buffer.drain(..start);
    }

    let end = buffer.iter().position(|&byte| byte == 0xF7)?;

    let message = buffer[..=end].to_vec();

    buffer.drain(..=end);

    Some(message)
}

async fn write_all(
    stream: &mut TcpStream,
    data: &[u8],
) -> io::Result<()> {
    stream.write_all(data).await?;
    println!("[qu-16 ] TX: {}", hex(data));
    Ok(())
}
