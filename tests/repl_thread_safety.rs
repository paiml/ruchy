//! RED Phase Test for DEFECT-001-A-TICKET-2
//!
//! This test MUST FAIL with Rc-based Values (current state)
//! This test MUST PASS after Arc refactoring (target state)
//!
//! Purpose: Prove that Repl can be shared across threads after Arc refactoring

#![allow(clippy::expect_used)]
#![allow(missing_docs)]


/// Test that Repl is Send (can cross thread boundaries)
///
/// This is the RED phase test - it MUST fail with Rc, MUST pass with Arc.
/// Currently fails due to Rc<`markup5ever_rcdom::Node`> in HTML parsing.
#[test]
#[ignore = "RED phase: Fails due to Rc in markup5ever_rcdom - requires Arc refactoring"]
fn test_repl_is_send() {
    // Compile-time check that would fail with Rc
    #[allow(dead_code)]
    fn assert_send<T: Send>() {}

    // UNCOMMENT WHEN READY FOR ARC REFACTORING:
    // assert_send::<Repl>(); // FAILS with Rc, PASSES with Arc

    // For now, document the requirement:
    println!("RED: Repl is not Send due to Rc in HtmlDocument");
    println!("GREEN: After Arc refactoring, uncomment assert_send::<Repl>()");
}

/// Test that Repl can actually be used across threads
///
/// This proves the practical use case: sharing Repl in Arc<Mutex<Repl>>
#[test]
#[ignore = "RED phase: Cannot compile with Rc-based Values - requires Arc refactoring"]
fn test_repl_shared_across_threads() {
    // UNCOMMENT WHEN READY FOR ARC REFACTORING:
    /*
    use std::sync::{Arc, Mutex};
    use std::thread;

    // Create REPL
    let repl = Repl::new(std::env::current_dir().unwrap()).expect("Failed to create REPL");
    let shared_repl = Arc::new(Mutex::new(repl));

    // Thread 1: Set variable x = 10
    let repl1 = Arc::clone(&shared_repl);
    let handle1 = thread::spawn(move || {
        let mut repl = repl1.lock().unwrap();
        repl.eval("x = 10").expect("Failed to eval")
    });

    let result1 = handle1.join().expect("Thread 1 panicked");
    assert_eq!(result1.trim(), "10");

    // Thread 2: Read variable x (should be 10)
    let repl2 = Arc::clone(&shared_repl);
    let handle2 = thread::spawn(move || {
        let mut repl = repl2.lock().unwrap();
        repl.eval("x * 2").expect("Failed to eval")
    });

    let result2 = handle2.join().expect("Thread 2 panicked");
    assert_eq!(result2.trim(), "20");
    */

    // For now, document the requirement:
    println!("RED: Repl cannot be shared across threads due to Rc in HtmlDocument");
    println!("GREEN: After Arc refactoring, uncomment the thread::spawn code above");
}
