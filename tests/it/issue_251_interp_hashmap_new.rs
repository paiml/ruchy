//! Issue #251: `ruchy run` must evaluate `HashMap::new()` (with or without
//! `use std::collections::HashMap`), as compiled output does.

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
fn test_issue_251_hashmap_new_after_use_import() {
    let (ok, stdout, stderr) = run_ruchy(
        "use std::collections::HashMap;\nfun main() {\n  let mut m = HashMap::new()\n  m.insert(\"a\", 1)\n  println!(\"{}\", m.len())\n}\n",
    );
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn test_issue_251_hashmap_new_without_import() {
    let (ok, stdout, stderr) = run_ruchy(
        "fun main() {\n  let mut m = HashMap::new()\n  m.insert(\"a\", 1)\n  m.insert(\"b\", 2)\n  println!(\"{}\", m.len())\n}\n",
    );
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn test_issue_251_hashmap_methods() {
    let (ok, stdout, stderr) = run_ruchy(
        "fun main() {\n  let mut m = HashMap::new()\n  println!(\"{}\", m.is_empty())\n  m.insert(\"a\", 1)\n  m.insert(\"b\", 2)\n  println!(\"{} {} {}\", m.get(\"a\"), m.contains_key(\"b\"), m.contains_key(\"z\"))\n  println!(\"{}\", m.keys().len())\n  m.remove(\"a\")\n  println!(\"{}\", m.len())\n  m.clear()\n  println!(\"{}\", m.is_empty())\n}\n",
    );
    assert!(ok, "ruchy run failed: {stderr}");
    assert_eq!(stdout.trim(), "true\n1 true false\n2\n1\ntrue");
}
