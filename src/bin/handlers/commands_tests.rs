use super::*;
use ruchy::frontend::ast::{Expr, ExprKind};
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// Helper function to create a test expression
fn create_test_expr() -> Expr {
    Expr::new(
        ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(42, None)),
        ruchy::frontend::ast::Span::default(),
    )
}

// Helper function to create a temporary file with content
fn create_temp_file_with_content(content: &str) -> Result<NamedTempFile> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;
    Ok(temp_file)
}

// ========== AST Command Tests ==========
#[test]
fn test_generate_json_output() {
    let expr = create_test_expr();
    let result = generate_json_output(&expr);
    assert!(result.is_ok());
    let json = result.expect("generate_json_output should succeed");
    assert!(json.contains("Integer"));
    assert!(json.contains("42"));
}

#[test]
fn test_generate_graph_output() {
    let result = generate_graph_output();
    assert!(result.contains("digraph AST"));
    assert!(result.contains("node [shape=box]"));
}

#[test]
fn test_generate_metrics_output() {
    let expr = create_test_expr();
    let result = generate_metrics_output(&expr);
    assert!(result.contains("=== AST Metrics ==="));
    assert!(result.contains("Nodes:"));
    assert!(result.contains("Depth:"));
    assert!(result.contains("Complexity:"));
}

#[test]
fn test_generate_symbols_output() {
    let expr = create_test_expr();
    let result = generate_symbols_output(&expr);
    assert!(result.contains("=== Symbol Analysis ==="));
    assert!(result.contains("Defined:"));
    assert!(result.contains("Used:"));
    assert!(result.contains("Unused:"));
}

#[test]
fn test_generate_deps_output() {
    let result = generate_deps_output();
    assert!(result.contains("=== Dependencies ==="));
    assert!(result.contains("No external dependencies"));
}

#[test]
fn test_generate_default_output() {
    let expr = create_test_expr();
    let result = generate_default_output(&expr);
    assert!(result.contains("kind"));
    assert!(result.contains("Integer"));
}

#[test]
fn test_generate_ast_output_json() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, true, false, false, false, false);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("Integer"));
}

#[test]
fn test_generate_ast_output_graph() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, false, true, false, false, false);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("digraph AST"));
}

#[test]
fn test_generate_ast_output_metrics() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, false, false, true, false, false);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("AST Metrics"));
}

#[test]
fn test_generate_ast_output_symbols() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, false, false, false, true, false);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("Symbol Analysis"));
}

#[test]
fn test_generate_ast_output_deps() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, false, false, false, false, true);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("Dependencies"));
}

#[test]
fn test_generate_ast_output_default() {
    let expr = create_test_expr();
    let result = generate_ast_output(&expr, false, false, false, false, false);
    assert!(result.is_ok());
    let output = result.expect("generate_ast_output should succeed");
    assert!(output.contains("kind"));
}

#[test]
fn test_write_ast_output_to_stdout() {
    let result = write_ast_output("test content".to_string(), None);
    assert!(result.is_ok());
}

#[test]
fn test_write_ast_output_to_file() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let output_path = temp_dir.path().join("output.txt");
    let result = write_ast_output("test content".to_string(), Some(&output_path));
    assert!(result.is_ok());
    let content = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|_| panic!("Failed to read output file: {}", output_path.display()));
    assert_eq!(content, "test content");
}

// ========== Format Command Tests ==========
#[test]
fn test_determine_fmt_mode_check() {
    let mode = determine_fmt_mode(true, false, false, false);
    assert!(matches!(mode, FmtMode::Check));
}

#[test]
fn test_determine_fmt_mode_stdout() {
    let mode = determine_fmt_mode(false, true, false, false);
    assert!(matches!(mode, FmtMode::Stdout));
}

#[test]
fn test_determine_fmt_mode_diff() {
    let mode = determine_fmt_mode(false, false, true, false);
    assert!(matches!(mode, FmtMode::Diff));
}

#[test]
fn test_determine_fmt_mode_write() {
    let mode = determine_fmt_mode(false, false, false, true);
    assert!(matches!(mode, FmtMode::Write));
}

#[test]
fn test_determine_fmt_mode_default() {
    let mode = determine_fmt_mode(false, false, false, false);
    assert!(matches!(mode, FmtMode::Default));
}

