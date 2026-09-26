//! Parser for incoming Qu MIDI protocol data.
//!
//! Converts a stream of MIDI bytes into structured Qu events, handling MIDI
//! message framing, running status, SysEx messages, and Qu NRPN messages.

use crate::{
    channels::{Channel, SendDestination},
    messages::{MeterValue, MidiMessage, QuEvent},
    parameters::{MeterBlock, Parameter},
    protocol::ACTIVE_SENSE,
};

/// Tracks the components of the currently assembled Qu NRPN message.
///
/// Qu encodes a parameter change across multiple MIDI Control Change
/// messages, so the parser retains the selected channel, parameter, and
/// value components until a complete NRPN can be emitted.
#[derive(Debug, Default)]
struct NRPNState {
    channel: Option<u8>,
    parameter: Option<u8>,
    value_msb: Option<u8>,
    value_lsb: Option<u8>,
}

impl NRPNState {
    /// Clears the currently accumulated NRPN value components.
    /// The selected channel and parameter remain unchanged.
    fn reset_value(&mut self) {
        self.value_msb = None;
        self.value_lsb = None;
    }
}

#[derive(Debug, Default)]
pub struct Parser {
    running_status: Option<u8>,
    data: Vec<u8>,
    sysex: Option<Vec<u8>>,
    nrpn: NRPNState,
}

