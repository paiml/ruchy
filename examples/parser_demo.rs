use ruchy::Parser;

fn main() {
    let examples = [
        // Simple arithmetic
        "1 + 2 * 3",
        // Function definition
        "fun add(x: i32, y: i32) -> i32 { x + y }",
        // If expression
        "if x > 0 { positive } else { negative }",
        // Pipeline
        "[1, 2, 3] |> map(double) |> filter(even)",
        // Let binding
        "let x = 42 in x + 1",
        // Complex expression
        "fun fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }",
    ];

    for (i, input) in examples.iter().enumerate() {
        println!("\n=== Example {} ===", i + 1);
        println!("Input: {}", input);

        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(ast) => {
                println!("✓ Parsed successfully!");
                println!("AST: {:#?}", ast);
            }
            Err(e) => {
                println!("✗ Parse error: {}", e);
            }
        }
    }
}
