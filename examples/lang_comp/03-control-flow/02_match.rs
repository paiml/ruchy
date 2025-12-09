fn main() {
    let number = 2;
    let description = match number {
        1 => "one",
        2 => "two",
        3 => "three",
        _ => "other",
    };
    println!("Number {} is {}", number, description);
    let day = 3;
    let day_type = match day {
        1 | 2 | 3 | 4 | 5 => "weekday",
        6 | 7 => "weekend",
        _ => "invalid",
    };
    println!("Day {} is a {}", day, day_type);
}
