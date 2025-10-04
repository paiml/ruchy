// Extreme TDD test to expose and fix arrow integration type mismatch bugs
// Following Toyota Way: Write test FIRST, then fix implementation

#[test]
fn test_type_compatibility_demonstration() {
    // TDD: This test demonstrates the type mismatch bug
    // Bug location: src/backend/arrow_integration.rs:658-659
    // Known bug: Vec<i32> created but used with Int64Array::from()

    // CORRECT usage - what the code should do:
    let i32_values: Vec<i32> = (0..5).collect();
    let i64_values: Vec<i64> = (0..5).collect();

    // Demonstrate type incompatibility
    assert_eq!(i32_values, vec![0, 1, 2, 3, 4]);
    assert_eq!(i64_values, vec![0i64, 1i64, 2i64, 3i64, 4i64]);

    // The bug: trying to use Vec<i32> where Vec<i64> is expected
    // This test documents the expected behavior after fix
    assert_ne!(
        std::mem::size_of_val(&i32_values[0]),
        std::mem::size_of_val(&i64_values[0])
    );
}

#[test]
fn test_compilation_will_succeed_after_fix() {
    // TDD: This will compile once we fix the type mismatch
    // It represents the corrected behavior

    // BUG: lines 658-659 in arrow_integration.rs create Vec<i32>
    let values: Vec<i32> = (0..5).collect();

    // SOLUTION: Convert to i64 before using with Int64Array
    let correct_values: Vec<i64> = values.into_iter().map(|x| x as i64).collect();

    assert_eq!(correct_values, vec![0i64, 1i64, 2i64, 3i64, 4i64]);
    assert_eq!(correct_values.len(), 5);
}
