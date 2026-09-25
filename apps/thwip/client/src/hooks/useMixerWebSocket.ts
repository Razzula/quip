import { useEffect, useRef, useState } from 'react';
import type { MixerState } from '@quip/quip';
import { applyChange, isMixerChange, isMixerState, patchState } from '../utils/qu';
import { parseMessage } from '../utils/ipc';

export type QuStatus =
    | { state: 'disconnected' }
    | { state: 'discovering' }
    | { state: 'connecting'; name: string; address: string }
    | { state: 'synchronising' }
    | { state: 'connected'; name: string; address: string }
    | { state: 'error'; message: string };

function isQuStatus(message: unknown): message is QuStatus {
    if (
        typeof message !== 'object' ||
        message === null ||
        !('state' in message) ||
        typeof message.state !== 'string'
    ) {
        return false;
    }

    switch (message.state) {
        case 'disconnected':
        case 'discovering':
        case 'synchronising':
            return true;

        case 'connecting':
        case 'connected':
            return (
                'name' in message &&
                typeof message.name === 'string' &&
                'address' in message &&
                typeof message.address === 'string'
            );

        case 'error':
            return 'message' in message && typeof message.message === 'string';

        default:
            return false;
    }
}

export function useMixerWebSocket(initialState: MixerState) {
    const [state, setState] = useState<MixerState>(initialState);
    const [isConnected, setIsConnected] = useState(false);
    const [quStatus, setQuStatus] = useState<QuStatus>({
        state: 'disconnected',
    });

    const socketRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        let retryTimeout: ReturnType<typeof setTimeout> | null = null;
        let disposed = false;

        const connect = () => {
            if (disposed) return;

            const protocol =
                window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const url = `${protocol}//${window.location.hostname}:3000/ws`;

            console.log(`Connecting to WebSocket: ${url}`);

            const socket = new WebSocket(url);
            socketRef.current = socket;

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
                    `WebSocket connection closed (code ${event.code}): ${
                        event.reason || 'no reason given'
                    }`,
                );

                if (socketRef.current === socket) {
                    socketRef.current = null;
                }

                setIsConnected(false);
                setQuStatus({ state: 'disconnected' });

                if (!disposed) {
                    retryTimeout = setTimeout(connect, 2000);
                }
            }

            socket.onmessage = (event) => {
                console.log('WebSocket message received:', event.data);

                try {
                    const message = parseMessage(event.data);

                    if (isQuStatus(message)) {
                        setQuStatus(message);
                        return;
                    }

                    if (isMixerState(message)) {
                        setState((current) => patchState(current, message));
                        return;
                    }

                    if (isMixerChange(message)) {
                        setState((current) => applyChange(current, message));
                        return;
                    }

                    console.error('[WebSocket] Unknown message:', message);
                }
                catch (error) {
                    console.error('Invalid WebSocket message:', error);
                }
            }
        }

        connect();

        return () => {
            disposed = true;

            if (retryTimeout) {
                clearTimeout(retryTimeout);
            }

            if (socketRef.current) {
                socketRef.current.close();
                socketRef.current = null;
            }
        }
    }, []);

    return {
        state,
        setState,
        socket: socketRef,
        isConnected,
        quStatus,
    };
}
