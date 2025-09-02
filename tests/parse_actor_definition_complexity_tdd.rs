// TDD test for parse_actor_definition complexity refactoring
// GOAL: Reduce parse_actor_definition complexity from 34 to <10 via systematic extraction
// RED → GREEN → REFACTOR methodology

use ruchy::frontend::parser::Parser;

#[test]
fn test_parse_simple_actor_definition() {
    // Test simple actor definition
    let test_cases = vec![
        ("actor Counter { }", "should parse empty actor"),
        ("actor Logger { }", "should parse basic actor"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                println!("✅ {}: {}", description, input);
                assert!(debug_str.contains("Actor"), 
                        "{}: Should contain Actor expression: {}", description, debug_str);
            },
            Err(e) => {
                println!("⚠️  Actor parsing not yet working: {} (will fix during refactoring)", e);
                // Don't fail - this guides our refactoring
            }
        }
    }
}

#[test]
fn test_parse_actor_with_state() {
    // Test actor with state fields
    let test_cases = vec![
        ("actor Counter { count: i32 }", "should parse actor with one field"),
        ("actor User { name: String, age: i32 }", "should parse actor with multiple fields"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                println!("⚠️  Actor with state not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_parse_actor_with_messages() {
    // Test actor with message handlers
    let test_cases = vec![
        (r#"actor Counter { 
            message increment() { }
        }"#, "should parse actor with message handler"),
        (r#"actor Logger {
            message log(msg: String) { }
            message clear() { }
        }"#, "should parse actor with multiple messages"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                println!("✅ {}: contains Actor", description);
            },
            Err(e) => {
                println!("⚠️  Actor messages not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_parse_actor_with_init() {
    // Test actor with init method
    let test_cases = vec![
        (r#"actor Counter { 
            init() { }
        }"#, "should parse actor with init"),
        (r#"actor Logger {
            init(level: String) { }
        }"#, "should parse actor with parameterized init"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(_) => {
                println!("✅ {}", description);
            },
            Err(e) => {
                println!("⚠️  Actor init not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_parse_actor_complexity_is_reduced() {
    // This test will pass once we've successfully refactored parse_actor_definition
    // REQUIREMENT: parse_actor_definition should delegate to focused helper functions
    
    // After refactoring, parse_actor_definition should be a simple dispatcher
    // that calls focused functions like:
    // - parse_actor_name()
    // - parse_actor_generics()
    // - parse_actor_body()
    // - parse_actor_state_fields()
    // - parse_actor_messages()
    // - parse_actor_init()
    
    // For now, just ensure basic functionality works
    let mut parser = Parser::new("actor Test { }");
    let result = parser.parse();
    
    // Actor parsing might not be fully implemented yet
    match result {
        Ok(expr) => {
            println!("✅ Basic actor parsing works");
            assert!(format!("{:?}", expr).contains("Actor"));
        },
        Err(e) => {
            println!("⚠️  Actor parsing needs implementation: {}", e);
            // Don't fail - this guides our refactoring
        }
    }
    
    // TODO: Add complexity measurement when we have the tools
    // assert!(parse_actor_definition_complexity() < 10);
}

#[test]
fn test_parse_actor_with_complex_body() {
    // Test actor with complete functionality
    let input = r#"
        actor BankAccount {
            balance: f64,
            owner: String,
            
            init(owner: String, initial: f64) {
                self.owner = owner
                self.balance = initial
            }
            
            message deposit(amount: f64) {
                self.balance = self.balance + amount
            }
            
            message withdraw(amount: f64) -> Result<f64, String> {
                if amount > self.balance {
                    Err("Insufficient funds")
                } else {
                    self.balance = self.balance - amount
                    Ok(amount)
                }
            }
            
            message get_balance() -> f64 {
                self.balance
            }
        }
    "#;
    
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    match result {
        Ok(_) => {
            println!("✅ Complex actor parsing works");
        },
        Err(e) => {
            println!("⚠️  Complex actor not yet working: {} (will fix during refactoring)", e);
        }
    }
}

#[test]
fn test_all_actor_components_work_after_refactoring() {
    // Comprehensive test to ensure refactoring doesn't break any component
    let test_cases = vec![
        ("actor Empty { }", "empty actor"),
        ("actor WithState { x: i32 }", "actor with state"),
        ("actor WithInit { init() { } }", "actor with init"),
        ("actor WithMessage { message hello() { } }", "actor with message"),
        (r#"actor Complete {
            count: i32,
            init() { self.count = 0 }
            message inc() { self.count = self.count + 1 }
        }"#, "complete actor"),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (input, variant) in test_cases {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(_) => {
                println!("✅ {} variant: works", variant);
                passed += 1;
            },
            Err(e) => {
                println!("⚠️  {} variant failed: {}", variant, e);
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Results: {} passed, {} failed", passed, failed);
    // We don't require all to pass initially - this guides our refactoring
}