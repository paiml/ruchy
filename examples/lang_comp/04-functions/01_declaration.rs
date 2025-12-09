fn greet() {
    println!("Hello from Ruchy!")
}
fn get_answer() -> i32 {
    return 42;
}
fn main() {
    greet();
    let answer = get_answer();
    println!("The answer is {}", answer);
}
