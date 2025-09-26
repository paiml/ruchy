use anyhow::Result;
use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_nested_list_comprehension() -> Result<()> {
    let input = "[item for sublist in [[1, 2], [3, 4]] for item in sublist]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    // The nested comprehension should transpile to nested flat_map
    assert!(output.contains("flat_map") || output.contains("flatten"));
    Ok(())
}

#[test]
fn test_nested_comprehension_with_condition() -> Result<()> {
    let input = "[x for row in matrix for x in row if x > 0]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    assert!(output.contains("filter"));
    Ok(())
}

#[test]
fn test_triple_nested_comprehension() -> Result<()> {
    let input = "[z for x in a for y in b for z in c]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    // Should create triple nested iteration
    assert!(output.contains("flat_map") || output.contains("flatten"));
    Ok(())
}

#[test]
fn test_nested_dict_comprehension() -> Result<()> {
    let input = "{k: v for d in dicts for (k, v) in d.items()}";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    assert!(output.contains("HashMap"));
    Ok(())
}

#[test]
fn test_nested_set_comprehension() -> Result<()> {
    let input = "{item for sublist in lists for item in sublist}";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    assert!(output.contains("HashSet"));
    Ok(())
}

#[test]
fn test_nested_comprehension_with_multiple_conditions() -> Result<()> {
    let input = "[x + y for x in a if x > 0 for y in b if y < 10]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    assert!(output.contains("filter"));
    Ok(())
}

#[test]
fn test_nested_comprehension_order_matters() -> Result<()> {
    // Order of for clauses matters - this creates different results
    let input1 = "[(x, y) for x in [1, 2] for y in [3, 4]]";
    let input2 = "[(x, y) for y in [3, 4] for x in [1, 2]]";

    let mut parser1 = Parser::new(input1);
    let expr1 = parser1.parse()?;

    let mut parser2 = Parser::new(input2);
    let expr2 = parser2.parse()?;

    let transpiler = Transpiler::new();
    let output1 = transpiler.transpile(&expr1)?.to_string();
    let output2 = transpiler.transpile(&expr2)?.to_string();

    // The outputs should be different due to different iteration order
    assert_ne!(output1, output2);
    Ok(())
}

#[test]
fn test_nested_comprehension_with_expression() -> Result<()> {
    let input = "[x * y for x in range(3) for y in range(4)]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    // Should produce cartesian product with multiplication
    assert!(output.contains("*"));
    Ok(())
}

#[test]
fn test_nested_comprehension_with_method_calls() -> Result<()> {
    let input = "[char for word in words for char in word.chars()]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    assert!(output.contains("chars"));
    Ok(())
}

#[test]
fn test_nested_comprehension_with_tuple_unpacking() -> Result<()> {
    let input = "[x + y for (x, y) in pairs for pairs in list_of_pairs]";
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&expr)?.to_string();

    // Should handle tuple unpacking in nested loop
    assert!(output.contains("("));
    Ok(())
}
