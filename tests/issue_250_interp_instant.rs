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
