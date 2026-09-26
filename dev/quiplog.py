#!/usr/bin/env python3

import socket
from datetime import datetime


HOST = '192.168.1.25'
PORT = 51325
LOG_FILE = 'quip.log'


# ---------------------------------------------------------------------------
# Qu MIDI
# ---------------------------------------------------------------------------

GET_SYSTEM_STATE = bytes.fromhex(
    'F0 00 00 1A 50 11 01 00 7F 10 00 F7'
)

METER_ON = bytes.fromhex(
    'F0 00 00 1A 50 11 01 00 00 12 01 F7'
)
METER_OFF = bytes.fromhex(
    'F0 00 00 1A 50 11 01 00 00 12 00 F7'
)

# Qu channel-strip numbers.
# Input 1-32 = 0x20-0x3F
# Stereo inputs = 0x40-0x42
# Mixes = 0x60-0x66
# LR = 0x67
CHANNEL_NAMES = {}

for i in range(32):
    CHANNEL_NAMES[0x20 + i] = f'CH{i + 1}'

CHANNEL_NAMES.update({
    0x40: 'ST1',
    0x41: 'ST2',
    0x42: 'ST3',

    0x60: 'Mix 1',
    0x61: 'Mix 2',
    0x62: 'Mix 3',
    0x63: 'Mix 4',
    0x64: 'Mix 5-6',
    0x65: 'Mix 7-8',
    0x66: 'Mix 9-10',
    0x67: 'LR Main',
})

for i in range(4):
    CHANNEL_NAMES[0x50 + i] = f'Mute Group {i + 1}'

for i in range(4):
    CHANNEL_NAMES[0x10 + i] = f'DCA {i + 1}'


MIX_NAMES = {
    0x00: 'Mix 1',
    0x01: 'Mix 2',
    0x02: 'Mix 3',
    0x03: 'Mix 4',
    0x04: 'Mix 5-6',
    0x05: 'Mix 7-8',
    0x06: 'Mix 9-10',
    0x07: 'LR',
    0x08: 'Group 1-2',
    0x09: 'Group 3-4',
    0x0A: 'Group 5-6',
    0x0B: 'Group 7-8',
    0x0C: 'Matrix 1-2',
    0x0D: 'Matrix 3-4',
    0x10: 'FX Send 1',
    0x11: 'FX Send 2',
    0x12: 'FX Send 3',
    0x13: 'FX Send 4',
}


# NRPN parameter IDs from the Qu MIDI protocol.
PARAMETERS = {
    0x01: 'LF EQ Gain',
    0x02: 'LF EQ Frequency',
    0x03: 'LF EQ Width',
    0x04: 'LF EQ Type',

    0x05: 'LM EQ Gain',
    0x06: 'LM EQ Frequency',
    0x07: 'LM EQ Width',

    0x09: 'HM EQ Gain',
    0x0A: 'HM EQ Frequency',
    0x0B: 'HM EQ Width',

    0x0D: 'HF EQ Gain',
    0x0E: 'HF EQ Frequency',
    0x0F: 'HF EQ Width',
    0x10: 'HF EQ Type',

    0x11: 'PEQ In/Out',
    0x12: 'USB Source',
    0x13: 'HPF Frequency',
    0x14: 'HPF In/Out',

    0x16: 'Pan',
    0x17: 'Fader',
    0x18: 'LR Assign',

    0x19: 'Local Gain',

    0x20: 'Send Level',

    0x40: 'DCA Assignment',

    0x41: 'Gate Attack',
    0x42: 'Gate Release',
    0x43: 'Gate Hold',
    0x44: 'Gate Threshold',
    0x45: 'Gate Depth',
    0x46: 'Gate In/Out',

    0x48: 'FX Delay Time MSB',
    0x49: 'FX Delay Time LSB',

    0x4C: 'Delay Time',
    0x4D: 'Delay In/Out',

    0x50: 'Mix Pre/Post',
    0x51: 'PAFL',
    0x52: 'Digital Trim',

    0x54: 'Stereo Trim',
    0x55: 'Mix Assignment',

    0x57: 'Preamp Source',
    0x58: 'dSNAKE Gain',
    0x59: 'dSNAKE Pad',
    0x5A: 'dSNAKE 48V',

    0x5C: 'Mute Group Assignment',
    0x5D: 'dSNAKE Patch',
    0x5E: 'Group/Mix Mode',
    0x5F: 'Remote Shutdown',

    0x61: 'Compressor Type',
    0x62: 'Compressor Attack',
    0x63: 'Compressor Release',
    0x64: 'Compressor Knee',
    0x65: 'Compressor Ratio',
    0x66: 'Compressor Threshold',
    0x67: 'Compressor Gain',
    0x68: 'Compressor In/Out',

    0x69: '48V Phantom Power',
    0x6A: 'Polarity',
    0x6B: 'Insert In/Out',
    0x6C: 'Delay Time',
    0x6D: 'Delay In/Out',

    0x70: 'GEQ Gain',
    0x71: 'GEQ In/Out',
}


