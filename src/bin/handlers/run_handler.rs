//! Run Command Handler
//!
//! Handles execution of Ruchy files via compilation or interpretation.

use anyhow::{Context, Result};
use ruchy::frontend::ast::Expr;
use ruchy::{Parser as RuchyParser, Transpiler};
use std::fs;
use std::path::{Path, PathBuf};

/// VM execution mode (OPT-004)
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum VmMode {
    /// Use AST interpreter (default, stable)
    Ast,
    /// Use bytecode VM (experimental, 40-60% faster)
    Bytecode,
}

/// Handle run command - compile and execute a Ruchy file
pub fn handle_run_command(file: &Path, verbose: bool, vm_mode: VmMode) -> Result<()> {
    log_run_start(file, verbose);

    if verbose {
        println!("Execution mode: {:?}", vm_mode);
    }

    // FIX Issue #80: Support stdin input with `-` argument (Unix convention)
    let source = if file.to_str() == Some("-") {
        use std::io::Read;
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    } else {
        super::read_file_with_context(file)?
    };

    // FIX CLI-CONTRACT-RUN-001: Parse the entire file FIRST to catch syntax errors
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("✗ Syntax error: {e}");
            eprintln!("Error: Syntax error: {e}");
            std::process::exit(1);
        }
    };

    // ISSUE-106: Module resolution for interpreter path
    // LIMITATION: mod declarations work for compilation but not interpretation
    // RATIONALE: The REPL API needs to support AST-based evaluation for this to work cleanly

    match vm_mode {
        VmMode::Ast => {
            // CLI-UNIFY-002: Use interpreter (like handle_file_execution), not compiler
            // This matches Deno/Python/Ruby/Node behavior: `run` = interpret immediately
            // For compilation to binary, use: `ruchy compile`
            let mut repl = super::create_repl()?;

            match repl.eval(&source) {
                Ok(_result) => {
                    // FIX CLI-CONTRACT-RUN-002: Don't print file evaluation results
                    // The user's code uses println() for output. We should NOT print the
                    // final value of file evaluation (that's REPL behavior, not script behavior).
                    // This matches Python/Ruby/Node: `python script.py` doesn't print the last value.

                    // After evaluating the file, check if main() function exists and call it
                    // (but also don't print main's return value - it's not a println)
                    // FIX Issue #81: Handle main() errors (panic!, undefined functions, etc.)
                    match repl.eval("main()") {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            eprintln!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
        VmMode::Bytecode => {
            // OPT-004: Bytecode VM execution path (40-60% faster than AST)
            use ruchy::runtime::bytecode::{Compiler, VM};

            let mut compiler = Compiler::new("main".to_string());
            if let Err(e) = compiler.compile_expr(&ast) {
                eprintln!("✗ Compilation error: {}", e);
                eprintln!("Error: Compilation error: {}", e);
                std::process::exit(1);
            }

            let chunk = compiler.finalize();
            let mut vm = VM::new();

            match vm.execute(&chunk) {
                Ok(_result) => {
                    // Don't print result (same as AST mode)
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ VM execution error: {}", e);
                    eprintln!("Error: VM execution error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Log run command start (complexity: 2)
fn log_run_start(file: &Path, verbose: bool) {
    if verbose {
        eprintln!("Running file: {}", file.display());
    }
}

/// Transpile AST for execution with context (complexity: 3)
#[allow(dead_code)]
pub fn transpile_for_execution(ast: &Expr, file: &Path) -> Result<String> {
    let mut transpiler = Transpiler::new();
    transpiler
        .transpile_to_program_with_context(ast, Some(file))
        .map(|tokens| tokens.to_string())
        .with_context(|| "Failed to transpile to Rust")
}

/// Prepare compilation artifacts (complexity: 4)
#[allow(dead_code)]
pub fn prepare_compilation(
    rust_code: &str,
    verbose: bool,
) -> Result<(tempfile::NamedTempFile, PathBuf)> {
    let temp_source =
        tempfile::NamedTempFile::new().with_context(|| "Failed to create temporary file")?;
    fs::write(temp_source.path(), rust_code).with_context(|| "Failed to write temporary file")?;
    if verbose {
        eprintln!("Temporary Rust file: {}", temp_source.path().display());
        eprintln!("Compiling and running...");
    }
    // Create unique binary path using process ID + timestamp to avoid parallel test collisions
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let binary_path = std::env::temp_dir().join(format!(
        "ruchy_temp_bin_{}_{}",
        std::process::id(),
        unique_id
    ));
    Ok((temp_source, binary_path))
}

/// Compile Rust code using rustc (complexity: 5)
#[allow(dead_code)]
pub fn compile_rust_code(source_path: &Path, binary_path: &Path) -> Result<()> {
    let output = std::process::Command::new("rustc")
        .arg("--edition=2018")
        .arg("--crate-name=ruchy_temp")
        .arg("-o")
        .arg(binary_path)
        .arg(source_path)
        .output()
        .with_context(|| "Failed to run rustc")?;
    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Compilation failed"));
    }
    Ok(())
}

/// Execute compiled binary and handle output (complexity: 5)
#[allow(dead_code)]
pub fn execute_binary(binary_path: &Path) -> Result<()> {
    let run_output = std::process::Command::new(binary_path)
        .output()
        .with_context(|| "Failed to run compiled binary")?;
    print!("{}", String::from_utf8_lossy(&run_output.stdout));
    if !run_output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&run_output.stderr));
    }
    if !run_output.status.success() {
        return Err(anyhow::anyhow!(
            "Program exited with code {}",
            run_output.status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vm_mode_values() {
        assert_eq!(VmMode::Ast, VmMode::Ast);
        assert_eq!(VmMode::Bytecode, VmMode::Bytecode);
        assert_ne!(VmMode::Ast, VmMode::Bytecode);
    }

    #[test]
    fn test_vm_mode_debug() {
        let ast_debug = format!("{:?}", VmMode::Ast);
        let bytecode_debug = format!("{:?}", VmMode::Bytecode);
        assert!(ast_debug.contains("Ast"));
        assert!(bytecode_debug.contains("Bytecode"));
    }

    #[test]
    fn test_vm_mode_clone() {
        let mode = VmMode::Ast;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_log_run_start_verbose() {
        // Just verify it doesn't panic
        let file = Path::new("test.ruchy");
        log_run_start(file, true);
        log_run_start(file, false);
    }

    // ===== EXTREME TDD Round 149 - Run Handler Tests =====

    #[test]
    fn test_vm_mode_copy() {
        let mode = VmMode::Bytecode;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_vm_mode_all_variants() {
        let modes = [VmMode::Ast, VmMode::Bytecode];
        assert_eq!(modes.len(), 2);
    }

    #[test]
    fn test_log_run_start_various_paths() {
        let paths = [
            Path::new("simple.ruchy"),
            Path::new("path/to/file.ruchy"),
            Path::new("/absolute/path.ruchy"),
        ];
        for path in paths {
            log_run_start(path, true);
            log_run_start(path, false);
        }
    }

    #[test]
    fn test_transpile_for_execution_nonexistent() {
        use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
        let ast = Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpile_for_execution(&ast, Path::new("/test.ruchy"));
        // May succeed or fail
        let _ = result;
    }

    #[test]
    fn test_prepare_compilation_basic() {
        let result = prepare_compilation("fn main() {}", false);
        if let Ok((temp_source, binary_path)) = result {
            assert!(temp_source.path().exists());
            // Binary path should not exist yet (not compiled)
            let _ = binary_path;
        }
    }

    #[test]
    fn test_prepare_compilation_verbose() {
        let result = prepare_compilation("fn main() { println!(\"test\"); }", true);
        // Just verify it runs with verbose mode
        let _ = result;
    }

    #[test]
    fn test_compile_rust_code_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("invalid.rs");
        std::fs::write(&source_path, "invalid rust code !!!").unwrap();
        let binary_path = temp_dir.path().join("output");
        let result = compile_rust_code(&source_path, &binary_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_binary_nonexistent() {
        let result = execute_binary(Path::new("/nonexistent/binary"));
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_mode_pattern_matching() {
        let mode = VmMode::Ast;
        let msg = match mode {
            VmMode::Ast => "ast",
            VmMode::Bytecode => "bytecode",
        };
        assert_eq!(msg, "ast");
    }

    #[test]
    fn test_vm_mode_bytecode_pattern() {
        let mode = VmMode::Bytecode;
        let msg = match mode {
            VmMode::Ast => "ast",
            VmMode::Bytecode => "bytecode",
        };
        assert_eq!(msg, "bytecode");
    }

    #[test]
    fn test_log_run_start_unicode_path() {
        let path = Path::new("日本語/test.ruchy");
        log_run_start(path, true);
        log_run_start(path, false);
    }

    #[test]
    fn test_log_run_start_empty_path() {
        let path = Path::new("");
        log_run_start(path, true);
        log_run_start(path, false);
    }
}
