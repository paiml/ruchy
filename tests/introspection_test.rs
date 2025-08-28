// TDD Test Suite for Introspection Features
// Testing ?object and ??object introspection, str(), summary()

use ruchy::runtime::repl::Repl;

#[test]
fn test_single_question_introspection_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Define a variable
    repl.eval("let x = 42").unwrap();
    
    // Test ?x introspection  
    let result = repl.eval("?x").unwrap();
    assert!(result.contains("Type") || result.contains("type") || result.contains("Integer"),
        "?x should show type information, got: {}", result);
    assert!(result.contains("42") || result.contains("Value"),
        "?x should show value information, got: {}", result);
}

#[test]
fn test_double_question_introspection() {
    let mut repl = Repl::new().unwrap();
    
    // Define a function
    repl.eval("fn add(a, b) { a + b }").unwrap();
    
    // Test ??add for detailed introspection
    let result = repl.eval("??add").unwrap();
    assert!(result.contains("Source") || result.contains("source") || result.contains("fn add"),
        "??add should show source code, got: {}", result);
    assert!(result.contains("Parameters") || result.contains("params") || result.contains("a, b"),
        "??add should show parameters, got: {}", result);
}

#[test]
fn test_introspection_on_lists() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let nums = [1, 2, 3, 4, 5]").unwrap();
    
    // Test ?nums
    let result = repl.eval("?nums").unwrap();
    assert!(result.contains("List") || result.contains("Array"),
        "?nums should show List type, got: {}", result);
    assert!(result.contains("5") || result.contains("length"),
        "?nums should show length info, got: {}", result);
}

#[test]
fn test_introspection_on_objects() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let person = {name: \"Alice\", age: 30}").unwrap();
    
    // Test ?person
    let result = repl.eval("?person").unwrap();
    assert!(result.contains("Object") || result.contains("Dict"),
        "?person should show Object type, got: {}", result);
    assert!(result.contains("name") || result.contains("age"),
        "?person should show field names, got: {}", result);
}

#[test]
fn test_str_function() {
    let mut repl = Repl::new().unwrap();
    
    // Test str() on various types
    let result = repl.eval("str(42)").unwrap();
    assert_eq!(result.trim().trim_matches('"'), "42",
        "str(42) should return \"42\"");
    
    let result = repl.eval("str(true)").unwrap();
    assert_eq!(result.trim().trim_matches('"'), "true",
        "str(true) should return \"true\"");
    
    let result = repl.eval("str([1, 2, 3])").unwrap();
    assert!(result.contains("1") && result.contains("2") && result.contains("3"),
        "str([1,2,3]) should contain all elements, got: {}", result);
}

#[test]
fn test_summary_function() {
    let mut repl = Repl::new().unwrap();
    
    // Create a larger data structure
    repl.eval("let data = [[1,2,3], [4,5,6], [7,8,9]]").unwrap();
    
    // Test summary() function
    let result = repl.eval("summary(data)").unwrap();
    assert!(result.contains("List") || result.contains("Array") || result.contains("3x3"),
        "summary should describe structure, got: {}", result);
}

#[test]
fn test_introspection_on_builtins() {
    let mut repl = Repl::new().unwrap();
    
    // Test introspection on builtin functions
    let result = repl.eval("?println").unwrap();
    assert!(result.contains("builtin") || result.contains("Builtin") || result.contains("Function"),
        "?println should indicate builtin function, got: {}", result);
}

#[test]
fn test_introspection_undefined() {
    let mut repl = Repl::new().unwrap();
    
    // Test introspection on undefined variable
    let result = repl.eval("?undefined_var");
    assert!(result.as_ref().is_err() || 
            result.as_ref().unwrap().contains("undefined") || 
            result.as_ref().unwrap().contains("not found") || 
            result.as_ref().unwrap().contains("Error"),
        "Should handle undefined variables gracefully");
}

#[test]
fn test_introspection_chain() {
    let mut repl = Repl::new().unwrap();
    
    // Create nested structure
    repl.eval("let nested = {a: {b: {c: 42}}}").unwrap();
    
    // Test introspection on top-level object
    let result = repl.eval("?nested").unwrap();
    assert!(result.contains("Object") || result.contains("a"),
        "Should introspect nested object, got: {}", result);
    
    // For now, nested field introspection is not supported
    // This would require enhancing the parser
}

#[test]
fn test_type_function() {
    let mut repl = Repl::new().unwrap();
    
    // Note: 'type' is a reserved keyword, so we use 'typeof' instead
    // Or we can use introspection with ?
    
    // Test getting type using introspection
    repl.eval("let x = 42").unwrap();
    let result = repl.eval("?x").unwrap();
    assert!(result.contains("Integer"),
        "?x should show Integer type, got: {}", result);
    
    repl.eval("let s = \"hello\"").unwrap();
    let result = repl.eval("?s").unwrap();
    assert!(result.contains("String"),
        "?s should show String type, got: {}", result);
    
    repl.eval("let l = []").unwrap();
    let result = repl.eval("?l").unwrap();
    assert!(result.contains("List"),
        "?l should show List type, got: {}", result);
}

#[test]
fn test_dir_function() {
    let mut repl = Repl::new().unwrap();
    
    // Create object with methods
    repl.eval("let obj = {x: 10, get_x: fn() { this.x }}").unwrap();
    
    // Test dir() function
    let result = repl.eval("dir(obj)").unwrap();
    assert!(result.contains("x") && result.contains("get_x"),
        "dir() should list object members, got: {}", result);
}

#[test]
fn test_help_function() {
    let mut repl = Repl::new().unwrap();
    
    // Test help() on builtin
    let result = repl.eval("help(println)").unwrap();
    assert!(result.contains("print") || result.contains("output") || result.contains("stdout"),
        "help(println) should show documentation, got: {}", result);
    
    // Test help() on user function
    repl.eval("fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }").unwrap();
    let result = repl.eval("help(factorial)").unwrap();
    assert!(result.contains("factorial") || result.contains("n"),
        "help(factorial) should show function info, got: {}", result);
}

#[test]
fn test_double_question_source_code() {
    let mut repl = Repl::new().unwrap();
    
    // Define a multi-line function
    repl.eval("fn fibonacci(n) { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }").unwrap();
    
    // Test ??fibonacci shows full source
    let result = repl.eval("??fibonacci").unwrap();
    assert!(result.contains("fibonacci") && result.contains("n") && 
            (result.contains("n - 1") || result.contains("n-1") || result.contains("n - Integer(1)")),
        "??fibonacci should show recursive calls in source, got: {}", result);
}

#[test]
fn test_introspection_with_metadata() {
    let mut repl = Repl::new().unwrap();
    
    // Create various types
    repl.eval("let i = 42").unwrap();
    repl.eval("let f = 3.14").unwrap();
    repl.eval("let s = \"hello\"").unwrap();
    repl.eval("let b = true").unwrap();
    
    // Test that introspection shows metadata
    let result = repl.eval("?i").unwrap();
    assert!(result.contains("42"),
        "?i should show value 42, got: {}", result);
    
    let result = repl.eval("?f").unwrap();
    assert!(result.contains("3.14"),
        "?f should show value 3.14, got: {}", result);
    
    let result = repl.eval("?s").unwrap();
    assert!(result.contains("hello"),
        "?s should show value hello, got: {}", result);
    
    let result = repl.eval("?b").unwrap();
    assert!(result.contains("true"),
        "?b should show value true, got: {}", result);
}