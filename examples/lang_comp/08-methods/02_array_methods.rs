fn main() {
    let numbers = [1, 2, 3, 4, 5];
    let len = numbers.len();
    let first = numbers.first();
    let last = numbers.last();
    let has_three = numbers.contains(&3);
    let has_ten = numbers.contains(&10);
    println!("{:?}", numbers);
    println!("{:?}", len);
    println!("{:?}", first);
    println!("{:?}", last);
    println!("{:?}", has_three);
    println!("{:?}", has_ten);
}
