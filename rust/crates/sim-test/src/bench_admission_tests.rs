//! Admission parser controls: no sleeping and no processes launched.
use super::{admission_decision, parse_admission_load};

#[test]
fn child_start_is_checked_independently_of_parent_admission() {
    assert!(super::validate_timed_start(&serde_json::json!({"load_before": "7.99 10 20"})).is_ok());
    for run in [
        serde_json::json!({"load_before": "8.00 1 1"}),
        serde_json::json!({"load_before": "unknown"}),
        serde_json::json!({}),
    ] {
        assert!(super::validate_timed_start(&run).is_err());
    }
}

#[test]
fn accepts_both_uptime_formats_and_retains_all_three() {
    assert_eq!(
        parse_admission_load("7.99 8.5 9.0").unwrap(),
        [7.99, 8.5, 9.0]
    );
    assert_eq!(
        parse_admission_load("7.99, 8.5, 9.0").unwrap(),
        [7.99, 8.5, 9.0]
    );
}

#[test]
fn malformed_loads_fail_closed() {
    for raw in [
        "unknown", "", "1 2", "1 2 3 4", "NaN 1 2", "inf 1 2", "-1 2 3",
    ] {
        assert!(parse_admission_load(raw).is_err(), "{raw}");
    }
}
#[test]
fn threshold_timeout_and_invalid_are_fail_closed() {
    assert_eq!(admission_decision(&Ok([7.999, 9.0, 9.0]), false), "ADMIT");
    assert_eq!(admission_decision(&Ok([8.0, 1.0, 1.0]), false), "WAIT");
    assert_eq!(admission_decision(&Ok([8.0, 1.0, 1.0]), true), "TIMEOUT");
    assert_eq!(admission_decision(&Err("unknown".into()), false), "INVALID");
}
