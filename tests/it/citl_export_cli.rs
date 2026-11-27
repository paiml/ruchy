//! CITL export (ruchy#165): `ruchy transpile input.rb --citl-export diagnostics.json`
//! writes one `CitlExport` record per rustc diagnostic on the transpiled output.

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;

const TYPE_ERROR: &str = "fun main() {\n    let x: i32 = \"a\";\n    println(x);\n}\n";
const CLEAN: &str = "fun main() {\n    println(\"ok\");\n}\n";

fn transpile_with_citl(dir: &Path, source: &str) -> (String, Value) {
    let input = dir.join("input.rb");
    std::fs::write(&input, source).unwrap();
    let export = dir.join("diagnostics.json");
    let out = Command::cargo_bin("ruchy")
        .expect("ruchy binary is built")
        .current_dir(dir)
        .args(["transpile", "input.rb", "--citl-export", "diagnostics.json"])
        .output()
        .expect("ruchy runs");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::fs::read_to_string(&export).expect("diagnostics.json written");
    let json: Value = serde_json::from_str(&text).expect("export is JSON");
    (String::from_utf8_lossy(&out.stdout).into_owned(), json)
}

#[test]
fn test_ruchy_165_citl_export_writes_error_record_with_issue_fields() {
    let dir = tempfile::tempdir().unwrap();
    let (_stdout, json) = transpile_with_citl(dir.path(), TYPE_ERROR);

    let records = json.as_array().expect("export is a JSON array");
    let e0308 = records
        .iter()
        .find(|r| r["error_code"] == "E0308")
        .unwrap_or_else(|| panic!("no E0308 record in {json}"));

    assert_eq!(e0308["level"], "error");
    assert!(e0308["message"].as_str().unwrap().contains("mismatched"));
    assert_eq!(e0308["source_file"], "input.rb");
    assert!(e0308["clippy_lint"].is_null());
    assert!(e0308["oip_category"].is_null());
    assert_eq!(e0308["confidence"].as_f64(), Some(1.0));
    assert!(e0308["timestamp"].as_i64().unwrap() > 0);

    let span = &e0308["span"];
    assert!(span["line_start"].as_u64().unwrap() >= 1, "{span}");
    assert!(span["column_start"].as_u64().unwrap() >= 1, "{span}");
    assert!(span.get("file_name").is_some(), "{span}");
    assert!(span.get("line_end").is_some(), "{span}");
    assert!(span.get("column_end").is_some(), "{span}");
    assert!(e0308.get("suggestion").is_some(), "suggestion key present");
}

#[test]
fn test_ruchy_165_citl_export_clean_program_writes_empty_array() {
    let dir = tempfile::tempdir().unwrap();
    let (_stdout, json) = transpile_with_citl(dir.path(), CLEAN);
    assert_eq!(json, serde_json::json!([]));
}

#[test]
fn test_ruchy_165_citl_export_keeps_transpiled_output_on_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let (stdout, _json) = transpile_with_citl(dir.path(), CLEAN);
    assert!(stdout.contains("fn main()"), "{stdout}");
}
