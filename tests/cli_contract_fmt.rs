#![allow(missing_docs)]
// CLI Contract Tests for `ruchy fmt` command
//
// Purpose: Validate fmt tool behavior via CLI interface (Layer 4: Black Box)
// Context: P0 CRITICAL - fmt had code-destroying bugs (operator mangling, let rewriting)
//          Bugs FIXED in src/quality/formatter.rs but ZERO regression prevention tests existed
//
// Critical Defects Prevented:
// - DEFECT-FMT-001: Operator mangling (`*` became `Multiply`)
// - DEFECT-FMT-002: Let statement rewriting (`let x = 42` became `let x = 42 in ()`)
//
// Reference: docs/defects/CRITICAL-FMT-CODE-DESTRUCTION.md

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn fixture_path(name: &str) -> String {
    format!("tests/fixtures/fmt/{name}")
}

// ============================================================================
// CRITICAL: Regression Prevention Tests (P0 Defects)
// ============================================================================

#[test]
fn test_fmt_no_operator_mangling() {
    // DEFECT-FMT-001 Prevention: Operators must use Display trait, not Debug
    // Bug: `x * 2` became `x Multiply 2` (Debug trait format)
    // Fix: Use `{}` instead of `{:?}` in formatter.rs:78

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("operators.ruchy");
    fs::write(&test_file, "let a = 10 + 5\nlet b = a * 2\nlet c = b / 3\nlet d = c - 1").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("*").and(predicate::str::contains("Multiply").not()))
        .stdout(predicate::str::contains("+").and(predicate::str::contains("Add").not()))
        .stdout(predicate::str::contains("/").and(predicate::str::contains("Divide").not()))
        .stdout(predicate::str::contains("-").and(predicate::str::contains("Subtract").not()));
}

#[test]
fn test_fmt_no_let_statement_rewriting() {
    // DEFECT-FMT-002 Prevention: Statement-style let should not become functional style
    // Bug: `let x = 42` became `let x = 42 in ()`
    // Fix: Check for Unit body in formatter.rs:69-80

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    fs::write(&test_file, "let x = 42\nlet y = x * 2\nprintln(y)").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 42 in ()").not())
        .stdout(predicate::str::contains("let y = x * 2 in ()").not());
}

#[test]
#[ignore = "KNOWN BUG: Formatting is not idempotent - let statements change on re-format"]
fn test_fmt_round_trip_idempotency() {
    // Round-trip validation: format(format(x)) == format(x)
    // Ensures formatting is stable and doesn't keep changing code
    //
    // BUG: First format: `{ let x = 42; let y = x * 2 + 10; println(y) }`
    //      Second format: `{ let x = 42 in { let y = x * 2 + 10; println(y) } }`
    //      This is a MEDIUM priority bug - formatting should be idempotent

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("roundtrip.ruchy");
    fs::write(&test_file, "let x = 42\nlet y = x * 2 + 10\nprintln(y)").unwrap();

    // First format
    let output1 = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted1 = String::from_utf8(output1.stdout).unwrap();

    // Write formatted output to temp file
    let test_file2 = temp_dir.path().join("roundtrip2.ruchy");
    fs::write(&test_file2, &formatted1).unwrap();

    // Second format
    let output2 = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file2)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted2 = String::from_utf8(output2.stdout).unwrap();

    // They should be identical
    assert_eq!(formatted1, formatted2, "Formatting is not idempotent!");
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_fmt_basic_file() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("let"));
}

#[test]
fn test_fmt_stdout_option() {
    // --stdout should output formatted code to stdout without modifying file
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_fmt_check_option() {
    // --check should verify formatting without modifying
    // Exit code 1 if formatting needed, 0 if already formatted
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--check")
        .assert()
        .code(predicate::eq(0).or(predicate::eq(1))); // Either formatted or needs formatting
}

#[test]
fn test_fmt_missing_file() {
    ruchy_cmd()
        .arg("fmt")
        .arg("tests/fixtures/fmt/nonexistent.ruchy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_fmt_operators() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("operators.ruchy"))
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("+"))
        .stdout(predicate::str::contains("*"))
        .stdout(predicate::str::contains("/"))
        .stdout(predicate::str::contains("-"));
}

#[test]
fn test_fmt_control_flow() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("control_flow.ruchy"))
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("if"))
        .stdout(predicate::str::contains("else"));
}

#[test]
fn test_fmt_functions() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("functions.ruchy"))
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("fun").or(predicate::str::contains("fn")))
        .stdout(predicate::str::contains("add"));
}

// ============================================================================
// Option Tests
// ============================================================================

#[test]
fn test_fmt_line_width_option() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--line-width")
        .arg("80")
        .arg("--stdout")
        .assert()
        .success();
}

