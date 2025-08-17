use ruchy::get_parse_error;

fn main() {
    // Test different error cases
    let test_cases = ["if", "fun (", "let x =", "match", "struct"];
    
    for case in test_cases {
        if let Some(error) = get_parse_error(case) {
            println!("Input: '{}' -> Error: {}", case, error);
            println!("  Contains 'Expected'? {}", error.contains("Expected"));
        } else {
            println!("Input: '{}' -> No error", case);
        }
        println!();
    }
}