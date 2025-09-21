use ruchy::compile;

#[test]
fn test_import_std() {
    let code = "import std";
    let result = compile(code);
    match result {
        Ok(output) => {
            println!("Output: {}", output);
            assert!(
                output.contains("use std;") || output.contains("use std ::"),
                "Expected 'use std;' in output, got: {}",
                output
            );
        }
        Err(e) => panic!("Failed to compile: {}", e),
    }
}

#[test]
fn test_import_std_collections() {
    let code = "import std.collections.HashMap";
    let result = compile(code);
    match result {
        Ok(output) => {
            println!("Output: {}", output);
            assert!(
                output.contains("use std::collections::HashMap"),
                "Expected 'use std::collections::HashMap' in output, got: {}",
                output
            );
        }
        Err(e) => panic!("Failed to compile: {}", e),
    }
}
