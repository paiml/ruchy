fn main() {
    let number = 42;
    let result = match number {
        0 => "zero",
        1 => "one",
        42 => "the answer",
        _ => "something else",
    };
    println!("{:?}", result);
    let status = "success";
    let message = match status {
        "success" => "Operation completed",
        "error" => "Operation failed",
        "pending" => "Operation in progress",
        _ => "Unknown status",
    };
    println!("{:?}", message);
    let flag = 1;
    { println!("enabled") };
}
