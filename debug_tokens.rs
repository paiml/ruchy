use ruchy::frontend::lexer::*;
use logos::Logos;

fn main() {
    let input = "myactor ? request";
    let mut lex = Token::lexer(input);
    
    while let Some(token) = lex.next() {
        match token {
            Ok(t) => println!("Token: {:?} at span {:?}", t, lex.span()),
            Err(_) => println!("Error token at span {:?}", lex.span()),
        }
    }
}