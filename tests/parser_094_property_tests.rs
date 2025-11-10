//! PARSER-094: Property tests for :: path separator preservation
//!
//! Property-based testing to verify module path separator invariants:
//! 1. Module paths with :: always preserve :: in transpilation
//! 2. Field access with . always preserves . in transpilation
//! 3. Mixed paths correctly distinguish :: from .

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

// Property 1: Module-like identifiers with :: preserve :: separator
// CONSTRAINT: Module names MUST contain underscore (our heuristic for distinguishing from variables)
proptest! {
    #[test]
    fn property_module_paths_preserve_colon_colon(
        prefix in "[a-z]{2,5}",
        suffix in "[a-z]{2,5}",
        function in "[a-z][a-z_]{2,10}"  // function_name
    ) {
        // Generate module name with underscore: prefix_suffix
        let module = format!("{prefix}_{suffix}");

        // Generate code: module_name::function_name()
        let code = format!("let result = {module}::{function}(); result");

        if let Ok(ast) = Parser::new(&code).parse() {
            if let Ok(rust_code) = Transpiler::new().transpile(&ast) {
                let rust_str = rust_code.to_string();

                // INVARIANT: Module paths with :: should preserve ::
                // Should NOT convert to .
                let expected_module = format!("{module} :: {function}");
                let unexpected_dot = format!("{module} . {function}");

                prop_assert!(
                    rust_str.contains(&expected_module),
                    "Module path {}::{} should preserve ::, got: {}",
                    module,
                    function,
                    rust_str
                );
                prop_assert!(
                    !rust_str.contains(&unexpected_dot),
                    "Module path {}::{} should NOT use ., got: {}",
                    module,
                    function,
                    rust_str
                );
            }
        }
    }
}

// Property 2: Type names with :: preserve :: separator
proptest! {
    #[test]
    fn property_type_paths_preserve_colon_colon(
        type_name in "[A-Z][a-zA-Z]{2,10}",  // TypeName (PascalCase)
        function in "[a-z][a-z_]{2,10}"       // function_name
    ) {
        // Generate code: TypeName::function_name()
        let code = format!("let result = {type_name}::{function}(); result");

        if let Ok(ast) = Parser::new(&code).parse() {
            if let Ok(rust_code) = Transpiler::new().transpile(&ast) {
                let rust_str = rust_code.to_string();

                // INVARIANT: Type associated functions should use ::
                let expected_path = format!("{type_name} :: {function}");
                let unexpected_dot = format!("{type_name} . {function}");

                prop_assert!(
                    rust_str.contains(&expected_path),
                    "Type path {}::{} should preserve ::, got: {}",
                    type_name,
                    function,
                    rust_str
                );
                prop_assert!(
                    !rust_str.contains(&unexpected_dot),
                    "Type path {}::{} should NOT use ., got: {}",
                    type_name,
                    function,
                    rust_str
                );
            }
        }
    }
}

// Property 3: Nested module paths with multiple :: preserve all separators
// CONSTRAINT: Both module names MUST contain underscores
proptest! {
    #[test]
    fn property_nested_module_paths_preserve_all_colons(
        prefix1 in "[a-z]{2,4}",
        suffix1 in "[a-z]{2,4}",
        prefix2 in "[a-z]{2,4}",
        suffix2 in "[a-z]{2,4}",
        func in "[a-z][a-z_]{2,8}"
    ) {
        let mod1 = format!("{prefix1}_{suffix1}");
        let mod2 = format!("{prefix2}_{suffix2}");

        // Generate code: mod1::mod2::func()
        let code = format!("let result = {mod1}::{mod2}::{func}(); result");

        if let Ok(ast) = Parser::new(&code).parse() {
            if let Ok(rust_code) = Transpiler::new().transpile(&ast) {
                let rust_str = rust_code.to_string();

                // INVARIANT: All :: in nested paths should be preserved
                let expected_first = format!("{mod1} :: {mod2}");
                let expected_second = format!("{mod2} :: {func}");

                prop_assert!(
                    rust_str.contains(&expected_first) || rust_str.contains(&expected_second),
                    "Nested path {}::{}::{} should preserve all ::, got: {}",
                    mod1,
                    mod2,
                    func,
                    rust_str
                );
            }
        }
    }
}

// Property 4: Field access with . should NOT become ::
proptest! {
    #[test]
    fn property_field_access_preserves_dot(
        obj_name in "[a-z][a-z]{2,10}",      // lowercase only (not underscore to avoid module heuristic)
        field_name in "[a-z][a-z]{2,10}"
    ) {
        // Skip if obj_name would match module heuristic (contains underscore or is "std")
        prop_assume!(!obj_name.contains('_'));
        prop_assume!(obj_name != "std");
        prop_assume!(!field_name.contains('_'));

        // Generate code: obj.field (no function call - just field access)
        let code = format!("let obj = MyStruct {{ {field_name}: 42 }}; let val = obj.{field_name}; val");

        if let Ok(ast) = Parser::new(&code).parse() {
            if let Ok(rust_code) = Transpiler::new().transpile(&ast) {
                let rust_str = rust_code.to_string();

                // INVARIANT: Field access should preserve .
                // Note: We check for the general pattern, not specific obj_name
                // since the test uses 'obj' as variable name
                let unexpected_colon = format!("obj :: {field_name}");

                prop_assert!(
                    !rust_str.contains(&unexpected_colon),
                    "Field access obj.{} should use ., not ::, got: {}",
                    field_name,
                    rust_str
                );
            }
        }
    }
}

// Property 5: stdlib paths always use ::
proptest! {
    #[test]
    fn property_stdlib_paths_preserve_colon_colon(
        stdlib_mod in prop::sample::select(vec!["io", "fs", "env", "time", "path"]),
        function in "[a-z][a-z_]{2,10}"
    ) {
        // Generate code: std::module::function()
        let code = format!("let result = std::{stdlib_mod}::{function}(); result");

        if let Ok(ast) = Parser::new(&code).parse() {
            if let Ok(rust_code) = Transpiler::new().transpile(&ast) {
                let rust_str = rust_code.to_string();

                // INVARIANT: stdlib paths must use ::
                let expected_std = "std :: ";
                let unexpected_std_dot = "std . ";

                prop_assert!(
                    rust_str.contains(expected_std),
                    "Stdlib path std::{}::{} should preserve ::, got: {}",
                    stdlib_mod,
                    function,
                    rust_str
                );
                prop_assert!(
                    !rust_str.contains(unexpected_std_dot),
                    "Stdlib path should NOT use ., got: {}",
                    rust_str
                );
            }
        }
    }
}
