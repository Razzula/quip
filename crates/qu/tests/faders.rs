use qu::faders::{db_to_fader, fader_to_db};

fn assert_db(actual: f32, expected: f32) {
    const EPSILON: f32 = 0.001;

    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {expected} dB, got {actual} dB"
    );
}

mod concepts {
    use super::*;

    #[test]
    fn zero_represents_negative_infinity() {
        let db = fader_to_db(0x00);

        assert!(db.is_infinite());
        assert!(db.is_sign_negative());
        assert_eq!(db_to_fader(f32::NEG_INFINITY), 0x00);
    }

    #[test]
    fn fader_to_db_is_monotonic() {
        let mut previous = fader_to_db(0x17);

        for raw in 0x17..=0x7f {
            let db = fader_to_db(raw);

            assert!(
                db >= previous,
                "raw 0x{raw:02X}: {db} < {previous}"
            );

            previous = db;
        }
    }

    #[test]
    fn db_to_fader_is_monotonic() {
        let mut previous = db_to_fader(f32::NEG_INFINITY);

        for db in -35..=10 {
            let raw = db_to_fader(db as f32);

            assert!(
                raw >= previous,
                "{db} dB: 0x{raw:02X} < 0x{previous:02X}"
            );

            previous = raw;
        }
    }

    #[test]
    fn fader_to_db_interpolates_between_calibration_points() {
        // 0x3F = -10 dB
        // 0x4F =  -5 dB
        // 0x47 is halfway between them.
        assert_db(fader_to_db(0x47), -7.5);

        // 0x62 = 0 dB
        // 0x72 = +5 dB
        // 0x6A is halfway between them.
        assert_db(fader_to_db(0x6a), 2.5);
    }

    #[test]
    fn db_to_fader_interpolates_between_calibration_points() {
        assert_eq!(db_to_fader(-7.5), 0x47);
        assert_eq!(db_to_fader(2.5), 0x6a);
    }

    #[test]
    fn conversion_is_bounded_at_positive_end() {
        assert_eq!(db_to_fader(10.0), 0x7f);
        assert_eq!(db_to_fader(100.0), 0x7f);
        assert_eq!(fader_to_db(0x7f), 10.0);
    }

    #[test]
    fn conversion_is_bounded_at_negative_end() {
        assert_eq!(db_to_fader(f32::NEG_INFINITY), 0x00);
        assert_eq!(fader_to_db(0x00), f32::NEG_INFINITY);
    }
}

mod data {
    use super::*;

    // BASED OFF OF qu-16-fader-mix.log

    // These are the positions explicitly observed on the Qu-16
    // during the captured full-range sweep.
    const QU16_CALIBRATION: &[(u8, f32)] = &[
        (0x00, f32::NEG_INFINITY),
        (0x17, -35.0),
        (0x1f, -30.0),
        (0x36, -15.0),
        (0x3f, -10.0),
        (0x4f, -5.0),
        (0x62, 0.0),
        (0x72, 5.0),
        (0x7f, 10.0),
    ];

    #[test]
    fn qu16_raw_values_produce_expected_db_values() {
        for &(raw, expected_db) in QU16_CALIBRATION {
            let actual_db = fader_to_db(raw);

            if expected_db.is_infinite() {
                assert!(
                    actual_db.is_infinite() && actual_db.is_sign_negative(),
                    "raw 0x{raw:02X}: expected -inf dB, got {actual_db}"
                );
            }
            else {
                assert_db(actual_db, expected_db);
            }
        }
    }

    #[test]
    fn qu16_db_values_produce_expected_raw_values() {
        for &(expected_raw, db) in QU16_CALIBRATION {
            assert_eq!(
                db_to_fader(db),
                expected_raw,
                "{db} dB should map to raw 0x{expected_raw:02X}"
            );
        }
    }

    #[test]
    fn qu16_calibration_points_round_trip() {
        for &(raw, expected_db) in QU16_CALIBRATION {
            let db = fader_to_db(raw);
            let result = db_to_fader(db);

            assert_eq!(
                result, raw,
                "raw 0x{raw:02X} -> {expected_db} dB -> 0x{result:02X}"
            );
        }
    }
}
