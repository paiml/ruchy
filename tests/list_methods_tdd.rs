// TDD tests for additional list methods
// This captures the requirement for comprehensive list manipulation

use ruchy::runtime::repl::Repl;

#[test]
fn test_list_find_method() {
    let mut repl = Repl::new().unwrap();
    
    // Find first element matching predicate
    let result = repl.eval("[1, 2, 3, 4, 5].find(x => x > 3)").unwrap();
    assert_eq!(result, "Option::Some(4)");
    
    // Find returns None when no match
    let result = repl.eval("[1, 2, 3].find(x => x > 10)").unwrap();
    assert_eq!(result, "Option::None");
}

#[test]
fn test_list_any_method() {
    let mut repl = Repl::new().unwrap();
    
    // Any returns true if any element matches
    let result = repl.eval("[1, 2, 3, 4].any(x => x > 3)").unwrap();
    assert_eq!(result, "true");
    
    // Any returns false if no element matches
    let result = repl.eval("[1, 2, 3].any(x => x > 10)").unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_list_all_method() {
    let mut repl = Repl::new().unwrap();
    
    // All returns true if all elements match
    let result = repl.eval("[2, 4, 6].all(x => x % 2 == 0)").unwrap();
    assert_eq!(result, "true");
    
    // All returns false if any element doesn't match
    let result = repl.eval("[2, 3, 4].all(x => x % 2 == 0)").unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_list_sum_method() {
    let mut repl = Repl::new().unwrap();
    
    // Sum integers
    let result = repl.eval("[1, 2, 3, 4].sum()").unwrap();
    assert_eq!(result, "10");
    
    // Sum floats
    let result = repl.eval("[1.5, 2.5, 3.0].sum()").unwrap();
    assert_eq!(result, "7");
}

#[test]
fn test_list_product_method() {
    let mut repl = Repl::new().unwrap();
    
    // Product of integers
    let result = repl.eval("[2, 3, 4].product()").unwrap();
    assert_eq!(result, "24");
    
    // Product with zero
    let result = repl.eval("[1, 0, 3].product()").unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_list_min_max_methods() {
    let mut repl = Repl::new().unwrap();
    
    // Min method
    let result = repl.eval("[3, 1, 4, 1, 5].min()").unwrap();
    assert_eq!(result, "Option::Some(1)");
    
    // Max method
    let result = repl.eval("[3, 1, 4, 1, 5].max()").unwrap();
    assert_eq!(result, "Option::Some(5)");
    
    // Empty list returns None
    let result = repl.eval("[].min()").unwrap();
    assert_eq!(result, "Option::None");
}

#[test]
fn test_list_unique_method() {
    let mut repl = Repl::new().unwrap();
    
    // Remove duplicates
    let result = repl.eval("[1, 2, 2, 3, 3, 3, 4].unique()").unwrap();
    assert_eq!(result, "[1, 2, 3, 4]");
    
    // Already unique
    let result = repl.eval("[1, 2, 3].unique()").unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_list_flatten_method() {
    let mut repl = Repl::new().unwrap();
    
    // Flatten nested lists
    let result = repl.eval("[[1, 2], [3, 4], [5]].flatten()").unwrap();
    assert_eq!(result, "[1, 2, 3, 4, 5]");
    
    // Mixed depths (only flattens one level)
    let result = repl.eval("[[1], [[2]], [3]].flatten()").unwrap();
    assert_eq!(result, "[1, [2], 3]");
}

#[test]
fn test_list_take_drop_methods() {
    let mut repl = Repl::new().unwrap();
    
    // Take first n elements
    let result = repl.eval("[1, 2, 3, 4, 5].take(3)").unwrap();
    assert_eq!(result, "[1, 2, 3]");
    
    // Drop first n elements
    let result = repl.eval("[1, 2, 3, 4, 5].drop(2)").unwrap();
    assert_eq!(result, "[3, 4, 5]");
    
    // Take more than available
    let result = repl.eval("[1, 2].take(5)").unwrap();
    assert_eq!(result, "[1, 2]");
}