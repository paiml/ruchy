// RED TEST: Tool Suite Enum/Struct Recognition (Issue #112)
// TOOL-DEFECT-001: ruchy lint, score, quality-gate, mutations don't recognize enum/struct types
//
// Root Cause: Quality tools use outdated scope tracking that only handles functions/variables
// Expected: All quality tools recognize enum/struct type declarations (v3.155.0+)
//
// This test will FAIL until tools are updated to track type declarations in scope

use assert_cmd::Command;

/// RED TEST 1: ruchy lint should NOT report "undefined variable" for enum types
#[test]
fn test_tool_defect_001_01_lint_recognizes_enums_red() {
    // Minimal enum definition + usage
    let ruchy_code = r"
enum Priority {
    High,
    Medium,
    Low,
}

fun get_priority() -> Priority {
    Priority::High
}

let p = get_priority();
";

    // Write test file
    std::fs::write("/tmp/test_enum_lint.ruchy", ruchy_code).unwrap();

    // Run ruchy lint
    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("lint")
        .arg("/tmp/test_enum_lint.ruchy")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assertions: Should NOT report "undefined variable: Priority"
    assert!(
        !stdout.contains("undefined variable: Priority"),
        "ruchy lint incorrectly reports Priority enum as undefined.\nStdout: {stdout}\nStderr: {stderr}"
    );

    assert!(
        !stdout.contains("undefined variable") || stdout.contains("No linting issues"),
        "ruchy lint should recognize enum types.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Cleanup
    std::fs::remove_file("/tmp/test_enum_lint.ruchy").ok();
}

/// RED TEST 2: ruchy lint should NOT report "undefined variable" for struct types
#[test]
fn test_tool_defect_001_02_lint_recognizes_structs_red() {
    let ruchy_code = r"
struct Config {
    max_retries: i32,
    timeout_ms: i32,
}

fun create_config() -> Config {
    Config {
        max_retries: 3,
        timeout_ms: 1000,
    }
}

let cfg = create_config();
";

    std::fs::write("/tmp/test_struct_lint.ruchy", ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("lint")
        .arg("/tmp/test_struct_lint.ruchy")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assertions: Should NOT report "undefined variable: Config"
    assert!(
        !stdout.contains("undefined variable: Config"),
        "ruchy lint incorrectly reports Config struct as undefined.\nStdout: {stdout}\nStderr: {stderr}"
    );

    assert!(
        !stdout.contains("undefined variable") || stdout.contains("No linting issues"),
        "ruchy lint should recognize struct types.\nStdout: {stdout}\nStderr: {stderr}"
    );

    std::fs::remove_file("/tmp/test_struct_lint.ruchy").ok();
}

/// RED TEST 3: ruchy mutations should find mutants in enum/struct code
#[test]
fn test_tool_defect_001_03_mutations_finds_enum_mutants_red() {
    let ruchy_code = r"
enum Priority {
    High,
    Medium,
    Low,
}

fun priority_score(p: Priority) -> i32 {
    match p {
        Priority::High => 10,
        Priority::Medium => 5,
        Priority::Low => 1,
    }
}

println(priority_score(Priority::High));
";

    std::fs::write("/tmp/test_mutations_enum.ruchy", ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("mutations")
        .arg("/tmp/test_mutations_enum.ruchy")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assertions: Should find mutants (e.g., changing 10 to 11, 5 to 6, etc.)
    assert!(
        stdout.contains("mutants") || stdout.contains("mutations"),
        "ruchy mutations should analyze enum-based code.\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should NOT say "0 mutants found" - there are clear mutation opportunities
    assert!(
        !stdout.contains("0 mutants"),
        "ruchy mutations should find mutants in match arms (10 → 11, 5 → 6, 1 → 2).\nStdout: {stdout}\nStderr: {stderr}"
    );

    std::fs::remove_file("/tmp/test_mutations_enum.ruchy").ok();
}

/// RED TEST 4: ruchy quality-gate should correctly assess complexity of enum/struct code
#[test]
fn test_tool_defect_001_04_quality_gate_enum_complexity_red() {
    // Simple enum with low complexity (should pass quality gates)
    let ruchy_code = r"
enum Status {
    Active,
    Inactive,
}

fun is_active(s: Status) -> bool {
    match s {
        Status::Active => true,
        Status::Inactive => false,
    }
}

println(is_active(Status::Active));
";

    std::fs::write("/tmp/test_quality_enum.ruchy", ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("quality-gate")
        .arg("/tmp/test_quality_enum.ruchy")
        .arg("--min-score")
        .arg("80")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should pass (simple enum with complexity ≤10)
    assert!(
        output.status.success(),
        "ruchy quality-gate should pass for simple enum code (complexity ≤10).\nStdout: {stdout}\nStderr: {stderr}"
    );

    // Should NOT report false SATD violations
    assert!(
        !stdout.contains("SATD") || !stdout.contains("TODO") || !stdout.contains("FIXME"),
        "ruchy quality-gate should not report false SATD violations for enum keywords.\nStdout: {stdout}\nStderr: {stderr}"
    );

    std::fs::remove_file("/tmp/test_quality_enum.ruchy").ok();
}
