#[cfg(test)]
mod binary_compilation_book_tdd {
    use std::process::Command;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_compile_simple_hello_world() {
        // RED: Test that ruchy compile works for simple hello world
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let source_file = temp_dir.path().join("hello.ruchy");
        let output_file = temp_dir.path().join("hello");
        
        // Write simple hello world program
        fs::write(&source_file, r#"
            fun main() {
                println("Hello from compiled Ruchy!");
            }
        "#).expect("Failed to write source file");
        
        // Compile the program
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", "compile", 
                    source_file.to_str().unwrap(), 
                    "-o", output_file.to_str().unwrap()])
            .output()
            .expect("Failed to run ruchy compile");
        
        // RED: This should fail if compile command doesn't work
        assert!(output.status.success(), 
            "Compilation should succeed, stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        // Check binary was created
        assert!(output_file.exists(), "Binary should be created at {:?}", output_file);
        
        // Run the compiled binary
        let run_output = Command::new(&output_file)
            .output()
            .expect("Failed to run compiled binary");
        
        assert!(run_output.status.success(), "Binary should run successfully");
        assert_eq!(
            String::from_utf8_lossy(&run_output.stdout).trim(),
            "Hello from compiled Ruchy!",
            "Binary should print expected output"
        );
    }
    
    #[test]
    fn test_compile_with_arguments() {
        // RED: Test that compiled programs can use command-line arguments
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let source_file = temp_dir.path().join("args.ruchy");
        let output_file = temp_dir.path().join("args");
        
        // Write program that uses arguments
        fs::write(&source_file, r#"
            fun main() {
                let args = std::env::args();
                if args.len() < 2 {
                    println("Usage: args <name>");
                } else {
                    println("Hello, {}!", args[1]);
                }
            }
        "#).expect("Failed to write source file");
        
        // Compile the program
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", "compile",
                    source_file.to_str().unwrap(),
                    "-o", output_file.to_str().unwrap()])
            .output()
            .expect("Failed to run ruchy compile");
        
        // RED: This should fail if std::env::args() doesn't work
        assert!(output.status.success(),
            "Compilation should succeed, stderr: {}",
            String::from_utf8_lossy(&output.stderr));
        
        // Run without arguments
        let run_output = Command::new(&output_file)
            .output()
            .expect("Failed to run compiled binary");
        
        assert!(String::from_utf8_lossy(&run_output.stdout).contains("Usage"),
            "Should show usage message when no args");
        
        // Run with arguments
        let run_output = Command::new(&output_file)
            .arg("World")
            .output()
            .expect("Failed to run compiled binary");
        
        assert_eq!(
            String::from_utf8_lossy(&run_output.stdout).trim(),
            "Hello, World!",
            "Should greet with provided name"
        );
    }
    
    #[test]
    fn test_compile_default_output_name() {
        // RED: Test that compile generates a.out by default
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let source_file = temp_dir.path().join("test.ruchy");
        
        fs::write(&source_file, r#"
            fun main() {
                println("Default output");
            }
        "#).expect("Failed to write source file");
        
        // Change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        // Compile without -o flag
        let output = Command::new(original_dir.join("target/debug/ruchy"))
            .args(&["compile", source_file.file_name().unwrap().to_str().unwrap()])
            .output()
            .expect("Failed to run ruchy compile");
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        // RED: This should fail if default output isn't created
        assert!(output.status.success(),
            "Compilation should succeed, stderr: {}",
            String::from_utf8_lossy(&output.stderr));
        
        // Check a.out was created
        let default_output = temp_dir.path().join("a.out");
        assert!(default_output.exists(), "Default a.out should be created");
    }
    
    #[test]
    #[ignore] // Mark as integration test
    fn test_compile_shows_binary_size() {
        // RED: Test that compile command shows binary size
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let source_file = temp_dir.path().join("size.ruchy");
        
        fs::write(&source_file, r#"
            fun main() {
                println("Size test");
            }
        "#).expect("Failed to write source file");
        
        // Compile and capture output
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", "compile",
                    source_file.to_str().unwrap()])
            .output()
            .expect("Failed to run ruchy compile");
        
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        
        // RED: This should fail if size info isn't shown
        assert!(stderr_str.contains("Binary size") || stderr_str.contains("bytes"),
            "Should show binary size information, got: {}", stderr_str);
    }
}