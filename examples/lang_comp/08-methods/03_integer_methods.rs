fn main() {
    let num = 42;
    let as_string = num.to_string();
    let as_float = (num as f64);
    let negative: i32 = -5;
    let abs_val = negative.abs();
    let base: i32 = 2;
    let power = base.pow(3);
    println!("{:?}", 42);
    println!("{:?}", as_string);
    println!("{:?}", as_float);
    println!("{:?}", abs_val);
    println!("{:?}", power);
}
