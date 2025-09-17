//! Basic tests that actually compile to establish baseline coverage

#[test]
fn test_arithmetic() {
    assert_eq!(2 + 2, 4);
    assert_eq!(10 - 5, 5);
    assert_eq!(3 * 4, 12);
    assert_eq!(20 / 4, 5);
}

#[test]
fn test_strings() {
    let s1 = String::from("hello");
    let s2 = String::from("world");
    assert_eq!(s1.len(), 5);
    assert_eq!(s2.len(), 5);

    let combined = format!("{} {}", s1, s2);
    assert_eq!(combined, "hello world");
}

#[test]
fn test_vectors() {
    let mut v = vec![1, 2, 3];
    v.push(4);
    assert_eq!(v.len(), 4);
    assert_eq!(v[3], 4);

    let sum: i32 = v.iter().sum();
    assert_eq!(sum, 10);
}

#[test]
fn test_options() {
    let some_value: Option<i32> = Some(42);
    let none_value: Option<i32> = None;

    assert!(some_value.is_some());
    assert!(none_value.is_none());

    assert_eq!(some_value.unwrap_or(0), 42);
    assert_eq!(none_value.unwrap_or(0), 0);
}

#[test]
fn test_results() {
    let ok_result: Result<i32, String> = Ok(100);
    let err_result: Result<i32, String> = Err("error".to_string());

    assert!(ok_result.is_ok());
    assert!(err_result.is_err());

    assert_eq!(ok_result.unwrap_or(-1), 100);
    assert_eq!(err_result.unwrap_or(-1), -1);
}

#[test]
fn test_iterators() {
    let data = vec![1, 2, 3, 4, 5];

    let doubled: Vec<i32> = data.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6, 8, 10]);

    let filtered: Vec<i32> = data.iter().filter(|x| **x > 2).cloned().collect();
    assert_eq!(filtered, vec![3, 4, 5]);
}

#[test]
fn test_closures() {
    let add = |a: i32, b: i32| a + b;
    assert_eq!(add(5, 3), 8);

    let multiply_by = |n: i32| move |x: i32| x * n;
    let times_three = multiply_by(3);
    assert_eq!(times_three(4), 12);
}

#[test]
fn test_pattern_matching() {
    let value = Some(42);

    let result = match value {
        Some(x) if x > 40 => "big",
        Some(_) => "small",
        None => "nothing",
    };

    assert_eq!(result, "big");
}

#[test]
fn test_error_handling() {
    fn divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 {
            Err("division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    assert_eq!(divide(10, 2), Ok(5));
    assert!(divide(10, 0).is_err());
}

#[test]
fn test_struct_operations() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 10, y: 20 };

    assert_eq!(p1, p2);
    assert_eq!(p1.x + p1.y, 30);
}