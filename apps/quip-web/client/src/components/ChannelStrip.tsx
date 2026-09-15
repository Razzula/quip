import type { ChannelState } from '@quip/quip'
import './ChannelStrip.scss'

interface ChannelStripProps {
    channel: ChannelState;
    onFaderChange: (value: number) => void;
    onMuteChange: (muted: boolean) => void;
    disabled?: boolean;
}

function formatFader(value: number) {
    if (value <= -60) {
        return '-∞';
    }

    return `${value.toFixed(1)} dB`;
}

export function ChannelStrip({
    channel,
    onFaderChange,
    onMuteChange,
    disabled,
}: ChannelStripProps) {
    return (
        <div
            className={`channel-strip${
                channel.muted ? ' channel-strip--muted' : ''
            }`}
        >
            <div className="channel-strip__name">{channel.name}</div>

            <div className="channel-strip__value">
                {formatFader(channel.fader)}
            </div>

            <div className="channel-strip__fader">
                <div className="channel-strip__scale">
                    <span>+10</span>
                    <span>+5</span>
                    <span>0</span>
                    <span>-5</span>
                    <span>-10</span>
                    <span>-20</span>
                    <span>-40</span>
                    <span>-∞</span>
                </div>

                <input
                    type="range"
                    disabled={disabled}
                    min="-60"
                    max="10"
                    step="0.1"
                    value={Math.max(-60, channel.fader)}
                    onChange={(event) =>
                        onFaderChange(Number(event.target.value))
                    }
                    aria-label={`${channel.name} fader`}
                />
            </div>

            <button
                className={`channel-strip__mute${
                    channel.muted
                        ? ' channel-strip__mute--active'
                        : ''
                }`}
                type="button"
                disabled={disabled}
                onClick={() => onMuteChange(!channel.muted)}
            >
                MUTE
            </button>
        </div>
    )
}
