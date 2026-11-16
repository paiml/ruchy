//! Numeric literal forms: `_` separators, 0b/0o/0x prefixes, range and digit errors.
//! Each row runs through the interpreter (`-e`) and the transpiler (`transpile`).

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn eval(src: &str) -> (bool, String, String) {
    let out = ruchy().arg("-e").arg(src).output().expect("run ruchy");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn transpile(src: &str) -> String {
    let dir = tempdir().unwrap();
    let input = dir.path().join("t.ruchy");
    fs::write(
        &input,
        format!("fun main() {{ println!(\"{{}}\", {src}) }}\n"),
    )
    .unwrap();
    let out = ruchy()
        .arg("transpile")
        .arg(&input)
        .output()
        .expect("run ruchy");
    assert!(out.status.success(), "transpile failed for {src}");
    String::from_utf8_lossy(&out.stdout).to_string()
}

const ACCEPTED: &[(&str, &str)] = &[
    ("1_000", "1000"),
    ("1_0_0_0", "1000"),
    ("1_", "1"),
    ("1__0", "10"),
    ("0b101", "5"),
    ("0B11", "3"),
    ("0b1_01", "5"),
    ("0o17", "15"),
    ("0O7", "7"),
    ("0o1_7", "15"),
    ("0xFF", "255"),
    ("0xff_ff", "65535"),
    ("0x7FFFFFFFFFFFFFFF", "9223372036854775807"),
    ("1_000.5", "1000.5"),
    ("1_0.0_1", "10.01"),
    ("1e1_0", "10000000000.0"),
    ("0b101u8", "5"),
    ("100i64", "100"),
    ("1_000i64", "1000"),
];

const REJECTED: &[(&str, &str)] = &[
    (
        "0xFFFFFFFFFFFFFFFF",
        "0xFFFFFFFFFFFFFFFF is out of range for i64 (-9223372036854775808..=9223372036854775807)",
    ),
    (
        "9223372036854775808",
        "9223372036854775808 is out of range for i64",
    ),
    ("0b", "has no digits"),
    ("0o", "has no digits"),
    ("0x_", "has no digits"),
    ("0b102", "invalid digit for base 2"),
    ("0o18", "invalid digit for base 8"),
    ("0xFG", "invalid digit for base 16"),
    ("0b1z", "invalid digit for base 2"),
    ("0b1u9", "invalid suffix"),
];

#[test]
fn accepted_forms_evaluate() {
    for (src, want) in ACCEPTED {
        let (ok, out, err) = eval(&format!("println({src})"));
        assert!(ok, "{src} failed: {err}");
        assert_eq!(out.trim(), *want, "value of {src}");
    }
}

#[test]
fn accepted_forms_transpile_to_same_value() {
    for (src, want) in ACCEPTED.iter().filter(|(_, w)| !w.contains('.')) {
        let rs = transpile(src);
        let digits = want.to_string();
        assert!(
            rs.contains(&format!("\"{{}}\", {digits}")),
            "transpile of {src} lacks {digits}: {rs}"
        );
    }
}

#[test]
fn rejected_forms_report_clear_errors() {
    for (src, msg) in REJECTED {
        let (ok, out, err) = eval(&format!("println({src})"));
        assert!(!ok, "{src} should fail, printed {out}");
        assert!(err.contains(msg), "{src}: expected `{msg}` in: {err}");
    }
}

#[test]
fn rejected_forms_fail_transpile() {
    for (src, _) in REJECTED {
        let dir = tempdir().unwrap();
        let input = dir.path().join("t.ruchy");
        fs::write(
            &input,
            format!("fun main() {{ println!(\"{{}}\", {src}) }}\n"),
        )
        .unwrap();
        ruchy().arg("transpile").arg(&input).assert().failure();
    }
}

#[test]
fn leading_underscore_is_an_identifier() {
    let (ok, _, err) = eval("println(_1)");
    assert!(!ok && err.contains("_1"), "{err}");
}