#[test]
fn test_handle_stdout_mode() {
    // Test doesn't crash
    handle_stdout_mode("formatted code");
}

#[test]
fn test_handle_diff_mode() {
    let temp_file =
        create_temp_file_with_content("original").expect("Failed to create temporary test file");
    handle_diff_mode(temp_file.path(), "original", "formatted");
}

#[test]
fn test_handle_default_mode() {
    // Test doesn't crash
    handle_default_mode("formatted code");
}

// ========== Linter Helper Tests ==========
#[test]
fn test_configure_linter_default() {
    let linter = configure_linter(None, false);
    // Test doesn't crash and creates a linter
    let _linter = linter;
}

#[test]
fn test_configure_linter_with_rules() {
    let linter = configure_linter(Some("test,rules"), false);
    // Test doesn't crash and creates a linter
    let _linter = linter;
}

#[test]
fn test_configure_linter_strict() {
    let linter = configure_linter(None, true);
    // Test doesn't crash and creates a linter
    let _linter = linter;
}

#[test]
fn test_count_issue_types_empty() {
    let issues = vec![];
    let (warnings, errors) = count_issue_types(&issues);
    assert_eq!(warnings, 0);
    assert_eq!(errors, 0);
}

// ========== Provability Tests ==========
#[test]
fn test_generate_provability_header() {
    let temp_file =
        create_temp_file_with_content("test").expect("Failed to create temporary test file");
    let expr = create_test_expr();
    let analysis = ProvabilityAnalysis {
        verify: true,
        contracts: true,
        invariants: true,
        termination: true,
        bounds: true,
    };
    let result = generate_provability_header(temp_file.path(), &expr, analysis);
    assert!(result.contains("=== Provability Analysis ==="));
    assert!(result.contains("File:"));
    assert!(result.contains("Provability Score:"));
}

#[test]
fn test_add_verification_section() {
    let mut output = String::new();
    add_verification_section(&mut output);
    assert!(output.contains("=== Formal Verification ==="));
    assert!(output.contains("No unsafe operations detected"));
}

#[test]
fn test_add_contracts_section() {
    let mut output = String::new();
    add_contracts_section(&mut output);
    assert!(output.contains("=== Contract Verification ==="));
    assert!(output.contains("contracts") || output.contains("Contract"));
}

#[test]
fn test_add_invariants_section() {
    let mut output = String::new();
    add_invariants_section(&mut output);
    assert!(output.contains("=== Loop Invariants ==="));
    assert!(output.contains("Loop") || output.contains("loops"));
}

#[test]
fn test_add_termination_section() {
    let mut output = String::new();
    add_termination_section(&mut output);
    assert!(output.contains("=== Termination Analysis ==="));
    assert!(output.contains("Termination") || output.contains("terminate"));
}

#[test]
fn test_add_bounds_section() {
    let mut output = String::new();
    add_bounds_section(&mut output);
    assert!(output.contains("=== Bounds Checking ==="));
    assert!(output.contains("bounds") || output.contains("Bounds"));
}

#[test]
fn test_write_provability_output_stdout() {
    let result = write_provability_output("test content".to_string(), None);
    assert!(result.is_ok());
}

#[test]
fn test_write_provability_output_file() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let output_path = temp_dir.path().join("provability.txt");
    let result = write_provability_output("test content".to_string(), Some(&output_path));
    assert!(result.is_ok());
    let content = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|_| panic!("Failed to read output file: {}", output_path.display()));
    assert_eq!(content, "test content");
}

// ========== Runtime Tests ==========
#[test]
fn test_generate_runtime_header() {
    let temp_file =
        create_temp_file_with_content("test").expect("Failed to create temporary test file");
    let result = generate_runtime_header(temp_file.path());
    assert!(result.contains("=== Performance Analysis ==="));
    assert!(result.contains("File:"));
}

#[test]
fn test_add_profile_section() {
    let mut output = String::new();
    add_profile_section(&mut output);
    assert!(output.contains("=== Execution Profiling ==="));
    assert!(output.contains("Function") || output.contains("times"));
}