/// Parses a stream of MIDI bytes into structured [`QuEvent`] values.
///
/// The parser maintains state between calls so that MIDI messages split
/// across multiple input buffers can be assembled correctly. It handles
/// running status, SysEx framing, MIDI real-time messages, and the
/// multi-message NRPN sequences used by the Qu protocol.
impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes a sequence of MIDI bytes and returns all complete events
    /// produced from them.
    ///
    /// Parsing state is retained after the method returns, allowing a message
    /// that spans multiple calls to be completed by a later call.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<QuEvent> {
        let mut events = Vec::new();

        for &byte in bytes {
            self.push_byte(byte, &mut events);
        }

        events
    }

    /// Processes a single MIDI byte and updates the parser state.
    fn push_byte(&mut self, byte: u8, events: &mut Vec<QuEvent>) {
        // MIDI real-time messages may occur anywhere, including inside
        // another MIDI message or SysEx.
        if byte >= 0xF8 {
            if byte == ACTIVE_SENSE {
                events.push(QuEvent::ActiveSense);
            }

            return;
        }

        // SysEx start.
        if byte == 0xF0 {
            self.sysex = Some(vec![byte]);
            self.running_status = None;
            self.data.clear();
            return;
        }

        // Continue SysEx.
        if let Some(sysex) = &mut self.sysex {
            sysex.push(byte);

            if byte == 0xF7 {
                let data = std::mem::take(sysex);
                self.sysex = None;

                self.handle_sysex(data, events);
            }

            return;
        }

        // Status byte.
        if byte & 0x80 != 0 {
            self.handle_status(byte);
            return;
        }

        // Data byte without a running status.
        let Some(status) = self.running_status else {
            events.push(QuEvent::Unknown(vec![byte]));
            return;
        };

        self.data.push(byte);

        let required = match status & 0xF0 {
            0xC0 | 0xD0 => 1,
            _ => 2,
        };

        if self.data.len() < required {
            return;
        }

        let data = std::mem::take(&mut self.data);

        self.handle_message(status, &data, events);
    }

    /// Processes a MIDI status byte and updates the parser's message state.
    ///
    /// Channel Voice status bytes establish running status. System messages
    /// terminate the current running status, while SysEx start is handled by
    /// entering SysEx parsing mode.
    fn handle_status(&mut self, status: u8) {
        match status {
            0xF0 => {
                self.sysex = Some(vec![status]);
                self.running_status = None;
                self.data.clear();
            }

            // System Common messages.
            0xF1 | 0xF2 | 0xF3 | 0xF6 => {
                self.running_status = None;
                self.data.clear();
            }

            // Undefined/System Reset.
            0xF4 | 0xF5 | 0xF7 => {
                self.running_status = None;
                self.data.clear();
            }

            // Channel Voice messages.
            _ => {
                self.running_status = Some(status);
                self.data.clear();
            }
        }
    }

    /// Interprets a complete MIDI message and converts it into a [`QuEvent`].
    fn handle_message(
        &mut self,
        status: u8,
        data: &[u8],
        events: &mut Vec<QuEvent>,
    ) {
        let message_type = status & 0xF0;
        let midi_channel = status & 0x0F;

        match message_type {
            0x80 => {
                if data.len() >= 2 {
                    self.handle_note_off(
                        midi_channel,
                        data[0],
                        data[1],
                        events,
                    );
                }
            }

            0x90 => {
                if data.len() >= 2 {
                    self.handle_note_on(
                        midi_channel,
                        data[0],
                        data[1],
                        events,
                    );
                }
            }

            0xB0 => {
                if data.len() >= 2 {
                    self.handle_cc(
                        midi_channel,
                        data[0],
                        data[1],
                        events,
                    );
                }
            }

            0xC0 => {
                if let Some(&program) = data.first() {
                    events.push(QuEvent::ProgramChange {
                        channel: midi_channel,
                        program,
                    });
                }
            }

            _ => {
                events.push(QuEvent::Midi(MidiMessage::from_raw(
                    status,
                    data,
                )));
            }
        }
    }

    /// Processes a MIDI Control Change message.
    fn handle_cc(
        &mut self,
        midi_channel: u8,
        controller: u8,
        value: u8,
        events: &mut Vec<QuEvent>,
    ) {
        match controller {
            // NRPN MSB: selects the Qu channel.
            0x63 => {
                self.nrpn.channel = Some(value);
                self.nrpn.parameter = None;
                self.nrpn.reset_value();
            }

            // NRPN LSB: selects the parameter.
            0x62 => {
                self.nrpn.parameter = Some(value);
            }

            // NRPN value MSB.
            0x06 => {
                self.nrpn.value_msb = Some(value);
            }

            // NRPN value/index LSB.
            0x26 => {
                self.nrpn.value_lsb = Some(value);

                let (
                    Some(channel),
                    Some(parameter_id),
                    Some(value_msb),
                ) = (
                    self.nrpn.channel,
                    self.nrpn.parameter,
                    self.nrpn.value_msb,
                )
                else {
                    return;
                };

                let Some(parameter) = Parameter::from_id(parameter_id) else {
                    events.push(QuEvent::Unknown(vec![
                        0xB0 | midi_channel,
                        0x63,
                        channel,
                        0xB0 | midi_channel,
                        0x62,
                        parameter_id,
                        0xB0 | midi_channel,
                        0x06,
                        value_msb,
                        0xB0 | midi_channel,
                        0x26,
                        value,
                    ]));

                    return;
                };

                self.emit_nrpn(
                    Channel(channel),
                    parameter,
                    value_msb,
                    value,
                    events,
                );
            }

            // Other CC messages are not Qu NRPN messages.
            _ => {
                events.push(QuEvent::Midi(MidiMessage::ControlChange {
                    channel: midi_channel,
                    controller,
                    value,
                }));
            }
        }
    }

    fn handle_sysex(
        &mut self,
        data: Vec<u8>,
        events: &mut Vec<QuEvent>,
    ) {
        // Qu meter data SysEx:
        //
        // F0 00 00 1A 50 11 01 00 00 13 <MeterData...> F7
        //
        // 0x13 is the meter-data response. The meter payload is packed
        // into MIDI-safe 7-bit bytes.

        const METER_PREFIX: &[u8] = &[
            0xF0, 0x00, 0x00, 0x1A,
            0x50, 0x11, 0x01, 0x00, 0x00, 0x13,
        ];

        if data.starts_with(METER_PREFIX)
            && data.last() == Some(&0xF7)
        {
            let meter_data = &data[METER_PREFIX.len()..data.len() - 1];

            events.push(QuEvent::Meters {
                values: parse_meter_values(meter_data),
            });

            return;
        }

        // Qu channel-name SysEx:
        //
        // F0 00 00 1A 50 11 01 00 00 02
        //    [channel ID] [name bytes...] F7
        //
        // The channel ID uses the same numbering as the Qu channel
        // identifiers elsewhere in the protocol.

        const PREFIX: &[u8] = &[
            0xF0, 0x00, 0x00, 0x1A,
            0x50, 0x11, 0x01, 0x00, 0x00, 0x02,
        ];

        if data.len() >= PREFIX.len() + 2
            && data.starts_with(PREFIX)
            && data.last() == Some(&0xF7)
        {
            let channel_id = data[PREFIX.len()];
            let name_bytes = &data[PREFIX.len() + 1..data.len() - 1];

            if let Some(end) = name_bytes.iter().position(|&byte| byte == 0) {
                let name_bytes = &name_bytes[..end];

                if let Ok(name) = std::str::from_utf8(name_bytes) {
                    events.push(QuEvent::Name {
                        channel: Channel(channel_id),
                        name: name.to_owned(),
                    });

                    return;
                }
            }
            else if let Ok(name) = std::str::from_utf8(name_bytes) {
                events.push(QuEvent::Name {
                    channel: Channel(channel_id),
                    name: name.to_owned(),
                });

                return;
            }
        }

        // Anything that isn't a recognised channel-name or meter SysEx remains generic.
        events.push(QuEvent::SysEx(data));
    }

    /// Converts a completed Qu NRPN parameter change into a semantic event.
    fn emit_nrpn(
        &self,
        channel: Channel,
        parameter: Parameter,
        value: u8,
        index: u8,
        events: &mut Vec<QuEvent>,
    ) {
        match parameter {
            Parameter::Fader => {
                events.push(QuEvent::Fader {
                    channel,
                    value,
                });
            }

            Parameter::SendLevel => {
                events.push(QuEvent::SendLevel {
                    channel,
                    destination: SendDestination::from_index(index),
                    value,
                });
            }

            Parameter::Pan => {
                events.push(QuEvent::Pan {
                    channel,
                    destination: SendDestination::from_index(index),
                    value,
                });
            }

            Parameter::Local48v => {
                events.push(QuEvent::PhantomPower {
                    channel,
                    enabled: value != 0,
                });
            }

            Parameter::LRAssign => {
                events.push(QuEvent::LRAssign {
                    channel,
                    enabled: value != 0,
                });
            }

            Parameter::Polarity => {
                events.push(QuEvent::Polarity {
                    channel,
                    reversed: value != 0,
                });
            }

            Parameter::Pafl => {
                events.push(QuEvent::Pafl {
                    channel,
                    enabled: value != 0,
                });
            }

            _ => {
                events.push(QuEvent::Parameter {
                    channel,
                    parameter,
                    value,
                    index,
                });
            }
        }
    }

    /// Processes a MIDI Note On message according to the Qu protocol.
    fn handle_note_on(
        &mut self,
        midi_channel: u8,
        note: u8,
        velocity: u8,
        events: &mut Vec<QuEvent>,
    ) {
        // The note is the Qu channel identifier:
        //   0x20..=0x3F = Input 1..32
        //   0x40..=0x42 = Stereo 1..3
        //   0x50..=0x53 = Mute Groups 1..4
        //   0x60..=0x67 = Mix 1..9-10 and LR
        //   0x68..=0x6B = Groups 1-2..7-8
        //   0x6C..=0x6D = Matrix 1-2..3-4
        let is_qu_channel = matches!(
            note,
            0x20..=0x3F
            | 0x40..=0x42
            | 0x50..=0x53
            | 0x60..=0x6D
        );
        
        if is_qu_channel {
            if velocity == 0 {
                return;
            }
            
            // The Qu protocol defines:
            //   0x01..=0x3F = mute off
            //   0x40..=0x7F = mute on
            //   0x00       = ignored
            events.push(QuEvent::Mute {
                channel: Channel(note),
                muted: velocity >= 0x40,
            });

            return;
        }

        events.push(QuEvent::Midi(MidiMessage::NoteOn {
            channel: midi_channel,
            note,
            velocity,
        }));
    }

    /// Processes a MIDI Note Off message according to the Qu protocol.
    fn handle_note_off(
        &mut self,
        midi_channel: u8,
        note: u8,
        velocity: u8,
        events: &mut Vec<QuEvent>,
    ) {
        // Qu explicitly ignores Note Off messages for mute state.
        if (0x20..=0x3F).contains(&note) {
            return;
        }

        events.push(QuEvent::Midi(MidiMessage::NoteOff {
            channel: midi_channel,
            note,
            velocity,
        }));
    }
}

