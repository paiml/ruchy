// TDD: Test the actual println output issue
use std::process::Command;
use std::fs;

#[test]
fn test_while_loop_println_count() {
    // Create test file
    let code = r"let i = 0
while i < 3 {
    println(i)
    i = i + 1
}";
    
    fs::write("/tmp/test_while_println.ruchy", code).unwrap();
    
    // Run and capture output
    let output = Command::new("ruchy")
        .arg("/tmp/test_while_println.ruchy")
        .output()
        .expect("Failed to run ruchy");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    // Should print exactly 3 lines: 0, 1, 2
    assert_eq!(lines.len(), 3, 
        "Should print exactly 3 lines, got: {lines:?}");
    assert_eq!(lines[0], "0");
    assert_eq!(lines[1], "1");
    assert_eq!(lines[2], "2");
    // Should NOT have a 4th line with "3"
}

#[test]
fn test_while_loop_println_boundary() {
    // Test with different boundary
    let code = r"let i = 5
while i < 8 {
    println(i)
    i = i + 1
}";
    
    fs::write("/tmp/test_while_boundary.ruchy", code).unwrap();
    
    let output = Command::new("ruchy")
        .arg("/tmp/test_while_boundary.ruchy")
        .output()
        .expect("Failed to run ruchy");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    assert_eq!(lines.len(), 3, "Should print 5, 6, 7 only");
    assert_eq!(lines[0], "5");
    assert_eq!(lines[1], "6");
    assert_eq!(lines[2], "7");
}