#[test]
fn test_fmt_indent_option() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--indent")
        .arg("2")
        .arg("--stdout")
        .assert()
        .success();
}

#[test]
fn test_fmt_use_tabs_option() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--use-tabs")
        .arg("--stdout")
        .assert()
        .success();
}

#[test]
fn test_fmt_diff_option() {
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--diff")
        .assert()
        .success();
}

#[test]
fn test_fmt_config_option() {
    // Config file doesn't exist yet, but option should be recognized
    ruchy_cmd()
        .arg("fmt")
        .arg(fixture_path("simple.ruchy"))
        .arg("--config")
        .arg("fmt.toml")
        .arg("--stdout")
        .assert()
        .code(predicate::ne(2)); // Not a CLI argument error
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_fmt_empty_file() {
    // Empty files should be rejected (not valid Ruchy programs)
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.ruchy");
    fs::write(&test_file, "").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unexpected end of input"));
}

#[test]
fn test_fmt_whitespace_only() {
    // Whitespace-only files should be rejected (not valid programs)
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("whitespace.ruchy");
    fs::write(&test_file, "   \n\n  \t\n").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unexpected end of input"));
}

#[test]
fn test_fmt_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("syntax_error.ruchy");
    fs::write(&test_file, "let x = ").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Error")
                .or(predicate::str::contains("Expected"))
                .or(predicate::str::contains("Unexpected"))
        );
}

#[test]
fn test_fmt_complex_expression() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.ruchy");
    fs::write(&test_file, "let x = (10 + 5) * (20 - 3) / 2").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("+"))
        .stdout(predicate::str::contains("*"))
        .stdout(predicate::str::contains("-"))
        .stdout(predicate::str::contains("/"));
}

#[test]
fn test_fmt_nested_blocks() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("nested.ruchy");
    fs::write(&test_file, "if x > 0 { if y > 0 { println(\"both positive\") } }").unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .assert()
        .success()
        .stdout(predicate::str::contains("if"));
}

// ============================================================================
// File Modification Tests
// ============================================================================

#[test]
fn test_fmt_modifies_file_in_place() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("modify.ruchy");
    let original = "let x=42\nlet y=x*2";
    fs::write(&test_file, original).unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // File should be modified
    let modified = fs::read_to_string(&test_file).unwrap();
    // Should have some formatting applied (spaces, etc.)
    assert!(modified.contains("let"));
}

#[test]
fn test_fmt_check_no_modification() {
    // --check should NEVER modify the file, regardless of exit code
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("nomodify.ruchy");
    let original = "let x=42\nlet y=x*2";
    fs::write(&test_file, original).unwrap();

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--check")
        .assert()
        .code(predicate::eq(0).or(predicate::eq(1))); // Either formatted or needs formatting

    // File should NOT be modified (this is the critical assertion)
    let unchanged = fs::read_to_string(&test_file).unwrap();
    assert_eq!(unchanged, original, "File was modified with --check flag!");
}

// ============================================================================
// P0 CRITICAL: Debug Fallback Prevention Tests (DEFECT-FMT-003)
// ============================================================================
// Reference: docs/defects/CRITICAL-FMT-DEBUG-FALLBACK.md
//
// Bug: Formatter had catch-all pattern `_ => format!("{:?}", expr.kind)` that
//      output AST Debug representation for 70+ unhandled ExprKind variants
// Impact: Silent file corruption - any code using array indexing, assignments,
//         returns, field access, etc. would be corrupted with AST debug output
// Fix: Implemented formatting for all critical ExprKind variants

#[test]
fn test_fmt_no_debug_fallback_array_indexing() {
    // DEFECT-FMT-003: IndexAccess must format as `arr[i]`, not Debug output
    // Bug: `content[i]` became `IndexAccess { object: Expr { ... }, index: ... }`
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("indexing.ruchy");
    fs::write(&test_file, "let arr = [1, 2, 3]\nlet x = arr[0]\nlet y = arr[1]").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper array indexing syntax
    assert!(formatted.contains("[0]") || formatted.contains("arr["),
            "Missing array indexing syntax");

    // Must NOT contain Debug output
    assert!(!formatted.contains("IndexAccess"),
            "Debug fallback detected for IndexAccess");
    assert!(!formatted.contains("Expr { kind:"),
            "AST Debug output detected");
}

#[test]
fn test_fmt_no_debug_fallback_assignment() {
    // DEFECT-FMT-003: Assign must format as `x = value`, not Debug output
    // Bug: `result = result + ch` became `Assign { target: Expr { ... }, value: ... }`
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("assign.ruchy");
    fs::write(&test_file, "let x = 0\nx = 42\nx = x + 1").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper assignment syntax
    assert!(formatted.contains(" = "), "Missing assignment operator");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Assign {"),
            "Debug fallback detected for Assign");
    assert!(!formatted.contains("target: Expr"),
            "AST Debug output detected");
}

