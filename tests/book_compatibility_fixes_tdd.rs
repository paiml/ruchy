// TDD Tests for book compatibility fixes
use ruchy::Parser;

#[test]
fn test_let_statement_parsing_issue() {
    // Based on lint error: "Expected identifier after 'let'"
    let problematic_cases = vec![
        "let x = 5",
        "let mut x = 5", 
        "let (x, y) = (1, 2)",
        "let [a, b] = [1, 2]",
    ];
    
    for case in problematic_cases {
        let mut parser = Parser::new(case);
        let result = parser.parse();
        assert!(result.is_ok(), "Let statement '{}' should parse: {:?}", case, result.err());
    }
}

#[test]
fn test_return_in_if_statement() {
    // Based on lint error: "Expected body after if condition... Unexpected token: Return"
    let problematic_cases = vec![
        "if x > 0 return x",
        "if x > 0 return x else return 0",
        "if condition return early_value",
        "if error return Err(error)",
    ];
    
    for case in problematic_cases {
        let mut parser = Parser::new(case);
        let result = parser.parse();
        assert!(result.is_ok(), "Return in if '{}' should parse: {:?}", case, result.err());
    }
}

#[test] 
fn test_while_accumulation_pattern() {
    // From failing test: test_02_while_accumulation.ruchy
    let code = r"
    let mut sum = 0
    let mut i = 1
    while i <= 10 {
        sum += i
        i += 1
    }
    sum
    ";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "While accumulation should parse: {:?}", result.err());
}

#[test]
fn test_multiple_returns_pattern() {
    // From failing test: test_02_multiple_returns.ruchy
    let code = r"
    fun classify(x) {
        if x > 0 return 'positive'
        if x < 0 return 'negative' 
        return 'zero'
    }
    ";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Multiple returns should parse: {:?}", result.err());
}

#[test]
fn test_recursive_function_pattern() {
    // From failing test: test_03_recursive_function.ruchy  
    let code = r"
    fun factorial(n) {
        if n <= 1 return 1
        return n * factorial(n - 1)
    }
    ";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Recursive function should parse: {:?}", result.err());
}

#[test]
fn test_function_composition_pattern() {
    // From failing test: test_01_function_composition.ruchy
    let code = r"
    fun add(x, y) { x + y }
    fun multiply(x, y) { x * y }
    fun compose(f, g, x) { f(g(x, 2), 3) }
    ";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Function composition should parse: {:?}", result.err());
}

#[test]
fn test_simulated_file_operations() {
    // From failing test: test_01_simulated_file.ruchy
    let code = r"
    let file_content = 'Hello, World!'
    let lines = file_content.split('\n')
    lines.map(|line| line.trim())
    ";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "File operations should parse: {:?}", result.err());
}

#[test]
fn test_control_flow_variations() {
    // From failing control flow tests
    let control_flows = vec![
        "if x > 0 { return x }",
        "if x > 0 return x",
        "if condition { early_return() } else { continue_processing() }",
        "match value { Some(x) => return x, None => return 0 }",
    ];
    
    for cf in control_flows {
        let mut parser = Parser::new(cf);
        let result = parser.parse();
        assert!(result.is_ok(), "Control flow '{}' should parse: {:?}", cf, result.err());
    }
}

#[test]
fn test_string_methods_and_operations() {
    // From various failing string operation tests
    let string_ops = vec![
        "'hello'.upper()",
        "'HELLO'.lower()",
        "'  hello  '.trim()",
        "'a,b,c'.split(',')",
        "'hello' + ' ' + 'world'",
    ];
    
    for op in string_ops {
        let mut parser = Parser::new(op);
        let result = parser.parse();
        assert!(result.is_ok(), "String op '{}' should parse: {:?}", op, result.err());
    }
}

#[test]
fn test_object_inspection_patterns() {
    // From failing REPL object inspection tests
    let inspection_patterns = vec![
        "obj.field",
        "obj.method()",
        "nested.obj.deep.access",
        "array[index].property",
    ];
    
    for pattern in inspection_patterns {
        let mut parser = Parser::new(pattern);
        let result = parser.parse();
        assert!(result.is_ok(), "Inspection '{}' should parse: {:?}", pattern, result.err());
    }
}

#[test]
fn test_compound_assignment_operators() {
    // From accumulation and loop patterns
    let compound_ops = vec![
        "x += 1",
        "sum += value", 
        "count -= 1",
        "total *= factor",
        "result /= divisor",
        "remainder %= modulus",
    ];
    
    for op in compound_ops {
        let mut parser = Parser::new(op);
        let result = parser.parse();
        assert!(result.is_ok(), "Compound assignment '{}' should parse: {:?}", op, result.err());
    }
}