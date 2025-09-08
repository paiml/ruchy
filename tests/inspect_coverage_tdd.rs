//! TDD tests for inspect.rs module  
//! Target: Improve inspect.rs from 0% to 90%+ coverage

use ruchy::runtime::inspect::{Inspect, Inspector, InspectStyle, DisplayForm, CompositeForm, OpaqueHandle};
use std::collections::HashMap;
use std::fmt::Write;

#[test]
fn test_inspector_new() {
    let inspector = Inspector::new();
    assert_eq!(inspector.depth(), 0);
    assert!(inspector.has_budget());
    assert!(!inspector.at_max_depth());
    assert!(inspector.output.is_empty());
}

#[test]
fn test_inspector_default() {
    let inspector = Inspector::default();
    assert_eq!(inspector.depth(), 0);
    assert!(inspector.has_budget());
    assert!(!inspector.at_max_depth());
}

#[test]
fn test_inspector_with_style() {
    let style = InspectStyle {
        max_elements: 5,
        max_string_len: 20,
        use_colors: true,
        indent: "    ".to_string(),
    };
    
    let inspector = Inspector::with_style(style.clone());
    assert_eq!(inspector.style.max_elements, 5);
    assert_eq!(inspector.style.max_string_len, 20);
    assert!(inspector.style.use_colors);
    assert_eq!(inspector.style.indent, "    ");
}

#[test]
fn test_inspect_style_default() {
    let style = InspectStyle::default();
    assert_eq!(style.max_elements, 10);
    assert_eq!(style.max_string_len, 100);
    assert!(!style.use_colors);
    assert_eq!(style.indent, "  ");
}

#[test]
fn test_inspector_enter_exit() {
    let mut inspector = Inspector::new();
    let value = 42;
    
    assert_eq!(inspector.depth(), 0);
    
    // Enter should succeed
    assert!(inspector.enter(&value));
    assert_eq!(inspector.depth(), 1);
    
    // Exit should decrease depth
    inspector.exit();
    assert_eq!(inspector.depth(), 0);
}

#[test]
fn test_inspector_cycle_detection() {
    let mut inspector = Inspector::new();
    let value = 42;
    
    // First enter should succeed
    assert!(inspector.enter(&value));
    
    // Second enter with same address should fail (cycle detected)
    assert!(!inspector.enter(&value));
}

#[test]
fn test_inspector_budget_tracking() {
    let mut inspector = Inspector::new();
    let initial_budget = inspector.budget;
    
    assert!(inspector.has_budget());
    
    // Consume some budget
    inspector.consume_budget(100);
    assert_eq!(inspector.budget, initial_budget - 100);
    assert!(inspector.has_budget());
    
    // Consume all budget
    inspector.consume_budget(inspector.budget);
    assert_eq!(inspector.budget, 0);
    assert!(!inspector.has_budget());
}

#[test]
fn test_inspector_budget_saturation() {
    let mut inspector = Inspector::new();
    
    // Consuming more than available should saturate at 0
    inspector.consume_budget(inspector.budget + 1000);
    assert_eq!(inspector.budget, 0);
}

#[test]
fn test_inspector_max_depth() {
    let mut inspector = Inspector::new();
    inspector.max_depth = 2;
    
    assert!(!inspector.at_max_depth());
    inspector.depth = 1;
    assert!(!inspector.at_max_depth());
    inspector.depth = 2;
    assert!(inspector.at_max_depth());
    inspector.depth = 3;
    assert!(inspector.at_max_depth());
}

#[test]
fn test_inspector_write_trait() {
    let mut inspector = Inspector::new();
    let initial_budget = inspector.budget;
    
    write!(inspector, "hello").unwrap();
    assert_eq!(inspector.output, "hello");
    assert_eq!(inspector.budget, initial_budget - 5); // "hello" is 5 chars
    
    write!(inspector, " world").unwrap();
    assert_eq!(inspector.output, "hello world");
    assert_eq!(inspector.budget, initial_budget - 11);
}

#[test]
fn test_inspect_i32() {
    let value = 42i32;
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "42");
}

#[test]
fn test_inspect_i64() {
    let value = 1234567890i64;
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "1234567890");
}

#[test]
fn test_inspect_f64() {
    let value = 3.14159f64;
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "3.14159");
}

