use super::*;
// Issue #250: deterministic unit pins for Instant / Duration methods.

fn t_obj(type_name: &str, nanos: i64) -> std::collections::HashMap<String, Value> {
    let Value::Object(map) = time_object(type_name, nanos) else {
        panic!("time_object must build an Object");
    };
    (*map).clone()
}

fn dur_call(nanos: i64, method: &str) -> Value {
    eval_time_object_method("Duration", &t_obj("Duration", nanos), method, &[]).unwrap()
}

fn nanos_of(v: &Value) -> i64 {
    match v {
        Value::Object(m) => match m.get("nanos") {
            Some(Value::Integer(n)) => *n,
            other => panic!("nanos missing: {other:?}"),
        },
        other => panic!("not an object: {other:?}"),
    }
}

#[test]
fn test_issue_250_duration_unit_conversions_exact() {
    let n = 3_456_789_012_i64; // 3.456789012 s
    assert_eq!(dur_call(n, "as_nanos"), Value::Integer(3_456_789_012));
    assert_eq!(dur_call(n, "as_micros"), Value::Integer(3_456_789));
    assert_eq!(dur_call(n, "as_millis"), Value::Integer(3_456));
    assert_eq!(dur_call(n, "as_secs"), Value::Integer(3));
    assert_eq!(dur_call(n, "as_secs_f64"), Value::Float(3.456_789_012));
}

#[test]
fn test_issue_250_duration_unit_boundaries() {
    assert_eq!(dur_call(999, "as_micros"), Value::Integer(0));
    assert_eq!(dur_call(1_000, "as_micros"), Value::Integer(1));
    assert_eq!(dur_call(999_999, "as_millis"), Value::Integer(0));
    assert_eq!(dur_call(1_000_000, "as_millis"), Value::Integer(1));
    assert_eq!(dur_call(999_999_999, "as_secs"), Value::Integer(0));
    assert_eq!(dur_call(1_000_000_000, "as_secs"), Value::Integer(1));
    assert_eq!(dur_call(500_000_000, "as_secs_f64"), Value::Float(0.5));
}

#[test]
fn test_issue_250_instant_duration_since_exact() {
    let later = t_obj("Instant", 5_000_000_000);
    let earlier = time_object("Instant", 2_000_000_000);
    let d = eval_time_object_method("Instant", &later, "duration_since", &[earlier]).unwrap();
    assert_eq!(nanos_of(&d), 3_000_000_000);
    assert_eq!(
        d,
        time_object("Duration", 3_000_000_000),
        "result must be a Duration object"
    );
}

#[test]
fn test_issue_250_instant_duration_since_saturates_at_zero() {
    let earlier = t_obj("Instant", 2_000_000_000);
    let later = time_object("Instant", 5_000_000_000);
    let d = eval_time_object_method("Instant", &earlier, "duration_since", &[later]).unwrap();
    assert_eq!(nanos_of(&d), 0);
}

#[test]
fn test_issue_250_instant_duration_since_rejects_non_instant() {
    let a = t_obj("Instant", 1);
    let err = eval_time_object_method("Instant", &a, "duration_since", &[Value::Integer(1)]);
    assert!(err.is_err());
    let bad = time_object("Instant", 0);
    let Value::Object(m) = &bad else {
        unreachable!()
    };
    let mut no_nanos = (**m).clone();
    no_nanos.remove("nanos");
    let err = eval_time_object_method(
        "Instant",
        &a,
        "duration_since",
        &[Value::Object(Arc::new(no_nanos))],
    );
    assert!(err.unwrap_err().to_string().contains("expects an Instant"));
}

#[test]
fn test_issue_250_instant_elapsed_is_positive_and_bounded() {
    // An Instant 5 s in the past: elapsed must be >= 5 s (and not wildly more).
    let past = now_nanos().unwrap() - 5_000_000_000;
    let i = t_obj("Instant", past);
    let d = eval_time_object_method("Instant", &i, "elapsed", &[]).unwrap();
    let n = nanos_of(&d);
    assert!(n >= 5_000_000_000, "elapsed too small: {n}");
    assert!(n < 60_000_000_000, "elapsed too large: {n}");
    // An Instant in the future saturates to 0.
    let future = t_obj("Instant", now_nanos().unwrap() + 3_600_000_000_000);
    let d = eval_time_object_method("Instant", &future, "elapsed", &[]).unwrap();
    assert_eq!(nanos_of(&d), 0);
}

#[test]
fn test_issue_250_instant_now_is_epoch_nanos_instant() {
    let v = instant_now().unwrap();
    let n = nanos_of(&v);
    // 2020-01-01 in nanos since the epoch: guards against a wrong clock unit.
    assert!(n > 1_577_836_800_000_000_000, "not epoch nanos: {n}");
    let Value::Object(m) = &v else { unreachable!() };
    assert_eq!(
        m.get("__type"),
        Some(&Value::from_string("Instant".to_string()))
    );
}

#[test]
fn test_issue_250_time_object_unknown_method_errors() {
    let d = t_obj("Duration", 1);
    assert!(eval_time_object_method("Duration", &d, "bogus", &[]).is_err());
}
