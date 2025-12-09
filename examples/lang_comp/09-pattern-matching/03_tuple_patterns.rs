fn main() {
    let point = (10, 20);
    let location = match point {
        (0, 0) => "origin",
        (x, 0) => "on x-axis",
        (0, y) => "on y-axis",
        (x, y) => "in quadrant",
    };
    println!("{:?}", location);
    let pair = (42, "answer");
    match pair {
        (num, text) => {
            println!("{:?}", num);
            println!("{:?}", text)
        }
    };
    let nested = ((1, 2), (3, 4));
    let result = match nested {
        ((a, b), (c, d)) => {
            println!("{:?}", a);
            println!("{:?}", b);
            println!("{:?}", c);
            println!("{:?}", d)
        }
    };
    if std::any::type_name_of_val(&result) == "()" {} else {
        println!("{:?}", result);
    }
}
