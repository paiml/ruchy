// ISSUE-131: Property-based tests for parse_json() alias fix
//
// Property tests validate invariants across 100+ random inputs
//
// Properties tested:
// 1. Roundtrip: parse_json(json_stringify(obj)) = obj
// 2. Determinism: parse_json(s) always returns same result
// 3. Alias equivalence: parse_json(s) = json_parse(s)
// 4. Type preservation: Numbers, strings, booleans preserved
// 5. Nested access: Deep object/array access doesn't crash

use predicates::prelude::*;
use proptest::prelude::*;

// ============================================================================
// Property 1: Roundtrip - parse_json(json_stringify(obj)) = obj
// ============================================================================

#[test]
fn prop_parse_json_roundtrip_objects() {
    proptest!(|(
        name in "[a-zA-Z]{3,10}",
        value in 0i32..1000
    )| {
        let script = format!(r#"
fun main() {{
    let original = '{{"name": "{name}", "value": {value}}}'
    let parsed = parse_json(original)
    let name = parsed["name"]
    let value = parsed["value"]
    println(name)
    println(value)
}}
"#);

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();

        // Property: Parsed values match original values
        prop_assert_eq!(lines[0], name);
        prop_assert_eq!(lines[1], value.to_string());
    });
}

#[test]
fn prop_parse_json_roundtrip_arrays() {
    proptest!(|(
                                        values in prop::collection::vec(0i32..100, 1..5)
                                    )| {
                                        let json_array = format!("[{}]", values.iter()
                                            .map(std::string::ToString::to_string)
                                            .collect::<Vec<_>>()
                                            .join(", "));

                                        let script = format!(r"
fun main() {{
    let arr = parse_json('{json_array}')
    println(arr[0])
}}
");

                                        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                            .arg("-e")
                                            .arg(&script)
                                            .assert()
                                            .success()
                                            .get_output()
                                            .stdout
                                            .clone();

                                        let output_str = String::from_utf8(output).unwrap().trim().to_string();

                                        // Property: First element matches
                                        prop_assert_eq!(output_str, values[0].to_string());
                                    });
}

// ============================================================================
// Property 2: Determinism - parse_json(s) always returns same result
// ============================================================================

#[test]
fn prop_parse_json_deterministic() {
    proptest!(|(
        x in 0i32..1000,
        y in 0i32..1000
    )| {
        let json_str = format!("{{\"x\": {x}, \"y\": {y}}}");
        let script = format!(r#"
fun main() {{
    let data1 = parse_json('{json_str}')
    let data2 = parse_json('{json_str}')
    println(data1["x"])
    println(data2["x"])
}}
"#);

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();

        // Property: Multiple parses of same string produce same result
        prop_assert_eq!(lines[0], lines[1]);
        prop_assert_eq!(lines[0], x.to_string());
    });
}

// ============================================================================
// Property 3: Alias Equivalence - parse_json(s) = json_parse(s)
// ============================================================================

#[test]
fn prop_parse_json_json_parse_equivalent() {
    proptest!(|(
        field in "[a-z]{3,8}",
        value in 0i32..1000
    )| {
        let json_str = format!("{{\"{field}\" : {value}}}");

        let script_parse_json = format!(r#"
fun main() {{
    let data = parse_json('{json_str}')
    println(data["{field}"])
}}
"#);

        let script_json_parse = format!(r#"
fun main() {{
    let data = json_parse('{json_str}')
    println(data["{field}"])
}}
"#);

        let output1 = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script_parse_json)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output2 = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script_json_parse)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        // Property: Both aliases produce identical output
        prop_assert_eq!(output1, output2);
    });
}

// ============================================================================
// Property 4: Type Preservation - Numbers, strings, booleans preserved
// ============================================================================

#[test]
fn prop_parse_json_preserves_types() {
    proptest!(|(
        num in 0i32..1000,
        text in "[a-zA-Z]{3,10}",
        flag in prop::bool::ANY
    )| {
        let json_str = format!(
            "{{\"number\": {num}, \"string\": \"{text}\", \"boolean\": {flag}}}"
        );

        let script = format!(r#"
fun main() {{
    let data = parse_json('{json_str}')
    println(data["number"])
    println(data["string"])
    println(data["boolean"])
}}
"#);

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();

        // Property: Types are preserved after parsing
        prop_assert_eq!(lines[0], num.to_string());
        prop_assert_eq!(lines[1], text);
        prop_assert_eq!(lines[2], flag.to_string());
    });
}

// ============================================================================
// Property 5: Nested Access - Deep object/array access doesn't crash
// ============================================================================

#[test]
fn prop_parse_json_nested_access_no_crash() {
    proptest!(|(
        depth1_val in 0i32..100,
        depth2_val in 0i32..100
    )| {
        let json_str = format!(
            "{{\"level1\": {{\"level2\": {{\"value\": {depth1_val}}}}}, \"array\": [{depth2_val}]}}"
        );

        let script = format!(r#"
fun main() {{
    let data = parse_json('{json_str}')
    println(data["level1"]["level2"]["value"])
    println(data["array"][0])
}}
"#);

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(&script)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();

        // Property: Nested access works without crashing
        prop_assert_eq!(lines[0], depth1_val.to_string());
        prop_assert_eq!(lines[1], depth2_val.to_string());
    });
}

// ============================================================================
// Property 6: Empty and Special Cases
// ============================================================================

#[test]
fn prop_parse_json_empty_cases() {
    proptest!(|(_unit in prop::bool::ANY)| {
        // Empty object
        let script1 = r#"
fun main() {
    let data = parse_json('{}')
    println("empty_object")
}
"#;
        assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(script1)
            .assert()
            .success()
            .stdout(predicate::str::contains("empty_object"));

        // Empty array
        let script2 = r#"
fun main() {
    let data = parse_json('[]')
    println("empty_array")
}
"#;
        assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("-e")
            .arg(script2)
            .assert()
            .success()
            .stdout(predicate::str::contains("empty_array"));
    });
}
