import type { ChannelRef, ChannelState, MuteGroupState } from '@quip/quip';
import { channelRef } from '@quip/quip';

export interface MuteState {
    muted: boolean;
    local: boolean;
    group: boolean;
}

function sameChannel(a: ChannelRef, b: ChannelRef): boolean {
    if (a.kind !== b.kind) {
        return false;
    }

    if (a.kind === 'Lr' && b.kind === 'Lr') {
        return true;
    }

    return (
        'number' in a &&
        'number' in b &&
        a.number === b.number
    );
}

export function getMuteState(
    channel: ChannelState,
    muteGroups: MuteGroupState[],
): MuteState {
    const ref = channelRef(channel);

    const local = channel.muted === true;

    const group = muteGroups.some(
        (muteGroup) =>
            muteGroup.muted === true &&
            Array.from(muteGroup.channels).some(
                (member) => sameChannel(member, ref),
            ),
    );

    return {
        muted: local || group,
        local,
        group,
    };
}
