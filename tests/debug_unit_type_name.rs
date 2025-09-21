// Debug test to understand what type_name_of_val returns for match expressions

#[test]
fn test_unit_type_name_matching() {
    let unit_result = ();
    let type_name = std::any::type_name_of_val(&unit_result);
    println!("Unit type name: '{}'", type_name);

    // Test the exact conditions used in transpiler
    assert_eq!(type_name, "()");
    assert!(!type_name.contains("String"));
    assert!(!type_name.contains("&str"));

    // Test match expression result
    let number = 2;
    let match_result = match number {
        1 => println!("One"),
        2 => println!("Two"),
        _ => println!("Other"),
    };
    let match_type_name = std::any::type_name_of_val(&match_result);
    println!("Match expression type name: '{}'", match_type_name);

    assert_eq!(match_type_name, "()");
    assert!(!match_type_name.contains("String"));
    assert!(!match_type_name.contains("&str"));
}
