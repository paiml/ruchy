//! Integration tests from sister projects
//!
//! [TEST-COV-012] Tests imported from ruchy-book and rosetta-ruchy

use assert_cmd::Command;
use predicates::prelude::*;

// Tests from ruchy-book examples
mod book_tests {
    use super::*;

    #[test]
    fn test_fibonacci_recursive() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun fib(n) {
                    if n <= 1 { n } else { fib(n-1) + fib(n-2) }
                }
                println(fib(10))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("55"));
    }

    #[test]
    fn test_factorial() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun factorial(n) {
                    if n <= 1 { 1 } else { n * factorial(n-1) }
                }
                println(factorial(5))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("120"));
    }

    #[test]
    fn test_list_operations() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let list = [1, 2, 3, 4, 5]
                println(list.len())
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("5"));
    }

    #[test]
    fn test_string_manipulation() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let s = "hello"
                println(s.upper())
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("HELLO"));
    }

    #[test]
    fn test_map_filter_reduce() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let nums = [1, 2, 3, 4, 5]
                let doubled = nums.map(|x| x * 2)
                println(doubled)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("[2, 4, 6, 8, 10]"));
    }

    #[test]
    fn test_pattern_matching() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let x = 2
                let result = match x {
                    1 => "one",
                    2 => "two",
                    _ => "other"
                }
                println(result)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("two"));
    }

    #[test]
    fn test_tuple_destructuring() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let (a, b) = (10, 20)
                println(a + b)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("30"));
    }

    #[test]
    fn test_object_creation() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let person = {name: "Alice", age: 30}
                println(person.name)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("Alice"));
    }

    #[test]
    fn test_range_iteration() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let sum = 0
                for i in 1..5 {
                    sum = sum + i
                }
                println(sum)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("10"));
    }

    #[test]
    fn test_while_loop() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let count = 0
                let sum = 0
                while count < 5 {
                    sum = sum + count
                    count = count + 1
                }
                println(sum)
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("10"));
    }
}

// Tests from rosetta-ruchy algorithms
mod rosetta_tests {
    use super::*;

    #[test]
    fn test_quicksort() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun quicksort(arr) {
                    if arr.len() <= 1 { 
                        arr 
                    } else {
                        let pivot = arr[0]
                        let less = arr.filter(|x| x < pivot)
                        let equal = arr.filter(|x| x == pivot)
                        let greater = arr.filter(|x| x > pivot)
                        quicksort(less) + equal + quicksort(greater)
                    }
                }
                let arr = [3, 1, 4, 1, 5, 9, 2, 6]
                println(quicksort(arr))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("[1, 1, 2, 3, 4, 5, 6, 9]"));
    }

    #[test]
    fn test_binary_search() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun binary_search(arr, target) {
                    let left = 0
                    let right = arr.len() - 1
                    
                    while left <= right {
                        let mid = (left + right) / 2
                        if arr[mid] == target {
                            return mid
                        } else if arr[mid] < target {
                            left = mid + 1
                        } else {
                            right = mid - 1
                        }
                    }
                    return -1
                }
                
                let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9]
                println(binary_search(arr, 5))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("4"));
    }

    #[test]
    fn test_bubble_sort() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun bubble_sort(arr) {
                    let n = arr.len()
                    for i in 0..n {
                        for j in 0..(n-i-1) {
                            if arr[j] > arr[j+1] {
                                let temp = arr[j]
                                arr[j] = arr[j+1]
                                arr[j+1] = temp
                            }
                        }
                    }
                    arr
                }
                let arr = [5, 2, 8, 1, 9]
                println(bubble_sort(arr))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("[1, 2, 5, 8, 9]"));
    }

    #[test]
    fn test_gcd() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun gcd(a, b) {
                    if b == 0 { a } else { gcd(b, a % b) }
                }
                println(gcd(48, 18))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("6"));
    }

    #[test]
    fn test_is_prime() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun is_prime(n) {
                    if n <= 1 { 
                        false 
                    } else {
                        let i = 2
                        while i * i <= n {
                            if n % i == 0 { return false }
                            i = i + 1
                        }
                        true
                    }
                }
                println(is_prime(17))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("true"));
    }

    #[test]
    fn test_sum_of_digits() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun sum_digits(n) {
                    let sum = 0
                    while n > 0 {
                        sum = sum + (n % 10)
                        n = n / 10
                    }
                    sum
                }
                println(sum_digits(12345))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("15"));
    }

    #[test]
    fn test_reverse_string() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun reverse_string(s) {
                    let result = ""
                    for c in s {
                        result = c + result
                    }
                    result
                }
                println(reverse_string("hello"))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("olleh"));
    }

    #[test]
    fn test_palindrome_check() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun is_palindrome(s) {
                    let i = 0
                    let j = s.len() - 1
                    while i < j {
                        if s[i] != s[j] { return false }
                        i = i + 1
                        j = j - 1
                    }
                    true
                }
                println(is_palindrome("racecar"))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("true"));
    }
}

// Tests for advanced features from sister projects
mod advanced_tests {
    use super::*;

    #[test]
    fn test_hashmap_usage() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let map = HashMap()
                map.insert("key1", 100)
                map.insert("key2", 200)
                println(map.get("key1"))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("100"));
    }

    #[test]
    fn test_hashset_usage() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let set = HashSet()
                set.insert(1)
                set.insert(2)
                set.insert(1)
                println(set.len())
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("2"));
    }

    #[test]
    fn test_option_type() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let x = Some(42)
                match x {
                    Some(v) => println(v),
                    None => println("nothing")
                }
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }

    #[test]
    fn test_result_type() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun divide(a, b) {
                    if b == 0 {
                        Err("Division by zero")
                    } else {
                        Ok(a / b)
                    }
                }
                
                match divide(10, 2) {
                    Ok(v) => println(v),
                    Err(e) => println(e)
                }
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("5"));
    }

    #[test]
    fn test_closure_capture() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                let x = 10
                let add_x = |y| x + y
                println(add_x(5))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("15"));
    }

    #[test]
    fn test_higher_order_functions() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("-e")
            .arg(r#"
                fun apply_twice(f, x) {
                    f(f(x))
                }
                let double = |x| x * 2
                println(apply_twice(double, 5))
            "#)
            .assert()
            .success()
            .stdout(predicate::str::contains("20"));
    }
}