import { useEffect, useRef, useState } from 'react'
import type { ChannelState } from '@quip/quip'
import './ChannelStrip.scss'

interface ChannelStripProps {
    channel: ChannelState;
    onFaderChange: (value: number) => void;
    onMuteChange: (muted: boolean) => void;
    disabled?: boolean;
}

function formatFader(value: number | null) {
    if (value === null || value === -Infinity) {
        return '-∞ dB';
    }

    return `${value.toFixed(1)} dB`;
}

export function ChannelStrip({
    channel,
    onFaderChange,
    onMuteChange,
    disabled,
}: ChannelStripProps) {
    const fader = channel.fader ?? -Infinity;
    const [displayedFader, setDisplayedFader] = useState(fader);
    const dragging = useRef(false);
    const animationFrame = useRef<number | null>(null);

    const faderMin = -60;

    useEffect(() => {
        if (dragging.current) {
            setDisplayedFader(fader);
            return;
        }

        const target = Math.max(fader, faderMin);
        const start = displayedFader;

        if (Math.abs(target - start) < 0.01) {
            setDisplayedFader(fader);
            return;
        }

        if (animationFrame.current !== null) {
            cancelAnimationFrame(animationFrame.current);
        }

        const startTime = performance.now();
        const duration = 999;

        const animate = (time: number) => {
            const progress = Math.min(
                (time - startTime) / duration,
                1,
            );

            if (progress >= 1 || Math.abs(target - start) < 0.01) {
                setDisplayedFader(target);
                animationFrame.current = null;
                return;
            }

            const eased = 1 - Math.pow(1 - progress, 3);
            const value = start + (target - start) * eased;

            if (Math.abs(target - value) < 1) {
                setDisplayedFader(fader);
                animationFrame.current = null;
                return;
            }

            setDisplayedFader(value);
            animationFrame.current = requestAnimationFrame(animate);
        };

        animationFrame.current = requestAnimationFrame(animate);

        return () => {
            if (animationFrame.current !== null) {
                cancelAnimationFrame(animationFrame.current);
                animationFrame.current = null;
            }
        };
    }, [fader]);

    const handleFaderChange = (value: number) => {
        dragging.current = true;
        setDisplayedFader(value);
        onFaderChange(value);
    };

    const handleFaderPointerUp = () => {
        dragging.current = false;
    };

    const faderValue = Math.max(faderMin, displayedFader);

    return (
        <div
            className={`channel-strip${channel.muted ? ' channel-strip--muted' : ''
                }`}
        >
            <div className="channel-strip__name">
                {channel.name}
            </div>

            <div className="channel-strip__value">
                {formatFader(displayedFader)}
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
                    <span>-45</span>
                    <span>-∞</span>
                </div>

                <input
                    type="range"
                    disabled={disabled}
                    min={faderMin}
                    max="10"
                    step="0.1"
                    value={faderValue}
                    onChange={(event) =>
                        handleFaderChange(Number(event.target.value))
                    }
                    onPointerDown={() => {
                        dragging.current = true;
                    }}
                    onPointerUp={handleFaderPointerUp}
                    onPointerCancel={handleFaderPointerUp}
                    aria-label={`${channel.name} fader`}
                />
            </div>

            <button
                className={`channel-strip__mute${channel.muted
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
