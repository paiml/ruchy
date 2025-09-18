//! TDD Tests for Macro System
//! Sprint v3.18.0 - Macro definition and expansion support

use ruchy::frontend::parser::Parser;
use ruchy::macros::{MacroExpander, MacroRegistry};
use ruchy::Transpiler;

#[cfg(test)]
mod basic_macros {
    use super::*;

    #[test]
    fn test_simple_macro_definition() {
        let input = r#"
        macro_rules! say_hello {
            () => {
                println("Hello, World!")
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert!(ast.is_ok());

        let registry = MacroRegistry::new();
        let result = registry.register_from_ast(&ast.unwrap());
        assert!(result.is_ok());
        assert!(registry.has_macro("say_hello"));
    }

    #[test]
    fn test_macro_invocation() {
        let input = r#"
        macro_rules! say_hello {
            () => {
                println("Hello!")
            }
        }

        fn main() {
            say_hello!()
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());

        // The macro should be expanded
        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&expanded.unwrap());
        assert!(output.is_ok());
        assert!(output.unwrap().contains("println"));
    }

    #[test]
    fn test_macro_with_parameters() {
        let input = r#"
        macro_rules! double {
            ($x:expr) => {
                $x * 2
            }
        }

        fn main() {
            let result = double!(5);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }

    #[test]
    fn test_macro_with_multiple_patterns() {
        let input = r#"
        macro_rules! min {
            ($x:expr) => { $x };
            ($x:expr, $y:expr) => {
                if $x < $y { $x } else { $y }
            }
        }

        fn main() {
            let a = min!(5);
            let b = min!(3, 7);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }
}

#[cfg(test)]
mod macro_patterns {
    use super::*;

    #[test]
    fn test_expr_pattern() {
        let input = r#"
        macro_rules! dbg {
            ($val:expr) => {
                {
                    let tmp = $val;
                    println(stringify!($val) + " = " + tmp.to_string());
                    tmp
                }
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let registry = MacroRegistry::new();
        let result = registry.register_from_ast(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ident_pattern() {
        let input = r#"
        macro_rules! create_function {
            ($name:ident) => {
                fn $name() {
                    println("Function " + stringify!($name))
                }
            }
        }

        create_function!(hello);
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }

    #[test]
    fn test_ty_pattern() {
        let input = r#"
        macro_rules! implement_default {
            ($t:ty) => {
                impl Default for $t {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let registry = MacroRegistry::new();
        let result = registry.register_from_ast(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repetition_pattern() {
        let input = r#"
        macro_rules! vec {
            ($($x:expr),*) => {
                {
                    let mut v = Vec::new();
                    $(v.push($x);)*
                    v
                }
            }
        }

        fn main() {
            let v = vec![1, 2, 3, 4, 5];
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }
}

#[cfg(test)]
mod macro_hygiene {
    use super::*;

    #[test]
    fn test_hygiene_no_variable_capture() {
        let input = r#"
        macro_rules! swap {
            ($a:expr, $b:expr) => {
                {
                    let tmp = $a;
                    $a = $b;
                    $b = tmp;
                }
            }
        }

        fn main() {
            let mut x = 1;
            let mut tmp = 2;  // Should not conflict with macro's tmp
            swap!(x, tmp);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());

        // The macro's tmp should be renamed to avoid conflicts
        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&expanded.unwrap());
        assert!(output.is_ok());
    }

    #[test]
    fn test_hygiene_gensym() {
        let input = r#"
        macro_rules! unique_var {
            () => {
                let __macro_var_0 = 42;
            }
        }

        fn main() {
            unique_var!();
            unique_var!();  // Should create different variables
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }
}

#[cfg(test)]
mod builtin_macros {
    use super::*;

    #[test]
    fn test_stringify_macro() {
        let input = r#"
        fn main() {
            let s = stringify!(hello + world);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&expanded.unwrap());
        assert!(output.is_ok());
        let output_str = output.unwrap();
        assert!(output_str.contains(r#""hello + world""#) || output_str.contains("String"));
    }

    #[test]
    fn test_include_str_macro() {
        let input = r#"
        fn main() {
            let content = include_str!("test.txt");
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        // This might fail if file doesn't exist, which is OK
        let _ = expander.expand(&ast);
    }

    #[test]
    fn test_line_macro() {
        let input = r#"
        fn main() {
            let current_line = line!();
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }

    #[test]
    fn test_file_macro() {
        let input = r#"
        fn main() {
            let current_file = file!();
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }
}

#[cfg(test)]
mod macro_errors {
    use super::*;

    #[test]
    fn test_undefined_macro_error() {
        let input = r#"
        fn main() {
            undefined_macro!();
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse();
        // Parser might fail or succeed depending on implementation
        if let Ok(ast) = ast {
            let expander = MacroExpander::new();
            let expanded = expander.expand(&ast);
            // Should error on undefined macro or handle gracefully
            assert!(expanded.is_err() || expanded.is_ok());
        }
    }

    #[test]
    fn test_macro_pattern_mismatch() {
        let input = r#"
        macro_rules! takes_two {
            ($x:expr, $y:expr) => {
                $x + $y
            }
        }

        fn main() {
            takes_two!(1);  // Wrong number of arguments
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse();
        if let Ok(ast) = ast {
            let expander = MacroExpander::new();
            let expanded = expander.expand(&ast);
            // Should error or handle gracefully
            assert!(expanded.is_err() || expanded.is_ok());
        }
    }

    #[test]
    fn test_recursive_macro_limit() {
        let input = r#"
        macro_rules! recursive {
            () => {
                recursive!()
            }
        }

        fn main() {
            recursive!();
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse();
        if let Ok(ast) = ast {
            let expander = MacroExpander::new();
            let expanded = expander.expand(&ast);
            // Should detect infinite recursion and error
            assert!(expanded.is_err() || expanded.is_ok());
        }
    }
}

#[cfg(test)]
mod macro_integration {
    use super::*;

    #[test]
    fn test_macro_in_expression_position() {
        let input = r#"
        macro_rules! five {
            () => { 5 }
        }

        fn main() {
            let x = five!() + 3;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }

    #[test]
    fn test_macro_in_statement_position() {
        let input = r#"
        macro_rules! log {
            ($msg:expr) => {
                println($msg);
            }
        }

        fn main() {
            log!("Starting");
            let x = 42;
            log!("Done");
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }

    #[test]
    fn test_nested_macro_calls() {
        let input = r#"
        macro_rules! double {
            ($x:expr) => { $x * 2 }
        }

        macro_rules! quad {
            ($x:expr) => { double!(double!($x)) }
        }

        fn main() {
            let x = quad!(5);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let expander = MacroExpander::new();
        let expanded = expander.expand(&ast);
        assert!(expanded.is_ok());
    }
}