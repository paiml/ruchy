fn main() {
    let coordinates = (100, 200);
    let (x, y) = coordinates;
    println!("{:?}", x);
    println!("{:?}", y);
    let nested = ((1, 2), (3, 4));
    let ((a, b), (c, d)) = nested;
    println!("{:?}", a);
    println!("{:?}", d);
    let numbers = [10, 20, 30];
    let first = numbers[0 as usize].clone();
    let second = numbers[1 as usize].clone();
    println!("{:?}", first);
    println!("{:?}", second);
    let point = (5, 10);
    let result = match point {
        (0, 0) => println!("origin"),
        (0, y) => println!("{:?}", y),
        (x, 0) => println!("{:?}", x),
        (x, y) => {
            println!("{:?}", x);
            println!("{:?}", y)
        }
    };
    if std::any::type_name_of_val(&result) == "()" {} else {
        println!("{:?}", result);
    }
}
