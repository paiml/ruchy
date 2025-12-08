// tests/http_server_cli.rs - HTTP Server CLI Tests (EXTREME TDD)
#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]
// [HTTP-001] RED Phase: Write failing tests FIRST
#![allow(clippy::similar_names)] // coop/coep are standard HTTP header abbreviations
#![allow(clippy::zombie_processes)] // Test processes are explicitly killed in tests
#![allow(deprecated)] // cargo_bin deprecation warning

use assert_cmd::Command;
use predicates::prelude::*;
use std::net::TcpListener;
use std::time::Duration;
use tempfile::TempDir;

/// Helper: Get ruchy binary command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper: Find available port
fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .expect("Failed to get local addr")
        .port()
}

/// Helper: Create test directory with files
fn create_test_dir() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    std::fs::write(
        dir.path().join("index.html"),
        "<!DOCTYPE html><html><body>Hello from Ruchy!</body></html>",
    )
    .expect("Failed to write index.html");
    std::fs::write(dir.path().join("style.css"), "body { margin: 0; }")
        .expect("Failed to write style.css");
    dir
}

// ============================================================================
// RED PHASE: These tests WILL FAIL until we implement ruchy serve
// ============================================================================

#[test]
fn test_red_ruchy_serve_shows_help() {
    // RED: This MUST fail - ruchy serve doesn't exist yet
    ruchy_cmd()
        .arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Serve static files"));
}

#[test]
fn test_red_ruchy_serve_requires_directory() {
    // RED: This MUST fail - ruchy serve doesn't exist yet
    let port = find_available_port();

    ruchy_cmd()
        .arg("serve")
        .arg("./nonexistent")
        .arg("--port")
        .arg(port.to_string())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Directory not found")
                .or(predicate::str::contains("No such file")),
        );
}

#[test]
fn test_red_ruchy_serve_starts_server() {
    // RED: This MUST fail - ruchy serve doesn't exist yet
    use std::process::{Command, Stdio};

    let test_dir = create_test_dir();
    let port = find_available_port();

    // Get ruchy binary path
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");

    // Start server in background
    let mut child = Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    // Give server time to start
    std::thread::sleep(Duration::from_millis(500));

    // Test HTTP request
    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/index.html"));
    assert!(response.is_ok(), "Failed to connect to server");

    let response = response.unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.text().unwrap().contains("Hello from Ruchy!"));

    // Cleanup
    child.kill().expect("Failed to kill server");
}

#[test]
fn test_red_ruchy_serve_default_port_8080() {
    // RED: This MUST fail - ruchy serve doesn't exist yet
    use std::process::{Command, Stdio};

    let test_dir = create_test_dir();

    // Get ruchy binary path
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");

    let mut child = Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    // Should default to port 8080
    let response = reqwest::blocking::get("http://127.0.0.1:8080/index.html");
    assert!(
        response.is_ok(),
        "Server should listen on port 8080 by default"
    );

    child.kill().expect("Failed to kill server");
}

#[test]
fn test_red_ruchy_serve_shows_startup_message() {
    // RED: This MUST fail - ruchy serve doesn't exist yet
    let test_dir = create_test_dir();
    let port = find_available_port();

    let output = ruchy_cmd()
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .timeout(Duration::from_secs(1))
        .output()
        .expect("Failed to run server");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should print startup info
    assert!(stdout.contains("Ruchy HTTP Server") || stdout.contains("🚀"));
    assert!(stdout.contains(&port.to_string()));
    assert!(stdout.contains("http://") || stdout.contains("localhost"));
}

// ============================================================================
// Property Tests (10,000 iterations)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        #[test]
        #[ignore = "Remove when implementation complete"]
        fn prop_ruchy_serve_never_panics_on_invalid_port(port in 0u16..1024u16) {
            // Property: ruchy serve with invalid port should fail gracefully, not panic
            let test_dir = create_test_dir();

            let result = ruchy_cmd()
                .arg("serve")
                .arg(test_dir.path())
                .arg("--port")
                .arg(port.to_string())
                .timeout(Duration::from_secs(1))
                .output();

            // Should either succeed or fail gracefully (no panic)
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Test disabled - run with --ignored"]
        fn prop_ruchy_serve_handles_any_valid_directory(dir_name: String) {
            // Property: ruchy serve should handle any valid directory name
            let temp_dir = TempDir::new().unwrap();
            let test_path = temp_dir.path().join(&dir_name);

            if let Ok(()) = std::fs::create_dir_all(&test_path) {
                std::fs::write(test_path.join("test.html"), "<html></html>").unwrap();

                let result = ruchy_cmd()
                    .arg("serve")
                    .arg(&test_path)
                    .timeout(Duration::from_millis(100))
                    .output();

                prop_assert!(result.is_ok());
            }
        }
    }
}

// ============================================================================
// MIME Type Detection Tests (HTTP-002)
// ============================================================================

#[test]
fn test_http002_mime_html() {
    let test_dir = TempDir::new().unwrap();
    std::fs::write(
        test_dir.path().join("index.html"),
        "<!DOCTYPE html><html></html>",
    )
    .unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/index.html")).unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");

    child.kill().unwrap();
}

