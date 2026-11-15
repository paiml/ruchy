//! Issue #250: `ruchy run` (interpreter) must provide `std::time::Instant`,
//! `Duration` accessors and the `time_micros()` builtin, as compiled output does.

use std::fs;
use tempfile::tempdir;

fn run_ruchy(src: &str) -> (bool, String, String) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("t.ruchy");
    fs::write(&path, src).unwrap();
    let out = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .arg(&path)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn test_issue_250_instant_now_elapsed_as_nanos() {
    let (ok, stdout, stderr) = run_ruchy(
        "use std::time::Instant;\nfun main() {\n  let t0 = Instant::now()\n  println!(\"{}\", t0.elapsed().as_nanos() >= 0)\n}\n",
    );
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "true");
}

#[test]
fn test_issue_250_duration_accessors() {
    let (ok, stdout, stderr) = run_ruchy(
        "use std::time::Instant;\nfun main() {\n  let d = Instant::now().elapsed()\n  println!(\"{} {} {} {}\", d.as_micros() >= 0, d.as_millis() >= 0, d.as_secs() >= 0, d.as_secs_f64() >= 0.0)\n}\n",
    );
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "true true true true");
}

#[test]
fn test_issue_250_time_micros_builtin() {
    let (ok, stdout, stderr) =
        run_ruchy("fun main() {\n  let t = time_micros()\n  println!(\"{}\", t > 0)\n}\n");
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "true");
}

/// Run a script and parse whitespace-separated stdout fields as f64.
fn run_nums(src: &str) -> Vec<f64> {
    let (ok, stdout, stderr) = run_ruchy(src);
    assert!(ok, "ruchy run failed: {stderr}");
    stdout
        .split_whitespace()
        .map(|s| {
            s.parse::<f64>()
                .unwrap_or_else(|_| panic!("not a number: {s:?}"))
        })
        .collect()
}

#[test]
fn test_issue_250_elapsed_units_after_known_sleep() {
    // Sleep 200 ms: each unit must land in a band only the right divisor satisfies.
    let v = run_nums(
        "use std::time::Instant;\nfun main() {\n  let t0 = Instant::now()\n  sleep(200)\n  let d = t0.elapsed()\n  println!(\"{} {} {} {} {}\", d.as_nanos(), d.as_micros(), d.as_millis(), d.as_secs(), d.as_secs_f64())\n}\n",
    );
    let (ns, us, ms, s, sf) = (v[0], v[1], v[2], v[3], v[4]);
    assert!((2.0e8..5.0e9).contains(&ns), "as_nanos {ns}");
    assert!((2.0e5..5.0e6).contains(&us), "as_micros {us}");
    assert!((200.0..5000.0).contains(&ms), "as_millis {ms}");
    assert!((0.0..5.0).contains(&s), "as_secs {s}");
    assert!((0.2..5.0).contains(&sf), "as_secs_f64 {sf}");
    // Cross-unit consistency (truncating integer division).
    assert_eq!(us, (ns / 1000.0).floor(), "micros == nanos/1000");
    assert_eq!(ms, (ns / 1_000_000.0).floor(), "millis == nanos/1e6");
    assert_eq!(s, (ns / 1e9).floor(), "secs == nanos/1e9");
    assert!((sf - ns / 1e9).abs() < 1e-9, "secs_f64 == nanos/1e9");
}

#[test]
fn test_issue_250_duration_since_units_and_order() {
    let v = run_nums(
        "use std::time::Instant;\nfun main() {\n  let a = Instant::now()\n  sleep(150)\n  let b = Instant::now()\n  let fwd = b.duration_since(a)\n  let rev = a.duration_since(b)\n  println!(\"{} {} {} {}\", fwd.as_millis(), fwd.as_micros(), fwd.as_secs_f64(), rev.as_nanos())\n}\n",
    );
    let (ms, us, sf, rev) = (v[0], v[1], v[2], v[3]);
    assert!((150.0..5000.0).contains(&ms), "fwd millis {ms}");
    assert!((1.5e5..5.0e6).contains(&us), "fwd micros {us}");
    assert!((0.15..5.0).contains(&sf), "fwd secs_f64 {sf}");
    assert_eq!(rev, 0.0, "earlier.duration_since(later) saturates to 0");
}

#[test]
fn test_issue_250_elapsed_grows_with_time() {
    // Catches `now - nanos` -> `now + nanos` and a dropped sign: a later
    // elapsed must be at least 100 ms larger than an earlier one.
    let v = run_nums(
        "use std::time::Instant;\nfun main() {\n  let t0 = Instant::now()\n  let e1 = t0.elapsed().as_millis()\n  sleep(100)\n  let e2 = t0.elapsed().as_millis()\n  println!(\"{} {}\", e1, e2)\n}\n",
    );
    assert!(v[0] >= 0.0 && v[0] < 5000.0, "e1 {}", v[0]);
    assert!(v[1] - v[0] >= 99.0, "e2-e1 = {}", v[1] - v[0]);
}

#[test]
fn test_issue_250_time_micros_unit_against_timestamp_ms() {
    // timestamp() is epoch milliseconds: time_micros()/1000 must sit between two reads.
    let v = run_nums(
        "fun main() {\n  let m0 = timestamp()\n  let u = time_micros()\n  let m1 = timestamp()\n  println!(\"{} {} {}\", m0, u, m1)\n}\n",
    );
    let (m0, u, m1) = (v[0], v[1], v[2]);
    assert!(u > 1.5e15, "time_micros not epoch micros: {u}");
    let um = (u / 1000.0).floor();
    assert!(
        m0 <= um && um <= m1,
        "micros/1000 {um} not within [{m0}, {m1}]"
    );
}

#[test]
fn test_issue_250_arg_count_errors() {
    let (ok, _, stderr) =
        run_ruchy("fun main() {\n  let t = time_micros(1)\n  println!(\"{}\", t)\n}\n");
    assert!(
        !ok && stderr.contains("time_micros() expects no arguments"),
        "{stderr}"
    );
    let (ok, _, stderr) =
        run_ruchy("fun main() {\n  let t = Instant::now(1)\n  println!(\"{}\", t)\n}\n");
    assert!(
        !ok && stderr.contains("Instant::now() expects no arguments"),
        "{stderr}"
    );
}
