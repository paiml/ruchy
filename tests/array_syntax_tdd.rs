// TDD Test Suite for ARRAY-SYNTAX-001
// Known bug: Array type syntax [i32; 5] not parsing in function parameters
// Target: Fix 4+ examples in Ch04, Ch15

use ruchy::runtime::repl::Repl;
use std::env;

fn eval(code: &str) -> String {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.eval(code).unwrap_or_else(|e| format!("Error: {}", e))
}

#[test]
fn test_fixed_size_array_parameter() {
    let code = r#"
        fun sum_array(arr: [i32; 5]) -> i32 {
            let mut total = 0;
            let mut i = 0;
            while i < 5 {
                total = total + arr[i];
                i = i + 1;
            }
            total
        }
        
        sum_array([1, 2, 3, 4, 5])
    "#;
    assert_eq!(eval(code), "15");
}

#[test]
fn test_array_return_type() {
    let code = r#"
        fun create_array() -> [i32; 3] {
            [10, 20, 30]
        }
        
        let arr = create_array();
        arr[1]
    "#;
    assert_eq!(eval(code), "20");
}

#[test]
fn test_multiple_array_parameters() {
    let code = r#"
        fun dot_product(a: [f64; 3], b: [f64; 3]) -> f64 {
            a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
        }
        
        dot_product([1.0, 2.0, 3.0], [4.0, 5.0, 6.0])
    "#;
    assert_eq!(eval(code), "32"); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
}

#[test]
fn test_nested_array_type() {
    let code = r#"
        fun get_matrix_element(matrix: [[i32; 3]; 3], row: i32, col: i32) -> i32 {
            matrix[row][col]
        }
        
        let m = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        get_matrix_element(m, 1, 2)
    "#;
    assert_eq!(eval(code), "6");
}

#[test]
fn test_array_with_generic_size() {
    let code = r#"
        fun first_element(arr: [i32; 10]) -> i32 {
            arr[0]
        }
        
        let numbers = [99, 98, 97, 96, 95, 94, 93, 92, 91, 90];
        first_element(numbers)
    "#;
    assert_eq!(eval(code), "99");
}

// Ch04 specific test - practical patterns with fixed arrays
#[test]
fn test_ch04_calculate_total() {
    let code = r#"
        fun calculate_total(prices: [i32; 5]) -> i32 {
            let mut total = 0;
            let mut i = 0;
            
            while i < 5 {
                total = total + prices[i];
                i = i + 1;
            }
            
            total
        }
        
        let prices = [10, 25, 5, 15, 8];
        calculate_total(prices)
    "#;
    assert_eq!(eval(code), "63");
}

#[test]
fn test_ch04_find_maximum() {
    let code = r#"
        fun find_maximum(numbers: [i32; 5]) -> i32 {
            let mut max_value = numbers[0];
            let mut i = 1;
            
            while i < 5 {
                if numbers[i] > max_value {
                    max_value = numbers[i];
                }
                i = i + 1;
            }
            
            max_value
        }
        
        let numbers = [10, 25, 5, 30, 15];
        find_maximum(numbers)
    "#;
    assert_eq!(eval(code), "30");
}

#[test]
fn test_ch04_count_positives() {
    let code = r#"
        fun count_positives(numbers: [i32; 5]) -> i32 {
            let mut count = 0;
            let mut i = 0;
            
            while i < 5 {
                if numbers[i] > 0 {
                    count = count + 1;
                }
                i = i + 1;
            }
            
            count
        }
        
        let numbers = [-3, 7, -1, 12, 0];
        count_positives(numbers)
    "#;
    assert_eq!(eval(code), "2");
}

// Ch15 specific test - binary compilation with arrays
#[test]
fn test_ch15_array_processing() {
    let code = r#"
        fun process_data(input: [i32; 4]) -> i32 {
            input[0] * 2 + input[1] * 2 + input[2] * 2 + input[3] * 2
        }
        
        let data = [1, 2, 3, 4];
        process_data(data)
    "#;
    assert_eq!(eval(code), "20"); // (1+2+3+4) * 2 = 20
}

#[test]
fn test_ch15_array_initialization() {
    let code = r#"
        fun init_array() -> [i32; 4] {
            let arr: [i32; 4] = [10; 4];  // Initialize all elements to 10
            arr[1] + arr[2]
        }
        
        init_array()
    "#;
    assert_eq!(eval(code), "20"); // 10 + 10 = 20
}

// Check array size in local variable declarations
#[test]
fn test_local_array_declaration() {
    let code = r#"
        fun create_buffer() -> i32 {
            let buffer: [i32; 3] = [42; 3];  // Initialize with 42
            buffer[0]
        }
        
        create_buffer()
    "#;
    assert_eq!(eval(code), "42");
}

// Check const generic-like behavior
#[test]
fn test_array_length_constant() {
    let code = r#"
        fun sum_array(arr: [i32; 5]) -> i32 {
            arr[0] + arr[1] + arr[2] + arr[3] + arr[4]
        }
        
        sum_array([10, 20, 30, 40, 50])
    "#;
    assert_eq!(eval(code), "150");
}
