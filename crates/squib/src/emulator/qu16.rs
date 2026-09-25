//! Qu mixer emulator.
//!
//! Implements the behaviour of a virtual Qu mixer, responding to incoming
//! Qu MIDI protocol messages and providing simulated mixer state over TCP.

use qu::messages::hex;

use rand::RngExt;
use std::io;

use qu::{
    channels::Channel,
    parser::Parser,
    protocol::{
        self,
        ACTIVE_SENSE,
        end_sync,
        get_system_state,
        QU16_BOX_ID,
    },
};
use quip::{
    state::{ChannelRef, MeterState, MixerState},
    qu_from_state,
    qu_to_state::handle_event,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{self, Duration},
};

/// Firmware version reported by the Squib emulator.
const FIRMWARE_MAJOR: u8 = 1;
const FIRMWARE_MINOR: u8 = 99;

/// Interval between simulated meter packets.
const METER_INTERVAL: Duration = Duration::from_millis(50);

pub async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut parser = Parser::new();
    let mut state = MixerState::default();
    let mut meters = MeterState::default();

    let mut meters_enabled = false;

    // TCP is a byte stream, not a message stream.
    // Keep bytes here so a SysEx message split across TCP reads
    // can still be detected.
    let mut rx_buffer = Vec::<u8>::new();

    // Qu sends Active Sense approximately every 300 ms.
    let mut active_sense_interval = time::interval(Duration::from_millis(300));

    // Meter data is continuously transmitted once enabled.
    let mut meter_interval = time::interval(METER_INTERVAL);

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

                    handle_event(&mut state, &mut meters, event);
                }

                // echo
                // XXX: needs to actually store the changes for multi-device
                write_all(&mut stream, data).await?;

                // Keep a copy for protocol-level SysEx detection.
                rx_buffer.extend_from_slice(data);

                while let Some(message) = next_sysex(&mut rx_buffer) {
                    if let Some(enabled) = handle_meter_control(&message) {
                        meters_enabled = enabled;

                        println!(
                            "[qu-16 ] Meter stream: {}",
                            if enabled { "ON" } else { "OFF" }
                        );

                        continue;
                    }

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

            _ = meter_interval.tick(), if meters_enabled => {
                randomise_meters(&mut meters);

                let message = meter_data(&meters);

                write_all(&mut stream, &message).await?;
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
    if message == get_system_state() {
        println!("[qu-16 ] RX: Get System State");
        send_system_state(stream, state).await?;
    }

    Ok(())
}

/// Handle a Qu meter control SysEx message.
///
/// `12 01` enables continuous meter transmission.
/// `12 00` disables continuous meter transmission.
fn handle_meter_control(message: &[u8]) -> Option<bool> {
    if message.len() != 12 {
        return None;
    }

    if message[0..8] != protocol::SYSEX_HEADER {
        return None;
    }

    if message[9] != protocol::SYSEX_METER_CONTROL {
        return None;
    }

    if message[11] != 0xf7 {
        return None;
    }

    match message[10] {
        0x00 => Some(false),
        0x01 => Some(true),
        _ => None,
    }
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
    write_all(stream, &end_sync()).await?;

    Ok(())
}

async fn send_current_state(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    // CHANNEL STATE [CH 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, ST 1,2,3, MIX 1,2,3,4,5-6,7-8,9-10,LR]
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
        
    // BASIC ??? STATE [0x00,0x01]
        // fader
        // mute group assignments (8)
        // dca assignments (4)
        
    // MUTE GROUPS / DCAs
    send_mute_group_state(stream, state).await?;
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
    send_channel_names(stream, state).await?;

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

        // mutestate
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
        send_mute_group_assigns(stream, state, &channel).await?;
        // dca assignments
    }

    for number in 1..=3 {
        // ST 1,2,3
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

        // mute group assignments
        send_mute_group_assigns(stream, state, &channel).await?;
    }

    for number in 1..=8 {
        // MIX 1,2,3,4,5-6,7-8,9-10,LR
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

        // mute group assignments
        send_mute_group_assigns(stream, state, &channel).await?;
    }

    Ok(())
}

async fn send_mute_group_state(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    for number in 1..=4 {
        // MG 1,2,3,4
        let channel = Channel::mute_group(number).unwrap();

        // mute state
        write_all(
            stream,
            &qu_from_state::mute(&state, channel).unwrap(),
        )
        .await?;
    }

    Ok(())
}

async fn send_mute_group_assigns(
    stream: &mut TcpStream,
    state: &MixerState,
    channel: &Channel,
) -> io::Result<()> {
    let Some(channel) = ChannelRef::from_channel(*channel) else {
        return Ok(());
    };

    for number in 1..=4 {
        let group = Channel::mute_group(number).unwrap();

        let Some(group) = ChannelRef::from_channel(group) else {
            continue;
        };

        write_all(
            stream,
            &qu_from_state::mute_group_assignment(state, channel, group)
                .unwrap(),
        )
        .await?;
    }

    Ok(())
}

