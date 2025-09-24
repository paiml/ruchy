//! EXTREME TDD: Import Syntax Tests (EXTR-001)
//!
//! Test-first development for comprehensive import syntax support
//! Target: Fix ignored test and achieve full import functionality
//! Complexity: All test functions ≤10 cyclomatic complexity
//! Coverage: 100% of import variations

use ruchy::compile;

#[cfg(test)]
mod import_syntax_tests {
    use super::*;

    // Basic import statements
    #[test]
    fn test_import_std() {
        let result = compile("import std");
        assert!(result.is_ok(), "Failed to compile: import std");
        let output = result.unwrap();
        assert!(output.contains("use std"), "Should transpile to 'use std'");
    }

    #[test]
    fn test_import_std_collections() {
        let result = compile("import std.collections");
        assert!(result.is_ok(), "Failed to compile: import std.collections");
        let output = result.unwrap();
        // Check for the import with or without spaces
        let has_import =
            output.contains("use std::collections") || output.contains("use std :: collections");
        assert!(
            has_import,
            "Should transpile to 'use std::collections', got: {output}"
        );
    }

    #[test]
    fn test_import_std_collections_hashmap() {
        let result = compile("import std.collections.HashMap");
        assert!(
            result.is_ok(),
            "Failed to compile: import std.collections.HashMap"
        );
        let output = result.unwrap();
        // Check for the import with or without spaces
        let has_import = output.contains("use std::collections::HashMap")
            || output.contains("use std :: collections :: HashMap");
        assert!(
            has_import,
            "Should transpile to 'use std::collections::HashMap', got: {output}"
        );
    }

    // From...import statements
    #[test]
    fn test_from_std_import_println() {
        let result = compile("from std import println");
        assert!(result.is_ok(), "Failed to compile: from std import println");
        let output = result.unwrap();
        println!("from std import println output: {output}");
        // Check with or without spaces
        let has_import = output.contains("use std::println")
            || output.contains("use std :: println")
            || output.contains("use std::{println}")
            || output.contains("use std :: { println }");
        assert!(
            has_import,
            "Should transpile to 'use std::println', got: {output}"
        );
    }

    #[test]
    #[ignore = "Import feature not fully implemented yet"]
    fn test_from_collections_import_multiple() {
        let result = compile("from std.collections import HashMap, HashSet, BTreeMap");
        assert!(
            result.is_ok(),
            "Failed to compile: from std.collections import multiple"
        );
        let output = result.unwrap();
        assert!(
            output.contains("use std::collections::{HashMap, HashSet, BTreeMap}"),
            "Should transpile to 'use std::collections::{{HashMap, HashSet, BTreeMap}}'"
        );
    }

    // Import with aliases
    #[test]
    fn test_import_as_alias() {
        let result = compile("import std.collections.HashMap as Map");
        assert!(result.is_ok(), "Failed to compile: import...as");
        let output = result.unwrap();
        println!("Import with alias output: {output}");
        // Check with or without spaces
        let has_alias = output.contains("use std::collections::HashMap as Map")
            || output.contains("use std :: collections :: HashMap as Map");
        assert!(
            has_alias,
            "Should transpile to 'use std::collections::HashMap as Map', got: {output}"
        );
    }

    #[test]
    fn test_from_import_with_alias() {
        let result = compile("from std.collections import HashMap as Map, HashSet as Set");
        assert!(result.is_ok(), "Failed to compile: from...import...as");
        let output = result.unwrap();
        assert!(
            output.contains("HashMap as Map") && output.contains("HashSet as Set"),
            "Should support multiple aliases"
        );
    }

    // Wildcard imports
    #[test]
    fn test_import_wildcard() {
        let result = compile("from std.collections import *");
        assert!(result.is_ok(), "Failed to compile: from...import *");
        let output = result.unwrap();
        println!("Wildcard import output: {output}");
        // Check with or without spaces
        let has_wildcard = output.contains("use std::collections::*")
            || output.contains("use std :: collections :: *");
        assert!(
            has_wildcard,
            "Should transpile to 'use std::collections::*', got: {output}"
        );
    }

    // Nested module imports
    #[test]
    fn test_import_nested_modules() {
        let result = compile("import tokio.sync.mpsc");
        assert!(result.is_ok(), "Failed to compile: import tokio.sync.mpsc");
        let output = result.unwrap();
        // Check with or without spaces
        let has_import = output.contains("use tokio::sync::mpsc")
            || output.contains("use tokio :: sync :: mpsc");
        assert!(
            has_import,
            "Should transpile nested module paths, got: {output}"
        );
    }

    // Import with braces (JS-style)
    #[test]
    fn test_import_braces_syntax() {
        let result = compile("import { readFile, writeFile } from fs");
        if let Err(e) = &result {
            println!("JS-style import error: {e}");
        }
        assert!(result.is_ok(), "Failed to compile: import {{...}} from");
        let output = result.unwrap();
        println!("JS-style import output: {output}");
        // Check with or without spaces
        let has_import = output.contains("use fs::{readFile, writeFile}")
            || output.contains("use fs :: { readFile , writeFile }");
        assert!(has_import, "Should support JS-style imports, got: {output}");
    }

