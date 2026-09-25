import type {
    ChannelState,
    MuteGroupState,
} from '@quip/quip'
import { ChannelStrip } from './ChannelStrip'
import './ChannelBank.scss'

interface ChannelBankProps {
    channels: ChannelState[];
    meters?: (number | [number | null, number | null] | null)[];
    muteGroups: MuteGroupState[];
    compact?: boolean;
    onFaderChange: (channel: ChannelState, value: number) => void;
    onMuteChange: (channel: ChannelState, muted: boolean) => void;
    disabled?: boolean;
}

export function ChannelBank({
    channels,
    meters,
    muteGroups,
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
            {channels.map((channel, index) => (
                <ChannelStrip
                    key={channel.id}
                    channel={channel}
                    meter={meters?.[index]}
                    muteGroups={muteGroups}
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
