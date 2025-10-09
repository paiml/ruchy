// DEFECT-ENUM-OK-RESERVED: Parser should accept reserved keywords as enum variant names
//
// BUG: Parser rejects 'Ok' and 'Err' as enum variant names inside functions
// ROOT CAUSE: Function body parser consumes reserved tokens before enum parser sees them
// FIX STRATEGY: Ensure enum parser has priority for reserved keywords in enum context

use ruchy::frontend::parser::Parser;

/// Helper to parse code and return Result
fn parse_code(code: &str) -> anyhow::Result<()> {
    let mut parser = Parser::new(code);
    parser.parse()?;
    Ok(())
}

// ============================================================================
// Unit Tests (RED → GREEN → REFACTOR)
// ============================================================================

#[test]
fn test_enum_variant_ok_alone_in_function() {
    // This works - enum alone
    let code = r#"
fn main() {
    enum HttpStatus {
        Ok = 200
    }
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum alone in function, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_variant_ok_with_enum_reference() {
    // BUG: Referencing enum variant Test::Ok triggers parser error
    let code = r#"
fn main() {
    enum HttpStatus {
        Ok = 200
    }
    let x = HttpStatus::Ok
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with variant reference, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_normal_variant_with_reference() {
    // This works - normal identifier variant
    let code = r#"
fn main() {
    enum Test {
        A = 1
    }
    let x = Test::A
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with normal variant reference, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_variant_ok_with_statement_before() {
    // This works - statement + enum after
    let code = r#"
fn main() {
    let x = 1
    enum HttpStatus {
        Ok = 200
    }
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with statement before, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_variant_err_inside_function() {
    let code = r#"
fn main() {
    enum Status {
        Err = 500
    }
    let x = Status::Err
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with 'Err' variant inside function, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_variant_some_inside_function() {
    let code = r#"
fn main() {
    enum Optional {
        Some = 1,
        None = 0
    }
    let x = Optional::Some
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with 'Some'/'None' variants inside function, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_enum_all_reserved_variants() {
    let code = r#"
fn main() {
    enum AllReserved {
        Ok = 1,
        Err = 2,
        Some = 3,
        None = 4
    }
    let a = AllReserved::Ok
    let b = AllReserved::Err
    let c = AllReserved::Some
    let d = AllReserved::None
}
"#;

    let result = parse_code(code);
    assert!(
        result.is_ok(),
        "Should parse enum with all reserved keywords as variants, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_example_file_04_enum_discriminants() {
    // This is the actual failing example from lang_comp
    let code = std::fs::read_to_string("examples/lang_comp/15-enums/04_enum_discriminants.ruchy")
        .expect("Failed to read example file");

    let result = parse_code(&code);
    assert!(
        result.is_ok(),
        "Should parse 04_enum_discriminants.ruchy, but got: {:?}",
        result.err()
    );
}

// ============================================================================
// Property Tests (10,000 cases for mathematical proof)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// List of reserved keywords that should work as enum variants
    const RESERVED_KEYWORDS: &[&str] = &["Ok", "Err", "Some", "None"];

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        /// Property 1: Any reserved keyword should work as an enum variant name
        #[test]
        fn prop_reserved_keywords_as_variants_never_panic(
            keyword_idx in 0..RESERVED_KEYWORDS.len(),
            discriminant in 1i64..1000i64
        ) {
            let keyword = RESERVED_KEYWORDS[keyword_idx];
            let code = format!(
                "fn main() {{\n    enum Test {{\n        {} = {}\n    }}\n    let x = Test::{}\n}}",
                keyword, discriminant, keyword
            );

            // Should not panic
            let _ = parse_code(&code);
        }

        /// Property 2: Enum with reserved variant inside function always parses
        #[test]
        fn prop_enum_inside_function_parses(
            keyword_idx in 0..RESERVED_KEYWORDS.len()
        ) {
            let keyword = RESERVED_KEYWORDS[keyword_idx];
            let code = format!(
                "fn main() {{\n    enum Status {{\n        {}\n    }}\n    let x = Status::{}\n}}",
                keyword, keyword
            );

            let result = parse_code(&code);
            prop_assert!(
                result.is_ok(),
                "Reserved keyword '{}' should parse as variant, got: {:?}",
                keyword,
                result.err()
            );
        }

        /// Property 3: Multiple reserved keywords in one enum
        #[test]
        fn prop_multiple_reserved_keywords_in_enum(
            count in 1..=4usize
        ) {
            let keywords: Vec<&str> = RESERVED_KEYWORDS.iter().take(count).copied().collect();
            let variants: Vec<String> = keywords
                .iter()
                .enumerate()
                .map(|(i, kw)| format!("{} = {}", kw, i + 1))
                .collect();

            let code = format!(
                "fn main() {{\n    enum Test {{\n        {}\n    }}\n    let x = Test::{}\n}}",
                variants.join(",\n        "),
                keywords[0]
            );

            let result = parse_code(&code);
            prop_assert!(
                result.is_ok(),
                "Enum with {} reserved keywords should parse, got: {:?}",
                count,
                result.err()
            );
        }
    }
}
