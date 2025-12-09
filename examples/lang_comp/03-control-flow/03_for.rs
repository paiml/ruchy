fn main() {
    println!("Counting 1 to 5:");
    for i in 1..6 {
        { println!("  {}", i) }
    }
    let mut sum = 0;
    for i in 1..11 {
        { sum = sum + i }
    }
    println!("Sum of 1 to 10: {}", sum);
}
