//! Examples demonstrating actor system functionality in Ruchy
//!
//! Run with: cargo run --example actors
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::ExprKind;
use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Ruchy Actor System Examples ===\n");

    // Example 1: Simple actor
    example_simple_actor();

    // Example 2: Actor with state
    example_stateful_actor();

    // Example 3: Message passing
    example_message_passing();

    // Example 4: Complex actor patterns
    example_complex_actor();
}

fn example_simple_actor() {
    println!("1. Simple Actor");
    println!("---------------");

    let input = r#"
        actor Greeter {
            receive {
                SayHello => { "Hello, World!" }
                SayGoodbye => { "Goodbye!" }
            }
        }
    "#;
    println!("Input: {}", &input[..60]);
    println!("...");

    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();

    let output_str = output.to_string();
    if output_str.contains("struct Greeter") && output_str.contains("enum GreeterMessage") {
        println!("✓ Actor transpiled to struct and message enum");
    }
    if output_str.contains("async fn handle_message") {
        println!("✓ Contains async message handler");
    }
    println!();
}

fn example_stateful_actor() {
    println!("2. Stateful Actor");
    println!("-----------------");

    let input = r"
        actor Counter {
            count: i32,
            
            receive {
                Increment => { 
                    self.count += 1
                    self.count
                }
                Decrement => {
                    self.count -= 1
                    self.count
                }
                Get => { self.count }
            }
        }
    ";
    println!("Actor with mutable state: Counter");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Actor {
                name,
                state,
                handlers,
            } = &ast.kind
            {
                println!("  Name: {name}");
                println!("  State fields: {}", state.len());
                println!("  Message handlers: {}", handlers.len());

                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(_) => println!("  ✓ Successfully transpiled"),
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
        }
        Err(e) => println!("  ✗ Parse error: {e}"),
    }
    println!();
}

fn example_message_passing() {
    println!("3. Message Passing");
    println!("------------------");

    let examples = vec![
        ("Send message", "counter ! Increment"),
        ("Ask pattern", "counter ? Get"),
        ("Send with data", r#"logger ! Log("Hello")"#),
    ];

    for (description, input) in examples {
        println!("{description}: {input}");

        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        let output = tokens.to_string();
                        if output.contains(".send(") {
                            println!("  ✓ Contains .send() for message passing");
                        } else if output.contains(".ask(") {
                            println!("  ✓ Contains .ask() for request-response");
                        } else {
                            println!("  Output: {}", &output[..output.len().min(60)]);
                        }
                    }
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
            Err(e) => println!("  ✗ Parse error: {e}"),
        }
    }
    println!();
}

fn example_complex_actor() {
    println!("4. Complex Actor Patterns");
    println!("-------------------------");

    let input = r"
        actor TaskManager {
            tasks: Vec<Task>,
            next_id: i32,
            
            receive {
                AddTask(description: String) => {
                    let task = Task::new(self.next_id, description)
                    self.tasks.push(task)
                    self.next_id += 1
                    self.next_id - 1
                }
                
                CompleteTask(id: i32) => {
                    self.tasks.retain(|t| t.id != id)
                    Ok(())
                }
                
                ListTasks => {
                    self.tasks.clone()
                }
            }
        }
    ";

    println!("Complex TaskManager actor with:");
    println!("  - Multiple state fields");
    println!("  - Parameterized messages");
    println!("  - Complex message handling logic");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Actor { handlers, .. } = &ast.kind {
                println!("\nMessage handlers:");
                for handler in handlers {
                    println!(
                        "  - {} with {} params",
                        handler.message_type,
                        handler.params.len()
                    );
                }

                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        let output = tokens.to_string();
                        if output.contains("enum TaskManagerMessage") {
                            println!("\n✓ Generated message enum");
                        }
                        if output.contains("impl TaskManager") {
                            println!("✓ Generated actor implementation");
                        }
                    }
                    Err(e) => println!("\n✗ Transpilation error: {e}"),
                }
            }
        }
        Err(e) => println!("\n✗ Parse error: {e}"),
    }

    println!("\n=== Actor System Examples Complete ===");
}
