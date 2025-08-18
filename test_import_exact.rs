fn main() {
    let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    
    let test_input = "use std :: collections :: { HashMap as Map } ;";
    let expected = "use std::collections::{HashMap as Map}";
    
    let normalized_input = normalize(test_input);
    let normalized_expected = normalize(expected);
    
    println!("Input normalized: '{}'", normalized_input);
    println!("Expected normalized: '{}'", normalized_expected);
    println!("Contains: {}", normalized_input.contains(&normalized_expected));
    
    // Check character by character
    let input_chars: Vec<char> = normalized_input.chars().collect();
    let expected_chars: Vec<char> = normalized_expected.chars().collect();
    
    for (i, (a, b)) in input_chars.iter().zip(expected_chars.iter()).enumerate() {
        if a != b {
            println!("Difference at position {}: '{}' vs '{}'", i, a, b);
            break;
        }
    }
}