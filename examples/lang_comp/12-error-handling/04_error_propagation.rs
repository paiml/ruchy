fn main() {
    fn step1(x: i32) -> Result<i32, String> {
        if x > 0 { Ok(x * 2) } else { Err("Step 1 failed".to_string()) }
    }
    fn step2(x: i32) -> Result<i32, String> {
        if x < 100 { Ok(x + 10) } else { Err("Step 2 failed".to_string()) }
    }
    fn process(input: i32) {
        match step1(input) {
            Ok(v1) => {
                match step2(v1) {
                    Ok(v2) => println!("{:?}", v2),
                    Err(e) => println!("{:?}", e),
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    process(10);
    process(-5);
    process(60);
    fn validate_age(age: i32) -> Result<i32, String> {
        if age < 0 {
            Err("Negative age".to_string())
        } else {
            if age > 150 { Err("Age too high".to_string()) } else { Ok(age) }
        }
    }
    fn validate_name(name: String) -> Result<String, String> {
        if name == "" { Err("Empty name".to_string()) } else { Ok(name) }
    }
    {
        let ages = [-5, 25, 200];
        for age in ages {
            {
                match validate_age(age.clone()) {
                    Ok(a) => println!("{:?}", a),
                    Err(msg) => println!("{:?}", msg),
                }
            }
        }
    }
}
