fn add(x: i32, y: i32) -> i32 {
    return x + y;
}
fn greet(name: &str, age: i32) -> i32 {
    println!("{:?}", name);
    return age + 1;
}
fn is_positive(n: i32) -> bool {
    return n > 0;
}
fn main() {
    let sum = add(10, 20);
    let next_age = greet("Bob", 25);
    let check = is_positive(-5);
    println!("{:?}", sum);
    println!("{:?}", next_age);
    println!("{:?}", check);
}
