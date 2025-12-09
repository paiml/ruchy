fn main() {
    let text = "  hello world  ";
    let result = text.trim().replace("world", "Ruchy");
    let numbers = [1, 2, 3, 4, 5];
    let first_val = numbers.first();
    let lowered = "LOUD TEXT".to_lowercase();
    let processed = lowered.trim();
    println!("  hello world  ");
    println!("{:?}", result);
    println!("{:?}", first_val);
    println!("{:?}", processed);
}
