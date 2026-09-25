import { useRef } from 'react';
import { DEFAULT_STATE, type ChannelState, type MuteGroupState } from '@quip/quip';
import { ChannelBank } from './components/ChannelBank';
import { MixerSection } from './components/MixerSection';
import { MuteButton } from './components/MuteButton';
import { useMixerWebSocket } from './hooks/useMixerWebSocket.ts';
import { handleFaderChange, handleMuteChange } from './utils/mixerActions';

import './App.scss';
import './_colours.scss';

const initialState = structuredClone(DEFAULT_STATE);

function App() {
    const {
        state,
        meters,
        socket,
        isConnected,
        quStatus,
    } = useMixerWebSocket(initialState);

    const mainMix = state.mixes[state.mixes.length - 1];
    const mixes = state.mixes.slice(0, -1);

    const mobileLayoutRef = useRef<HTMLDivElement>(null);

    const onMobileTouchStart = (
        event: React.TouchEvent<HTMLElement>,
    ) => {
        const touch = event.touches[0];
        const width = window.innerWidth;

        const edgeWidth = Math.min(80, width * 0.15);

        if (touch.clientX <= edgeWidth) {
            mobileLayoutRef.current?.setAttribute(
                'data-active-side',
                'left',
            );
        }
        else if (touch.clientX >= width - edgeWidth) {
            mobileLayoutRef.current?.setAttribute(
                'data-active-side',
                'right',
            );
        }
    };

    const onFaderChange = (
        channel: ChannelState,
        value: number,
    ) => {
        handleFaderChange(socket.current, channel, value);
    };

    const onMuteChange = (
        channel: ChannelState | MuteGroupState,
        muted: boolean,
    ) => {
        handleMuteChange(socket.current, channel, muted);
    };

    const mixerConnected =
        isConnected && quStatus.state === 'connected';

    const channelBankProps = {
        muteGroups: state.muteGroups,
        onFaderChange,
        onMuteChange,
        disabled: !mixerConnected,
    };

    const connectionLabel = !isConnected
        ? 'Quip Disconnected'
        : quStatus.state === 'disconnected'
            ? 'Qu Disconnected'
            : quStatus.state === 'discovering'
                ? 'Discovering Qu'
                : quStatus.state === 'connecting'
                    ? `Connecting to ${quStatus.name || 'Qu'}`
                    : quStatus.state === 'synchronising'
                        ? 'Synchronising Qu'
                        : quStatus.state === 'error'
                            ? 'Qu Error'
                            : `Connected to ${quStatus.name || 'Qu'}`;

    const connectionClass = !isConnected
        ? 'mixer__connection--disconnected'
        : quStatus.state === 'connected'
            ? 'mixer__connection--connected'
            : 'mixer__connection--connecting';

    return (
        <main
            className="mixer"
            onTouchStart={onMobileTouchStart}
        >
            <header className="mixer__header">
                <div>
                    <h1>Quip</h1>
                    <span
                        className={`mixer__connection ${connectionClass}`}
                    >
                        <span className="mixer__connection-indicator" />
                        {connectionLabel}
                    </span>
                </div>

                <div className="mixer__mute-groups">
                    {state.muteGroups.map((muteGroup) => (
                        <div
                            className={`mixer__mute-group${
                                muteGroup.muted
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
                                    onMuteChange(
                                        muteGroup,
                                        muted,
                                    )
                                }
                                disabled={!mixerConnected}
                            />
                        </div>
                    ))}
                </div>
            </header>

            <div className="mixer__tall-layout">
                <MixerSection title="Inputs">
                    <ChannelBank
                        channels={[
                            ...state.inputs,
                            ...state.stereo,
                        ]}
                        meters={[
                            ...meters.inputs,
                            ...meters.stereo,
                        ]}
                        {...channelBankProps}
                    />
                </MixerSection>

                <MixerSection title="Mixes">
                    <ChannelBank
                        channels={[...state.mixes]}
                        meters={meters.mixes}
                        {...channelBankProps}
                    />
                </MixerSection>
            </div>

            <div className="mixer__short-layout">
                <div className="mixer__io-scroll">
                    <MixerSection title="Inputs">
                        <ChannelBank
                            channels={[
                                ...state.inputs,
                                ...state.stereo,
                            ]}
                            meters={[
                                ...meters.inputs,
                                ...meters.stereo,
                            ]}
                            {...channelBankProps}
                        />
                    </MixerSection>

                    <MixerSection title="Mixes">
                        <ChannelBank
                            channels={mixes}
                            meters={meters.mixes.slice(0, -1)}
                            {...channelBankProps}
                        />
                    </MixerSection>
                </div>

                {mainMix && (
                    <MixerSection
                        title={mainMix.name || 'LR Main Mix'}
                        className="mixer__main-mix"
                    >
                        <ChannelBank
                            channels={[mainMix]}
                            meters={[meters.mixes[meters.mixes.length - 1]]}
                            {...channelBankProps}
                        />
                    </MixerSection>
                )}
            </div>

            <div
                ref={mobileLayoutRef}
                className="mixer__mobile-layout"
                data-active-side="right"
            >
                <MixerSection title="Inputs">
                    <ChannelBank
                        channels={[
                            ...state.inputs,
                            ...state.stereo,
                        ]}
                        meters={[
                            ...meters.inputs,
                            ...meters.stereo,
                        ]}
                        {...channelBankProps}
                    />
                </MixerSection>

                <MixerSection title="Mixes">
                    <ChannelBank
                        channels={[...state.mixes]}
                        meters={meters.mixes}
                        {...channelBankProps}
                    />
                </MixerSection>
            </div>
        </main>
    );
}

export default App;
