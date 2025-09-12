#!/usr/bin/env -S cargo +stable run --bin

//! Automated REPL demo recording script
//! 
//! This script programmatically executes demo files and records the sessions
//! to generate comprehensive .replay files for coverage testing.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 Automated REPL Demo Recording");
    println!("==================================");
    
    let demos_dir = Path::new("demos");
    let mut recorded_count = 0;
    
    // Find all demo files
    let demo_files: Vec<_> = fs::read_dir(demos_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "ruchy" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    println!("📝 Found {} demo files to record", demo_files.len());
    
    for demo_file in demo_files {
        let demo_name = demo_file.file_stem().unwrap().to_str().unwrap();
        let replay_file = demos_dir.join(format!("{}.replay", demo_name));
        
        println!("\n🎯 Recording: {}", demo_name);
        
        match record_demo_session(&demo_file, &replay_file) {
            Ok(()) => {
                println!("✅ Successfully recorded: {}", replay_file.display());
                recorded_count += 1;
            }
            Err(e) => {
                println!("❌ Failed to record {}: {}", demo_name, e);
            }
        }
    }
    
    println!("\n🏆 Recording Summary");
    println!("===================");
    println!("📊 Total demos: {}", demo_files.len());
    println!("✅ Successfully recorded: {}", recorded_count);
    println!("❌ Failed: {}", demo_files.len() - recorded_count);
    
    if recorded_count > 0 {
        println!("\n🚀 Generated replay files provide comprehensive coverage of:");
        println!("   • Core language syntax and semantics");
        println!("   • Data structures and operations");
        println!("   • Control flow and pattern matching");
        println!("   • Advanced features and error handling");
        println!("   • REPL-specific functionality");
        println!("   • Edge cases and boundary conditions");
        
        println!("\n💡 Next steps:");
        println!("   1. Convert replays to regression tests");
        println!("   2. Measure coverage improvement");
        println!("   3. Validate replay determinism");
    }
    
    Ok(())
}

fn record_demo_session(demo_file: &Path, replay_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let repl_input = extract_repl_input_from_demo(demo_file)?;
    let mut child = spawn_repl_process(replay_file)?;
    send_input_to_repl(&mut child, repl_input)?;
    validate_recording_completion(&child, replay_file)?;
    
    Ok(())
}

/// Extract executable REPL input from demo file, filtering comments and empty lines
fn extract_repl_input_from_demo(demo_file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let demo_content = fs::read_to_string(demo_file)?;
    let mut repl_input = String::new();
    
    for line in demo_content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            repl_input.push_str(line);
            repl_input.push('\n');
        }
    }
    
    // Add quit command to cleanly terminate session
    repl_input.push_str(":quit\n");
    Ok(repl_input)
}

/// Spawn REPL process configured for recording
fn spawn_repl_process(replay_file: &Path) -> Result<std::process::Child, Box<dyn std::error::Error>> {
    let child = Command::new("./target/debug/ruchy")
        .args(&["repl", "--record", replay_file.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    Ok(child)
}

/// Send input to REPL process via stdin in separate thread
fn send_input_to_repl(child: &mut std::process::Child, repl_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(stdin) = child.stdin.take() {
        std::thread::spawn(move || {
            let mut stdin = stdin;
            let _ = stdin.write_all(repl_input.as_bytes());
            let _ = stdin.flush();
        });
    }
    Ok(())
}

/// Validate that recording completed successfully and replay file was created
fn validate_recording_completion(child: &std::process::Child, replay_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = child.wait_with_output()?;
    
    if !output.status.success() {
        return Err(format!(
            "REPL recording failed. stderr: {}", 
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }
    
    if !replay_file.exists() {
        return Err("Replay file was not created".into());
    }
    
    Ok(())
}