export interface ChannelState {
    name: string;
    fader: number;
    muted: boolean;
}

export interface MixerState {
    inputs: ChannelState[];
    stereo: ChannelState[];
    mixes: ChannelState[];
}

function channel(name: string): ChannelState {
    return {
        name,
        fader: 0,
        muted: false,
    };
}

export type ChannelRef =
    | { kind: 'Input'; number: number }
    | { kind: 'Stereo'; number: number }
    | { kind: 'Mix'; number: number }
    | { kind: 'Lr' }

export type MixerChange =
    | {
        type: 'Fader'
        channel: ChannelRef
        value: number
    }
    | {
        type: 'Mute'
        channel: ChannelRef
        muted: boolean
    }

export function channelRef(name: string): ChannelRef {
    if (name.startsWith('CH')) {
        return {
            kind: 'Input',
            number: Number(name.slice(2)),
        }
    }

    if (name.startsWith('ST')) {
        return {
            kind: 'Stereo',
            number: Number(name.slice(2)),
        }
    }

    if (name.startsWith('MIX')) {
        return {
            kind: 'Mix',
            number: Number(name.slice(3).split('-')[0]),
        }
    }

    if (name === 'LR') {
        return {
            kind: 'Lr',
        }
    }

    throw new Error(`Unknown channel: ${name}`)
}

export const DEFAULT_STATE: MixerState = {
    inputs: Array.from({ length: 16 }, (_, i) => channel(`CH${i + 1}`)),

    stereo: Array.from({ length: 3 }, (_, i) => channel(`ST${i + 1}`)),

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
};