#[test]
fn test_add_bigo_section() {
    let mut output = String::new();
    let expr = create_test_expr();
    add_bigo_section(&mut output, &expr);
    assert!(output.contains("=== BigO Complexity Analysis ==="));
    assert!(output.contains("O("));
}

#[test]
fn test_add_benchmark_section() {
    let mut output = String::new();
    add_benchmark_section(&mut output);
    assert!(output.contains("=== Benchmark Results ==="));
    assert!(output.contains("execution time"));
}

#[test]
fn test_add_memory_section() {
    let mut output = String::new();
    add_memory_section(&mut output);
    assert!(output.contains("=== Memory Analysis ==="));
    assert!(output.contains("memory usage"));
}

#[test]
fn test_add_comparison_section() {
    let temp_file1 =
        create_temp_file_with_content("current").expect("Failed to create temporary test file");
    let temp_file2 =
        create_temp_file_with_content("baseline").expect("Failed to create temporary test file");
    let mut output = String::new();
    add_comparison_section(&mut output, temp_file1.path(), temp_file2.path());
    assert!(output.contains("=== Performance Comparison ==="));
    assert!(output.contains("Current:"));
    assert!(output.contains("Baseline:"));
}

#[test]
fn test_write_runtime_output_stdout() {
    let result = write_runtime_output("test content".to_string(), None);
    assert!(result.is_ok());
}

#[test]
fn test_write_runtime_output_file() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let output_path = temp_dir.path().join("runtime.txt");
    let result = write_runtime_output("test content".to_string(), Some(&output_path));
    assert!(result.is_ok());
    let content = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|_| panic!("Failed to read output file: {}", output_path.display()));
    assert_eq!(content, "test content");
}

// ========== Score Tests ==========
#[test]
fn test_calculate_average_empty() {
    let scores = HashMap::new();
    let result = calculate_average(&scores);
    assert_eq!(result, 0.0);
}

#[test]
fn test_calculate_average_single() {
    let mut scores = HashMap::new();
    scores.insert(PathBuf::from("test.ruchy"), 85.5);
    let result = calculate_average(&scores);
    assert_eq!(result, 85.5);
}

#[test]
fn test_calculate_average_multiple() {
    let mut scores = HashMap::new();
    scores.insert(PathBuf::from("test1.ruchy"), 80.0);
    scores.insert(PathBuf::from("test2.ruchy"), 90.0);
    let result = calculate_average(&scores);
    assert_eq!(result, 85.0);
}

#[test]
fn test_format_empty_directory_output_text() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let result = format_empty_directory_output(temp_dir.path(), "shallow", "text");
    assert!(result.is_ok());
    let output = result.expect("format_empty_directory_output should succeed");
    assert!(output.contains("=== Quality Score ==="));
    assert!(output.contains("Files: 0"));
    assert!(output.contains("shallow"));
}

#[test]
fn test_format_empty_directory_output_json() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let result = format_empty_directory_output(temp_dir.path(), "deep", "json");
    assert!(result.is_ok());
    let output = result.expect("format_empty_directory_output should succeed");
    assert!(output.contains("\"directory\""));
    assert!(output.contains("\"files\""));
    assert!(output.contains("\"depth\": \"deep\""));
}

#[test]
fn test_write_output_stdout() {
    let result = write_output("test content", None);
    assert!(result.is_ok());
}

#[test]
fn test_write_output_file() {
    let temp_dir = TempDir::new().expect("Failed to create temporary test directory");
    let output_path = temp_dir.path().join("output.txt");
    let result = write_output("test content", Some(&output_path));
    assert!(result.is_ok());
    let content = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|_| panic!("Failed to read output file: {}", output_path.display()));
    assert_eq!(content, "test content");
}

#[test]
fn test_check_score_threshold_pass() {
    // Test doesn't crash when score is above threshold
    assert!(check_score_threshold(85.0, Some(80.0)).is_ok());
}

#[test]
fn test_check_score_threshold_no_threshold() {
    // Test doesn't crash when no threshold is set
    assert!(check_score_threshold(50.0, None).is_ok());
}

#[test]
fn test_check_score_threshold_fail() {
    // Test returns error when score is below threshold
    assert!(check_score_threshold(75.0, Some(80.0)).is_err());
}

// ========== Quality Gate Tests ==========
#[test]
fn test_parse_source_file_valid() {
    let result = parse_source_file("42");
    assert!(result.is_ok());
}

