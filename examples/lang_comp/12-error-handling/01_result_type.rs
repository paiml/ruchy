fn main() {
    fn divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 { Err("Division by zero".to_string()) } else { Ok(a / b) }
    }
    {
        let result1 = divide(10, 2);
        {
            match result1 {
                Ok(value) => println!("{:?}", value),
                Err(msg) => println!("{:?}", msg),
            };
            {
                let result2 = divide(10, 0);
                {
                    match result2 {
                        Ok(value) => println!("{:?}", value),
                        Err(msg) => println!("{:?}", msg),
                    };
                    fn parse_number(s: String) -> Result<i32, String> {
                        if s == "42" { Ok(42) } else { Err("Parse error".to_string()) }
                    }
                    let s1: String = "42".to_string();
                    let s2: String = "abc".to_string();
                    {
                        let good = parse_number(s1);
                        {
                            let bad = parse_number(s2);
                            {
                                match good {
                                    Ok(n) => println!("{:?}", n),
                                    Err(e) => println!("{:?}", e),
                                };
                                match bad {
                                    Ok(n) => println!("{:?}", n),
                                    Err(e) => println!("{:?}", e),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