/// Parses the packed meter data from a Qu meter SysEx response.
///
/// Qu packs seven 8-bit bytes into eight MIDI-safe bytes. The first byte
/// contains the high bit for each of the following seven bytes.
fn decode_meter_data(data: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(data.len() * 7 / 8);

    for chunk in data.chunks(8) {
        if chunk.len() < 2 {
            break;
        }

        let high_bits = chunk[0];

        for i in 0..(chunk.len() - 1).min(7) {
            let high_bit = (high_bits >> (6 - i)) & 0x01;
            decoded.push(chunk[i + 1] | (high_bit << 7));
        }
    }

    decoded
}

/// Converts the packed Qu meter stream into individual meter values.
///
/// The Qu-16 meter stream contains:
///
/// 16 × Mono Input blocks (10 values)
/// 80 unused values
/// 3 × Stereo Input blocks (20 values)
/// 20 unused values
/// 4 × Mono Mix blocks (10 values)
/// 4 × Stereo Mix blocks (20 values)
/// 1 × Stereo Monitor block (16 values)
/// 4 × Stereo FX blocks (80 values)
fn parse_meter_values(data: &[u8]) -> Vec<MeterValue> {
    let decoded = decode_meter_data(data);

    decoded
        .chunks_exact(2)
        .enumerate()
        .filter_map(|(index, bytes)| {
            let (block, index_in_block) = match index {
                0..160 => (
                    MeterBlock::MonoInput((index / 10 + 1) as u8),
                    index % 10,
                ),

                160..240 => return None,

                240..300 => (
                    MeterBlock::StereoInput(((index - 240) / 20 + 1) as u8),
                    (index - 240) % 20,
                ),

                300..320 => return None,

                320..360 => (
                    MeterBlock::MonoMix(((index - 320) / 10 + 1) as u8),
                    (index - 320) % 10,
                ),

                360..440 => (
                    MeterBlock::StereoMix(((index - 360) / 20 + 1) as u8),
                    (index - 360) % 20,
                ),

                440..518 => (
                    MeterBlock::Monitor,
                    index - 440,
                ),

                518..598 => (
                    MeterBlock::Fx(((index - 518) / 20 + 1) as u8),
                    (index - 518) % 20,
                ),

                _ => return None,
            };

           let raw = u16::from_le_bytes([bytes[0], bytes[1]]);

            if index == 325 {
                println!(
                    "MonoMix(1)[5]: bytes={:02X} {:02X}, raw={:#06X}, db={}",
                    bytes[0],
                    bytes[1],
                    raw,
                    (raw.wrapping_sub(0x8000) as i16) as f32 / 256.0,
                );
            }

            // Signed 7Q8 fixed-point value with a 0x8000 offset.
            let value = raw.wrapping_sub(0x8000) as i16;

            Some(MeterValue {
                block,
                index: index_in_block,
                db: value as f32 / 256.0,
            })
        })
        .collect()
}

impl MidiMessage {
    /// Constructs a [`MidiMessage`] from a raw MIDI status byte and data bytes.
    fn from_raw(status: u8, data: &[u8]) -> Self {
        let channel = status & 0x0F;

        match status & 0xF0 {
            0x80 => Self::NoteOff {
                channel,
                note: data.first().copied().unwrap_or(0),
                velocity: data.get(1).copied().unwrap_or(0),
            },

            0x90 => Self::NoteOn {
                channel,
                note: data.first().copied().unwrap_or(0),
                velocity: data.get(1).copied().unwrap_or(0),
            },

            0xB0 => Self::ControlChange {
                channel,
                controller: data.first().copied().unwrap_or(0),
                value: data.get(1).copied().unwrap_or(0),
            },

            0xC0 => Self::ProgramChange {
                channel,
                program: data.first().copied().unwrap_or(0),
            },

            _ => Self::ControlChange {
                channel,
                controller: data.first().copied().unwrap_or(0),
                value: data.get(1).copied().unwrap_or(0),
            },
        }
    }
}
