use ruchy::compile;

fn main() {
    let source = "df![col1 => [1, 2, 3]]";
    match compile(source) {
        Ok(rust_code) => println!("Success:\n{}", rust_code),
        Err(e) => println!("Error: {}", e),
    }
}