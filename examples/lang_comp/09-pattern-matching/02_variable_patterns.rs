fn main() {
    let value = 100;
    let category = match value {
        0 => "zero".to_string(),
        x if x < 10 => "single digit".to_string(),
        x if x < 100 => "double digit".to_string(),
        x => format!("large number: {}", x),
    };
    println!("{:?}", category);
    let status_code = 404;
    let response = match status_code {
        200 => "OK",
        404 => "Not Found",
        500 => "Server Error",
        _ => "Unknown",
    };
    println!("{:?}", response);
}
