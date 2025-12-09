fn main() {
    let mut count = 1;
    println!("Counting with while:");
    while count <= 5 {
        {
            println!("  {}", count);
            count = count + 1;
        }
    }
    let mut sum = 0;
    let mut n = 1;
    println!("Sum 1 to 10:");
    while n <= 10 {
        {
            sum = sum + n;
            n = n + 1;
        }
    }
    println!("  Sum: {}", sum);
}
