fn main() {
    let numbers = [1, 2, 3, 4, 5];
    let strings = ["one", "two", "three"];
    let first = numbers[0 as usize].clone();
    let last = numbers[4 as usize].clone();
    let second_str = strings[1 as usize].clone();
    let matrix = [[1, 2], [3, 4]];
    let nested_val = matrix[0 as usize].clone()[1 as usize].clone();
    println!("{:?}", numbers);
    println!("{:?}", first);
    println!("{:?}", last);
    println!("{:?}", second_str);
    println!("{:?}", matrix);
    println!("{:?}", nested_val);
}