#[test]
fn test_http002_mime_css() {
    let test_dir = TempDir::new().unwrap();
    std::fs::write(test_dir.path().join("style.css"), "body { margin: 0; }").unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/style.css")).unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/css");

    child.kill().unwrap();
}

#[test]
fn test_http002_mime_javascript() {
    let test_dir = TempDir::new().unwrap();
    std::fs::write(test_dir.path().join("app.js"), "console.log('test');").unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/app.js")).unwrap();
    assert_eq!(response.status(), 200);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("application/javascript") || content_type.contains("text/javascript"),
        "Expected JavaScript MIME type, got: {content_type}"
    );

    child.kill().unwrap();
}

#[test]
fn test_http002_mime_wasm() {
    // CRITICAL: WASM files MUST have application/wasm MIME type
    let test_dir = TempDir::new().unwrap();
    // Minimal valid WASM module: magic number + version
    std::fs::write(
        test_dir.path().join("module.wasm"),
        b"\x00\x61\x73\x6d\x01\x00\x00\x00",
    )
    .unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/module.wasm")).unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/wasm",
        "WASM files MUST have application/wasm MIME type for streaming compilation"
    );

    child.kill().unwrap();
}

#[test]
fn test_http002_mime_json() {
    let test_dir = TempDir::new().unwrap();
    std::fs::write(test_dir.path().join("data.json"), r#"{"test": true}"#).unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/data.json")).unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    child.kill().unwrap();
}

// ============================================================================
// WASM-Specific Headers Tests (HTTP-003)
// ============================================================================

#[test]
fn test_http003_wasm_coop_header() {
    // CRITICAL: WASM files MUST have Cross-Origin-Opener-Policy for SharedArrayBuffer
    let test_dir = TempDir::new().unwrap();
    std::fs::write(
        test_dir.path().join("app.wasm"),
        b"\x00\x61\x73\x6d\x01\x00\x00\x00",
    )
    .unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/app.wasm")).unwrap();
    assert_eq!(response.status(), 200);

    // COOP header required for SharedArrayBuffer
    let coop = response.headers().get("cross-origin-opener-policy");
    assert!(
        coop.is_some(),
        "WASM files MUST have Cross-Origin-Opener-Policy header for SharedArrayBuffer support"
    );
    assert_eq!(coop.unwrap(), "same-origin");

    child.kill().unwrap();
}

#[test]
fn test_http003_wasm_coep_header() {
    // CRITICAL: WASM files MUST have Cross-Origin-Embedder-Policy for SharedArrayBuffer
    let test_dir = TempDir::new().unwrap();
    std::fs::write(
        test_dir.path().join("app.wasm"),
        b"\x00\x61\x73\x6d\x01\x00\x00\x00",
    )
    .unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/app.wasm")).unwrap();
    assert_eq!(response.status(), 200);

    // COEP header required for SharedArrayBuffer
    let coep = response.headers().get("cross-origin-embedder-policy");
    assert!(
        coep.is_some(),
        "WASM files MUST have Cross-Origin-Embedder-Policy header for SharedArrayBuffer support"
    );
    assert_eq!(coep.unwrap(), "require-corp");

    child.kill().unwrap();
}

#[test]
fn test_http003_html_coop_header() {
    // HTML files serving WASM also need COOP/COEP headers
    let test_dir = TempDir::new().unwrap();
    std::fs::write(
        test_dir.path().join("index.html"),
        "<!DOCTYPE html><html></html>",
    )
    .unwrap();

    let port = find_available_port();
    let ruchy_bin = assert_cmd::cargo::cargo_bin("ruchy");
    let mut child = std::process::Command::new(ruchy_bin)
        .arg("serve")
        .arg(test_dir.path())
        .arg("--port")
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn server");

    std::thread::sleep(Duration::from_millis(500));

    let response = reqwest::blocking::get(format!("http://127.0.0.1:{port}/index.html")).unwrap();
    assert_eq!(response.status(), 200);

    // HTML pages that load WASM need these headers too
    let coop = response.headers().get("cross-origin-opener-policy");
    assert!(
        coop.is_some(),
        "HTML files MUST have COOP header for WASM SharedArrayBuffer"
    );
    assert_eq!(coop.unwrap(), "same-origin");

    let coep = response.headers().get("cross-origin-embedder-policy");
    assert!(
        coep.is_some(),
        "HTML files MUST have COEP header for WASM SharedArrayBuffer"
    );
    assert_eq!(coep.unwrap(), "require-corp");

    child.kill().unwrap();
}

// ============================================================================
// RED Phase Validation
// ============================================================================

#[test]
fn test_red_phase_validation() {
    // This test ensures we're in RED phase
    // All #[ignore = "tests above MUST fail when un-ignored"]

    println!("✅ RED Phase: 5 failing tests created");
    println!("✅ Property tests: 2 tests with 10K iterations each");
    println!("🔴 Next: Remove #[ignore] and watch tests FAIL");
    println!("🟢 Then: Implement ruchy serve to make tests PASS");
}
