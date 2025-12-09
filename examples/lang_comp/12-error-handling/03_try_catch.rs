fn main() {
    fn risky_operation(x: i32) -> i32 {
        if x < 0 {
            panic!("Negative not allowed")
        }
        x * 2
    }
    match std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            {
                {
                    let result = risky_operation(5);
                    println!("{:?}", result)
                }
            }
        }),
    ) {
        Ok(val) => val,
        Err(panic_err) => {
            let e = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            { println!("{:?}", e) }
        }
    };
    match std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            {
                {
                    let result = risky_operation(-1);
                    println!("{:?}", result)
                }
            }
        }),
    ) {
        Ok(val) => val,
        Err(panic_err) => {
            let e = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            { println!("{:?}", e) }
        }
    };
    fn validate_and_process(n: i32) -> i32 {
        if n < 0 {
            panic!("Negative value")
        }
        if n > 100 {
            panic!("Too large")
        }
        n * n
    }
    match std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            {
                {
                    let r1 = validate_and_process(5);
                    println!("{:?}", r1)
                }
            }
        }),
    ) {
        Ok(val) => val,
        Err(panic_err) => {
            let e = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            { println!("{:?}", e) }
        }
    };
    match std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            {
                {
                    let r2 = validate_and_process(-5);
                    println!("{:?}", r2)
                }
            }
        }),
    ) {
        Ok(val) => val,
        Err(panic_err) => {
            let e = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            { println!("{:?}", e) }
        }
    };
    match std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| {
            {
                {
                    let r3 = validate_and_process(150);
                    println!("{:?}", r3)
                }
            }
        }),
    ) {
        Ok(val) => val,
        Err(panic_err) => {
            let e = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            { println!("{:?}", e) }
        }
    }
}
