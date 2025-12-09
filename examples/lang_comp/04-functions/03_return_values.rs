fn square(x: i32) -> i32 {
    x * x
}
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    }
    return b;
}
fn main() {
    println!("square(4) = {}", square(4));
    println!("max(10, 7) = {}", std::cmp::max(10, 7));
}
