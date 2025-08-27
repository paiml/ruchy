// TDD: Test list mutation behavior
use ruchy::runtime::Repl;

#[test]
fn test_list_push_returns_new_list() {
    // push() returns a new list, doesn't mutate
    let code = r#"
        let list1 = [1, 2]
        let list2 = list1.push(3)
        list2
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result.to_string(), "[1, 2, 3]", "push should return new list with item");
}

#[test]
fn test_list_push_doesnt_mutate_original() {
    let code = r#"
        let list1 = [1, 2]
        let list2 = list1.push(3)
        list1
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result.to_string(), "[1, 2]", "Original list should be unchanged");
}

#[test]
fn test_list_push_with_reassignment() {
    // Must reassign to "mutate"
    let code = r#"
        let result = []
        result = result.push(1)
        result = result.push(2)
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result.to_string(), "[1, 2]", "Reassignment should work");
}

#[test]
fn test_while_loop_with_reassignment() {
    // While loop with proper reassignment
    let code = r#"
        let i = 0
        let result = []
        while i < 3 {
            result = result.push(i)
            i = i + 1
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result.to_string(), "[0, 1, 2]", 
        "While loop with reassignment should work");
}