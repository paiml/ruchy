// TDD tests for list reduce method
// This captures the requirement for the fundamental reduce operation

use ruchy::runtime::repl::Repl;

#[test]
fn test_list_reduce_sum() {
    let mut repl = Repl::new().unwrap();
    
    // Reduce with sum
    let result = repl.eval("[1, 2, 3, 4].reduce(|acc, x| acc + x, 0)").unwrap();
    assert_eq!(result, "10");
    
    // Reduce with multiplication
    let result = repl.eval("[1, 2, 3, 4].reduce(|acc, x| acc * x, 1)").unwrap();
    assert_eq!(result, "24");
}

#[test]
fn test_list_reduce_strings() {
    let mut repl = Repl::new().unwrap();
    
    // Concatenate strings
    let result = repl.eval(r#"["a", "b", "c"].reduce(|acc, x| acc + x, "")"#).unwrap();
    assert_eq!(result, r#""abc""#);
}

#[test]
fn test_list_reduce_max() {
    let mut repl = Repl::new().unwrap();
    
    // Find maximum using reduce
    let code = r#"
        [3, 1, 4, 1, 5].reduce(|acc, x| {
            if x > acc {
                x
            } else {
                acc
            }
        }, 0)
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_list_reduce_count() {
    let mut repl = Repl::new().unwrap();
    
    // Count elements meeting condition
    let code = r#"
        [1, 2, 3, 4, 5].reduce(|acc, x| {
            if x > 2 {
                acc + 1
            } else {
                acc
            }
        }, 0)
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_list_reduce_empty() {
    let mut repl = Repl::new().unwrap();
    
    // Empty list returns initial value
    let result = repl.eval("[].reduce(|acc, x| acc + x, 42)").unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_list_reduce_nested() {
    let mut repl = Repl::new().unwrap();
    
    // Flatten nested lists with reduce
    let code = r#"
        [[1, 2], [3, 4], [5]].reduce(|acc, x| acc.concat(x), [])
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "[1, 2, 3, 4, 5]");
}