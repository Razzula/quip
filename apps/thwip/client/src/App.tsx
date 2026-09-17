import { useEffect, useRef, useState } from 'react'
import {
    DEFAULT_STATE,
    type ChannelRef,
    type ChannelState,
    type MixerChange,
    type MixerState,
} from '@quip/quip'
import { ChannelBank } from './components/ChannelBank'
import { MixerSection } from './components/MixerSection'

import './App.scss'

const initialState: MixerState = structuredClone(DEFAULT_STATE)

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
    }
}

function updateChannels(
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

function applyChange(
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
                        inputs: updateChannels(
                            state.inputs,
                            change.channel,
                            { fader: value },
                        ),
                    }

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannels(
                            state.stereo,
                            change.channel,
                            { fader: value },
                        ),
                    }

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannels(
                            state.mixes,
                            change.channel,
                            { fader: value },
                        ),
                    }
            }

        case 'Mute':
            switch (change.channel.kind) {
                case 'Input':
                    return {
                        ...state,
                        inputs: updateChannels(
                            state.inputs,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }

                case 'Stereo':
                    return {
                        ...state,
                        stereo: updateChannels(
                            state.stereo,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }

                case 'Mix':
                case 'Lr':
                    return {
                        ...state,
                        mixes: updateChannels(
                            state.mixes,
                            change.channel,
                            { muted: change.muted },
                        ),
                    }
            }
    }
}

function isMixerState(value: unknown): value is MixerState {
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

function isMixerChange(value: unknown): value is MixerChange {
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

function App() {
    const [state, setState] = useState<MixerState>(initialState)

    const socketRef = useRef<WebSocket | null>(null)
    const [isConnected, setIsConnected] = useState(false)

    useEffect(() => {
        let retryTimeout: ReturnType<typeof setTimeout> | null = null
        let disposed = false

        const connect = () => {
            if (disposed) return

            const protocol =
                window.location.protocol === 'https:' ? 'wss:' : 'ws:'
            const url = `${protocol}//${window.location.hostname}:3000/ws`

            console.log(`Connecting to WebSocket: ${url}`)

            const socket = new WebSocket(url)
            socketRef.current = socket

            socket.onopen = () => {
                console.log('WebSocket connection established');
                setIsConnected(true);
            }

            socket.onerror = (event) => {
                console.error('WebSocket connection error:', event);
                setIsConnected(false);
            }

            socket.onclose = (event) => {
                console.log(
                    `WebSocket connection closed (code ${event.code}): ${event.reason || 'no reason given'
                    }`,
                )

                if (socketRef.current === socket) {
                    socketRef.current = null
                }
                setIsConnected(false);

                // Try again after 2 seconds
                if (!disposed) {
                    retryTimeout = setTimeout(connect, 2000)
                }
            }

            socket.onmessage = (event) => {
                console.log('WebSocket message received:', event.data)

                try {
                    const message: unknown = JSON.parse(event.data)

                    if (isMixerState(message)) {
                        console.log('[STATE] Received full mixer state')
                        setState(message)
                        return
                    }

                    if (isMixerChange(message)) {
                        console.log('[STATE] Received mixer change:', message)
                        setState((current) => applyChange(current, message))
                        return
                    }

                    console.error('[WebSocket] Unknown message:', message)
                } catch (error) {
                    console.error('Invalid WebSocket message:', error)
                }
            }
        }

        connect()

        return () => {
            disposed = true

            if (retryTimeout) {
                clearTimeout(retryTimeout)
            }

            console.log('Closing WebSocket connection')

            if (socketRef.current) {
                socketRef.current.close()
                socketRef.current = null
            }
        }
    }, []);

    function sendChange(change: MixerChange) {
        const socket = socketRef.current

        if (socket?.readyState !== WebSocket.OPEN) {
            return
        }

        socket.send(JSON.stringify(change))
    }

    function channelRef(channel: ChannelState): ChannelRef {
        if (channel.name.startsWith('CH')) {
            return {
                kind: 'Input',
                number: Number(channel.name.slice(2)),
            }
        }

        if (channel.name.startsWith('ST')) {
            return {
                kind: 'Stereo',
                number: Number(channel.name.slice(2)),
            }
        }

        switch (channel.name) {
            case 'MIX1':
                return { kind: 'Mix', number: 1 }
            case 'MIX2':
                return { kind: 'Mix', number: 2 }
            case 'MIX3':
                return { kind: 'Mix', number: 3 }
            case 'MIX4':
                return { kind: 'Mix', number: 4 }
            case 'MIX5-6':
                return { kind: 'Mix', number: 5 }
            case 'MIX7-8':
                return { kind: 'Mix', number: 6 }
            case 'MIX9-10':
                return { kind: 'Mix', number: 7 }
            case 'LR':
                return { kind: 'Lr' }
        }

        throw new Error(`Unknown channel: ${channel.name}`)
    }

    function handleFaderChange(
        channel: ChannelState,
        value: number,
    ) {
        sendChange({
            type: 'Fader',
            channel: channelRef(channel),
            value,
        })
    }

    function handleMuteChange(
        channel: ChannelState,
        muted: boolean,
    ) {
        sendChange({
            type: 'Mute',
            channel: channelRef(channel),
            muted,
        })
    }

    return (
        <main className="mixer">
            <header className="mixer__header">
                <div>
                    <h1>Quip</h1>

                    <span
                        className={`mixer__connection${isConnected
                            ? ' mixer__connection--connected'
                            : ' mixer__connection--connecting'
                            }`}
                    >
                        <span className="mixer__connection-indicator" />
                        {isConnected ? 'Connected' : 'Not Connected'}
                    </span>
                </div>
            </header>

            <MixerSection title="Inputs">
                <ChannelBank
                    channels={[...state.inputs, ...state.stereo]}
                    onFaderChange={handleFaderChange}
                    onMuteChange={handleMuteChange}
                    disabled={!isConnected}
                />
            </MixerSection>

            <MixerSection title="Mixes">
                <ChannelBank
                    channels={state.mixes}
                    onFaderChange={handleFaderChange}
                    onMuteChange={handleMuteChange}
                    disabled={!isConnected}
                />
            </MixerSection>
        </main>
    )
}

export default App
