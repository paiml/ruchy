// TDD: While loop boundary condition test
use ruchy::runtime::Repl;

#[test]
fn test_while_loop_stops_at_boundary() {
    // While i < 3 should print 0, 1, 2 (NOT 3)
    let code = r"
        let i = 0
        let result = []
        while i < 3 {
            result = result.push(i)
            i = i + 1
        }
        result
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    
    // EXPECTED: [0, 1, 2]
    // ACTUAL BUG: Currently might include 3
    assert_eq!(result, "[0, 1, 2]", 
        "While i < 3 should stop when i reaches 3, not include it");
}

#[test]
fn test_while_loop_counter_after_loop() {
    let code = r"
        let i = 0
        while i < 3 {
            i = i + 1
        }
        i
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result, "3", 
        "Counter should be 3 after loop completes");
}

#[test]
fn test_while_loop_executes_correct_iterations() {
    let code = r"
        let count = 0
        let i = 0
        while i < 3 {
            count = count + 1
            i = i + 1
        }
        count
    ";
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    assert_eq!(result, "3", 
        "While loop should execute exactly 3 times");
}