# ---------------------------------------------------------------------------
# Formatting
# ---------------------------------------------------------------------------

def hex_bytes(data: bytes) -> str:
    return ' '.join(f'{b:02X}' for b in data)


def raw_suffix(data: bytes) -> str:
    return f' [{hex_bytes(data)}]'


def channel_name(ch: int) -> str:
    return CHANNEL_NAMES.get(ch, f'Channel 0x{ch:02X}')


def value_7bit(value: int) -> int:
    return value & 0x7F


def db_value(value: int) -> str:
    """
    Qu's standard fader/send level mapping.

    The protocol gives the important reference points:
        00 = -inf
        40 = -20 dB
        4F = -5 dB
        62 = 0 dB
        7F = +10 dB

    Intermediate values are reported as the raw MIDI value unless
    they correspond to a known reference point.
    """

    known = {
        0x00: '-inf dB',
        0x10: '-40 dB',
        0x17: '-35 dB',
        0x1F: '-30 dB',
        0x27: '-25 dB',
        0x2F: '-20 dB',
        0x36: '-15 dB',
        0x3F: '-10 dB',
        0x4F: '-5 dB',
        0x62: '0 dB',
        0x72: '+5 dB',
        0x7F: '+10 dB',
    }

    return known.get(value, f'raw {value:02X}')


# ---------------------------------------------------------------------------
# NRPN processing
# ---------------------------------------------------------------------------

class NRPNState:
    def __init__(self):
        self.channel = None
        self.parameter = None
        self.value_msb = None
        self.value_lsb = None

    def reset_value(self):
        self.value_msb = None
        self.value_lsb = None


def describe_parameter(
    channel: int,
    parameter: int,
    value: int,
    raw: bytes,
) -> str:
    ch = CHANNEL_NAMES.get(channel, f'0x{channel:02X}')
    raw_text = raw_suffix(raw)

    # Fader
    if parameter == 0x17:
        return (
            f'{ch} Fader: {db_value(value)} '
            f'(MIDI {value:02X})'
            f'{raw_text}'
        )

    # Pan
    if parameter == 0x16:
        return (
            f'{ch} Pan: {value:02X} '
            f'(MIDI {value:02X})'
            f'{raw_text}'
        )

    # LR assignment
    if parameter == 0x18:
        return (
            f'{ch} LR Assign: '
            f'{"ON" if value else "OFF"} '
            f'(MIDI {value:02X})'
            f'{raw_text}'
        )

    # Send level
    if parameter == 0x20:
        mix_name = MIX_NAMES.get(
            value >> 8,
            f'Mix {value >> 8}',
        )
        level = value & 0x7F

        return (
            f'{ch} {mix_name} Send: {db_value(level)} '
            f'(MIDI {level:02X})'
            f'{raw_text}'
        )

    # Binary parameters
    if parameter in {
        0x19,  # Polarity
        0x1A,  # Mute
        0x1B,  # PAFL
    }:
        return (
            f'{ch} {PARAMETERS.get(parameter, f"Parameter {parameter:02X}")}: '
            f'{"ON" if value else "OFF"} '
            f'(MIDI {value:02X})'
            f'{raw_text}'
        )

    # Known parameter name
    name = PARAMETERS.get(parameter)

    if name is not None:
        return (
            f'{ch} {name}: '
            f'raw {value:02X} '
            f'(MIDI {value:02X})'
            f'{raw_text}'
        )

    # Unknown parameter
    return (
        f'{ch} Parameter {parameter:02X}: '
        f'raw {value:02X} '
        f'(MIDI {value:02X})'
        f'{raw_text}'
    )


