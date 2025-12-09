fn main() {
    let double = move |x| x * 2;
    let add = move |a, b| a + b;
    println!("double(5) = {}", double(5));
    println!("add(3, 7) = {}", add(3, 7));
}
