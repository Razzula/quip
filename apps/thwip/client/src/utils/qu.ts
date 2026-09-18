import type { ChannelRef, ChannelState, MixerChange, MixerState, MuteGroupState } from "@quip/quip"

function channelName(channel: ChannelRef): string {
    switch (channel.kind) {
        case 'Input':
            return `CH${channel.number}`

        case 'Stereo':
            return `ST${channel.number}`

        case 'Mix':
            switch (channel.number) {
                case 1:
                case 2:
                case 3:
                case 4:
                    return `MIX${channel.number}`
                case 5:
                    return 'MIX5-6'
                case 6:
                    return 'MIX7-8'
                case 7:
                    return 'MIX9-10'
                case 8:
                    return 'LR'
                default:
                    throw new Error(`Unknown mix: ${channel.number}`)
            }
        case 'Lr':
            return 'LR'
        
        case 'MuteGroup':
            return `MG${channel.number}`;
    }
}

function updateChannel(
    channels: ChannelState[],
    channel: ChannelRef,
    update: Partial<ChannelState>,
): ChannelState[] {
    const name = channelName(channel)

    return channels.map((item) =>
        item.name === name
            ? { ...item, ...update }
            : item,
    )
}

function updateMuteGroup(
    groups: MuteGroupState[],
    channel: ChannelRef,
    update: Partial<MuteGroupState>,
): MuteGroupState[] {
    const name = channelName(channel)

    return groups.map((group) =>
        group.name === name
            ? { ...group, ...update }
            : group,
    )
}

export function applyChange(
    state: MixerState,
    change: MixerChange,
): MixerState {
    switch (change.type) {
        case 'Fader':
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
                    }

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannel(
                            state.stereo,
                            change.channel,
                            { fader: value },
                        ),
                    }

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannel(
                            state.mixes,
                            change.channel,
                            { fader: value },
                        ),
                    }
                
                default:
                    return {...state};
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
                    }

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannel(
                            state.stereo,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannel(
                            state.mixes,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }
                
                case 'MuteGroup':
                    return {
                        ...state,
                        muteGroups: updateMuteGroup(
                            state.muteGroups,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }
            }
    }

    throw new Error(`Unsupported mixer change: ${change.type}`)
}

export function patchState(
    current: MixerState,
    received: MixerState,
): MixerState {
    console.log(current.muteGroups, received.muteGroups);
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
        muteGroups:  received.muteGroups.map((channel, i) => ({
            ...current.muteGroups[i],
            ...channel,
        })),
    }
}

export function isMixerState(value: unknown): value is MixerState {
    if (!value || typeof value !== 'object') {
        return false
    }

    const state = value as Record<string, unknown>

    return (
        Array.isArray(state.inputs) &&
        Array.isArray(state.stereo) &&
        Array.isArray(state.mixes)
    )
}

export function isMixerChange(value: unknown): value is MixerChange {
    if (!value || typeof value !== 'object') {
        return false
    }

    const change = value as Record<string, unknown>

    return (
        (change.type === 'Fader' || change.type === 'Mute') &&
        typeof change.channel === 'object' &&
        change.channel !== null &&
        (
            change.type === 'Mute'
                ? typeof change.muted === 'boolean'
                : (
                    typeof change.value === 'number' ||
                    change.value === null
                )
        )
    )
}