def process_nrpn_byte(
    status: int,
    data1: int,
    data2: int,
    state: NRPNState,
):
    """
    Process one MIDI Control Change message.

    NRPN sequence:

        Bn 63 CH
        Bn 62 ID
        Bn 06 VA
        Bn 26 VX
    """

    controller = data1
    value = data2

    if controller == 0x63:
        state.channel = value
        state.parameter = None
        state.reset_value()

    elif controller == 0x62:
        state.parameter = value

    elif controller == 0x06:
        state.value_msb = value

    elif controller == 0x26:
        state.value_lsb = value

        if (
            state.channel is not None
            and state.parameter is not None
            and state.value_msb is not None
        ):
            raw = bytes([
                status,
                0x63,
                state.channel,
                0x62,
                state.parameter,
                0x06,
                state.value_msb,
                0x26,
                state.value_lsb,
            ])

            # The Qu uses the MSB as the actual 7-bit value for the
            # parameters currently being decoded.
            value = state.value_msb

            return describe_parameter(
                state.channel,
                state.parameter,
                value,
                raw,
            )

    return None


# ---------------------------------------------------------------------------
# MIDI message processing
# ---------------------------------------------------------------------------

class MIDIProcessor:
    def __init__(self):
        self.running_status = None
        self.pending = bytearray()
        self.nrpn = NRPNState()

    def process(self, data: bytes) -> tuple[list[str], bool]:
        events = []
        meter_received = False

        for byte in data:

            # Active Sense: explicitly ignore it.
            if byte == 0xFE:
                continue

            # MIDI real-time messages can occur anywhere.
            if byte >= 0xF8:
                continue

            # Start of SysEx.
            if byte == 0xF0:
                self.pending.clear()
                self.pending.append(byte)
                self.running_status = None
                continue

            # Inside SysEx.
            if self.pending and self.pending[0] == 0xF0:
                self.pending.append(byte)

                if byte == 0xF7:
                    raw = bytes(self.pending)

                    events.append(
                        f'SysEx: {hex_bytes(raw)}'
                    )

                    # -------------------------------------------------------
                    # Meter data
                    # -------------------------------------------------------

                    if (
                        len(raw) >= 10
                        and raw[9] == METER_DATA_COMMAND
                    ):
                        meter_received = True

                        events.extend(
                            parse_meter_sysex(raw)
                        )

                    self.pending.clear()

                continue

            # MIDI status byte.
            if byte & 0x80:
                self.running_status = byte
                self.pending.clear()
                continue

            # No running status means this is malformed/incomplete data.
            if self.running_status is None:
                raw = bytes([byte])

                events.append(
                    f'Unexpected MIDI data byte: {byte:02X}'
                    f'{raw_suffix(raw)}'
                )
                continue

            status = self.running_status
            message_type = status & 0xF0
            midi_channel = status & 0x0F

            # All messages we care about here have two data bytes.
            self.pending.append(byte)

            if len(self.pending) < 2:
                continue

            data1 = self.pending[0]
            data2 = self.pending[1]
            self.pending.clear()

            raw = bytes([
                status,
                data1,
                data2,
            ])

            # ---------------------------------------------------------------
            # Control Change
            # ---------------------------------------------------------------

            if message_type == 0xB0:
                event = process_nrpn_byte(
                    status,
                    data1,
                    data2,
                    self.nrpn,
                )

                if event:
                    events.append(event)

            # ---------------------------------------------------------------
            # Note On = Qu mutes / PAFL
            # ---------------------------------------------------------------

            elif message_type == 0x90:
                note = data1
                velocity = data2

                if 0x20 <= note <= 0x3F:
                    channel = note - 0x20

                    if velocity >= 0x40:
                        events.append(
                            f'CH{channel + 1} MUTE: ON '
                            f'(velocity {velocity:02X})'
                            f'{raw_suffix(raw)}'
                        )
                    else:
                        events.append(
                            f'CH{channel + 1} MUTE: OFF '
                            f'(velocity {velocity:02X})'
                            f'{raw_suffix(raw)}'
                        )

                elif 0x40 <= note <= 0x5F:
                    channel = note - 0x40

                    events.append(
                        f'CH{channel + 1} PAFL: '
                        f'{"ON" if velocity else "OFF"}'
                        f'{raw_suffix(raw)}'
                    )

                elif 0x60 <= note <= 0x7F:
                    events.append(
                        f'Note On: '
                        f'channel={midi_channel + 1} '
                        f'note={note:02X} '
                        f'velocity={velocity:02X}'
                        f'{raw_suffix(raw)}'
                    )

            # ---------------------------------------------------------------
            # Note Off
            # ---------------------------------------------------------------

            elif message_type == 0x80:
                note = data1
                velocity = data2

                if 0x20 <= note <= 0x3F:
                    channel = note - 0x20

                    events.append(
                        f'CH{channel + 1} MUTE key released '
                        f'(velocity {velocity:02X})'
                        f'{raw_suffix(raw)}'
                    )

                else:
                    events.append(
                        f'Note Off: '
                        f'channel={midi_channel + 1} '
                        f'note={note:02X} '
                        f'velocity={velocity:02X}'
                        f'{raw_suffix(raw)}'
                    )

            # ---------------------------------------------------------------
            # Program Change
            # ---------------------------------------------------------------

            elif message_type == 0xC0:
                # Program Change only has one data byte, but this parser
                # currently treats MIDI messages as two-byte messages.
                events.append(
                    f'Program Change: '
                    f'channel={midi_channel + 1} '
                    f'program={data1}'
                    f'{raw_suffix(bytes([status, data1]))}'
                )

            # ---------------------------------------------------------------
            # Generic MIDI
            # ---------------------------------------------------------------

            else:
                events.append(
                    f'MIDI: {status:02X} {data1:02X} {data2:02X}'
                    f'{raw_suffix(raw)}'
                )

        return events, meter_received


