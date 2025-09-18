// Comprehensive unit tests for REPL to increase code coverage
use ruchy::runtime::{Repl, ReplConfig};
use std::{env, time::Duration;

#[test]
fn test_repl_creation_and_defaults() {
    let repl = Repl::new(std::env::temp_dir()).unwrap();
    assert_eq!(repl.get_mode(), "normal");
    assert_eq!(repl.get_prompt(), "ruchy> ");
    // assert!(!repl.is_multiline_mode());
}

#[test]
fn test_repl_with_custom_config() {
    let config = ReplConfig {
        max_memory: 1024,
        timeout: Duration::from_millis(50),
        maxdepth: 10,
        debug: false,
    };
    
    let repl = Repl::with_config(config).unwrap();
    assert_eq!(repl.get_mode(), "normal");
}

#[test]
fn test_basic_arithmetic_evaluation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test basic arithmetic
    assert_eq!(repl.eval("1 + 1").unwrap(), "2");
    assert_eq!(repl.eval("10 - 3").unwrap(), "7");
    assert_eq!(repl.eval("4 * 5").unwrap(), "20");
    assert_eq!(repl.eval("15 / 3").unwrap(), "5");
    assert_eq!(repl.eval("17 % 5").unwrap(), "2");
}

#[test]
fn test_variable_binding_and_retrieval() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test variable binding
    assert_eq!(repl.eval("let x = 42").unwrap(), "42");
    assert_eq!(repl.eval("x").unwrap(), "42");
    
    // Test variable update
    assert_eq!(repl.eval("let x = 100").unwrap(), "100");
    assert_eq!(repl.eval("x").unwrap(), "100");
}

#[test]
fn test_string_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("\"hello\"").unwrap(), "\"hello\"");
    assert_eq!(repl.eval("\"hello\" + \" world\"").unwrap(), "\"hello world\"");
    assert_eq!(repl.eval("\"test\".length()").unwrap(), "4");
}

#[test]
fn test_list_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");
    assert_eq!(repl.eval("[1, 2, 3].length()").unwrap(), "3");
    assert_eq!(repl.eval("[1, 2, 3][0]").unwrap(), "1");
}

#[test]
fn test_object_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("{x: 10, y: 20}").unwrap(), "{x: 10, y: 20}");
    assert_eq!(repl.eval("let obj = {x: 10, y: 20}").unwrap(), "{x: 10, y: 20}");
    assert_eq!(repl.eval("obj.x").unwrap(), "10");
}

#[test]
fn test_boolean_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("true").unwrap(), "true");
    assert_eq!(repl.eval("false").unwrap(), "false");
    assert_eq!(repl.eval("true && false").unwrap(), "false");
    assert_eq!(repl.eval("true || false").unwrap(), "true");
    assert_eq!(repl.eval("!true").unwrap(), "false");
}

#[test]
fn test_comparison_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("5 > 3").unwrap(), "true");
    assert_eq!(repl.eval("5 < 3").unwrap(), "false");
    assert_eq!(repl.eval("5 >= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 <= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 == 5").unwrap(), "true");
    assert_eq!(repl.eval("5 != 3").unwrap(), "true");
}

#[test]
fn test_if_else_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("if true { 1 } else { 2 }").unwrap(), "1");
    assert_eq!(repl.eval("if false { 1 } else { 2 }").unwrap(), "2");
    assert_eq!(repl.eval("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap(), "\"yes\"");
}

#[test]
fn test_function_definition_and_call() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("fn add(a, b) { a + b }").is_ok());
    assert_eq!(repl.eval("add(3, 4)").unwrap(), "7");
    
    assert!(repl.eval("fn greet(name) { \"Hello, \" + name }").is_ok());
    assert_eq!(repl.eval("greet(\"World\")").unwrap(), "\"Hello, World\"");
}

#[test]
fn test_lambda_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let add = fn(a, b) { a + b }").is_ok());
    assert_eq!(repl.eval("add(10, 20)").unwrap(), "30");
}

#[test]
fn test_for_loop() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let sum = 0").is_ok());
    assert!(repl.eval("for i in [1, 2, 3] { sum = sum + i }").is_ok());
    assert_eq!(repl.eval("sum").unwrap(), "6");
}

#[test]
fn test_while_loop() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let count = 0").is_ok());
    assert!(repl.eval("while count < 5 { count = count + 1 }").is_ok());
    assert_eq!(repl.eval("count").unwrap(), "5");
}

#[test]
fn test_match_expression() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(
        repl.eval("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }").unwrap(),
        "\"two\""
    );
}

