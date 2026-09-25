import { useEffect, useRef, useState } from 'react'
import type {
    ChannelState,
    MuteGroupState,
} from '@quip/quip'
import {
    FADER_MAX_POSITION,
    FADER_MIN,
    faderPositionToValue,
    faderValueToPosition,
} from '../utils/fader'
import { getMuteState } from '../utils/mute'
import './ChannelStrip.scss'
import { MuteButton } from './MuteButton'

interface ChannelStripProps {
    channel: ChannelState;
    muteGroups: MuteGroupState[];
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
    muteGroups,
    onFaderChange,
    onMuteChange,
    disabled,
}: ChannelStripProps) {
    const fader = channel.fader ?? -Infinity;
    const [displayedFader, setDisplayedFader] = useState(fader);

    const mute = getMuteState(channel, muteGroups);

    const dragging = useRef(false);
    const animationFrame = useRef<number | null>(null);

    const faderPointer = useRef<{
        active: boolean;
        pointerType: string;
    }>({
        active: false,
        pointerType: '',
    });

    useEffect(() => {
        if (dragging.current) {
            setDisplayedFader(fader);
            return;
        }

        const target = Math.max(fader, FADER_MIN);
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

            if (
                progress >= 1 ||
                Math.abs(target - start) < 0.01
            ) {
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
            animationFrame.current =
                requestAnimationFrame(animate);
        };

        animationFrame.current =
            requestAnimationFrame(animate);

        return () => {
            if (animationFrame.current !== null) {
                cancelAnimationFrame(animationFrame.current);
                animationFrame.current = null;
            }
        };
    }, [fader]);

    const handleFaderChange = (position: number) => {
        const value = faderPositionToValue(position);

        dragging.current = true;
        setDisplayedFader(value);
        onFaderChange(value);
    };

    const getFaderPosition = (
        event: React.PointerEvent<HTMLDivElement>,
    ) => {
        const track = event.currentTarget.querySelector(
            '.channel-strip__fader-track',
        );

        if (!track) {
            return 0;
        }

        const rect = track.getBoundingClientRect();

        if (rect.height <= 0) {
            return 0;
        }

        const position =
            ((rect.bottom - event.clientY) / rect.height) *
            FADER_MAX_POSITION;

        return Math.max(
            0,
            Math.min(FADER_MAX_POSITION, position),
        );
    };

    const handleFaderPointerDown = (
        event: React.PointerEvent<HTMLDivElement>,
    ) => {
        if (disabled) return;

        const target = event.target as HTMLElement;

        const isThumb =
            target.closest(
                '[data-fader-thumb="true"]',
            ) !== null;

        /*
         * Touch and stylus interaction must begin on the thumb.
         * The track itself does nothing.
         *
         * Mouse interaction may begin anywhere on the control.
         */
        if (
            event.pointerType !== 'mouse' &&
            !isThumb
        ) {
            return;
        }

        faderPointer.current = {
            active: true,
            pointerType: event.pointerType,
        };

        dragging.current = true;

        event.currentTarget.setPointerCapture(
            event.pointerId,
        );

        /*
         * Mouse can jump directly to the clicked position.
         *
         * Touch/stylus deliberately does not update here:
         * the pointer is already on the thumb and movement
         * will determine the new value.
         */
        if (event.pointerType === 'mouse') {
            handleFaderChange(
                getFaderPosition(event),
            );
        }
    };

    const handleFaderPointerMove = (
        event: React.PointerEvent<HTMLDivElement>,
    ) => {
        if (!faderPointer.current.active) {
            return;
        }

        handleFaderChange(
            getFaderPosition(event),
        );
    };

    const handleFaderPointerUp = (
        event: React.PointerEvent<HTMLDivElement>,
    ) => {
        if (faderPointer.current.active) {
            event.currentTarget.releasePointerCapture(
                event.pointerId,
            );
        }

        faderPointer.current.active = false;
        dragging.current = false;
    };

    const handleFaderPointerCancel = (
        event: React.PointerEvent<HTMLDivElement>,
    ) => {
        if (faderPointer.current.active) {
            event.currentTarget.releasePointerCapture(
                event.pointerId,
            );
        }

        faderPointer.current.active = false;
        dragging.current = false;
    };

    const faderPosition =
        faderValueToPosition(displayedFader);

    const faderPercentage =
        (faderPosition / FADER_MAX_POSITION) * 100;

    return (
        <div
            className={`channel-strip${
                mute.muted
                    ? ' channel-strip--muted'
                    : ''
            }`}
        >
            <div className="channel-strip__name">
                {channel.name || channel.id}
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
                    <span>-30</span>
                    <span>-40</span>
                    <span>-∞</span>
                </div>

                <div
                    className="channel-strip__fader-control"
                    onPointerDown={
                        handleFaderPointerDown
                    }
                    onPointerMove={
                        handleFaderPointerMove
                    }
                    onPointerUp={
                        handleFaderPointerUp
                    }
                    onPointerCancel={
                        handleFaderPointerCancel
                    }
                    aria-label={`${
                        channel.name || channel.id
                    } fader`}
                    role="slider"
                    aria-orientation="vertical"
                    aria-valuemin={FADER_MIN}
                    aria-valuemax={10}
                    aria-valuenow={
                        displayedFader === -Infinity
                            ? FADER_MIN
                            : displayedFader
                    }
                    aria-disabled={disabled}
                >
                    <div className="channel-strip__fader-track">
                        <div
                            className="channel-strip__fader-thumb"
                            data-fader-thumb="true"
                            style={{
                                bottom: `${faderPercentage}%`,
                            }}
                        />
                    </div>
                </div>
            </div>

            <MuteButton
                muted={mute.local}
                onChange={onMuteChange}
                disabled={disabled}
            />
        </div>
    );
}
