//! Function call examples for Ruchy REPL
//!
//! This example demonstrates various function call patterns
//! that are supported in the Ruchy REPL.

#![allow(clippy::print_stdout)] // Examples need to print
#![allow(clippy::uninlined_format_args)] // Examples for clarity

use ruchy::runtime::Repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Ruchy Function Call Examples");
    println!("{}", "=".repeat(50));

    let mut repl = Repl::new()?;

    // Basic println examples
    println!("\n📝 Basic println Examples:");

    println!("1. Simple string:");
    repl.eval(r#"println("Hello, World!")"#)?;

    println!("2. Multiple arguments:");
    repl.eval(r#"println("Hello", "World", "!")"#)?;

    println!("3. Different types:");
    repl.eval("println(42, 3.14, true, \"mixed\")")?;

    // Variables in function calls
    println!("\n🔗 Variables in Function Calls:");

    repl.eval("let name = \"Alice\"")?;
    repl.eval("let age = 30")?;
    repl.eval("let score = 95.5")?;

    println!("4. Variables as arguments:");
    repl.eval(r#"println("Name:", name, "Age:", age, "Score:", score)"#)?;

    // Expressions in function calls
    println!("\n🧮 Expressions in Function Calls:");

    println!("5. Arithmetic expressions:");
    repl.eval("println(\"Sum:\", 10 + 20)")?;
    repl.eval("println(\"Product:\", 6 * 7)")?;

    println!("6. Complex expressions:");
    repl.eval("println(\"Result:\", (5 + 3) * 2 - 1)")?;

    // String formatting patterns
    println!("\n📋 String Formatting Patterns:");

    println!("7. Formatted output:");
    repl.eval("let x = 42")?;
    repl.eval("let y = 58")?;
    repl.eval(r#"println("x =", x, ", y =", y, ", x + y =", x + y)"#)?;

    // Print vs println
    println!("\n🖨️  Print vs Println:");

    println!("8. Print without newline:");
    repl.eval(r#"print("A")"#)?;
    repl.eval(r#"print("B")"#)?;
    repl.eval(r#"println("C")"#)?;

    println!("9. Building output incrementally:");
    repl.eval(r#"print("Result: ")"#)?;
    repl.eval("print(2 + 2)")?;
    repl.eval(r#"println(" (calculated)")"#)?;

    // Function calls in control flow
    println!("\n🔄 Function Calls in Control Flow:");

    println!("10. Function calls in if expressions:");
    repl.eval(r#"if true { println("True branch") } else { println("False branch") }"#)?;

    println!("11. Function calls with boolean logic:");
    repl.eval("let debug = true")?;
    repl.eval(r#"if debug { println("Debug: Variables loaded successfully") }"#)?;

    // Error handling examples
    println!("\n⚠️  Error Handling:");

    println!("12. Unknown function (should error):");
    match repl.eval("unknown_function()") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   ✅ Correctly errored: {e}"),
    }

    println!("\n🎉 Function call examples completed!");
    println!("All function calls returned unit type '()' as expected.");

    Ok(())
}