async fn send_channel_names(
    stream: &mut TcpStream,
    state: &MixerState,
) -> io::Result<()> {
    // SYSEX CHANNEL NAMES
    for number in 1..=16 {
        // CH 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16
        let channel = Channel::input(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::name(&state, channel).unwrap(),
        )
        .await?;
    }

    for number in 1..=3 {
        // ST 1,2,3
        let channel = Channel::stereo(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::name(&state, channel).unwrap(),
        )
        .await?;
    }

    for number in 1..=8 {
        // MIX 1,2,3,4,5-6,7-8,9-10,LR
        let channel = Channel::mix(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::name(&state, channel).unwrap(),
        )
        .await?;
    }

    for number in 1..=4 {
        // MG 1,2,3,4
        let channel = Channel::mute_group(number).unwrap();

        // fader
        write_all(
            stream,
            &qu_from_state::name(&state, channel).unwrap(),
        )
        .await?;
    }

    Ok(())
}

/// Generate random meter values for the subset of the Qu-16 meter stream
/// currently represented by `MeterState`.
fn randomise_meters(meters: &mut MeterState) {
    let mut rng = rand::rng();

    for value in &mut meters.inputs {
        *value = rng.random_range(-60.0..=10.0);
    }

    for stereo in &mut meters.stereo {
        stereo[0] = rng.random_range(-60.0..=10.0);
        stereo[1] = rng.random_range(-60.0..=10.0);
    }

    for mix in &mut meters.mixes {
        mix[0] = rng.random_range(-60.0..=10.0);
        mix[1] = rng.random_range(-60.0..=10.0);
    }
}

/// Construct a Qu-16 meter data SysEx message.
///
/// Only the input, stereo input, and mix meters represented by `MeterState`
/// contain simulated values. Other meter positions are populated with
/// -128 dB values.
fn meter_data(meters: &MeterState) -> Vec<u8> {
    let mut raw = Vec::new();

    // ---------------------------------------------------------------------
    // Mono Input blocks
    // ---------------------------------------------------------------------

    for input in &meters.inputs {
        // Post Preamp
        push_meter(&mut raw, *input);

        // Remaining 9 meters in the block.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }
    }

    // 80 unused meters.
    for _ in 0..80 {
        push_meter(&mut raw, -128.0);
    }

    // ---------------------------------------------------------------------
    // Stereo Input blocks
    // ---------------------------------------------------------------------

    for stereo in &meters.stereo {
        // Post Preamp L
        push_meter(&mut raw, stereo[0]);

        // Remaining 9 L meters.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }

        // Post Preamp R
        push_meter(&mut raw, stereo[1]);

        // Remaining 9 R meters.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }
    }

    // 20 unused meters.
    for _ in 0..20 {
        push_meter(&mut raw, -128.0);
    }

    // ---------------------------------------------------------------------
    // Mono Mix blocks
    // ---------------------------------------------------------------------

    for mix in &meters.mixes[..4] {
        // TB/SigGen
        push_meter(&mut raw, mix[0]);

        // Remaining 9 meters in the block.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }
    }

    // ---------------------------------------------------------------------
    // Stereo Mix blocks
    // ---------------------------------------------------------------------

    for mix in &meters.mixes[4..8] {
        // TB/SigGen L
        push_meter(&mut raw, mix[0]);

        // Remaining 9 L meters.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }

        // TB/SigGen R
        push_meter(&mut raw, mix[1]);

        // Remaining 9 R meters.
        for _ in 0..9 {
            push_meter(&mut raw, -128.0);
        }
    }

    // ---------------------------------------------------------------------
    // Stereo Monitor
    // ---------------------------------------------------------------------

    for _ in 0..16 {
        push_meter(&mut raw, -128.0);
    }

    // ---------------------------------------------------------------------
    // Stereo FX
    // ---------------------------------------------------------------------

    for _ in 0..4 {
        for _ in 0..83 {
            push_meter(&mut raw, -128.0);
        }
    }

    let encoded = encode_meter_data(&raw);

    let mut message = Vec::with_capacity(
        10 + encoded.len() + 1,
    );

    message.extend_from_slice(&protocol::SYSEX_HEADER);
    message.push(protocol::SYSEX_METER_DATA);
    message.extend_from_slice(&encoded);
    message.push(0xf7);

    message
}

/// Convert a dB value into the Qu meter's 16-bit 7Q8 representation.
///
/// Qu stores the signed value with an 0x8000 offset:
///
///     raw = (dB * 256) + 0x8000
fn push_meter(raw: &mut Vec<u8>, db: f32) {
    let value = if db <= -128.0 {
        0x0000u16
    }
    else {
        let signed = (db * 256.0)
            .round()
            .clamp(-32768.0, 32767.0) as i16;

        (signed as i32 + 0x8000) as u16
    };

    raw.extend_from_slice(&value.to_be_bytes());
}

/// Encode raw meter bytes into Qu's 7-bit SysEx representation.
///
/// Seven raw bytes are represented by:
///
///     1 byte containing their high bits
///     7 bytes containing their low 7 bits
fn encode_meter_data(raw: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(
        raw.len() + raw.len().div_ceil(7),
    );

    for chunk in raw.chunks(7) {
        let mut high_bits = 0u8;

        for (index, byte) in chunk.iter().enumerate() {
            high_bits |= ((byte >> 7) & 0x01) << index;
        }

        encoded.push(high_bits);

        for byte in chunk {
            encoded.push(byte & 0x7f);
        }
    }

    encoded
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