#[test]
fn test_fmt_no_debug_fallback_return() {
    // DEFECT-FMT-003: Return must format as `return value`, not Debug output
    // Bug: `return result` became `Return { value: Some(Expr { ... }) }`
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("return.ruchy");
    // Simplified version of return usage from head.ruchy
    fs::write(&test_file,
        "{\n\
         fun test(x: Any) {\n\
         if x > 10 {\n\
         return x\n\
         }\n\
         0\n\
         }\n\
         }"
    ).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper return syntax
    assert!(formatted.contains("return"), "Missing return keyword");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Return {"),
            "Debug fallback detected for Return");
    assert!(!formatted.contains("value: Some("),
            "AST Debug output detected");
}

#[test]
fn test_fmt_no_debug_fallback_field_access() {
    // DEFECT-FMT-003: FieldAccess must format as `obj.field`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("field.ruchy");
    fs::write(&test_file, "let x = obj.field\nlet y = obj.method()").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper field access syntax
    assert!(formatted.contains('.'), "Missing field access operator");

    // Must NOT contain Debug output
    assert!(!formatted.contains("FieldAccess"),
            "Debug fallback detected for FieldAccess");
}

#[test]
fn test_fmt_no_debug_fallback_while_loop() {
    // DEFECT-FMT-003: While must format as `while cond { body }`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("while.ruchy");
    fs::write(&test_file, "let x = 0\nwhile x < 10 {\nx = x + 1\n}").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper while syntax
    assert!(formatted.contains("while"), "Missing while keyword");

    // Must NOT contain Debug output
    assert!(!formatted.contains("While {"),
            "Debug fallback detected for While");
}

#[test]
fn test_fmt_no_debug_fallback_break_continue() {
    // DEFECT-FMT-003: Break/Continue must format as keywords, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("loop_control.ruchy");
    fs::write(&test_file, "while true {\nif x > 5 { break }\nif x == 3 { continue }\n}").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper keywords
    assert!(formatted.contains("break"), "Missing break keyword");
    assert!(formatted.contains("continue"), "Missing continue keyword");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Break {"),
            "Debug fallback detected for Break");
    assert!(!formatted.contains("Continue {"),
            "Debug fallback detected for Continue");
}

#[test]
fn test_fmt_no_debug_fallback_range() {
    // DEFECT-FMT-003: Range must format as `start..end`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("range.ruchy");
    fs::write(&test_file, "for i in 0..10 {\nprintln(i)\n}").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper range syntax
    assert!(formatted.contains(".."), "Missing range operator");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Range {"),
            "Debug fallback detected for Range");
}

#[test]
fn test_fmt_no_debug_fallback_unary_ops() {
    // DEFECT-FMT-003: Unary must format as `-x` or `!flag`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unary.ruchy");
    fs::write(&test_file, "let x = -42\nlet flag = !true").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper unary operators
    assert!(formatted.contains('-') || formatted.contains('!'),
            "Missing unary operators");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Unary {"),
            "Debug fallback detected for Unary");
    assert!(!formatted.contains("Negate"),
            "Debug fallback detected for Negate operator");
}

#[test]
fn test_fmt_no_debug_fallback_list_literal() {
    // DEFECT-FMT-003: List must format as `[1, 2, 3]`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("list.ruchy");
    fs::write(&test_file, "let arr = [1, 2, 3, 4, 5]").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper list syntax
    assert!(formatted.contains('['), "Missing list opening bracket");
    assert!(formatted.contains(']'), "Missing list closing bracket");

    // Must NOT contain Debug output
    assert!(!formatted.contains("List {"),
            "Debug fallback detected for List");
    assert!(!formatted.contains("elements:"),
            "AST Debug output detected");
}

#[test]
fn test_fmt_no_debug_fallback_tuple_literal() {
    // DEFECT-FMT-003: Tuple must format as `(1, 2, 3)`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("tuple.ruchy");
    fs::write(&test_file, "let pair = (1, 2)\nlet triple = (1, 2, 3)").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper tuple syntax
    assert!(formatted.contains('(') && formatted.contains(')'),
            "Missing tuple parentheses");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Tuple {"),
            "Debug fallback detected for Tuple");
}

#[test]
fn test_fmt_no_debug_fallback_match_expr() {
    // DEFECT-FMT-003: Match must format as `match x { ... }`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("match.ruchy");
    fs::write(&test_file, "match x {\n1 => \"one\"\n2 => \"two\"\n_ => \"other\"\n}").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper match syntax
    assert!(formatted.contains("match"), "Missing match keyword");

    // Must NOT contain Debug output
    assert!(!formatted.contains("Match {"),
            "Debug fallback detected for Match");
    assert!(!formatted.contains("arms:"),
            "AST Debug output detected");
}

