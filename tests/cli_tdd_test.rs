// [RUCHY-207] TDD Tests for CLI Module Implementation
// PMAT Complexity: <10 per function

use ruchy::cli::{Cli, Command, NotebookCommand, WasmCommand, TestCommand};
use std::path::PathBuf;
use clap::Parser;

#[test]
fn test_cli_parse_notebook_serve() {
    let args = vec!["ruchy", "notebook", "serve", "--port", "9000"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Notebook(NotebookCommand::Serve { port }) => {
            assert_eq!(port, 9000);
        }
        _ => panic!("Expected Notebook Serve command"),
    }
}

#[test]
fn test_cli_parse_notebook_test() {
    let args = vec!["ruchy", "notebook", "test", "example.ipynb"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Notebook(NotebookCommand::Test { path, .. }) => {
            assert_eq!(path, PathBuf::from("example.ipynb"));
        }
        _ => panic!("Expected Notebook Test command"),
    }
}

#[test]
fn test_cli_parse_wasm_compile() {
    let args = vec!["ruchy", "wasm", "compile", "script.ruchy", "-o", "output.wasm"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Wasm(WasmCommand::Compile { input, output, .. }) => {
            assert_eq!(input, PathBuf::from("script.ruchy"));
            assert_eq!(output, Some(PathBuf::from("output.wasm")));
        }
        _ => panic!("Expected Wasm Compile command"),
    }
}

#[test]
fn test_cli_parse_test_command() {
    let args = vec!["ruchy", "test", "src/", "--coverage"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Test(TestCommand::Run { path, coverage, .. }) => {
            assert_eq!(path, PathBuf::from("src/"));
            assert!(coverage);
        }
        _ => panic!("Expected Test Run command"),
    }
}

#[test]
fn test_cli_parse_repl_command() {
    let args = vec!["ruchy", "repl"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Repl => {}
        _ => panic!("Expected REPL command"),
    }
}

#[test]
fn test_cli_parse_run_command() {
    let args = vec!["ruchy", "run", "script.ruchy"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Run { path } => {
            assert_eq!(path, PathBuf::from("script.ruchy"));
        }
        _ => panic!("Expected Run command"),
    }
}

#[test]
fn test_cli_parse_format_command() {
    let args = vec!["ruchy", "fmt", "src/", "--check"];
    let cli = Cli::parse_from(args);
    
    match cli.command {
        Command::Format { path, check } => {
            assert_eq!(path, PathBuf::from("src/"));
            assert!(check);
        }
        _ => panic!("Expected Format command"),
    }
}

#[test]
fn test_cli_parse_version_flag() {
    let args = vec!["ruchy", "--version"];
    let result = Cli::try_parse_from(args);
    
    // Version flag causes early exit, so we expect an error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("3.0.1"));
}

#[test]
fn test_cli_parse_verbose_flag() {
    let args = vec!["ruchy", "--verbose", "repl"];
    let cli = Cli::parse_from(args);
    
    assert!(cli.verbose);
    assert!(matches!(cli.command, Command::Repl));
}

#[test]
fn test_cli_parse_quiet_flag() {
    let args = vec!["ruchy", "--quiet", "test", "."];
    let cli = Cli::parse_from(args);
    
    assert!(cli.quiet);
}

// Property test for CLI parsing
#[test]
fn test_cli_property_all_commands_have_help() {
    let commands = vec![
        vec!["ruchy", "notebook", "--help"],
        vec!["ruchy", "wasm", "--help"],
        vec!["ruchy", "test", "--help"],
        vec!["ruchy", "--help"],
    ];
    
    for args in commands {
        let result = Cli::try_parse_from(args);
        assert!(result.is_err()); // Help causes early exit
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Usage"));
    }
}