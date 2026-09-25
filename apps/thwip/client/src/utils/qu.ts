import type {
    ChannelRef,
    ChannelState,
    MeterState,
    MixerChange,
    MixerState,
    MuteGroupState,
} from "@quip/quip";

export const DEFAULT_METERS: MeterState = {
    inputs: Array(16).fill(-Infinity),
    stereo: Array.from({ length: 3 }, () => [-Infinity, -Infinity]),
    mixes: Array.from({ length: 8 }, () => [-Infinity, -Infinity]),
};

function channelID(channel: ChannelRef): string {
    switch (channel.kind) {
        case 'Input':
            return `CH${channel.number}`;

        case 'Stereo':
            return `ST${channel.number}`;

        case 'Mix':
            switch (channel.number) {
                case 1:
                case 2:
                case 3:
                case 4:
                    return `MIX${channel.number}`;

                case 5:
                    return 'MIX5-6';

                case 6:
                    return 'MIX7-8';

                case 7:
                    return 'MIX9-10';

                case 8:
                    return 'LR';

                default:
                    throw new Error(`Unknown mix: ${channel.number}`);
            }

        case 'Lr':
            return 'LR';

        case 'MuteGroup':
            return `MG${channel.number}`;
    }
}

function updateChannel(
    channels: ChannelState[],
    channel: ChannelRef,
    update: Partial<ChannelState>,
): ChannelState[] {
    const id = channelID(channel);

    return channels.map((item) =>
        item.id === id
            ? { ...item, ...update }
            : item,
    );
}

function updateMuteGroup(
    groups: MuteGroupState[],
    channel: ChannelRef,
    update: Partial<MuteGroupState>,
): MuteGroupState[] {
    const id = channelID(channel);

    return groups.map((group) =>
        group.id === id
            ? { ...group, ...update }
            : group,
    );
}

export function applyChange(
    state: MixerState,
    change: MixerChange,
): MixerState {
    switch (change.type) {
        case 'Fader': {
            const value = change.value ?? -Infinity;

            switch (change.channel.kind) {
                case 'Input':
                    return {
                        ...state,
                        inputs: updateChannel(
                            state.inputs,
                            change.channel,
                            { fader: value },
                        ),
                    };

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannel(
                            state.stereo,
                            change.channel,
                            { fader: value },
                        ),
                    };

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannel(
                            state.mixes,
                            change.channel,
                            { fader: value },
                        ),
                    };

                default:
                    return state;
            }
        }

        case 'Mute':
            switch (change.channel.kind) {
                case 'Input':
                    return {
                        ...state,
                        inputs: updateChannel(
                            state.inputs,
                            change.channel,
                            { muted: change.muted },
                        ),
                    };

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannel(
                            state.stereo,
                            change.channel,
                            { muted: change.muted },
                        ),
                    };

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannel(
                            state.mixes,
                            change.channel,
                            { muted: change.muted },
                        ),
                    };

                case 'MuteGroup':
                    return {
                        ...state,
                        muteGroups: updateMuteGroup(
                            state.muteGroups,
                            change.channel,
                            { muted: change.muted },
                        ),
                    };
            }

        case 'Name':
            switch (change.channel.kind) {
                case 'Input':
                    return {
                        ...state,
                        inputs: updateChannel(
                            state.inputs,
                            change.channel,
                            { name: change.value },
                        ),
                    };

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannel(
                            state.stereo,
                            change.channel,
                            { name: change.value },
                        ),
                    };

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannel(
                            state.mixes,
                            change.channel,
                            { name: change.value },
                        ),
                    };

                case 'MuteGroup':
                    return {
                        ...state,
                        muteGroups: updateMuteGroup(
                            state.muteGroups,
                            change.channel,
                            { name: change.value },
                        ),
                    };
            }
    }
}

export function patchState(
    current: MixerState,
    received: MixerState,
): MixerState {
    return {
        inputs: received.inputs.map((channel, i) => ({
            ...current.inputs[i],
            ...channel,
        })),
        stereo: received.stereo.map((channel, i) => ({
            ...current.stereo[i],
            ...channel,
        })),
        mixes: received.mixes.map((channel, i) => ({
            ...current.mixes[i],
            ...channel,
        })),
        muteGroups: received.muteGroups.map((group, i) => ({
            ...current.muteGroups[i],
            ...group,
        })),
    };
}

export function isMixerState(value: unknown): value is MixerState {
    if (!value || typeof value !== 'object') {
        return false;
    }

    const state = value as Record<string, unknown>;

    return (
        state.type === 'state' &&
        Array.isArray(state.inputs) &&
        Array.isArray(state.stereo) &&
        Array.isArray(state.mixes) &&
        Array.isArray(state.muteGroups)
    );
}

export function isMeterState(value: unknown): value is MeterState {
    if (!value || typeof value !== 'object') {
        return false;
    }

    const meters = value as Record<string, unknown>;

    return (
        meters.type === 'meters' &&
        Array.isArray(meters.inputs) &&
        Array.isArray(meters.stereo) &&
        Array.isArray(meters.mixes)
    );
}

export function isMixerChange(value: unknown): value is MixerChange {
    if (!value || typeof value !== 'object') {
        return false;
    }

    const change = value as Record<string, unknown>;

    if (
        typeof change.channel !== 'object' ||
        change.channel === null
    ) {
        return false;
    }

    switch (change.type) {
        case 'Fader':
            return (
                typeof change.value === 'number' ||
                change.value === null
            );

        case 'Mute':
            return typeof change.muted === 'boolean';

        case 'Name':
            return typeof change.name === 'string';

        default:
            return false;
    }
}
