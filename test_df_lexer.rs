use ruchy::frontend::lexer::{Lexer, Token};

fn main() {
    let input = "df![col1 => [1, 2, 3]]";
    println!("Lexing: {}", input);
    let lexer = Lexer::new(input);
    for (token, _span) in lexer {
        println!("  {:?}", token);
    }
}