# ---------------------------------------------------------------------------
# Meter processing
# ---------------------------------------------------------------------------

METER_DATA_COMMAND = 0x13

METER_NAMES = [
    'Post Preamp',
    'Post PEQ',
    'Post Compressor',
    'Post Delay',
    'Gate Side Chain',
    'Compressor Side Chain',
    'Direct Out',
    'Gate GR',
    'Compressor GR',
    'Ducker GR',
]


def unpack_7bit(data: bytes) -> bytes:
    """
    Decode Qu's 7-bit SysEx packing.

    Every group contains:
        1 byte  = high-bit flags
        7 bytes = payload bytes with their high bits removed

    The flag byte is:

        0ABCDEFG

    Therefore:
        bit 6 -> payload byte 0
        bit 5 -> payload byte 1
        ...
        bit 0 -> payload byte 6
    """

    raw = bytearray()
    index = 0

    while index < len(data):
        flags = data[index]
        index += 1

        remaining = min(7, len(data) - index)

        for i in range(remaining):
            value = data[index + i]

            if flags & (0x40 >> i):
                value |= 0x80

            raw.append(value)

        index += remaining

    return bytes(raw)


def meter_db(raw: bytes) -> float:
    """
    Convert a Qu 7Q8 meter value into dB.

    The protocol stores:
        unsigned = signed_value + 0x8000
        dB = signed_value / 256

    The two bytes are transmitted low byte first.
    """

    if len(raw) != 2:
        raise ValueError('Meter value must contain exactly two bytes')

    value = int.from_bytes(raw, 'little')
    signed = value - 0x8000

    return signed / 256.0


def describe_meter(
    raw: bytes,
    offset: int,
) -> str:
    value = raw[offset:offset + 2]

    if len(value) != 2:
        return '<missing>'

    return f'{meter_db(value):7.2f} dB'


