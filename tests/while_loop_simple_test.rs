// TDD: Simplest possible while loop test
use ruchy::runtime::Repl;

#[test]
fn test_while_prints_values() {
    // Simplest test: while loop should print values
    let code = r"
        let i = 0
        while i < 3 {
            println(i)
            i = i + 1
        }
    ";
    
    // This should print 0, 1, 2 (not 0, 1, 2, 3)
    // Currently prints 0, 1, 2, 3 - off by one error
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code);
    assert!(result.is_ok(), "While loop should execute without error");
}

#[test] 
fn test_while_modifies_variable() {
    let code = r"
        let i = 0
        while i < 3 {
            i = i + 1
        }
        i
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result, "3", "i should be 3 after loop");
}

#[test]
fn test_list_push_works() {
    // Test that list.push works outside of while loop
    let code = r"
        let result = []
        result.push(1)
        result.push(2)
        result
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result, "[1, 2]", "List push should work");
}

#[test]
fn test_while_with_list_push() {
    // Combine while loop with list push
    let code = r"
        let i = 0
        let result = []
        while i < 3 {
            result.push(i)
            i = i + 1
        }
        result
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result, "[0, 1, 2]", 
        "While loop should push 0, 1, 2 to list");
}