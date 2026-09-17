export const FADER_POSITIONS = [ 10, 5, 0, -5, -10, -20, -30, -40, -Infinity ] as const;
export const FADER_MIN = -50;
export const FADER_MAX_POSITION = (FADER_POSITIONS.length - 1) * 10;

/**
 * Convert a linear slider position into a fader value.
 *
 * The slider has 10 units between each labelled position:
 *
 *   +10   0
 *    +5  10
 *     0  20
 *    -5  30
 *   -10  40
 *   -20  50
 *   -30  60
 *   -40  70
 *    -∞  80
 */
export function faderPositionToValue(position: number): number {
    if (position <= 0) {
        return FADER_POSITIONS[FADER_POSITIONS.length - 1];
    }

    if (position >= FADER_MAX_POSITION) {
        return FADER_POSITIONS[0];
    }

    const reversedPosition = FADER_MAX_POSITION - position;

    const index = Math.floor(reversedPosition / 10);
    const fraction = (reversedPosition % 10) / 10;

    const from = FADER_POSITIONS[index];
    const to = FADER_POSITIONS[index + 1];

    // There is no numerical interpolation to -∞.
    // The actual Qu-16 taper can be implemented here.
    if (to === -Infinity) {
        return from;
    }

    return from + (to - from) * fraction;
}

/**
 * Convert a fader value into the corresponding linear slider position.
 */
export function faderValueToPosition(value: number): number {
    if (value === -Infinity) {
        return 0;
    }

    if (value >= FADER_POSITIONS[0]) {
        return FADER_MAX_POSITION;
    }

    for (let index = 0; index < FADER_POSITIONS.length - 1; index++) {
        const from = FADER_POSITIONS[index];
        const to = FADER_POSITIONS[index + 1];

        if (to === -Infinity) {
            if (value <= from) {
                return FADER_MAX_POSITION - index * 10;
            }

            continue;
        }

        if (value <= from && value >= to) {
            const fraction = (from - value) / (from - to);

            return FADER_MAX_POSITION - (index * 10 + fraction * 10);
        }
    }

    return 0;
}

