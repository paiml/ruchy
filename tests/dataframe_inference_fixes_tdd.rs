// TDD Tests for DataFrame type inference fixes
use ruchy::{Parser, Transpiler};
use ruchy::middleend::infer::InferenceContext;

#[test]
fn test_dataframe_literal_parsing() {
    // Test basic DataFrame literal syntax
    let code = "df![\"name\" => [\"Alice\", \"Bob\"], \"age\" => [25, 30]]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse DataFrame literal");
    
    // Should parse without errors
    assert!(matches!(ast.kind, ruchy::frontend::ast::ExprKind::DataFrame { .. }));
}

#[test] 
fn test_dataframe_type_inference_with_quotes() {
    let code = "df![\"name\" => [\"Alice\", \"Bob\"], \"age\" => [25, 30]]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse DataFrame");
    
    let mut inferencer = InferenceContext::new();
    let result = inferencer.infer(&ast);
    
    assert!(result.is_ok(), "Should infer DataFrame type: {:?}", result.err());
}

#[test]
fn test_dataframe_type_inference_without_quotes() {
    // This is the failing case - identifiers instead of strings
    let code = "df![age => [25, 30], name => [\"Alice\", \"Bob\"]]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse DataFrame");
    
    let mut inferencer = InferenceContext::new();
    let result = inferencer.infer(&ast);
    
    assert!(result.is_ok(), "Should infer DataFrame type: {:?}", result.err());
}

#[test]
fn test_dataframe_operations_inference() {
    let code = "df![\"x\" => [1, 2]].select([\"x\"])";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse DataFrame operations");
    
    let mut inferencer = InferenceContext::new();
    let result = inferencer.infer(&ast);
    
    assert!(result.is_ok(), "Should infer DataFrame operation type: {:?}", result.err());
}

#[test]
fn test_series_type_inference() {
    let code = "[1, 2, 3].sum()";
    let mut parser = Parser::new(code);  
    let ast = parser.parse().expect("Should parse Series operations");
    
    let mut inferencer = InferenceContext::new();
    let result = inferencer.infer(&ast);
    
    assert!(result.is_ok(), "Should infer Series type: {:?}", result.err());
}

#[test]
fn test_actor_compilation() {
    // This is the syntax the test expects
    let code = r"actor Counter {
        count: i32,
        
        receive {
            Inc => 1,
            Get => 0
        }
    }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse actor");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should compile actor: {:?}", result.err());
}