#[test]
fn test_inspect_bool_true() {
    let value = true;
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "true");
}

#[test]
fn test_inspect_bool_false() {
    let value = false;
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "false");
}

#[test]
fn test_inspect_string_short() {
    let value = "hello".to_string();
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "\"hello\"");
}

#[test]
fn test_inspect_string_long() {
    let long_string = "a".repeat(150);
    let mut inspector = Inspector::new();
    inspector.style.max_string_len = 10;
    
    long_string.inspect(&mut inspector).unwrap();
    assert!(inspector.output.starts_with("\"aaaaaaaaaa...\""));
    assert!(inspector.output.contains("(150 chars)"));
}

#[test]
fn test_inspect_str_short() {
    let value = "world";
    let mut inspector = Inspector::new();
    
    value.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "\"world\"");
}

#[test]
fn test_inspect_str_long() {
    let long_str = &"b".repeat(80);
    let mut inspector = Inspector::new();
    inspector.style.max_string_len = 5;
    
    long_str.inspect(&mut inspector).unwrap();
    assert!(inspector.output.starts_with("\"bbbbb...\""));
    assert!(inspector.output.contains("(80 chars)"));
}

#[test]
fn test_inspect_vec_empty() {
    let vec: Vec<i32> = vec![];
    let mut inspector = Inspector::new();
    
    vec.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "[]");
}

#[test]
fn test_inspect_vec_single_element() {
    let vec = vec![42];
    let mut inspector = Inspector::new();
    
    vec.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "[42]");
}

#[test]
fn test_inspect_vec_multiple_elements() {
    let vec = vec![1, 2, 3, 4, 5];
    let mut inspector = Inspector::new();
    
    vec.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "[1, 2, 3, 4, 5]");
}

#[test]
fn test_inspect_vec_max_elements_limit() {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let mut inspector = Inspector::new();
    inspector.style.max_elements = 5;
    
    vec.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("[1, 2, 3, 4, 5, ...7 more]"));
}

#[test]
fn test_inspect_vec_at_max_depth() {
    let vec = vec![1, 2, 3];
    let mut inspector = Inspector::new();
    inspector.max_depth = 0; // At max depth immediately
    
    vec.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "[3 elements]");
}

#[test]
fn test_inspect_vec_budget_exhausted() {
    let vec = vec![1, 2, 3, 4, 5];
    let mut inspector = Inspector::new();
    inspector.budget = 5; // Very small budget
    
    vec.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("..."));
}

#[test]
fn test_inspect_hashmap_empty() {
    let map: HashMap<i32, String> = HashMap::new();
    let mut inspector = Inspector::new();
    
    map.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "{}");
}

#[test]
fn test_inspect_hashmap_single_entry() {
    let mut map = HashMap::new();
    map.insert(1, "one".to_string());
    let mut inspector = Inspector::new();
    
    map.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("1: \"one\""));
    assert!(inspector.output.starts_with('{'));
    assert!(inspector.output.ends_with('}'));
}

#[test]
fn test_inspect_hashmap_multiple_entries() {
    let mut map = HashMap::new();
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    let mut inspector = Inspector::new();
    
    map.inspect(&mut inspector).unwrap();
    // Order is not guaranteed in HashMap, but should contain both entries
    assert!(inspector.output.contains("1: \"one\"") || inspector.output.contains("2: \"two\""));
    assert!(inspector.output.contains(", "));
    assert!(inspector.output.starts_with('{'));
    assert!(inspector.output.ends_with('}'));
}

#[test]
fn test_inspect_hashmap_max_elements_limit() {
    let mut map = HashMap::new();
    for i in 1..=15 {
        map.insert(i, format!("value{i}"));
    }
    let mut inspector = Inspector::new();
    inspector.style.max_elements = 3;
    
    map.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("...12 more"));
}

#[test]
fn test_inspect_hashmap_at_max_depth() {
    let mut map = HashMap::new();
    map.insert(1, "one".to_string());
    let mut inspector = Inspector::new();
    inspector.max_depth = 0;
    
    map.inspect(&mut inspector).unwrap();
    assert_eq!(inspector.output, "{1 entries}");
}