def parse_meter_sysex(data: bytes) -> list[str]:
    """
    Parse a Qu-16 meter-data SysEx message.

    Qu-16 meter layout:

        16 Mono Input blocks
        80 unused meters
        3 Stereo Input blocks
        20 unused meters
        4 Mono Mix blocks
        4 Stereo Mix blocks
        1 Stereo Monitor block
        4 Stereo FX blocks

    Each meter is a signed 7Q8 value stored as two bytes.

    For normal channel metering, the first meter in each block
    is the post-preamp / signal meter.
    """

    if len(data) < 10:
        return []

    if data[:10] != bytes.fromhex(
        'F0 00 00 1A 50 11 01 00 00 13'
    ):
        return []

    if data[9] != METER_DATA_COMMAND:
        return []

    if data[-1] != 0xF7:
        return []

    packed = data[10:-1]
    raw = unpack_7bit(packed)

    events = []

    events.append(
        f'  Meter Data: '
        f'packed={len(packed)} bytes, '
        f'unpacked={len(raw)} bytes'
    )

    events.append(
        f'    Packed:   {hex_bytes(packed)}'
    )

    events.append(
        f'    Unpacked: {hex_bytes(raw)}'
    )

    # -----------------------------------------------------------------------
    # The first meter in each channel block is the primary channel meter.
    #
    # Mono input:
    #     10 meters × 2 bytes
    #
    # Stereo input:
    #     20 meters × 2 bytes
    #
    # There are 80 unused meters between mono and stereo inputs.
    # -----------------------------------------------------------------------

    MONO_INPUT_METERS = 10
    STEREO_INPUT_METERS = 20
    MIX_MONO_METERS = 10
    MIX_STEREO_METERS = 20
    meter_size = 2

    # -----------------------------------------------------------------------
    # Mono inputs
    # -----------------------------------------------------------------------

    mono_input_start = 0

    input_values = []

    for channel in range(16):
        offset = (
            mono_input_start
            + channel * MONO_INPUT_METERS
        ) * meter_size

        input_values.append(
            describe_meter(raw, offset)
        )

    # -----------------------------------------------------------------------
    # Stereo inputs
    # -----------------------------------------------------------------------

    stereo_input_start = (
        16 * MONO_INPUT_METERS
        + 80
    )

    stereo_values = []

    for channel in range(3):
        offset = (
            stereo_input_start
            + channel * STEREO_INPUT_METERS
        ) * meter_size

        left = describe_meter(
            raw,
            offset,
        )

        right = describe_meter(
            raw,
            offset + 10 * meter_size,
        )

        stereo_values.append(
            f'L {left} / R {right}'
        )

    # -----------------------------------------------------------------------
    # Mono mixes
    # -----------------------------------------------------------------------

    mono_mix_start = (
        stereo_input_start
        + 3 * STEREO_INPUT_METERS
        + 20
    )

    mix_values = []

    for mix in range(4):
        offset = (
            mono_mix_start
            + mix * MIX_MONO_METERS
        ) * meter_size

        mix_values.append(
            f'Mix {mix + 1} {describe_meter(
                raw,
                offset + 5 * meter_size,
            )}'
        )

    # -----------------------------------------------------------------------
    # Stereo mixes
    #
    # Mix 5-6
    # Mix 7-8
    # Mix 9-10
    # LR
    # -----------------------------------------------------------------------

    stereo_mix_start = (
        mono_mix_start
        + 4 * MIX_MONO_METERS
    )

    stereo_mix_names = (
        'Mix 5-6',
        'Mix 7-8',
        'Mix 9-10',
        'LR',
    )

    for mix, name in enumerate(stereo_mix_names):
        offset = (
            stereo_mix_start
            + mix * MIX_STEREO_METERS
        ) * meter_size

        left = describe_meter(
            raw,
            offset + 5 * meter_size,   # Post Fader L
        )

        right = describe_meter(
            raw,
            offset + 15 * meter_size,  # Post Fader R
        )

        mix_values.append(
            f'{name} L {left} / R {right}'
        )

    # -----------------------------------------------------------------------
    # Readable summary
    # -----------------------------------------------------------------------

    events.append('  Meter Values:')

    events.append(
        '    Inputs: ['
        + ', '.join(input_values)
        + ']'
    )

    events.append(
        '    STs: ['
        + ', '.join(stereo_values)
        + ']'
    )

    events.append(
        '    Mixes: ['
        + ', '.join(mix_values)
        + ']'
    )

    return events


# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

def log(message: str, file) -> None:
    timestamp = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    line = f'[{timestamp}] {message}'

    file.write(line + '\n')
    file.flush()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    processor = MIDIProcessor()

    with open(LOG_FILE, 'a') as log_file:
        log_file.write(
            f'\n--- {datetime.now().isoformat()} ---\n'
        )
        log_file.flush()

        log(
            f'Connecting to {HOST}:{PORT}...',
            log_file,
        )

        with socket.create_connection((HOST, PORT)) as sock:
            log('Connected.', log_file)

            # Initial data, normally Active Sense.
            data = sock.recv(4096)

            if data:
                events, _ = processor.process(data)

                for event in events:
                    log(event, log_file)

            # log('Requesting system state...', log_file)
            # sock.sendall(GET_SYSTEM_STATE)

            log('Enabling meter data...', log_file)
            sock.sendall(METER_ON)

            while True:
                data = sock.recv(4096)

                if not data:
                    log('Connection closed.', log_file)
                    break

                events, meter_received = processor.process(data)

                for event in events:
                    log(event, log_file)

                if meter_received:
                    log('Disabling meter data...', log_file)
                    sock.sendall(METER_OFF)
                    break

            log('Done.', log_file)

if __name__ == '__main__':
    main()
