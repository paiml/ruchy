// TDD: Test for while loop off-by-one error
// Bug: while i < 3 prints 0,1,2,3 instead of 0,1,2

use std::process::Command;
use std::fs;

#[test]
fn test_while_loop_off_by_one() {
    let code = "let i = 0\nwhile i < 3 {\n    println(i)\n    i = i + 1\n}";
    fs::write("/tmp/test_while_off_by_one.ruchy", code).unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .arg("/tmp/test_while_off_by_one.ruchy")
        .output()
        .expect("Failed to run ruchy");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // EXPECTED: "0\n1\n2\n"
    // ACTUAL BUG: "0\n1\n2\n3\n"
    assert_eq!(stdout, "0\n1\n2\n", 
        "while i < 3 should print 0,1,2 only, got: {stdout:?}");
}

#[test]
fn test_object_items_method() {
    let code = r#"let obj = { "a": 1, "b": 2 }
obj.items()"#;
    fs::write("/tmp/test_object_items.ruchy", code).unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .arg("/tmp/test_object_items.ruchy")
        .output()
        .expect("Failed to run ruchy");
    
    assert!(output.status.success(), "obj.items() should work, got error: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // items() should return list of (key, value) tuples
    assert!(stdout.contains("(\"a\", 1)") && stdout.contains("(\"b\", 2)"),
        "obj.items() should return key-value pairs, got: {stdout:?}");
}