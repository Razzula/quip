//! Qu fader value conversion.
//!
//! The Qu protocol represents fader position as a 7-bit value from `0x00`
//! to `0x7f`. The relationship between the raw value and displayed dB is
//! non-linear.

/// Calibration points defining the Qu fader's non-linear MIDI-to-dB curve.
/// As given in protocl documentation.
///
/// Each tuple contains `(raw_value, db_value)`, where `raw_value` is the
/// 7-bit value transmitted by the Qu MIDI protocol and `db_value` is the
/// corresponding mixer fader level in decibels.
///
/// Values between calibration points are linearly interpolated.
pub const FADER_CURVE: &[(u8, f32)] = &[
    (0x00, f32::NEG_INFINITY),
    (0x0C, -45.0),
    (0x10, -40.0),
    (0x17, -35.0),
    (0x1F, -30.0),
    (0x27, -25.0),
    (0x2F, -20.0),
    (0x36, -15.0),
    (0x3F, -10.0),
    (0x4F, -5.0),
    (0x62, 0.0),
    (0x72, 5.0),
    (0x7F, 10.0),
];

/// Converts a raw Qu MIDI fader value to a dB value.
///
/// The Qu protocol represents fader positions using a 7-bit value from
/// `0x00` to `0x7F`. Because the relationship between this value and the
/// displayed fader level is non-linear, this function interpolates between
/// the calibration points in [`FADER_CURVE`].
///
/// The special value `0x00` represents negative infinity (`-∞ dB`).
///
/// Values above the final calibration point are clamped to `+10 dB`.
pub fn fader_to_db(value: u8) -> f32 {
    // handle infinity
    if value == 0 {
        return f32::NEG_INFINITY;
    }

    for window in FADER_CURVE.windows(2) {
        let [(raw_a, db_a), (raw_b, db_b)] = window else {
            unreachable!();
        };

        if *raw_a == 0 {
            continue;
        }

        if value >= *raw_a && value <= *raw_b {
            let fraction =
                (value - *raw_a) as f32 /
                (*raw_b - *raw_a) as f32;

            return db_a + fraction * (db_b - db_a);
        }
    }

    // clamp value
    10.0
}

/// Converts a dB fader value to the corresponding raw Qu MIDI value.
///
/// The Qu protocol represents fader positions using a 7-bit value from
/// `0x00` to `0x7F`. Because the relationship between this value and the
/// displayed fader level is non-linear, this function interpolates between
/// the calibration points in [`FADER_CURVE`].
///
/// Negative infinity is represented by the special raw value `0x00`.
///
/// Values at or above `+10 dB` are clamped to `0x7F`.
pub fn db_to_fader(db: f32) -> u8 {
    // handle infinity
    if db.is_infinite() && db.is_sign_negative() {
        return 0x00;
    }
    // clamp value
    if db >= 10.0 {
        return 0x7f;
    }

    for window in FADER_CURVE.windows(2) {
        let [(raw_a, db_a), (raw_b, db_b)] = window else {
            unreachable!();
        };

        if *raw_a == 0 {
            continue;
        }

        if db <= *db_b {
            let fraction = (db - *db_a) / (*db_b - *db_a);

            return (*raw_a as f32
                + fraction * (*raw_b - *raw_a) as f32)
                .round() as u8;
        }
    }

    0x7f
}
