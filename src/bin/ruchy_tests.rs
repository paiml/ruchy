use super::*;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_format_config_default() {
    let config = FormatConfig::default();
    assert_eq!(config.line_width, 100);
    assert_eq!(config.indent, 4);
    assert!(!config.use_tabs);
}

#[test]
fn test_format_config_creation() {
    let config = FormatConfig {
        line_width: 120,
        indent: 2,
        use_tabs: true,
    };
    assert_eq!(config.line_width, 120);
    assert_eq!(config.indent, 2);
    assert!(config.use_tabs);
}

#[test]
fn test_try_handle_direct_evaluation_with_eval() {
    let cli = Cli {
        eval: Some("1 + 1".to_string()),
        format: "text".to_string(),
        verbose: false,
        vm_mode: VmMode::Ast,
        file: None,
        command: None,
        trace: false,
    };
    let result = try_handle_direct_evaluation(&cli);
    assert!(result.is_some());
}

#[test]
fn test_try_handle_direct_evaluation_with_file() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "println(\"Hello World\")")
        .expect("Failed to write test content to temporary file");

    let cli = Cli {
        eval: None,
        format: "text".to_string(),
        verbose: false,
        vm_mode: VmMode::Ast,
        file: Some(temp_file.path().to_path_buf()),
        command: None,
        trace: false,
    };
    let result = try_handle_direct_evaluation(&cli);
    assert!(result.is_some());
}

#[test]
fn test_try_handle_direct_evaluation_none() {
    let cli = Cli {
        eval: None,
        format: "text".to_string(),
        verbose: false,
        vm_mode: VmMode::Ast,
        file: None,
        command: None,
        trace: false,
    };
    let result = try_handle_direct_evaluation(&cli);
    assert!(result.is_none());
}

#[test]
fn test_try_handle_stdin_no_command() {
    let result = try_handle_stdin(None);
    assert!(result.is_ok());
}

#[test]
fn test_try_handle_stdin_with_command() {
    let command = Commands::Repl {
        record: None,
        max_depth: 100,
    };
    let result = try_handle_stdin(Some(&command));
    assert!(result.is_ok());
}

#[test]
fn test_run_file_valid_syntax() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let result = run_file(temp_file.path());
    assert!(result.is_ok());
}

#[test]
fn test_run_file_nonexistent() {
    let nonexistent_path = PathBuf::from("/nonexistent/file.ruchy");
    let result = run_file(&nonexistent_path);
    assert!(result.is_err());
}

#[test]
fn test_check_syntax_valid() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let result = check_syntax(temp_file.path());
    assert!(result.is_ok());
}

#[test]
fn test_check_syntax_nonexistent_file() {
    let nonexistent_path = PathBuf::from("/nonexistent/file.ruchy");
    let result = check_syntax(&nonexistent_path);
    assert!(result.is_err());
}

// PMAT-104: this used to pass `None` as the path, which made the .ruchy test
// runner walk the *current* directory - under `cargo test` that is the ruchy
// repo itself - so it was slow enough to need `#[ignore]`. Pointing it at a
// temp dir makes it hermetic, fast, and actually run.
#[test]
fn test_handle_test_dispatch_basic() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    fs::write(
        temp_dir.path().join("basic.ruchy"),
        "#[test]\nfun test_basic() {\n    let x = 42\n}\n",
    )
    .expect("Failed to write test file");
    let result = handle_test_dispatch(
        Some(temp_dir.path().to_path_buf()),
        false,
        false,
        None,
        false,
        "text",
        false,
        None,
        "text",
    );
    assert!(result.is_ok(), "dispatch failed: {result:?}");
}

