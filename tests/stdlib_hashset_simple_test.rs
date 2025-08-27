// STDLIB-005: HashSet Operations Test Suite (Simplified)

use ruchy::runtime::repl::Repl;

fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {:?}", e))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {:?}", e))?;
    
    Ok(result)
}

#[test]
fn test_hashset_operations_exist() {
    // Test that HashSet operations exist and don't error
    
    // Test union
    let result = eval_in_repl("let s1 = HashSet(); let s2 = HashSet(); s1.union(s2)");
    assert!(result.is_ok(), "Union should not error: {:?}", result);
    
    // Test intersection  
    let result = eval_in_repl("let s1 = HashSet(); let s2 = HashSet(); s1.intersection(s2)");
    assert!(result.is_ok(), "Intersection should not error: {:?}", result);
    
    // Test difference
    let result = eval_in_repl("let s1 = HashSet(); let s2 = HashSet(); s1.difference(s2)");
    assert!(result.is_ok(), "Difference should not error: {:?}", result);
    
    // Test that methods return HashSet
    let result = eval_in_repl("let s1 = HashSet(); let s2 = HashSet(); s1.union(s2)").unwrap();
    assert!(result.contains("HashSet"));
}

#[test] 
fn test_hashset_basic_functionality() {
    // Test basic HashSet functionality works
    let result = eval_in_repl("let s = HashSet(); s").unwrap();
    assert!(result.contains("HashSet"));
    
    let result = eval_in_repl("let s = HashSet(); s.insert(42)").unwrap();
    assert!(result.contains("42"));
}