#[test]
fn test_parse_source_file_invalid() {
    let result = parse_source_file("invalid syntax +++");
    assert!(result.is_err());
}

#[test]
fn test_check_complexity_gate_simple() {
    let expr = create_test_expr();
    let (passed, message) = check_complexity_gate(&expr);
    assert!(passed);
    assert!(message.contains("Complexity") && message.contains("within limit"));
}

#[test]
fn test_check_satd_gate_clean() {
    let (passed, message) = check_satd_gate("// Clean code without SATD");
    assert!(passed);
    assert!(message.contains("No SATD comments"));
}

#[test]
fn test_check_satd_gate_with_todo() {
    let (passed, message) = check_satd_gate("// TODO: fix this");
    assert!(!passed);
    assert!(message.contains("Contains SATD comments"));
}

#[test]
fn test_contains_satd_comment_todo() {
    assert!(contains_satd_comment("// TODO: something"));
    // Block comments are not currently detected by contains_satd_comment
    assert!(!contains_satd_comment("/* TODO stuff */"));
}

#[test]
fn test_contains_satd_comment_fixme() {
    assert!(contains_satd_comment("// FIXME: broken"));
    // Block comments are not currently detected
    assert!(!contains_satd_comment("/* FIXME issue */"));
}

#[test]
fn test_contains_satd_comment_hack() {
    assert!(contains_satd_comment("// HACK: workaround"));
    // Block comments are not currently detected
    assert!(!contains_satd_comment("/* HACK solution */"));
}

#[test]
fn test_contains_satd_comment_clean() {
    assert!(!contains_satd_comment("// Normal comment"));
    assert!(!contains_satd_comment("/* Regular comment */"));
    assert!(!contains_satd_comment("println!(\"Hello\");"));
}

#[test]
fn test_should_fail_strict_true() {
    assert!(should_fail_strict(false, true));
    assert!(!should_fail_strict(true, true));
}

#[test]
fn test_should_fail_strict_false() {
    assert!(!should_fail_strict(false, false));
    assert!(!should_fail_strict(true, false));
}

// ========== Quality Metrics Tests ==========
#[test]
fn test_count_ast_nodes() {
    let expr = create_test_expr();
    let count = count_ast_nodes(&expr);
    assert!(count >= 1);
}

#[test]
fn test_calculate_ast_depth() {
    let expr = create_test_expr();
    let depth = calculate_ast_depth(&expr);
    assert!(depth >= 1);
}

#[test]
fn test_calculate_provability_score() {
    let expr = create_test_expr();
    let analysis = ProvabilityAnalysis {
        verify: true,
        contracts: true,
        invariants: true,
        termination: true,
        bounds: true,
    };
    let score = calculate_provability_score(&expr, analysis);
    assert!(score >= 0.0);
    assert!(score <= 100.0);
}

#[test]
fn test_calculate_quality_score() {
    let expr = create_test_expr();
    let score = calculate_quality_score(&expr, "42");
    assert!(score >= 0.0);
    assert!(score <= 100.0);
}

#[test]
fn test_detect_satd_in_source_clean() {
    assert!(!detect_satd_in_source("let x = 42;"));
    assert!(!detect_satd_in_source("// Normal comment"));
}

#[test]
fn test_detect_satd_in_source_with_todo() {
    assert!(detect_satd_in_source("// TODO: implement"));
    // Block comments are not currently detected
    assert!(!detect_satd_in_source("/* FIXME: bug here */"));
    assert!(detect_satd_in_source("// HACK: workaround"));
}

#[test]
fn test_collect_quality_metrics() {
    let expr = create_test_expr();
    let metrics = collect_quality_metrics(&expr, "42");
    // function_count is usize, always >= 0
    // total_identifiers is usize, always >= 0
    assert!(!metrics.has_satd);
}

#[test]
fn test_get_complexity_penalty_low() {
    let penalty = get_complexity_penalty(5);
    assert_eq!(penalty, 1.0); // Low complexity = no penalty (multiplier = 1.0)
}

#[test]
fn test_get_complexity_penalty_medium() {
    let penalty = get_complexity_penalty(15);
    assert_eq!(penalty, 0.85); // Medium complexity = 0.85 multiplier
}