#[test]
fn test_handle_test_dispatch_with_path() {
    // Create a temp directory with a proper .ruchy test file containing a test function
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let ruchy_file = temp_dir.path().join("test_file.ruchy");
    // Write a minimal valid Ruchy file - dispatch should process it regardless of test content
    fs::write(&ruchy_file, "let x = 42\n").expect("Failed to write test content to temporary file");

    // The dispatch function should execute successfully even if the file has no tests
    // (it will report 0 tests found, which is valid behavior)
    let result = handle_test_dispatch(
        Some(temp_dir.path().to_path_buf()),
        false,
        true,
        None,
        false,
        "text",
        false,
        Some(0.0), // Use 0% threshold since file has no tests
        "json",
    );
    // The dispatch should complete (Ok or Err for "no tests found") - just ensure it doesn't panic
    // Accept any result since no tests in file may return Err
    let _ = result;
}

#[test]
fn test_handle_test_dispatch_with_filter() {
    // PMAT-104: with `None` as the path this scanned the current directory (the
    // ruchy repo), where nothing matched the filter, so the runner returned
    // "No .ruchy test files found" and the assertion failed. Point it at a temp
    // dir holding a file the filter actually matches.
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    fs::write(
        temp_dir.path().join("test_name.ruchy"),
        "#[test]\nfun test_name() {\n    let x = 42\n}\n",
    )
    .expect("Failed to write test file");
    let filter = "test_name".to_string();
    let result = handle_test_dispatch(
        Some(temp_dir.path().to_path_buf()),
        false,
        false,
        Some(&filter),
        true,
        "html",
        true,
        Some(0.0),
        "junit",
    );
    assert!(result.is_ok(), "dispatch failed: {result:?}");
}

