fn main() {
    // Test what token "df" produces
    let test_cases = vec![
        "df",
        "df!",
        "df![",
        "dataframe",
    ];
    
    for test in test_cases {
        println!("Testing: '{}'", test);
        let mut parser = ruchy::Parser::new(test);
        match parser.parse() {
            Ok(ast) => println!("  Parsed OK: {:?}", ast),
            Err(e) => println!("  Parse error: {}", e),
        }
        println!();
    }
}