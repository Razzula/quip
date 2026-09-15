#!/usr/bin/env python3

import socket
from datetime import datetime


HOST = '127.0.0.1'
PORT = 51325
LOG_FILE = 'qu16.log'


# ---------------------------------------------------------------------------
# Qu MIDI
# ---------------------------------------------------------------------------

GET_SYSTEM_STATE = bytes.fromhex(
    'F0 00 00 1A 50 11 01 00 7F 10 00 F7'
)


# Qu channel-strip numbers.
# Input 1-32 = 0x20-0x3F
# Stereo inputs = 0x40-0x42
# Mixes = 0x60-0x66
# LR = 0x67
CHANNEL_NAMES = {}
for i in range(32):
    CHANNEL_NAMES[0x20 + i] = f'CH{i + 1}'

CHANNEL_NAMES[0x67] = 'LR'
CHANNEL_NAMES.update({
    0x40: 'ST1',
    0x41: 'ST2',
    0x42: 'ST3',
})
for i in range(10):
    if i < 8:
        CHANNEL_NAMES[0x60 + i] = f'Mix {i + 1}'
    else:
        CHANNEL_NAMES[0x60 + i] = f'Mix {i + 1}'

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


def describe_parameter(channel: int, parameter: int, value: int, index: int) -> str:
    ch = channel_name(channel)
    name = PARAMETERS.get(parameter, f'Unknown parameter 0x{parameter:02X}')

    # Fader
    if parameter == 0x17:
        return (
            f'{ch} Fader: {db_value(value)} '
            f'(MIDI {value:02X})'
        )

    # Send level
    if parameter == 0x20:
        destination = MIX_NAMES.get(index, f'Index 0x{index:02X}')
        return (
            f'{ch} {destination} send: {db_value(value)} '
            f'(MIDI {value:02X})'
        )

    # Pan
    if parameter == 0x16:
        if value == 0x00:
            position = 'full left'
        elif value == 0x25:
            position = 'centre'
        elif value == 0x4A:
            position = 'full right'
        else:
            position = f'raw {value:02X}'

        destination = MIX_NAMES.get(index, f'index 0x{index:02X}')

        return f'{ch} Pan ({destination}): {position}'

    # Binary parameters
    if parameter in {
        0x11, 0x14, 0x18, 0x46, 0x51,
        0x55, 0x57, 0x59, 0x5A,
        0x68, 0x69, 0x6B, 0x6D, 0x71
    }:
        state = 'ON' if value else 'OFF'
        return f'{ch} {name}: {state}'

    if parameter == 0x52:
        # Digital trim: 0x40 = 0 dB, range -24 to +24 dB.
        return f'{ch} Digital Trim: MIDI {value:02X}'

    if parameter == 0x19:
        return f'{ch} Local Gain: MIDI {value:02X}'

    if parameter in {0x01, 0x05, 0x09, 0x0D}:
        # EQ gain: 0x40 = 0 dB, range -12 to +12 dB.
        return f'{ch} {name}: MIDI {value:02X}'

    if parameter == 0x5E:
        mode = {
            0x00: 'Group mode',
            0x01: 'Mix mode',
        }.get(value, f'unknown mode {value:02X}')

        return f'{ch} {name}: {mode}'

    if parameter == 0x40:
        return f'{ch} DCA assignment: index {index:02X}, value {value:02X}'

    if parameter == 0x50:
        mode = 'Pre' if value else 'Post'
        destination = MIX_NAMES.get(index, f'index {index:02X}')
        return f'{ch} {destination}: {mode}-fader'

    if parameter == 0x5C:
        return f'{ch} Mute Group assignment: {value:02X}'

    if parameter == 0x6A:
        return f'{ch} Polarity: {"REVERSED" if value else "normal"}'

    if parameter == 0x6C:
        return f'{ch} Delay: MIDI {value:02X}'

    if parameter == 0x70:
        return f'{ch} GEQ band {index:02X}: MIDI {value:02X}'

    return (
        f'{ch} {name}: value={value:02X} '
        f'index={index:02X}'
    )


def process_nrpn_byte(status: int, data1: int, data2: int, state: NRPNState):
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

        # Some messages can be useful with MSB alone.
        # Don't emit yet because the following LSB identifies VX.

    elif controller == 0x26:
        state.value_lsb = value

        if (
            state.channel is not None
            and state.parameter is not None
            and state.value_msb is not None
        ):
            description = describe_parameter(
                state.channel,
                state.parameter,
                state.value_msb,
                state.value_lsb,
            )

            raw = (
                f'B{status & 0x0F:X} '
                f'63 {state.channel:02X} '
                f'62 {state.parameter:02X} '
                f'06 {state.value_msb:02X} '
                f'26 {state.value_lsb:02X}'
            )

            return f'{description} [{raw}]'

    return None


# ---------------------------------------------------------------------------
# MIDI message processing
# ---------------------------------------------------------------------------

class MIDIProcessor:
    def __init__(self):
        self.running_status = None
        self.pending = bytearray()
        self.nrpn = NRPNState()

    def process(self, data: bytes) -> list[str]:
        events = []

        for byte in data:

            # Active Sense: explicitly ignore it.
            if byte == 0xFE:
                continue

            # System real-time messages can occur anywhere.
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
                    events.append(
                        f'SysEx: {hex_bytes(bytes(self.pending))}'
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
                events.append(
                    f'Unexpected MIDI data byte: {byte:02X}'
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
            # Note On / Note Off = Qu mutes
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
                        )
                    else:
                        events.append(
                            f'CH{channel + 1} MUTE: OFF '
                            f'(velocity {velocity:02X})'
                        )

                elif 0x40 <= note <= 0x5F:
                    channel = note - 0x40
                    events.append(
                        f'CH{channel + 1} PAFL: '
                        f'{"ON" if velocity else "OFF"}'
                    )

                elif 0x60 <= note <= 0x7F:
                    events.append(
                        f'Note On: channel={midi_channel + 1} '
                        f'note={note:02X} velocity={velocity:02X}'
                    )

            # ---------------------------------------------------------------
            # Note Off
            # ---------------------------------------------------------------

            elif message_type == 0x80:
                note = data1

                if 0x20 <= note <= 0x3F:
                    channel = note - 0x20
                    events.append(
                        f'CH{channel + 1} MUTE key released'
                    )

            # ---------------------------------------------------------------
            # Program Change
            # ---------------------------------------------------------------

            elif message_type == 0xC0:
                events.append(
                    f'Program Change: channel={midi_channel + 1} '
                    f'program={data1}'
                )

            else:
                events.append(
                    f'MIDI: {status:02X} {data1:02X} {data2:02X}'
                )

        return events


# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

def log(message: str, file) -> None:
    timestamp = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    line = f'[{timestamp}] {message}'

    print(line)
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

            # Initial Active Sense.
            data = sock.recv(4096)

            if data:
                for event in processor.process(data):
                    log(event, log_file)

            log('Requesting system state...', log_file)

            sock.sendall(GET_SYSTEM_STATE)

            while True:
                data = sock.recv(4096)

                if not data:
                    log('Connection closed.', log_file)
                    break

                for event in processor.process(data):
                    log(event, log_file)


if __name__ == '__main__':
    main()