#[test]
fn test_inspect_option_some() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let opt = Some(42);
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&opt, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Some(42)");
}

#[test]
fn test_inspect_option_none() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let opt: Option<i32> = None;
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&opt, &mut inspector).unwrap();
    assert_eq!(inspector.output, "None");
}

#[test]
fn test_inspect_option_nested() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let opt = Some(Some(42));
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&opt, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Some(Some(42))");
}

#[test]
fn test_inspect_result_ok() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let res: Result<i32, String> = Ok(42);
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&res, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Ok(42)");
}

#[test]
fn test_inspect_result_err() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let res: Result<i32, String> = Err("error message".to_string());
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&res, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Err(\"error message\")");
}

#[test]
fn test_inspect_result_nested() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let res: Result<Result<i32, String>, String> = Ok(Ok(42));
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&res, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Ok(Ok(42))");
}

#[test]
fn test_display_form_atomic() {
    let form = DisplayForm::Atomic("hello".to_string());
    match form {
        DisplayForm::Atomic(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected Atomic"),
    }
}

#[test]
fn test_display_form_composite() {
    let composite = CompositeForm {
        opener: "[",
        elements: vec![(None, DisplayForm::Atomic("1".to_string()))],
        closer: "]",
        elided: None,
    };
    let form = DisplayForm::Composite(composite);
    
    match form {
        DisplayForm::Composite(comp) => {
            assert_eq!(comp.opener, "[");
            assert_eq!(comp.closer, "]");
            assert_eq!(comp.elements.len(), 1);
            assert!(comp.elided.is_none());
        }
        _ => panic!("Expected Composite"),
    }
}

#[test]
fn test_display_form_composite_with_elided() {
    let composite = CompositeForm {
        opener: "{",
        elements: vec![
            (Some("key".to_string()), DisplayForm::Atomic("value".to_string()))
        ],
        closer: "}",
        elided: Some(5),
    };
    let form = DisplayForm::Composite(composite);
    
    match form {
        DisplayForm::Composite(comp) => {
            assert_eq!(comp.elided, Some(5));
            assert_eq!(comp.elements.len(), 1);
            assert_eq!(comp.elements[0].0, Some("key".to_string()));
        }
        _ => panic!("Expected Composite"),
    }
}

#[test]
fn test_display_form_reference() {
    let inner = DisplayForm::Atomic("referenced".to_string());
    let form = DisplayForm::Reference(0x1000, Box::new(inner));
    
    match form {
        DisplayForm::Reference(addr, boxed) => {
            assert_eq!(addr, 0x1000);
            match *boxed {
                DisplayForm::Atomic(s) => assert_eq!(s, "referenced"),
                _ => panic!("Expected inner Atomic"),
            }
        }
        _ => panic!("Expected Reference"),
    }
}

#[test]
fn test_display_form_opaque() {
    let handle = OpaqueHandle {
        type_name: "Function".to_string(),
        id: Some("main".to_string()),
    };
    let form = DisplayForm::Opaque(handle);
    
    match form {
        DisplayForm::Opaque(h) => {
            assert_eq!(h.type_name, "Function");
            assert_eq!(h.id, Some("main".to_string()));
        }
        _ => panic!("Expected Opaque"),
    }
}

#[test]
fn test_opaque_handle_without_id() {
    let handle = OpaqueHandle {
        type_name: "Thread".to_string(),
        id: None,
    };
    
    assert_eq!(handle.type_name, "Thread");
    assert!(handle.id.is_none());
}

#[test]
fn test_visit_set_inline_storage() {
    let mut inspector = Inspector::new();
    let values = [1, 2, 3, 4, 5];
    
    // All should succeed with inline storage
    for value in &values {
        assert!(inspector.enter(value));
    }
    
    // All should be detected as cycles
    for value in &values {
        assert!(!inspector.enter(value));
    }
}

#[test]
fn test_visit_set_overflow_storage() {
    let mut inspector = Inspector::new();
    let mut values = Vec::new();
    
    // Create enough values to trigger overflow
    for i in 0..12 {
        values.push(i);
    }
    
    // All should succeed, triggering overflow storage
    for value in &values {
        assert!(inspector.enter(value));
    }
    
    // All should be detected as cycles
    for value in &values {
        assert!(!inspector.enter(value));
    }
}

#[test]
fn test_inspect_depth_default() {
    struct TestStruct;
    impl Inspect for TestStruct {
        fn inspect(&self, inspector: &mut Inspector) -> std::fmt::Result {
            write!(inspector, "TestStruct")
        }
    }
    
    let test_struct = TestStruct;
    assert_eq!(test_struct.inspect_depth(), 1); // Default implementation
}

#[test]
fn test_inspect_depth_custom() {
    struct DeepStruct;
    impl Inspect for DeepStruct {
        fn inspect(&self, inspector: &mut Inspector) -> std::fmt::Result {
            write!(inspector, "DeepStruct")
        }
        
        fn inspect_depth(&self) -> usize {
            5 // Custom depth
        }
    }
    
    let deep_struct = DeepStruct;
    assert_eq!(deep_struct.inspect_depth(), 5);
}

#[test]
fn test_nested_vec_inspection() {
    let nested = vec![vec![1, 2], vec![3, 4, 5]];
    let mut inspector = Inspector::new();
    
    nested.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("[["));
    assert!(inspector.output.contains("1, 2"));
    assert!(inspector.output.contains("3, 4, 5"));
    assert!(inspector.output.contains("]]"));
}

