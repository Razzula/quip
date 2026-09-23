import { DEFAULT_STATE, type ChannelState, type MuteGroupState } from '@quip/quip';
import { ChannelBank } from './components/ChannelBank';
import { MixerSection } from './components/MixerSection';
import { MuteButton } from './components/MuteButton';
import { useMixerWebSocket } from './hooks/useMixerWebSocket.ts';
import { handleFaderChange, handleMuteChange } from './utils/mixerActions';

import './App.scss';

const initialState = structuredClone(DEFAULT_STATE);

function App() {
    const { state, socket, isConnected } = useMixerWebSocket(initialState);

    const mainMix = state.mixes[state.mixes.length - 1];
    const mixes = state.mixes.slice(0, -1);

    const onFaderChange = (
        channel: ChannelState,
        value: number,
    ) => {
        handleFaderChange(socket.current, channel, value);
    }

    const onMuteChange = (
        channel: ChannelState | MuteGroupState,
        muted: boolean,
    ) => {
        handleMuteChange(socket.current, channel, muted);
    }

    const channelBankProps = {
        muteGroups: state.muteGroups,
        onFaderChange,
        onMuteChange,
        disabled: !isConnected,
    };

    return (
        <main className="mixer">
            <header className="mixer__header">
                <div>
                    <h1>Quip</h1>

                    <span
                        className={`mixer__connection${
                            isConnected
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
                                disabled={!isConnected}
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
                        {...channelBankProps}
                    />
                </MixerSection>

                <MixerSection title="Mixes">
                    <ChannelBank
                        channels={[...state.mixes]}
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
                            {...channelBankProps}
                        />
                    </MixerSection>

                    <MixerSection title="Mixes">
                        <ChannelBank
                            channels={mixes}
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
                            {...channelBankProps}
                        />
                    </MixerSection>
                )}
            </div>

            <div className="mixer__mobile-layout">
                <MixerSection title="Inputs">
                    <ChannelBank
                        channels={[
                            ...state.inputs,
                            ...state.stereo,
                        ]}
                        {...channelBankProps}
                    />
                </MixerSection>

                <MixerSection title="Mixes">
                    <ChannelBank
                        channels={[...state.mixes]}
                        {...channelBankProps}
                    />
                </MixerSection>
            </div>
        </main>
    )
}

export default App;