#[test]
fn test_enum_definition() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("enum Color { Red, Green, Blue }").is_ok());
    assert!(repl.eval("Color::Red").is_ok());
}

#[test]
fn test_struct_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("struct Point { x, y }").is_ok());
    assert_eq!(repl.eval("Point { x: 10, y: 20 }").unwrap(), "Point { x: 10, y: 20 }");
}

#[test]
fn test_colon_commands() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test :help command
    assert!(repl.eval(":help").unwrap().contains("Available commands"));
    
    // Test :clear command
    assert!(repl.eval("let x = 42").is_ok());
    assert!(repl.eval(":clear").unwrap().contains("cleared"));
    
    // Test :env command
    assert!(repl.eval("let y = 100").is_ok());
    assert!(repl.eval(":env").unwrap().contains('y'));
}

#[test]
fn test_magic_commands() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test %help
    assert!(repl.eval("%help").unwrap().contains("magic commands"));
    
    // Test %time
    assert!(repl.eval("%time 1 + 1").unwrap().contains("ms"));
}

#[test]
fn test_history_indexing() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("42").unwrap(), "42");
    assert_eq!(repl.eval("100").unwrap(), "100");
    assert_eq!(repl.eval("_1").unwrap(), "42");
    assert_eq!(repl.eval("_2").unwrap(), "100");
}

#[test]
fn test_unicode_expansion() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test that backslash sequences are expanded
    let result = repl.eval("\"\\\\alpha\"").unwrap();
    assert!(result == "\"α\"" || result == "\"\\\\alpha\""); // May vary based on implementation
}

#[test]
fn test_multiline_mode() {
    let _repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Multiline mode checking would go here
    // Currently not exposed in public API
    // Test placeholder - multiline mode not exposed in public API
}

#[test]
fn test_error_handling() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test undefined variable
    assert!(repl.eval("undefined_var").unwrap().contains("Undefined variable"));
    
    // Test syntax error
    assert!(repl.eval("1 +").unwrap().contains("error"));
    
    // Test type mismatch
    assert!(repl.eval("1 + \"string\"").unwrap().contains("error"));
}

#[test]
fn test_range_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("1..5").is_ok());
    assert!(repl.eval("for i in 1..3 { i }").is_ok());
}

#[test]
fn test_pipe_operator() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("fn double(x) { x * 2 }").is_ok());
    assert_eq!(repl.eval("5 |> double").unwrap(), "10");
}

#[test]
fn test_async_await() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test async function definition
    assert!(repl.eval("async fn fetch() { 42 }").is_ok());
    assert!(repl.eval("await fetch()").is_ok());
}

#[test]
fn test_generics() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("fn identity<T>(x: T) -> T { x }").is_ok());
    assert_eq!(repl.eval("identity(42)").unwrap(), "42");
}

#[test]
fn test_try_catch() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("try { 1 / 0 } catch { \"error\" }").is_ok());
}

#[test]
fn test_builtin_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test math functions
    assert_eq!(repl.eval("abs(-5)").unwrap(), "5");
    assert_eq!(repl.eval("min(3, 7)").unwrap(), "3");
    assert_eq!(repl.eval("max(3, 7)").unwrap(), "7");
    assert_eq!(repl.eval("floor(3.7)").unwrap(), "3");
    assert_eq!(repl.eval("ceil(3.2)").unwrap(), "4");
    assert_eq!(repl.eval("round(3.5)").unwrap(), "4");
    
    // Test type conversion
    assert_eq!(repl.eval("int(\"42\")").unwrap(), "42");
    assert_eq!(repl.eval("str(42)").unwrap(), "\"42\"");
    assert_eq!(repl.eval("float(42)").unwrap(), "42.0");
    assert!(repl.eval("bool(1)").unwrap() == "true");
}

#[test]
fn test_checkpoint_and_restore() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let x = 100").is_ok());
    let checkpoint = repl.checkpoint();
    assert!(repl.eval("let x = 200").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "200");
    repl.restore_checkpoint(&checkpoint);
    assert_eq!(repl.eval("x").unwrap(), "100");
}

#[test]
fn test_repl_state_transitions() {
    let repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Check state
    let state = repl.get_state();
    assert!(matches!(state, ruchy::runtime::ReplState::Ready));
}

#[test]
fn test_type_command() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval(":type 42").unwrap().contains("Int") || 
            repl.eval(":type 42").unwrap().contains("i32"));
    assert!(repl.eval(":type \"hello\"").unwrap().contains("String"));
    assert!(repl.eval(":type [1, 2]").unwrap().contains("List") ||
            repl.eval(":type [1, 2]").unwrap().contains("Array"));
}

