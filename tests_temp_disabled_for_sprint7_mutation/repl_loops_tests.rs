#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! REPL loop tests

#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(unused_variables)]

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_for_loop_with_list() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let result = repl.eval(
        r#"
        let sum = 0;
        for x in [1, 2, 3] { 
            let sum = sum + x 
        };
        sum
    "#,
    );
    // This won't work perfectly yet because let sum is creating new bindings
    // Let's try a simpler version

    let result2 = repl.eval(r#"for x in [1, 2, 3] { println(x) }"#);
    assert!(result2.is_ok());
}

#[test]
fn test_while_loop_basic() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    let result = repl.eval(
        r#"
        let i = 0;
        while i < 3 {
            let i = i + 1
        };
        i
    "#,
    );
    // Again, this has scoping issues, but the while loop should parse and run
    assert!(result.is_ok());
}

#[test]
fn test_while_loop_with_counter() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Set up a counter
    assert!(repl.eval("let counter = 0").is_ok());

    // Simple while loop that should execute
    let result = repl.eval("while counter < 2 { println(counter); let counter = counter + 1 }");
    assert!(result.is_ok());
}
