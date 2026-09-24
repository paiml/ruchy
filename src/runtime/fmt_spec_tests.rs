//! FMTSPEC-1 unit and property tests: every expected string is computed by
//! Rust's own `format!` in the test, so the oracle is std::fmt itself.

use super::*;
use proptest::prelude::*;

fn fmt(template: &str, args: &[Value]) -> Result<String, String> {
    format_str(template, args, &|_| None)
}

fn s(text: &str) -> Value {
    Value::from_string(text.to_string())
}

#[test]
fn test_fmtspec_1_brief_case() {
    let got = fmt(
        "[{:>5}] [{:.2}] [{0}]",
        &[Value::Integer(7), Value::Float(3.14159)],
    );
    assert_eq!(got.unwrap(), format!("[{:>5}] [{:.2}] [{0}]", 7, 3.14159));
}

#[test]
fn test_fmtspec_1_escapes() {
    assert_eq!(fmt("{{}} {{{}}}", &[Value::Integer(1)]).unwrap(), "{} {1}");
}

#[test]
fn test_fmtspec_1_named_resolver() {
    let got = format_str("{a}-{b:>3}", &[], &|n| match n {
        "a" => Some(Value::Integer(1)),
        "b" => Some(s("z")),
        _ => None,
    });
    assert_eq!(got.unwrap(), format!("{}-{:>3}", 1, "z"));
    let missing = format_str("{nope}", &[], &|_| None).unwrap_err();
    assert!(missing.contains("nope"), "{missing}");
}

#[test]
fn test_fmtspec_1_radix_sign_alternate_zero() {
    let args: Vec<Value> = [255, 255, 5, 8, 255, 5, 8, 5, 255, -1, 255]
        .iter()
        .map(|i| Value::Integer(*i))
        .collect();
    let got = fmt(
        "{:x} {:X} {:b} {:o} {:#x} {:#b} {:#o} {:08b} {:#06x} {:x} {:+X}",
        &args,
    );
    let want = format!(
        "{:x} {:X} {:b} {:o} {:#x} {:#b} {:#o} {:08b} {:#06x} {:x} {:+X}",
        255, 255, 5, 8, 255, 5, 8, 5, 255, -1i64, 255
    );
    assert_eq!(got.unwrap(), want);
}

#[test]
fn test_fmtspec_1_exp_forms() {
    let args = [
        Value::Float(1234.5),
        Value::Float(0.00012),
        Value::Integer(1234),
        Value::Float(1234.5),
        Value::Float(1.5),
    ];
    let got = fmt("{:e} {:E} {:e} {:>10.2e} {:+e}", &args);
    let want = format!(
        "{:e} {:E} {:e} {:>10.2e} {:+e}",
        1234.5, 0.00012, 1234i64, 1234.5, 1.5
    );
    assert_eq!(got.unwrap(), want);
}

#[test]
fn test_fmtspec_1_debug_forms() {
    let arr = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);
    let got = fmt(
        "{:?}|{:5?}|{:>6?}|{:#?}|{:08.2?}",
        &[arr.clone(), arr.clone(), s("ab"), arr, Value::Float(-1.5)],
    );
    let v = vec![1, 2];
    let want = format!("{:?}|{:5?}|{:>6?}|{:#?}|{:08.2?}", v, v, "ab", v, -1.5);
    assert_eq!(got.unwrap(), want);
}

#[test]
fn test_fmtspec_1_pretty_tuple_and_variant() {
    let tuple = Value::Tuple(vec![Value::Integer(1)].into());
    let some = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        data: Some(vec![Value::Integer(3)]),
    };
    let got = fmt("{:#?}|{:?}|{:#?}", &[tuple.clone(), tuple, some]);
    assert_eq!(
        got.unwrap(),
        format!("{:#?}|{:?}|{:#?}", (1,), (1,), Some(3))
    );
}

#[test]
fn test_fmtspec_1_argument_counts_are_refused() {
    for bad in ["{:1$}", "{:.*}", "{:w$}", "{:.1$}"] {
        let err = fmt(bad, &[Value::Integer(1), Value::Integer(2)]).unwrap_err();
        assert!(err.contains("$") || err.contains(".*"), "{bad}: {err}");
    }
}

#[test]
fn test_fmtspec_1_malformed_strings_are_errors() {
    for bad in ["{", "}", "{:q}", "{0", "{a b}", "{:.}"] {
        assert!(fmt(bad, &[Value::Integer(1)]).is_err(), "{bad}");
    }
    assert!(fmt("{} {}", &[Value::Integer(1)]).is_err());
    assert!(fmt("{3}", &[Value::Integer(1)]).is_err());
    assert!(fmt("{:x}", &[Value::Float(1.0)]).is_err());
    assert!(fmt("{:x}", &[s("a")]).is_err());
}

#[test]
fn test_fmtspec_1_format_call_legacy_join() {
    let named = |_: &str| None;
    let joined = format_call(&[s("a"), Value::Integer(1)], true, &named);
    assert_eq!(joined.unwrap(), "a 1");
    assert_eq!(format_call(&[s("x{{")], false, &named).unwrap(), "x{{");
    assert_eq!(format_call(&[s("x{{")], true, &named).unwrap(), "x{");
    assert_eq!(format_call(&[], true, &named).unwrap(), "");
}

