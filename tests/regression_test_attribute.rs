// Regression test for P0 bug #10: test attribute compilation failure
// https://github.com/paiml/ruchy/issues/10

#[test]
fn test_attribute_preservation_in_compilation() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let source = r#"
        #[test]
        fun test_simple() {
            assert_eq!(2, 2)
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");

    let generated = rust_code.to_string();

    // Verify test attribute is preserved (with or without spaces)
    assert!(
        generated.contains("# [test]") || generated.contains("#[test]"),
        "Test attribute was not preserved in transpilation. Generated: {}",
        generated
    );

    // Verify no return type for test functions
    assert!(
        !generated.contains("-> i32"),
        "Test function should not have return type"
    );

    // Verify assert_eq is present
    assert!(
        generated.contains("assert_eq"),
        "assert_eq macro should be present"
    );
}

#[test]
fn test_multiple_attributes() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let source = r#"
        #[test]
        #[ignore]
        fun test_with_multiple_attrs() {
            assert!(true)
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");

    let generated = rust_code.to_string();

    // Both attributes should be preserved (with or without spaces)
    assert!(
        generated.contains("# [test]") || generated.contains("#[test]"),
        "Test attribute was not preserved. Generated: {}",
        generated
    );
    assert!(
        generated.contains("# [ignore]") || generated.contains("#[ignore]"),
        "Ignore attribute was not preserved. Generated: {}",
        generated
    );
}

#[test]
fn test_test_attribute_with_main_function() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let source = r#"
        #[test]
        fun test_something() {
            assert_eq!(1, 1)
        }

        fun main() {
            println!("main function")
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");

    let generated = rust_code.to_string();

    // Test attribute should be preserved (with or without spaces)
    assert!(
        generated.contains("# [test]") || generated.contains("#[test]"),
        "Test attribute was not preserved. Generated: {}",
        generated
    );

    // Main function should exist
    assert!(
        generated.contains("fn main"),
        "Main function should be present. Generated: {}",
        generated
    );
}

#[test]
fn test_no_imports_preserves_attributes() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    // This test ensures that when there are no imports,
    // the optimization to skip module resolution still preserves attributes
    let source = r#"
        #[derive(Debug)]
        struct TestStruct {
            field: i32
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");

    let generated = rust_code.to_string();

    // Derive attribute should be preserved (with or without spaces)
    assert!(
        generated.contains("# [derive") || generated.contains("#[derive"),
        "Derive attribute was not preserved when no imports present. Generated: {}",
        generated
    );
}
