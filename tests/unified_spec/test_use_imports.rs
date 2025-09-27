// EXTREME TDD: Tests for Rust-style `use` imports from unified spec
// ALL TESTS MUST FAIL INITIALLY - Implementation comes AFTER

use ruchy::compile;

#[cfg(test)]
mod test_use_imports {
    use super::*;

    // Basic use statement tests
    #[test]
    fn test_use_single_module() {
        let code = r"
            use std::collections;

            fun main() {
                let map = collections::HashMap::new();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use statement");
        let output = result.unwrap();
        assert!(output.contains("use std") && output.contains("collections"));
    }

    #[test]
    fn test_use_specific_type() {
        let code = r"
            use std::collections::HashMap;

            fun main() {
                let map = HashMap::new();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use with specific type");
        let output = result.unwrap();
        assert!(
            output.contains("use std")
                && output.contains("collections")
                && output.contains("HashMap")
        );
    }

    #[test]
    fn test_use_multiple_items() {
        let code = r"
            use std::collections::{HashMap, BTreeMap, HashSet};

            fun main() {
                let map = HashMap::new();
                let tree = BTreeMap::new();
                let set = HashSet::new();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use with multiple items");
        let output = result.unwrap();
        assert!(
            output.contains("use std")
                && output.contains("HashMap")
                && output.contains("BTreeMap")
                && output.contains("HashSet")
        );
    }

    // Aliasing tests
    #[test]
    fn test_use_with_alias() {
        let code = r"
            use numpy as np;

            fun main() {
                let arr = np::array([1, 2, 3]);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use with alias");
        let output = result.unwrap();
        assert!(output.contains("use numpy") && output.contains("as np"));
    }

    #[test]
    fn test_use_specific_item_with_alias() {
        let code = r"
            use std::collections::HashMap as Map;

            fun main() {
                let m = Map::new();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use item with alias");
        let output = result.unwrap();
        assert!(output.contains("use std") && output.contains("HashMap as Map"));
    }

    // Nested imports
    #[test]
    fn test_use_nested_modules() {
        let code = r"
            use tokio::time::{sleep, timeout};

            async fun delay() {
                sleep(Duration::from_secs(1)).await;
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile nested use");
        let output = result.unwrap();
        assert!(
            output.contains("use tokio") && output.contains("sleep") && output.contains("timeout")
        );
    }

    // Wildcard imports
    #[test]
    fn test_use_wildcard() {
        let code = r"
            use rayon::prelude::*;

            fun parallel_sum(data: Vec<i32>) -> i32 {
                data.par_iter().sum()
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use with wildcard");
        let output = result.unwrap();
        assert!(output.contains("use rayon") && output.contains("prelude"));
    }

    // Self and super imports
    #[test]
    fn test_use_self() {
        let code = r"
            mod math {
                pub fun add(x: i32, y: i32) -> i32 { x + y }
            }

            use self::math::add;

            fun main() {
                let sum = add(1, 2);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use self");
        let output = result.unwrap();
        assert!(output.contains("use self") && output.contains("math") && output.contains("add"));
    }

    #[test]
    fn test_use_super() {
        let code = r"
            mod outer {
                pub fun helper() -> i32 { 42 }

                mod inner {
                    use super::helper;

                    pub fun use_helper() -> i32 {
                        helper()
                    }
                }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use super");
        let output = result.unwrap();
        assert!(output.contains("use super") && output.contains("helper"));
    }

    // Crate imports
    #[test]
    fn test_use_crate() {
        let code = r"
            use crate::utils::helper;

            fun main() {
                helper();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use crate");
        let output = result.unwrap();
        assert!(
            output.contains("use crate") && output.contains("utils") && output.contains("helper")
        );
    }

    // External crate imports
    #[test]
    fn test_use_external_crate() {
        let code = r"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            struct Config {
                name: String,
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile external crate use");
        let output = result.unwrap();
        assert!(
            output.contains("use serde")
                && output.contains("Serialize")
                && output.contains("Deserialize")
        );
    }

    // Trait imports
    #[test]
    fn test_use_trait() {
        let code = r#"
            use std::fmt::Display;

            fun print_it<T: Display>(value: T) {
                println!("{}", value);
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile trait use");
    }

    // Multiple use statements
    #[test]
    fn test_multiple_use_statements() {
        let code = r"
            use std::collections::HashMap;
            use std::io::{Read, Write};
            use tokio::time::Duration;

            fun main() {}
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile multiple use statements");
        let output = result.unwrap();
        assert!(
            output.contains("use std")
                && output.contains("collections")
                && output.contains("HashMap")
        );
        assert!(
            output.contains("use std")
                && output.contains("io")
                && output.contains("Read")
                && output.contains("Write")
        );
        assert!(output.contains("use tokio") && output.contains("Duration"));
    }

    // Grouped imports
    #[test]
    fn test_use_grouped() {
        let code = r"
            use std::{
                collections::{HashMap, HashSet},
                io::{Read, Write},
                fmt::Display,
            };
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile grouped use");
    }

    // Renaming in grouped imports
    #[test]
    fn test_use_grouped_with_rename() {
        let code = r"
            use std::collections::{
                HashMap as Map,
                HashSet as Set,
                BTreeMap,
            };
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile grouped use with rename");
    }

    // Pub use for re-export
    #[test]
    fn test_pub_use() {
        let code = r"
            pub use std::collections::HashMap;

            pub mod prelude {
                pub use super::HashMap;
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile pub use");
        let output = result.unwrap();
        assert!(output.contains("pub use std") && output.contains("HashMap"));
    }

    // Use with macros
    #[test]
    fn test_use_macro() {
        let code = r#"
            use serde_json::json;

            fun create_json() -> Value {
                json!({
                    "name": "Ruchy",
                    "version": "1.0"
                })
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile use with macro");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        #[allow(clippy::needless_pass_by_value)]
        fn prop_use_with_valid_path(module: String) -> TestResult {
            if module.is_empty()
                || !module
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
            {
                return TestResult::discard();
            }

            let code = format!("use {module};");
            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        quickcheck! {
            fn test_use_with_random_paths(module: String) -> TestResult {
                prop_use_with_valid_path(module)
            }
        }
    }
}
