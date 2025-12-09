fn double(x: i32) -> i32 {
    x * 2
}
fn add(a: i32, b: i32) -> i32 {
    a + b
}
fn greet_person(name: String) {
    println!("Hello, {}!", name)
}
fn main() {
    println!("double(5) = {}", double(5));
    println!("add(3, 7) = {}", add(3, 7));
    greet_person("Alice".to_string());
}
