use ruchy::frontend::lexer::TokenStream;

fn main() {
    let code = "import std::collections::{HashMap as Map}";
    let mut tokens = TokenStream::new(code);
    
    println!("Tokens for: {}", code);
    while let Some((token, _)) = tokens.advance() {
        println!("  {:?}", token);
    }
}