#[test]
fn test_get_complexity_penalty_high() {
    let penalty = get_complexity_penalty(25);
    assert_eq!(penalty, 0.45); // High complexity = 0.45 multiplier
}

#[test]
fn test_get_parameter_penalty() {
    let penalty_low = get_parameter_penalty(3);
    let penalty_high = get_parameter_penalty(8);
    assert_eq!(penalty_low, 1.0); // Low params = no penalty
    assert_eq!(penalty_high, 0.50); // High params = 0.50 multiplier
}

#[test]
fn test_get_nesting_penalty() {
    let penalty_low = get_nesting_penalty(2);
    let penalty_high = get_nesting_penalty(6);
    assert_eq!(penalty_low, 1.0); // Low nesting = no penalty
    assert_eq!(penalty_high, 0.30); // High nesting = 0.30 multiplier
}

#[test]
fn test_get_satd_penalty() {
    assert_eq!(get_satd_penalty(false), 1.0); // No SATD = no penalty
    assert_eq!(get_satd_penalty(true), 0.70); // Has SATD = 0.70 multiplier
}

#[test]
fn test_calculate_complexity() {
    let expr = create_test_expr();
    let complexity = calculate_complexity(&expr);
    assert!(complexity >= 1);
}

#[test]
fn test_analyze_complexity() {
    let expr = create_test_expr();
    let analysis = analyze_complexity(&expr);
    // analyze_complexity returns complexity like "1", "n", "n²", etc
    assert!(!analysis.is_empty());
    // For a simple literal expression, complexity should be "1" (constant)
    assert_eq!(analysis, "1");
}

#[test]
fn test_calculate_max_nesting() {
    let expr = create_test_expr();
    let _nesting = calculate_max_nesting(&expr);
    // nesting is usize, always >= 0
}

#[test]
fn test_count_assertions_recursive() {
    let expr = create_test_expr();
    let mut assertion_count = 0;
    let mut total_statements = 0;
    count_assertions_recursive(&expr, &mut assertion_count, &mut total_statements);
    assert_eq!(assertion_count, 0); // No assertions in a literal
    assert!(total_statements >= 1);
}

#[test]
fn test_check_method_assertion() {
    let mut count = 0;
    check_method_assertion("assert", &mut count);
    assert_eq!(count, 1);
    check_method_assertion("assert_eq", &mut count);
    assert_eq!(count, 2);
    check_method_assertion("regular_method", &mut count);
    assert_eq!(count, 2); // No change
}

#[test]
fn test_extract_symbols() {
    let expr = create_test_expr();
    let symbols = extract_symbols(&expr);
    assert_eq!(symbols.defined.len(), 2);
    assert_eq!(symbols.used.len(), 1);
    assert_eq!(symbols.unused.len(), 1);
    assert!(symbols.defined.contains(&"x".to_string()));
    assert!(symbols.defined.contains(&"y".to_string()));
    assert!(symbols.used.contains(&"x".to_string()));
    assert!(symbols.unused.contains(&"y".to_string()));
}

// ========== Integration Tests ==========
#[test]
fn test_run_quality_gates_simple_code() {
    let expr = create_test_expr();
    let (passed, results) = run_quality_gates(&expr, "42");
    assert!(passed);
    assert!(!results.is_empty());
}

#[test]
fn test_format_gate_results_json() {
    let results = vec!["Test result".to_string()];
    let formatted = format_gate_results(true, &results, true);
    assert!(formatted.is_ok());
    let json = formatted.expect("format_gate_results should succeed");
    assert!(json.contains("\"passed\""));
    assert!(json.contains("true"));
}

#[test]
fn test_format_gate_results_text() {
    let results = vec!["Test result".to_string()];
    let formatted = format_gate_results(false, &results, false);
    assert!(formatted.is_ok());
    let text = formatted.expect("format_gate_results should succeed");
    // format_gate_results just joins results with newlines
    assert!(text.contains("Test result"));
}

#[test]
fn test_output_results_quiet() {
    let result = output_results("test content", true, None);
    assert!(result.is_ok());
}

#[test]
fn test_output_results_verbose() {
    let result = output_results("test content", false, None);
    assert!(result.is_ok());
}