#[test]
fn test_fmtspec_1_struct_and_map_debug() {
    let mut fields = HashMap::new();
    fields.insert("a".to_string(), Value::Integer(1));
    let st = Value::Struct {
        name: "P".to_string(),
        fields: std::sync::Arc::new(fields.clone()),
    };
    let map = Value::Object(std::sync::Arc::new(fields));
    let got = fmt("{:?}|{:#?}|{:?}|{:#?}", &[st.clone(), st, map.clone(), map]);
    #[derive(Debug)]
    #[allow(dead_code)]
    struct P {
        a: i64,
    }
    let m: std::collections::BTreeMap<&str, i64> = [("a", 1)].into_iter().collect();
    let want = format!("{:?}|{:#?}|{:?}|{:#?}", P { a: 1 }, P { a: 1 }, m, m);
    assert_eq!(got.unwrap(), want);
}

/// Rust's own output for a spec prefix chosen by index, with runtime width
/// and precision.
macro_rules! rust_ref {
    ($idx:expr, $v:expr, $w:expr, $p:expr) => {
        match $idx {
            0 => format!("{:w$.p$}", $v, w = $w, p = $p),
            1 => format!("{:<w$.p$}", $v, w = $w, p = $p),
            2 => format!("{:^w$.p$}", $v, w = $w, p = $p),
            3 => format!("{:>w$.p$}", $v, w = $w, p = $p),
            4 => format!("{:*<w$.p$}", $v, w = $w, p = $p),
            5 => format!("{:*^w$.p$}", $v, w = $w, p = $p),
            6 => format!("{:*>w$.p$}", $v, w = $w, p = $p),
            7 => format!("{:0w$.p$}", $v, w = $w, p = $p),
            _ => format!("{:+w$.p$}", $v, w = $w, p = $p),
        }
    };
}

const PREFIXES: [&str; 9] = ["", "<", "^", ">", "*<", "*^", "*>", "0", "+"];

fn ours(idx: usize, value: &Value, w: usize, p: usize) -> String {
    let spec = format!("{}{w}.{p}", PREFIXES[idx]);
    fmt(&format!("{{:{spec}}}"), std::slice::from_ref(value)).expect("valid spec")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn prop_fmtspec_1_ints_match_rust(idx in 0usize..9, v in -100_000i64..100_000, w in 1usize..14, p in 0usize..6) {
        prop_assert_eq!(ours(idx, &Value::Integer(v), w, p), rust_ref!(idx, v, w, p));
    }

    #[test]
    fn prop_fmtspec_1_floats_match_rust(idx in 0usize..9, v in -1.0e5f64..1.0e5, w in 1usize..14, p in 0usize..6) {
        prop_assert_eq!(ours(idx, &Value::Float(v), w, p), rust_ref!(idx, v, w, p));
    }

    #[test]
    fn prop_fmtspec_1_strings_match_rust(idx in 0usize..9, v in "[a-z]{0,8}", w in 1usize..14, p in 0usize..10) {
        prop_assert_eq!(ours(idx, &s(&v), w, p), rust_ref!(idx, v.as_str(), w, p));
    }
}

#[test]
fn test_fmtextra_1_extra_positional_argument_is_an_error() {
    let err = fmt("{}", &[Value::Integer(1), Value::Integer(2)]).unwrap_err();
    assert!(err.contains("argument never used"), "{err}");
    assert!(err.contains("argument 1"), "{err}");
}

#[test]
fn test_fmtextra_1_unreferenced_explicit_index_is_an_error() {
    let err = fmt("{0} {0}", &[Value::Integer(1), Value::Integer(2)]).unwrap_err();
    assert!(err.contains("argument 1"), "{err}");
    let err = fmt("{1}", &[Value::Integer(1), Value::Integer(2)]).unwrap_err();
    assert!(err.contains("argument 0"), "{err}");
}

#[test]
fn test_fmtextra_1_explicit_indices_count_as_uses() {
    let args = [Value::Integer(1), Value::Integer(2)];
    assert_eq!(
        fmt("{1} {0} {}", &args).unwrap(),
        format!("{1} {0} {}", 1, 2)
    );
    assert_eq!(fmt("{} {0}", &args[..1]).unwrap(), format!("{} {0}", 1));
    assert_eq!(fmt("{0}{}{}", &args).unwrap(), format!("{0}{}{}", 1, 2));
}

#[test]
fn test_fmtextra_1_named_fields_leave_positional_arguments_unused() {
    let named = |n: &str| (n == "x").then_some(Value::Integer(5));
    let err = format_str("{x}", &[Value::Integer(1)], &named).unwrap_err();
    assert!(err.contains("argument never used"), "{err}");
    assert_eq!(
        format_str("{x} {}", &[Value::Integer(1)], &named).unwrap(),
        "5 1"
    );
}

#[test]
fn test_fmtextra_1_format_call_template_with_extra_argument_errors() {
    let named = |_: &str| None;
    let values = [s("{}"), Value::Integer(1), Value::Integer(2)];
    assert!(format_call(&values, true, &named).is_err());
    assert_eq!(format_call(&values, false, &named).unwrap(), "{} 1 2");
}
