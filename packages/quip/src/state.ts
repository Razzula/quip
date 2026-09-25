export interface ChannelState {
    id: string;
    name: string | null;
    fader: number | null;
    muted: boolean | null;
}

export interface MuteGroupState {
    id: string;
    name: string | null;
    muted: boolean | null;
    channels: Set<ChannelRef>;
}

export interface MixerState {
    inputs: ChannelState[];
    stereo: ChannelState[];
    mixes: ChannelState[];
    muteGroups: MuteGroupState[];
}

export interface MeterState {
    inputs: (number | null)[];
    stereo: [number | null, number | null][];
    mixes: [number | null, number | null][];
}

function channel(id: string): ChannelState {
    return {
        id,
        name: null,
        fader: null,
        muted: null,
    };
}

function muteGroup(id: string): MuteGroupState {
    return {
        id,
        name: null,
        muted: null,
        channels: new Set([]),
    };
}

export type ChannelRef =
    | { kind: 'Input'; number: number; }
    | { kind: 'Stereo'; number: number; }
    | { kind: 'Mix'; number: number; }
    | { kind: 'Lr'; }
    | { kind: 'MuteGroup'; number: number; };

export type MixerChange =
    | {
        type: 'Fader';
        channel: ChannelRef;
        value: number | null;
    }
    | {
        type: 'Mute';
        channel: ChannelRef;
        muted: boolean;
    }
    | {
        type: 'Name';
        channel: ChannelRef;
        value: string;
    };

export function channelRef(channel: ChannelState | MuteGroupState): ChannelRef {
    if (channel.id.startsWith('CH')) {
        return {
            kind: 'Input',
            number: Number(channel.id.slice(2)),
        };
    }

    if (channel.id.startsWith('ST')) {
        return {
            kind: 'Stereo',
            number: Number(channel.id.slice(2)),
        };
    }

    switch (channel.id) {
        case 'MIX1':
            return { kind: 'Mix', number: 1 };

        case 'MIX2':
            return { kind: 'Mix', number: 2 };

        case 'MIX3':
            return { kind: 'Mix', number: 3 };

        case 'MIX4':
            return { kind: 'Mix', number: 4 };

        case 'MIX5-6':
            return { kind: 'Mix', number: 5 };

        case 'MIX7-8':
            return { kind: 'Mix', number: 6 };

        case 'MIX9-10':
            return { kind: 'Mix', number: 7 };

        case 'LR':
            return { kind: 'Lr' };
    }

    if (channel.id.startsWith('MG')) {
        return {
            kind: 'MuteGroup',
            number: Number(channel.id.slice(2)),
        };
    }

    throw new Error(`Unknown channel: ${channel.name}`);
}

export const DEFAULT_STATE: MixerState = {
    inputs: Array.from(
        { length: 16 },
        (_, i) => channel(`CH${i + 1}`),
    ),

    stereo: Array.from(
        { length: 3 },
        (_, i) => channel(`ST${i + 1}`),
    ),

    mixes: [
        channel('MIX1'),
        channel('MIX2'),
        channel('MIX3'),
        channel('MIX4'),
        channel('MIX5-6'),
        channel('MIX7-8'),
        channel('MIX9-10'),
        channel('LR'),
    ],

    muteGroups: Array.from(
        { length: 4 },
        (_, i) => muteGroup(`MG${i + 1}`),
    ),
};

export const DEFAULT_METERS: MeterState = {
    inputs: Array.from({ length: 16 }, () => null),

    stereo: Array.from(
        { length: 3 },
        () => [null, null],
    ),

    mixes: Array.from(
        { length: 8 },
        () => [null, null],
    ),
};
