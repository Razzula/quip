import './MuteButton.scss'

interface MuteButtonProps {
    muted: boolean | null;
    onChange: (muted: boolean) => void;
    disabled?: boolean;
    label?: string;
}

export function MuteButton({
    muted,
    onChange,
    disabled,
    label = 'MUTE',
}: MuteButtonProps) {
    return (
        <button
            className={`mute-button${
                muted ? ' mute-button--active' : ''
            }`}
            type="button"
            disabled={disabled}
            onClick={() => onChange(!muted)}
        >
            {label}
        </button>
    );
}
