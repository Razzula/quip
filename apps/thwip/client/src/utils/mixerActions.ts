import type {
    ChannelState,
    MixerChange,
    MuteGroupState,
} from '@quip/quip'
import { channelRef } from '@quip/quip'

export function sendChange(
    socket: WebSocket | null,
    change: MixerChange,
) {
    if (socket?.readyState !== WebSocket.OPEN) {
        return;
    }

    socket.send(JSON.stringify(change));
}

export function handleFaderChange(
    socket: WebSocket | null,
    channel: ChannelState,
    value: number,
) {
    sendChange(socket, {
        type: 'Fader',
        channel: channelRef(channel),
        value,
    })
}

export function handleMuteChange(
    socket: WebSocket | null,
    channel: ChannelState | MuteGroupState,
    muted: boolean,
) {
    sendChange(socket, {
        type: 'Mute',
        channel: channelRef(channel),
        muted,
    })
}