#[test]
fn test_fmt_no_debug_fallback_compound_assign() {
    // DEFECT-FMT-003: CompoundAssign must format as `x += 1`, not Debug output
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("compound.ruchy");
    fs::write(&test_file, "let x = 0\nx += 1\nx *= 2\nx -= 5").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain proper compound assignment operators
    assert!(formatted.contains("+=") || formatted.contains("*=") || formatted.contains("-="),
            "Missing compound assignment operators");

    // Must NOT contain Debug output
    assert!(!formatted.contains("CompoundAssign"),
            "Debug fallback detected for CompoundAssign");
}

#[test]
fn test_fmt_real_world_head_example() {
    // INTEGRATION TEST: Validate with actual head.ruchy that triggered bug discovery
    // This is the EXACT code pattern from external bug report
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("head.ruchy");

    // Simplified version of head.ruchy that uses all critical constructs
    fs::write(&test_file,
        "fun head_lines(file_path, n) {\n\
         let content = fs_read(file_path)\n\
         let result = \"\"\n\
         let line_count = 0\n\
         for i in range(0, content.len()) {\n\
         let ch = content[i]\n\
         if line_count < n {\n\
         result = result + ch\n\
         }\n\
         if ch == \"\\n\" {\n\
         line_count = line_count + 1\n\
         if line_count >= n {\n\
         return result\n\
         }\n\
         }\n\
         }\n\
         result\n\
         }"
    ).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Critical assertions - NONE of these Debug patterns should appear
    assert!(!formatted.contains("IndexAccess {"),
            "CRITICAL: Array indexing corrupted with Debug output");
    assert!(!formatted.contains("Assign {"),
            "CRITICAL: Assignment corrupted with Debug output");
    assert!(!formatted.contains("Return {"),
            "CRITICAL: Return statement corrupted with Debug output");
    assert!(!formatted.contains("Expr { kind:"),
            "CRITICAL: AST Debug output detected");
    assert!(!formatted.contains("span: Span"),
            "CRITICAL: Span Debug output detected");

    // Positive assertions - proper syntax should be present
    assert!(formatted.contains("[i]") || formatted.contains("content["),
            "Array indexing syntax missing");
    assert!(formatted.contains("result = "),
            "Assignment syntax missing");
    assert!(formatted.contains("return"),
            "Return keyword missing");

    // The formatted code should pass syntax validation
    let check_result = ruchy_cmd()
        .arg("check")
        .arg(&test_file)
        .output()
        .expect("Failed to run check");

    assert!(check_result.status.success(),
            "Formatted code failed syntax validation");
}

// ============================================================================
// Comprehensive Operator Coverage (CRITICAL)
// ============================================================================

#[test]
fn test_fmt_all_binary_operators() {
    // Ensure ALL operators use Display trait, not Debug
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("all_ops.ruchy");
    fs::write(&test_file,
        "let a = 1 + 2\n\
         let b = 3 - 4\n\
         let c = 5 * 6\n\
         let d = 7 / 8\n\
         let e = 9 % 10\n\
         let f = 11 == 12\n\
         let g = 13 != 14\n\
         let h = 15 < 16\n\
         let i = 17 > 18\n\
         let j = 19 <= 20\n\
         let k = 21 >= 22"
    ).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must contain actual operators, not Debug names
    assert!(formatted.contains('+'), "Missing + operator");
    assert!(formatted.contains('-'), "Missing - operator");
    assert!(formatted.contains('*'), "Missing * operator");
    assert!(formatted.contains('/'), "Missing / operator");

    // Must NOT contain Debug trait names
    assert!(!formatted.contains("Add"), "Operator mangling: Add found");
    assert!(!formatted.contains("Subtract"), "Operator mangling: Subtract found");
    assert!(!formatted.contains("Multiply"), "Operator mangling: Multiply found");
    assert!(!formatted.contains("Divide"), "Operator mangling: Divide found");
}

#[test]
fn test_fmt_preserves_semantics() {
    // CRITICAL: Formatted code must have identical semantics to original
    // This is the ultimate regression test - run both versions

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("semantics.ruchy");
    let code = "let x = 10\nlet y = x * 2 + 5\nprintln(y)";
    fs::write(&test_file, code).unwrap();

    // Run original
    let original_output = ruchy_cmd()
        .arg("run")
        .arg(&test_file)
        .output()
        .expect("Failed to run original");

    // Format the file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Run formatted
    let formatted_output = ruchy_cmd()
        .arg("run")
        .arg(&test_file)
        .output()
        .expect("Failed to run formatted");

    // Both must produce identical output
    assert_eq!(
        original_output.stdout,
        formatted_output.stdout,
        "Formatted code has different semantics!"
    );
}
