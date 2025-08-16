use ruchy::is_valid_syntax;

fn main() {
    println!("fun (): {}", is_valid_syntax("fun ()"));
    println!("fun test(): {}", is_valid_syntax("fun test()"));
    println!("fun test() {{{{ 1 }}}}: {}", is_valid_syntax("fun test() { 1 }"));
}