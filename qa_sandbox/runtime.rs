fn fib(n: i32) -> i32 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}
fn main() {
    print!("Fib 10: {}", fib(10));
    let outer = "outer";
    {
        {
            let outer = "inner";
            print!("Inside: {}", outer)
        }
    }
    print!("Outside: {}", outer);
}
