fn main() {
    let pair = (1, 2);
    let triple = (1, "two", 3f64);
    let nested = ((1, 2), (3, 4));
    let first = pair.0;
    let second = pair.1;
    let person = ("Alice", 30, "NYC");
    let name = person.0;
    let age = person.1;
    let city = person.2;
    println!("{:?}", pair);
    println!("{:?}", triple);
    println!("{:?}", first);
    println!("{:?}", name);
    println!("{:?}", age);
}