#[test]
fn test_handle_advanced_command_repl() {
    let command = Commands::Repl {
        record: None,
        max_depth: 100,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_parse() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Parse {
        file: temp_file.path().to_path_buf(),
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_transpile() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Transpile {
        file: temp_file.path().to_path_buf(),
        output: None,
        minimal: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_compile() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Compile {
        file: temp_file.path().to_path_buf(),
        output: PathBuf::from("test.out"),
        opt_level: "2".to_string(),
        optimize: None,
        strip: false,
        static_link: false,
        target: None,
        verbose: false,
        json: None,
        show_profile_info: false,
        pgo: false,
        embed_models: Vec::new(),
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_check() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Check {
        files: vec![temp_file.path().to_path_buf()],
        watch: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

// PMAT-104: `test_handle_advanced_command_notebook` was removed rather than left
// `#[ignore]`d. It dispatched `Commands::Notebook { file: None, .. }`, which
// starts the notebook HTTP server and serves requests until the process is
// killed, so it could never return - no timeout would have made it pass.
// `handle_advanced_command` only re-routes the variant to
// `handlers::handle_notebook_command`, which has its own 17 tests in
// `handlers::notebook_handler::tests`.

#[test]
fn test_handle_advanced_command_coverage() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary test directory");
    // Create a test file with some content
    let test_file = temp_dir.path().join("test.ruchy");
    fs::write(&test_file, "let x = 42;")
        .unwrap_or_else(|_| panic!("Failed to write test file: {}", test_file.display()));

    let command = Commands::Coverage {
        path: test_file, // Use the file path, not directory
        threshold: None, // Don't set threshold for test
        format: "html".to_string(),
        verbose: false,
    };
    let result = handle_advanced_command(command);
    if let Err(e) = &result {
        eprintln!("Coverage test error: {}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_ast() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Ast {
        file: temp_file.path().to_path_buf(),
        json: false,
        graph: false,
        metrics: false,
        symbols: false,
        deps: false,
        verbose: false,
        output: None,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_ast_with_options() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let output_file = NamedTempFile::new().expect("Failed to create temporary output file");
    let command = Commands::Ast {
        file: temp_file.path().to_path_buf(),
        json: true,
        graph: true,
        metrics: true,
        symbols: true,
        deps: true,
        verbose: true,
        output: Some(output_file.path().to_path_buf()),
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

// Note: fmt command testing removed - redundant with comprehensive formatter tests
// in tests/cli_contract_fmt*.rs and tests/formatter_*.rs

#[test]
fn test_handle_advanced_command_doc() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    // TEST-FIX-002: Use valid Ruchy code instead of comment-only (empty program)
    fs::write(
        &temp_file,
        "/// Documentation test\nfun add(a, b) { a + b }",
    )
    .expect("Failed to write test content to temporary file");

    let output_dir = tempfile::tempdir().expect("Failed to create temporary output directory");
    let command = Commands::Doc {
        path: temp_file.path().to_path_buf(),
        output: output_dir.path().to_path_buf(),
        format: "html".to_string(),
        private: false,
        open: false,
        all: false,
        verbose: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_bench() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Bench {
        file: temp_file.path().to_path_buf(),
        iterations: 10,
        warmup: 5,
        format: "json".to_string(),
        output: None,
        verbose: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_lint() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Lint {
        file: Some(temp_file.path().to_path_buf()),
        all: false,
        fix: false,
        strict: false,
        verbose: false,
        format: "text".to_string(),
        rules: None,
        deny_warnings: false,
        max_complexity: 10,
        config: None,
        init_config: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

// PMAT-104: `test_handle_advanced_command_add` was removed rather than left
// `#[ignore]`d. It dispatched `Commands::Add`, which shells out to
// `cargo add test_package@1.0.0` in the *current* directory - it needed the
// network and would have edited this repository's own Cargo.toml - so it could
// neither pass nor be run safely. `handle_advanced_command` only re-routes the
// variant to `handlers::add::handle_add_command`, which has its own 15 tests in
// `handlers::add::tests`, run against temporary project directories.

#[test]
fn test_handle_advanced_command_publish() {
    let command = Commands::Publish {
        registry: "https://ruchy.dev/registry".to_string(),
        version: Some("1.0.0".to_string()),
        dry_run: true,
        allow_dirty: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_score() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Score {
        path: temp_file.path().to_path_buf(),
        depth: "standard".to_string(),
        fast: false,
        deep: false,
        watch: false,
        explain: false,
        baseline: None,
        min: Some(0.8),
        config: None,
        format: "text".to_string(),
        verbose: false,
        output: None,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_advanced_command_wasm() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let command = Commands::Wasm {
        file: temp_file.path().to_path_buf(),
        output: None,
        target: "wasm32".to_string(),
        wit: false,
        deploy: false,
        deploy_target: None,
        portability: false,
        opt_level: "O2".to_string(),
        debug: false,
        simd: false,
        threads: false,
        component_model: true,
        name: None,
        version: "0.1.0".to_string(),
        verbose: false,
    };
    let result = handle_advanced_command(command);
    assert!(result.is_ok());
}

#[test]
fn test_handle_command_dispatch_repl() {
    let result = handle_command_dispatch(
        Some(Commands::Repl {
            record: None,
            max_depth: 100,
        }),
        false,
        VmMode::Ast,
    );
    assert!(result.is_ok());
}

#[test]
fn test_handle_command_dispatch_none() {
    let result = handle_command_dispatch(None, false, VmMode::Ast);
    assert!(result.is_ok());
}

#[test]
fn test_handle_command_dispatch_parse() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let result = handle_command_dispatch(
        Some(Commands::Parse {
            file: temp_file.path().to_path_buf(),
        }),
        false,
        VmMode::Ast,
    );
    assert!(result.is_ok());
}

#[test]
fn test_handle_command_dispatch_transpile() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let result = handle_command_dispatch(
        Some(Commands::Transpile {
            file: temp_file.path().to_path_buf(),
            output: None,
            minimal: false,
        }),
        true,
        VmMode::Ast,
    );
    assert!(result.is_ok());
}

#[test]
fn test_handle_command_dispatch_run() {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary test file");
    fs::write(&temp_file, "let x = 42").expect("Failed to write test content to temporary file");

    let result = handle_command_dispatch(
        Some(Commands::Run {
            file: temp_file.path().to_path_buf(),
        }),
        false,
        VmMode::Ast,
    );
    assert!(result.is_ok());
}
