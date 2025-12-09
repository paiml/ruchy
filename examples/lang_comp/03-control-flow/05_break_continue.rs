fn main() {
    println!("Finding first multiple of 7 after 20:");
    let mut i = 20;
    while true {
        {
            if i % 7 == 0 {
                println!("Found: {}", i);
                break;
            }
            i = i + 1;
        }
    }
    println!("Even numbers 1-10:");
    for n in 1..11 {
        {
            if n % 2 != 0 {
                continue;
            }
            println!("  {}", n)
        }
    }
}
