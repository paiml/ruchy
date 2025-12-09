fn main() {
    let numbers: [i32; 5] = [1, 2, 3, 4, 5];
    let floats: [f64; 3] = [1.1f64, 2.2f64, 3.3f64];
    let pair: (i32, i32) = (10, 20);
    let coords: (f64, f64) = (3.14f64, 2.71f64);
    let first = numbers[0 as usize].clone();
    let x = coords.0;
    let y = coords.1;
    println!("{:?}", numbers);
    println!("{:?}", floats);
    println!("{:?}", pair);
    println!("{:?}", first);
    println!("{:?}", x);
    println!("{:?}", y);
}
