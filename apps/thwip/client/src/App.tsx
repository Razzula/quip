import { useEffect, useRef, useState } from 'react'
import {
    channelRef,
    DEFAULT_STATE,
    type ChannelState,
    type MixerChange,
    type MixerState,
    type MuteGroupState,
} from '@quip/quip'
import { ChannelBank } from './components/ChannelBank'
import { MixerSection } from './components/MixerSection'

import './App.scss'
import { applyChange, isMixerChange, isMixerState, patchState } from './utils/qu'
import { MuteButton } from './components/MuteButton'
import { parseMessage } from './utils/ipc'

const initialState: MixerState = structuredClone(DEFAULT_STATE)

function App() {
    const [state, setState] = useState<MixerState>(initialState)

    const socketRef = useRef<WebSocket | null>(null)
    const [isConnected, setIsConnected] = useState(false)

    console.log(state);

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
                    const message = parseMessage(event.data);

                    if (isMixerState(message)) {
                        console.log('[STATE] Received full mixer state')
                        setState((current) => patchState(current, message))
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
        channel: ChannelState | MuteGroupState,
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

                <div className="mixer__mute-groups">
                    {state.muteGroups.map((muteGroup) => (
                        <div
                            className={`mixer__mute-group${muteGroup.muted
                                    ? ' mixer__mute-group--muted'
                                    : ''
                                }`}
                            key={muteGroup.id}
                        >
                            <div className="channel-strip__name">
                                {muteGroup.name || muteGroup.id}
                            </div>

                            <MuteButton
                                muted={muteGroup.muted}
                                onChange={(muted) =>
                                    handleMuteChange(muteGroup, muted)
                                }
                                disabled={!isConnected}
                            />
                        </div>
                    ))}
                </div>
            </header>

            <MixerSection title="Inputs">
                <ChannelBank
                    channels={[...state.inputs, ...state.stereo]}
                    muteGroups={state.muteGroups}
                    onFaderChange={handleFaderChange}
                    onMuteChange={handleMuteChange}
                    disabled={!isConnected}
                />
            </MixerSection>

            <MixerSection title="Mixes">
                <ChannelBank
                    channels={state.mixes}
                    muteGroups={state.muteGroups}
                    onFaderChange={handleFaderChange}
                    onMuteChange={handleMuteChange}
                    disabled={!isConnected}
                />
            </MixerSection>
        </main>
    )
}

export default App