#[test]
fn test_nested_option_inspection() {
    use ruchy::runtime::inspect::Inspect as InspectTrait;
    let nested = Some(Some(Some(42)));
    let mut inspector = Inspector::new();
    
    InspectTrait::inspect(&nested, &mut inspector).unwrap();
    assert_eq!(inspector.output, "Some(Some(Some(42)))");
}

#[test]
fn test_complex_nested_structures() {
    let mut map = HashMap::new();
    map.insert("numbers".to_string(), vec![1, 2, 3]);
    map.insert("empty".to_string(), vec![]);
    
    let mut inspector = Inspector::new();
    map.inspect(&mut inspector).unwrap();
    
    assert!(inspector.output.contains("numbers"));
    assert!(inspector.output.contains("empty"));
    assert!(inspector.output.contains("[1, 2, 3]"));
    assert!(inspector.output.contains("[]"));
}

#[test]
fn test_inspector_saturating_exit() {
    let mut inspector = Inspector::new();
    assert_eq!(inspector.depth(), 0);
    
    // Exit when already at depth 0 should stay at 0
    inspector.exit();
    assert_eq!(inspector.depth(), 0);
}

#[test]
fn test_style_clone() {
    let style = InspectStyle {
        max_elements: 15,
        max_string_len: 200,
        use_colors: true,
        indent: "\t".to_string(),
    };
    
    let cloned = style.clone();
    assert_eq!(cloned.max_elements, 15);
    assert_eq!(cloned.max_string_len, 200);
    assert!(cloned.use_colors);
    assert_eq!(cloned.indent, "\t");
}

#[test]
fn test_composite_form_clone() {
    let composite = CompositeForm {
        opener: "(",
        elements: vec![(Some("label".to_string()), DisplayForm::Atomic("value".to_string()))],
        closer: ")",
        elided: Some(10),
    };
    
    let cloned = composite.clone();
    assert_eq!(cloned.opener, "(");
    assert_eq!(cloned.closer, ")");
    assert_eq!(cloned.elided, Some(10));
    assert_eq!(cloned.elements.len(), 1);
}

#[test]
fn test_opaque_handle_clone() {
    let handle = OpaqueHandle {
        type_name: "CustomType".to_string(),
        id: Some("instance_42".to_string()),
    };
    
    let cloned = handle.clone();
    assert_eq!(cloned.type_name, "CustomType");
    assert_eq!(cloned.id, Some("instance_42".to_string()));
}

#[test]
fn test_display_form_clone() {
    let form = DisplayForm::Reference(
        0x2000,
        Box::new(DisplayForm::Atomic("test".to_string()))
    );
    
    let cloned = form.clone();
    match cloned {
        DisplayForm::Reference(addr, boxed) => {
            assert_eq!(addr, 0x2000);
            match *boxed {
                DisplayForm::Atomic(s) => assert_eq!(s, "test"),
                _ => panic!("Expected Atomic inner"),
            }
        }
        _ => panic!("Expected Reference"),
    }
}