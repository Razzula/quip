import type { ChannelState } from '@quip/quip'
import { ChannelStrip } from './ChannelStrip'
import './ChannelBank.scss'

interface ChannelBankProps {
    channels: ChannelState[];
    compact?: boolean;
    onFaderChange: (channel: ChannelState, value: number) => void;
    onMuteChange: (channel: ChannelState, muted: boolean) => void;
    disabled?: boolean;
}

export function ChannelBank({
    channels,
    compact = false,
    onFaderChange,
    onMuteChange,
    disabled,
}: ChannelBankProps) {
    return (
        <div
            className={`channel-bank${
                compact ? ' channel-bank--compact' : ''
            }`}
        >
            {channels.map((channel) => (
                <ChannelStrip
                    key={channel.name}
                    channel={channel}
                    onFaderChange={(value) =>
                        onFaderChange(channel, value)
                    }
                    onMuteChange={(muted) =>
                        onMuteChange(channel, muted)
                    }
                    disabled={disabled}
                />
            ))}
        </div>
    )
}