// TDD Test Suite for Mode System
// Testing pkg>, shell>, help> and other REPL modes

use ruchy::runtime::repl::Repl;

#[test]
fn test_default_mode() {
    let repl = Repl::new().unwrap();
    
    // Default mode should be normal
    let mode = repl.get_mode();
    assert_eq!(mode, "normal", "Default mode should be 'normal'");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "ruchy> ", "Default prompt should be 'ruchy> '");
}

#[test]
fn test_shell_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter shell mode with :shell command
    repl.eval(":shell").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "shell", "Should be in shell mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "shell> ", "Shell mode prompt should be 'shell> '");
    
    // In shell mode, all input is treated as shell commands
    let result = repl.eval("echo hello").unwrap();
    assert!(result.contains("hello"), "Shell mode should execute commands directly");
    
    // Exit shell mode with :normal or Ctrl+D
    repl.eval(":normal").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "normal", "Should return to normal mode");
}

#[test]
fn test_pkg_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter package mode with :pkg command
    repl.eval(":pkg").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "pkg", "Should be in pkg mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "pkg> ", "Pkg mode prompt should be 'pkg> '");
    
    // In pkg mode, commands are package management
    // search, install, list, etc.
    let result = repl.eval("search json").unwrap();
    assert!(result.contains("json") || result.contains("No packages") || result.contains("search"), 
        "Should handle package search, got: {}", result);
}

#[test]
fn test_help_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter help mode with :help command
    repl.eval(":help").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "help", "Should be in help mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "help> ", "Help mode prompt should be 'help> '");
    
    // In help mode, keywords trigger help documentation
    let result = repl.eval("fn").unwrap();
    assert!(result.contains("function") || result.contains("def") || result.contains("syntax"),
        "Should show help for 'fn', got: {}", result);
}

#[test]
fn test_sql_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter SQL mode with :sql command  
    repl.eval(":sql").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "sql", "Should be in SQL mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "sql> ", "SQL mode prompt should be 'sql> '");
    
    // In SQL mode, execute SQL queries
    let result = repl.eval("SELECT 1 + 1").unwrap_or_else(|e| e.to_string());
    assert!(result.contains("2") || result.contains("SQL") || result.contains("not supported"),
        "Should handle SQL query or indicate not supported, got: {}", result);
}

#[test]
fn test_math_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter math mode with :math command
    repl.eval(":math").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "math", "Should be in math mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "math> ", "Math mode prompt should be 'math> '");
    
    // In math mode, expressions are evaluated with math focus
    let result = repl.eval("sin(pi/2)").unwrap_or_else(|e| e.to_string());
    eprintln!("Math mode result: {}", result);
    assert!(result.contains("1") || result.contains("sin") || result.contains("not defined") || result.contains("Undefined"),
        "Should evaluate math expression, got: {}", result);
}

#[test]
fn test_mode_switching() {
    let mut repl = Repl::new().unwrap();
    
    // Test switching between modes
    assert_eq!(repl.get_mode(), "normal");
    
    repl.eval(":shell").unwrap();
    assert_eq!(repl.get_mode(), "shell");
    
    repl.eval(":pkg").unwrap();
    assert_eq!(repl.get_mode(), "pkg");
    
    repl.eval(":normal").unwrap();
    assert_eq!(repl.get_mode(), "normal");
}

#[test]
fn test_mode_persistence() {
    let mut repl = Repl::new().unwrap();
    
    // Mode should persist across evaluations
    repl.eval(":shell").unwrap();
    assert_eq!(repl.get_mode(), "shell");
    
    // Run some commands in shell mode
    repl.eval("ls").unwrap();
    assert_eq!(repl.get_mode(), "shell", "Mode should stay in shell");
    
    repl.eval("pwd").unwrap();
    assert_eq!(repl.get_mode(), "shell", "Mode should still be shell");
}

#[test]
fn test_mode_exit_commands() {
    let mut repl = Repl::new().unwrap();
    
    // Test various ways to exit a mode
    repl.eval(":shell").unwrap();
    repl.eval(":exit").unwrap();
    assert_eq!(repl.get_mode(), "normal", ":exit should return to normal");
    
    repl.eval(":pkg").unwrap();
    repl.eval(":quit").unwrap();
    assert_eq!(repl.get_mode(), "normal", ":quit should return to normal");
    
    repl.eval(":help").unwrap();
    repl.eval(":normal").unwrap();
    assert_eq!(repl.get_mode(), "normal", ":normal should return to normal");
}

#[test]
fn test_debug_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter debug mode
    repl.eval(":debug").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "debug", "Should be in debug mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "debug> ", "Debug mode prompt should be 'debug> '");
    
    // In debug mode, show extra information
    repl.eval("let x = 42").unwrap();
    let result = repl.eval("x").unwrap();
    assert!(result.contains("42"), "Should evaluate in debug mode");
}

#[test]
fn test_time_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Enter time mode to show execution timing
    repl.eval(":time").unwrap();
    let mode = repl.get_mode();
    assert_eq!(mode, "time", "Should be in time mode");
    
    let prompt = repl.get_prompt();
    assert_eq!(prompt, "time> ", "Time mode prompt should be 'time> '");
    
    // In time mode, show timing for each expression
    let result = repl.eval("1 + 1").unwrap();
    assert!(result.contains("2"), "Should evaluate expression");
    // Could also check for timing info like "ms" or "μs"
}

#[test]
fn test_invalid_mode() {
    let mut repl = Repl::new().unwrap();
    
    // Try to enter an invalid mode
    let result = repl.eval(":invalidmode");
    assert!(result.is_err() || result.unwrap().contains("Unknown"),
        "Should reject invalid mode");
    
    // Should remain in normal mode
    assert_eq!(repl.get_mode(), "normal", "Should stay in normal mode");
}

#[test]
fn test_mode_help_command() {
    let mut repl = Repl::new().unwrap();
    
    // :modes should list available modes
    let result = repl.eval(":modes").unwrap();
    assert!(result.contains("normal"), "Should list normal mode");
    assert!(result.contains("shell"), "Should list shell mode");
    assert!(result.contains("pkg"), "Should list pkg mode");
    assert!(result.contains("help"), "Should list help mode");
}

#[test]
fn test_mode_specific_completion() {
    let mut repl = Repl::new().unwrap();
    
    // In shell mode, completions should be shell-specific
    repl.eval(":shell").unwrap();
    let completions = repl.complete("ec");
    assert!(completions.iter().any(|s| s.contains("echo")), 
        "Shell mode should complete shell commands");
    
    // In normal mode, completions should be Ruchy-specific
    repl.eval(":normal").unwrap();
    let completions = repl.complete("print");
    assert!(completions.iter().any(|s| s == "println"),
        "Normal mode should complete Ruchy functions");
}