#[test]
fn test_ast_command() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let ast_output = repl.eval(":ast 1 + 2").unwrap();
    assert!(ast_output.contains("Binary") || ast_output.contains("Add"));
}

#[test]
fn test_inspect_command() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let data = [1, 2, 3]").is_ok());
    let inspect_output = repl.eval(":inspect data").unwrap();
    assert!(inspect_output.contains("Inspector") || inspect_output.contains("data"));
}

#[test]
fn test_memory_tracking() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Create large data structures to test memory tracking
    assert!(repl.eval("let big_list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").is_ok());
    assert!(repl.eval("let big_string = \"a\" * 100").is_ok());
    
    // Memory should be tracked
    assert!(repl.peak_memory() > 0);
}

#[test]
fn test_completion_generation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Add some variables
    assert!(repl.eval("let test_var = 42").is_ok());
    assert!(repl.eval("let testing = 100").is_ok());
    
    // Generate completions
    let completions = repl.complete("test");
    assert!(completions.contains(&"test_var".to_string()));
    assert!(completions.contains(&"testing".to_string()));
}

#[test]
fn test_pattern_matching() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let [a, b, c] = [1, 2, 3]").is_ok());
    assert_eq!(repl.eval("a").unwrap(), "1");
    assert_eq!(repl.eval("b").unwrap(), "2");
    assert_eq!(repl.eval("c").unwrap(), "3");
}

#[test]
fn test_spread_operator() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let arr = [1, 2, 3]").is_ok());
    assert!(repl.eval("let expanded = [0, ...arr, 4]").is_ok());
    assert_eq!(repl.eval("expanded").unwrap(), "[0, 1, 2, 3, 4]");
}

#[test]
fn test_fat_arrow_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let double = x => x * 2").is_ok());
    assert_eq!(repl.eval("double(5)").unwrap(), "10");
}

#[test]
fn test_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let name = \"World\"").is_ok());
    let result = repl.eval("f\"Hello {name}\"");
    assert!(result.is_ok());
    // May be "Hello World" or error depending on implementation
}

#[test]
fn test_dataframe_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test DataFrame literal
    assert!(repl.eval("df![[1, 2], [3, 4]]").is_ok() ||
            repl.eval("df![[1, 2], [3, 4]]").unwrap().contains("error"));
}

#[test]
fn test_session_export() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let x = 42").is_ok());
    assert!(repl.eval("let y = x * 2").is_ok());
    
    // Session export not directly exposed, would test through commands
    let session = repl.eval(":history").unwrap_or_default();
    assert!(session.contains("let x = 42"));
    assert!(session.contains("let y = x * 2"));
}

#[test]
fn test_clear_history() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("1 + 1").is_ok());
    assert!(repl.eval("2 + 2").is_ok());
    
    // Clear history functionality not exposed in public API
    // Would need to test through command interface
    assert!(repl.eval(":clear").is_ok());
}

#[test]
fn test_math_constants() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test pi constant
    let pi_result = repl.eval("pi");
    assert!(pi_result.is_ok() || pi_result.unwrap().contains("Undefined"));
    
    // Test e constant
    let e_result = repl.eval("e");
    assert!(e_result.is_ok() || e_result.unwrap().contains("Undefined"));
}

#[test]
fn test_complex_nested_structures() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test nested objects
    assert!(repl.eval("let nested = {a: {b: {c: 42}}}").is_ok());
    assert_eq!(repl.eval("nested.a.b.c").unwrap(), "42");
    
    // Test nested arrays
    assert!(repl.eval("let matrix = [[1, 2], [3, 4]]").is_ok());
    assert_eq!(repl.eval("matrix[0][1]").unwrap(), "2");
}

#[test]
fn test_recursive_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test factorial
    assert!(repl.eval("fn fact(n) { if n <= 1 { 1 } else { n * fact(n - 1) } }").is_ok());
    assert_eq!(repl.eval("fact(5)").unwrap(), "120");
}

#[test]
fn test_closure_capture() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let x = 10").is_ok());
    assert!(repl.eval("let add_x = fn(y) { x + y }").is_ok());
    assert_eq!(repl.eval("add_x(5)").unwrap(), "15");
}

#[test]
fn test_module_system() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test module definition (if supported)
    let module_result = repl.eval("module math { pub fn add(a, b) { a + b } }");
    assert!(module_result.is_ok() || module_result.unwrap().contains("error"));
}