// TDD Test Suite for Tab Completion
// Testing context-aware tab completion and auto-suggestions

use ruchy::runtime::repl::Repl;

#[test]
fn test_tab_complete_variable_names() {
    let mut repl = Repl::new().unwrap();
    
    // Define some variables
    repl.eval("let variable1 = 10").unwrap();
    repl.eval("let variable2 = 20").unwrap();
    repl.eval("let var_special = 30").unwrap();
    
    // Test completion for "var"
    let completions = repl.complete("var");
    assert!(completions.iter().any(|s| s == "variable1"), "Should suggest variable1");
    assert!(completions.iter().any(|s| s == "variable2"), "Should suggest variable2");
    assert!(completions.iter().any(|s| s == "var_special"), "Should suggest var_special");
}

#[test]
fn test_tab_complete_function_names() {
    let mut repl = Repl::new().unwrap();
    
    // Define some functions
    repl.eval("fn function1() { 1 }").unwrap();
    repl.eval("fn function2(x) { x }").unwrap();
    repl.eval("fn func_helper() { 42 }").unwrap();
    
    // Test completion for "func"
    let completions = repl.complete("func");
    assert!(completions.iter().any(|s| s == "function1"), "Should suggest function1");
    assert!(completions.iter().any(|s| s == "function2"), "Should suggest function2");
    assert!(completions.iter().any(|s| s == "func_helper"), "Should suggest func_helper");
}

#[test]
fn test_tab_complete_builtin_functions() {
    let repl = Repl::new().unwrap();
    
    // Test completion for "print"
    let completions = repl.complete("print");
    assert!(completions.iter().any(|s| s == "println"), "Should suggest println builtin");
    assert!(completions.iter().any(|s| s == "print"), "Should suggest print builtin");
}

#[test]
fn test_tab_complete_keywords() {
    let repl = Repl::new().unwrap();
    
    // Test completion for "f"
    let completions = repl.complete("f");
    assert!(completions.iter().any(|s| s == "fn"), "Should suggest fn keyword");
    assert!(completions.iter().any(|s| s == "for"), "Should suggest for keyword");
    assert!(completions.iter().any(|s| s == "false"), "Should suggest false literal");
}

#[test]
fn test_tab_complete_methods() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let s = \"hello\"").unwrap();
    
    // Test completion for string methods after dot
    let completions = repl.complete("s.");
    assert!(completions.iter().any(|s| s == "s.len"), "Should suggest len method");
    assert!(completions.iter().any(|s| s == "s.upper"), "Should suggest upper method");
    assert!(completions.iter().any(|s| s == "s.lower"), "Should suggest lower method");
    assert!(completions.iter().any(|s| s == "s.trim"), "Should suggest trim method");
}

#[test]
fn test_tab_complete_field_access() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let obj = {name: \"Alice\", age: 30, city: \"NYC\"}").unwrap();
    
    // Test completion for object fields
    let completions = repl.complete("obj.");
    assert!(completions.iter().any(|s| s == "obj.name"), "Should suggest name field");
    assert!(completions.iter().any(|s| s == "obj.age"), "Should suggest age field");
    assert!(completions.iter().any(|s| s == "obj.city"), "Should suggest city field");
}

#[test]
fn test_tab_complete_empty_prefix() {
    let repl = Repl::new().unwrap();
    
    // Empty prefix should return all available completions
    let completions = repl.complete("");
    assert!(completions.iter().any(|s| s == "fn"), "Should include keywords");
    assert!(completions.iter().any(|s| s == "let"), "Should include let");
    assert!(completions.iter().any(|s| s == "println"), "Should include builtins");
    assert!(completions.len() > 10, "Should have many completions");
}

#[test]
fn test_tab_complete_partial_expression() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let num = 42").unwrap();
    
    // Test completion in middle of expression
    let completions = repl.complete("num + pr");
    assert!(completions.iter().any(|s| s == "println"), "Should suggest println");
    assert!(completions.iter().any(|s| s == "print"), "Should suggest print");
}

#[test]
fn test_tab_complete_case_insensitive() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let MyVariable = 100").unwrap();
    
    // Test case-insensitive matching
    let completions = repl.complete("myvar");
    assert!(completions.iter().any(|s| s == "MyVariable"), 
        "Should match case-insensitively, got: {:?}", completions);
}

#[test]
fn test_tab_complete_special_commands() {
    let repl = Repl::new().unwrap();
    
    // Test completion for colon commands
    let completions = repl.complete(":l");
    assert!(completions.iter().any(|s| s == ":load"), "Should suggest :load command");
    
    let completions = repl.complete(":h");
    assert!(completions.iter().any(|s| s == ":help"), "Should suggest :help command");
    
    let completions = repl.complete(":q");
    assert!(completions.iter().any(|s| s == ":quit"), "Should suggest :quit command");
}

#[test]
fn test_tab_complete_nested_access() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let data = {user: {name: \"Bob\", email: \"bob@example.com\"}}").unwrap();
    
    // Test nested field access (if supported)
    let completions = repl.complete("data.user.");
    assert!(!completions.is_empty(), "Should provide completions for nested object");
}

#[test]
fn test_tab_complete_array_methods() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let arr = [1, 2, 3]").unwrap();
    
    // Test array method completions
    let completions = repl.complete("arr.");
    assert!(completions.iter().any(|s| s == "arr.len"), "Should suggest len for arrays");
    assert!(completions.iter().any(|s| s == "arr.push"), "Should suggest push method");
    assert!(completions.iter().any(|s| s == "arr.pop"), "Should suggest pop method");
}

#[test]
fn test_tab_complete_no_duplicates() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let test = 1").unwrap();
    repl.eval("let test2 = 2").unwrap();
    
    let completions = repl.complete("test");
    let test_count = completions.iter().filter(|s| s == &"test").count();
    assert_eq!(test_count, 1, "Should not have duplicate completions");
}

#[test]
fn test_tab_complete_sorted_results() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let aaa = 1").unwrap();
    repl.eval("let aab = 2").unwrap();
    repl.eval("let aac = 3").unwrap();
    
    let completions = repl.complete("aa");
    // Check that results are alphabetically sorted
    let mut sorted = completions.clone();
    sorted.sort();
    assert_eq!(completions, sorted, "Completions should be sorted");
}