    // Multiple imports in one file
    #[test]
    #[ignore = "Import feature not fully implemented yet"]
    fn test_multiple_imports() {
        let code = r"
import std
import std.collections.HashMap
from std.io import println, eprintln
import tokio.sync as sync
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile multiple imports");
        let output = result.unwrap();
        assert!(output.contains("use std;"), "Should have use std");
        assert!(
            output.contains("use std::collections::HashMap;"),
            "Should have HashMap import"
        );
        assert!(
            output.contains("use std::io::{println, eprintln}"),
            "Should have io imports"
        );
        assert!(
            output.contains("use tokio::sync as sync"),
            "Should have aliased import"
        );
    }

    // Import in different contexts
    #[test]
    fn test_import_in_function() {
        let code = r"
fn main() {
    import std.collections.HashMap
    let map = HashMap::new()
}
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile import inside function");
    }

    #[test]
    fn test_import_in_module() {
        let code = r"
mod utils {
    import std.fs
    fn read_file(path: String) -> String {
        fs::read_to_string(path)
    }
}
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile import inside module");
    }

    // Edge cases
    #[test]
    fn test_import_self() {
        let result = compile("import self");
        if let Err(e) = &result {
            println!("Import self error: {e}");
        }
        assert!(result.is_ok(), "Failed to compile: import self");
        let output = result.unwrap();
        assert!(
            output.contains("use self"),
            "Should transpile to 'use self'"
        );
    }

    #[test]
    fn test_import_super() {
        let result = compile("import super");
        assert!(result.is_ok(), "Failed to compile: import super");
        let output = result.unwrap();
        assert!(
            output.contains("use super"),
            "Should transpile to 'use super'"
        );
    }

    #[test]
    fn test_import_crate() {
        let result = compile("import crate.utils");
        if let Err(e) = &result {
            println!("Import crate.utils error: {e}");
        }
        assert!(result.is_ok(), "Failed to compile: import crate.utils");
        let output = result.unwrap();
        println!("Import crate.utils output: {output}");
        // Check with or without spaces
        let has_import =
            output.contains("use crate::utils") || output.contains("use crate :: utils");
        assert!(
            has_import,
            "Should transpile to 'use crate::utils', got: {output}"
        );
    }

    // Error cases (should give meaningful errors)
    #[test]
    fn test_import_empty_from() {
        let result = compile("from import something");
        assert!(result.is_err(), "Should fail: from import something");
    }

    #[test]
    fn test_import_missing_target() {
        let result = compile("from std.collections import");
        assert!(result.is_err(), "Should fail: missing import target");
    }

    #[test]
    fn test_import_invalid_syntax() {
        let result = compile("import from std");
        assert!(result.is_err(), "Should fail: invalid syntax");
    }
}

#[cfg(test)]
mod import_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_import_paths_never_panic(path in "[a-zA-Z_][a-zA-Z0-9_]{0,20}(\\.[a-zA-Z_][a-zA-Z0-9_]{0,20}){0,5}") {
            let code = format!("import {path}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_from_import_never_panic(
            module in "[a-zA-Z_][a-zA-Z0-9_]{0,20}(\\.[a-zA-Z_][a-zA-Z0-9_]{0,20}){0,3}",
            item in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let code = format!("from {module} import {item}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_import_alias_never_panic(
            path in "[a-zA-Z_][a-zA-Z0-9_]{0,20}(\\.[a-zA-Z_][a-zA-Z0-9_]{0,20}){0,3}",
            alias in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let code = format!("import {path} as {alias}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_multiple_imports_never_panic(
            items in prop::collection::vec("[a-zA-Z_][a-zA-Z0-9_]{0,20}", 1..10)
        ) {
            let imports = items.join(", ");
            let code = format!("from std.collections import {imports}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_deeply_nested_imports(depth in 1..20usize) {
            let path = (0..depth)
                .map(|i| format!("mod{i}"))
                .collect::<Vec<_>>()
                .join(".");
            let code = format!("import {path}");
            let _ = compile(&code); // Should not panic
        }
    }
}

#[cfg(test)]
mod import_integration_tests {
    use super::*;

    #[test]
    #[ignore = "Import feature not fully implemented yet"]
    fn test_import_with_code() {
        let code = r#"
import std.collections.HashMap

fn main() {
    let mut map = HashMap::new()
    map.insert("key", "value")
    println(map)
}
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile import with usage");
        let output = result.unwrap();
        assert!(
            output.contains("use std::collections::HashMap"),
            "Should have import"
        );
        assert!(
            output.contains("HashMap::new()"),
            "Should use imported type"
        );
    }

    #[test]
    #[ignore = "Import feature not fully implemented yet"]
    fn test_selective_imports() {
        let code = r"
from std.collections import HashMap, HashSet
from std.sync import Arc, Mutex

fn main() {
    let map = HashMap::new()
    let set = HashSet::new()
    let arc = Arc::new(42)
    let mutex = Mutex::new(0)
}
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile selective imports");
        let output = result.unwrap();
        assert!(output.contains("use std::collections::{HashMap, HashSet}"));
        assert!(output.contains("use std::sync::{Arc, Mutex}"));
    }

    #[test]
    fn test_import_resolution_order() {
        let code = r"
import std
import std.collections.HashMap

fn main() {
    // Should resolve to most specific import
    let map = HashMap::new()
}
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile with import resolution");
    }
}
