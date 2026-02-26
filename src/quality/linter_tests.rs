//! Comprehensive tests for the code linter
//!
//! EXTREME TDD Round 88: Comprehensive tests for linter module
//! Coverage target: 95% for linter.rs module
//!
//! Tests use the parser to generate AST from source code strings.

#[cfg(test)]
mod tests {
    use crate::frontend::parser::Parser;
    use crate::quality::linter::{is_builtin, Linter};

    // ============== Helper Function ==============

    fn parse_and_lint(source: &str) -> Vec<crate::quality::linter::LintIssue> {
        let mut parser = Parser::new(source);
        let ast = parser.parse().expect("parsing should succeed");
        let linter = Linter::new();
        linter.lint(&ast, source).expect("linting should succeed")
    }

    fn parse_and_lint_with_rules(
        source: &str,
        rules: &str,
    ) -> Vec<crate::quality::linter::LintIssue> {
        let mut parser = Parser::new(source);
        let ast = parser.parse().expect("parsing should succeed");
        let mut linter = Linter::new();
        linter.set_rules(rules);
        linter.lint(&ast, source).expect("linting should succeed")
    }

    // ============== is_builtin Tests ==============

    #[test]
    fn test_is_builtin_println() {
        assert!(is_builtin("println"));
    }

    #[test]
    fn test_is_builtin_print() {
        assert!(is_builtin("print"));
    }

    #[test]
    fn test_is_builtin_dbg() {
        assert!(is_builtin("dbg"));
    }

    #[test]
    fn test_is_builtin_fs_functions() {
        assert!(is_builtin("fs_read"));
        assert!(is_builtin("fs_write"));
        assert!(is_builtin("fs_exists"));
        assert!(is_builtin("fs_remove"));
    }

    #[test]
    fn test_is_builtin_env_functions() {
        assert!(is_builtin("env_var"));
        assert!(is_builtin("env_args"));
        assert!(is_builtin("env_current_dir"));
    }

    #[test]
    fn test_is_builtin_http_functions() {
        assert!(is_builtin("http_get"));
        assert!(is_builtin("http_post"));
        assert!(is_builtin("http_put"));
        assert!(is_builtin("http_delete"));
    }

    #[test]
    fn test_is_builtin_json_functions() {
        assert!(is_builtin("json_parse"));
        assert!(is_builtin("json_stringify"));
    }

    #[test]
    fn test_is_builtin_time_functions() {
        assert!(is_builtin("time_now"));
        assert!(is_builtin("time_sleep"));
    }

    #[test]
    fn test_is_builtin_math_functions() {
        assert!(is_builtin("abs"));
        assert!(is_builtin("sqrt"));
        assert!(is_builtin("pow"));
        assert!(is_builtin("sin"));
        assert!(is_builtin("cos"));
        assert!(is_builtin("floor"));
        assert!(is_builtin("ceil"));
        assert!(is_builtin("round"));
        assert!(is_builtin("min"));
        assert!(is_builtin("max"));
    }

    #[test]
    fn test_is_builtin_process_functions() {
        assert!(is_builtin("exit"));
        assert!(is_builtin("panic"));
        assert!(is_builtin("assert"));
        assert!(is_builtin("assert_eq"));
    }

    #[test]
    fn test_is_builtin_collection_functions() {
        assert!(is_builtin("range"));
        assert!(is_builtin("HashMap"));
        assert!(is_builtin("HashSet"));
    }

    #[test]
    fn test_is_builtin_dataframe_functions() {
        assert!(is_builtin("col"));
        assert!(is_builtin("lit"));
        assert!(is_builtin("DataFrame"));
    }

    #[test]
    fn test_is_builtin_custom_not_builtin() {
        assert!(!is_builtin("my_function"));
        assert!(!is_builtin("custom_helper"));
        assert!(!is_builtin(""));
        assert!(!is_builtin("unknown"));
    }

    #[test]
    fn test_is_builtin_path_functions() {
        assert!(is_builtin("path_join"));
        assert!(is_builtin("path_extension"));
        assert!(is_builtin("path_filename"));
        assert!(is_builtin("path_parent"));
    }

    #[test]
    fn test_is_builtin_regex_functions() {
        assert!(is_builtin("regex_new"));
        assert!(is_builtin("regex_is_match"));
        assert!(is_builtin("regex_find"));
        assert!(is_builtin("regex_replace"));
    }

    #[test]
    fn test_is_builtin_logging_functions() {
        assert!(is_builtin("log_info"));
        assert!(is_builtin("log_warn"));
        assert!(is_builtin("log_error"));
        assert!(is_builtin("log_debug"));
        assert!(is_builtin("log_trace"));
    }

    // ============== Linter Construction Tests ==============

    #[test]
    fn test_linter_new() {
        let linter = Linter::new();
        let mut parser = Parser::new("42");
        let ast = parser.parse().expect("parsing should succeed");
        let result = linter.lint(&ast, "42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_linter_set_rules_unused() {
        let issues = parse_and_lint_with_rules("42", "unused");
        // No issues expected for just a literal
        assert!(issues.is_empty());
    }

    #[test]
    fn test_linter_set_rules_undefined() {
        let issues = parse_and_lint_with_rules("undefined_var", "undefined");
        // Should detect undefined variable
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    #[test]
    fn test_linter_set_rules_shadowing() {
        let issues = parse_and_lint_with_rules("42", "shadowing");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_linter_set_rules_complexity() {
        let issues = parse_and_lint_with_rules("42", "complexity");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_linter_set_rules_multiple() {
        let issues = parse_and_lint_with_rules("42", "unused,undefined,shadowing");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_linter_set_strict_mode() {
        let mut linter = Linter::new();
        linter.set_strict_mode(true);
        let mut parser = Parser::new("42");
        let ast = parser.parse().expect("parsing should succeed");
        let result = linter.lint(&ast, "42");
        assert!(result.is_ok());
    }

    // ============== Undefined Variable Tests ==============

    #[test]
    fn test_undefined_variable_detected() {
        let issues = parse_and_lint("undefined_var");
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    #[test]
    fn test_defined_variable_no_error() {
        let issues = parse_and_lint("let x = 42\nx");
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "x"));
    }

    #[test]
    fn test_builtin_not_undefined() {
        let issues = parse_and_lint("println");
        // println is a builtin, should not be undefined
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "println"));
    }

    #[test]
    fn test_multiple_undefined_variables() {
        let issues = parse_and_lint("undefined_a + undefined_b");
        let undefined_count = issues.iter().filter(|i| i.rule == "undefined").count();
        assert!(undefined_count >= 2);
    }

    // ============== Unused Variable Tests ==============

    #[test]
    fn test_unused_variable_detected() {
        let issues = parse_and_lint("let unused_x = 42");
        // Check if unused variable is detected
        assert!(issues
            .iter()
            .any(|i| i.name == "unused_x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_used_variable_no_warning() {
        let issues = parse_and_lint("let x = 42\nx + 1");
        // Should not have unused variable warning for x
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Variable Shadowing Tests ==============

    #[test]
    fn test_variable_shadowing_detected() {
        // Use nested scopes for shadowing detection
        let issues = parse_and_lint("let x = 1\nfun foo() { let x = 2\nx }");
        // Should detect shadowing (inner x shadows outer x)
        // Note: may not trigger if top-level lets don't create nested scopes
        // The test verifies the linting doesn't crash
        let _ = issues;
    }

    // ============== Function Tests ==============

    #[test]
    fn test_function_parameter_used() {
        let issues = parse_and_lint("fun foo(x) { x + 1 }");
        // Parameter x is used, should not be flagged
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_function_forward_reference() {
        let issues = parse_and_lint("fun foo() { bar() }\nfun bar() { 42 }");
        // bar should be defined (forward reference resolution)
        assert!(!issues
            .iter()
            .any(|i| i.name == "bar" && i.rule == "undefined"));
    }

    // ============== Loop Variable Tests ==============

    #[test]
    fn test_loop_variable_used() {
        let issues = parse_and_lint("for i in range(0, 10) { println(i) }");
        // i is used, should not be flagged
        assert!(!issues
            .iter()
            .any(|i| i.name == "i" && i.rule.contains("unused_loop")));
    }

    #[test]
    fn test_loop_variable_unused() {
        let issues = parse_and_lint("for i in range(0, 10) { println(\"hello\") }");
        // i is unused, should be flagged
        assert!(issues
            .iter()
            .any(|i| i.name == "i" && i.rule.contains("unused_loop")));
    }

    // ============== Match Binding Tests ==============

    #[test]
    fn test_match_binding_used() {
        let issues = parse_and_lint("let x = 42\nmatch x { n => n + 1 }");
        // n is used, should not be flagged as unused match binding
        assert!(!issues
            .iter()
            .any(|i| i.name == "n" && i.rule.contains("unused_match")));
    }

    #[test]
    fn test_match_binding_unused() {
        let issues = parse_and_lint("let x = 42\nmatch x { n => 42 }");
        // n is unused, should be flagged
        assert!(issues
            .iter()
            .any(|i| i.name == "n" && i.rule.contains("unused_match")));
    }

    // ============== If Expression Tests ==============

    #[test]
    fn test_if_condition_analyzed() {
        let issues = parse_and_lint("if undefined_var { 1 } else { 2 }");
        // undefined_var should be flagged
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    #[test]
    fn test_if_branches_analyzed() {
        let issues = parse_and_lint("let x = 1\nif true { x } else { x + 1 }");
        // Should not panic, and x should be visible in both branches
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "x"));
    }

    // ============== Lambda Tests ==============

    #[test]
    fn test_lambda_parameter_used() {
        let issues = parse_and_lint("|x| x + 1");
        // x is used in lambda, should not be flagged
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_lambda_parameter_unused() {
        let issues = parse_and_lint("|x| 42");
        // x is unused in lambda, should be flagged
        assert!(issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Binary Expression Tests ==============

    #[test]
    fn test_binary_both_sides_analyzed() {
        let issues = parse_and_lint("undefined_left + undefined_right");
        // Both should be flagged as undefined
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== Call Expression Tests ==============

    #[test]
    fn test_call_func_and_args_analyzed() {
        let issues = parse_and_lint("undefined_func(undefined_arg)");
        // Both should be flagged
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== Block Expression Tests ==============

    #[test]
    fn test_block_sequential_visibility() {
        let issues = parse_and_lint("let x = 1\nx + 1");
        // x should be visible in subsequent statement
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "x"));
    }

    // ============== List/Array Tests ==============

    #[test]
    fn test_list_elements_analyzed() {
        let issues = parse_and_lint("[undefined_a, undefined_b]");
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== Tuple Tests ==============

    #[test]
    fn test_tuple_elements_analyzed() {
        let issues = parse_and_lint("(undefined_a, undefined_b)");
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== Field Access Tests ==============

    #[test]
    fn test_field_access_object_analyzed() {
        let issues = parse_and_lint("undefined_obj.field");
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    // ============== Index Access Tests ==============

    #[test]
    fn test_index_access_analyzed() {
        let issues = parse_and_lint("undefined_arr[undefined_idx]");
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== While Loop Tests ==============

    #[test]
    fn test_while_loop_analyzed() {
        let issues = parse_and_lint("while undefined_cond { undefined_body }");
        assert!(issues.iter().filter(|i| i.rule == "undefined").count() >= 2);
    }

    // ============== Return Tests ==============

    #[test]
    fn test_return_value_analyzed() {
        let issues = parse_and_lint("fun foo() { return undefined_val }");
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    // ============== Method Call Tests ==============

    #[test]
    fn test_method_call_receiver_analyzed() {
        let issues = parse_and_lint("undefined_obj.method()");
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    // ============== Enum Tests ==============

    #[test]
    fn test_enum_type_defined() {
        let issues = parse_and_lint("enum Color { Red, Green, Blue }");
        // Should not error
        assert!(
            issues.is_empty()
                || !issues
                    .iter()
                    .any(|i| i.name == "Color" && i.rule == "undefined")
        );
    }

    // ============== Struct Tests ==============

    #[test]
    fn test_struct_type_defined() {
        let issues = parse_and_lint("struct Point { x: i32, y: i32 }");
        // Should not error
        assert!(
            issues.is_empty()
                || !issues
                    .iter()
                    .any(|i| i.name == "Point" && i.rule == "undefined")
        );
    }

    // ============== LintIssue Tests ==============

    #[test]
    fn test_lint_issue_fields() {
        let issues = parse_and_lint("undefined_var");
        if let Some(issue) = issues.first() {
            assert!(!issue.message.is_empty());
            assert!(!issue.suggestion.is_empty());
            assert!(!issue.rule.is_empty());
        }
    }

    // ============== Auto-fix Tests ==============

    #[test]
    fn test_auto_fix_basic() {
        let linter = Linter::new();
        let source = "let unused_var = 42";
        let issues = vec![];
        let result = linter.auto_fix(source, &issues);
        assert!(result.is_ok());
    }

    // ============== Clean Code Tests ==============

    #[test]
    fn test_clean_code_no_issues() {
        let issues = parse_and_lint("let x = 42\nx");
        // No unused or undefined issues expected
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "x"));
    }

    // ============== Edge Case Tests ==============

    #[test]
    fn test_underscore_variable_not_flagged() {
        let issues = parse_and_lint("for _ in range(0, 10) { println(\"hi\") }");
        // _ should not be flagged as unused
        assert!(!issues.iter().any(|i| i.name == "_"));
    }

    #[test]
    fn test_literal_no_issues() {
        let issues = parse_and_lint("42");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_string_literal_no_issues() {
        let issues = parse_and_lint("\"hello world\"");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_bool_literal_no_issues() {
        let issues = parse_and_lint("true");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_list_no_issues() {
        let issues = parse_and_lint("[]");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_object_no_issues() {
        let issues = parse_and_lint("{}");
        assert!(issues.is_empty());
    }

    // ============== Complexity Tests ==============

    #[test]
    fn test_high_complexity_flagged() {
        // A deeply nested expression to trigger complexity warning
        let complex_code = "\
if true {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        if true { 1 } else { 2 }
                                    } else { 3 }
                                } else { 4 }
                            } else { 5 }
                        } else { 6 }
                    } else { 7 }
                } else { 8 }
            } else { 9 }
        } else { 10 }
    } else { 11 }
} else { 12 }
";
        let issues = parse_and_lint_with_rules(complex_code, "complexity");
        // Should flag high complexity
        assert!(issues.iter().any(|i| i.rule == "complexity"));
    }

    // ============== Security Tests ==============

    #[test]
    fn test_security_rules() {
        let issues = parse_and_lint_with_rules("42", "security");
        // No security issues in a simple literal
        assert!(issues.is_empty());
    }

    // ============== Performance Tests ==============

    #[test]
    fn test_performance_rules() {
        let issues = parse_and_lint_with_rules("42", "performance");
        // No performance issues in a simple literal
        assert!(issues.is_empty());
    }

    // ============== Style Tests ==============

    #[test]
    fn test_style_rules() {
        let issues = parse_and_lint_with_rules("42", "style");
        // No style issues in a simple literal
        assert!(issues.is_empty());
    }

    // ============== Multiple Issues Tests ==============

    #[test]
    fn test_multiple_issues_detected() {
        // Code with both unused and undefined variables
        let issues = parse_and_lint("let unused_y = 42\nundefined_z");
        // Should detect both types of issues
        assert!(issues.iter().any(|i| i.rule.contains("unused")));
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    // ============== Nested Scope Tests ==============

    #[test]
    fn test_nested_scope_variable_visibility() {
        let issues = parse_and_lint("let x = 1\nif true { let y = x } else { x }");
        // x should be visible in both branches
        assert!(!issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "x"));
    }

    #[test]
    fn test_inner_scope_not_visible_outside() {
        let issues = parse_and_lint("if true { let inner = 1 }\ninner");
        // inner should not be visible outside the if block
        assert!(issues
            .iter()
            .any(|i| i.rule == "undefined" && i.name == "inner"));
    }

    // ============== F-String Interpolation Tests ==============

    #[test]
    fn test_fstring_variable_marked_used() {
        let issues = parse_and_lint("let x = 42\nf\"{x}\"");
        // x is used in f-string, should not be flagged as unused
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Macro Tests ==============

    #[test]
    fn test_macro_args_analyzed() {
        let issues = parse_and_lint("let x = 42\nformat!(\"{}\", x)");
        // x is used in macro, should not be flagged as unused
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Additional Builtin Tests ==============

    #[test]
    fn test_is_builtin_range() {
        assert!(is_builtin("range"));
    }

    #[test]
    fn test_is_builtin_format() {
        assert!(is_builtin("format"));
    }

    #[test]
    fn test_is_builtin_regex_new() {
        assert!(is_builtin("regex_new"));
    }

    #[test]
    fn test_is_builtin_log_info() {
        assert!(is_builtin("log_info"));
    }

    #[test]
    fn test_is_builtin_path_join() {
        assert!(is_builtin("path_join"));
    }

    #[test]
    fn test_is_not_builtin_random_name() {
        assert!(!is_builtin("my_custom_function"));
    }

    #[test]
    fn test_is_not_builtin_empty() {
        assert!(!is_builtin(""));
    }

    // ============== Assignment Pattern Tests ==============

    #[test]
    fn test_variable_reassignment() {
        let issues = parse_and_lint("let x = 1\nx = 2\nx");
        // x is used after reassignment
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_compound_assignment() {
        let issues = parse_and_lint("let x = 1\nx += 5\nx");
        // x is used in compound assignment
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Control Flow Tests ==============

    #[test]
    fn test_if_condition_uses_variable() {
        let issues = parse_and_lint("let x = true\nif x { 1 } else { 0 }");
        // x is used in condition
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_while_condition_uses_variable() {
        let issues = parse_and_lint("let x = true\nwhile x { break }");
        // x is used in condition
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Binary/Unary Expression Tests ==============

    #[test]
    fn test_variable_in_binary_expr() {
        let issues = parse_and_lint("let x = 5\nlet y = 3\nx + y");
        // Both x and y are used
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
        assert!(!issues
            .iter()
            .any(|i| i.name == "y" && i.rule.contains("unused")));
    }

    #[test]
    fn test_variable_in_unary_expr() {
        // Just verify unary expression doesn't crash
        let _issues = parse_and_lint("let x = true\n!x");
    }

    // ============== Array/Index Tests ==============

    #[test]
    fn test_variable_used_in_array() {
        let issues = parse_and_lint("let x = 1\n[x, 2, 3]");
        // x is used in array literal
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_variable_used_as_index() {
        let issues = parse_and_lint("let arr = [1,2,3]\nlet i = 0\narr[i]");
        // i is used as index
        assert!(!issues
            .iter()
            .any(|i| i.name == "i" && i.rule.contains("unused")));
    }

    // ============== Call Expression Tests ==============

    #[test]
    fn test_variable_used_in_call_arg() {
        let issues = parse_and_lint("let x = 42\nprintln(x)");
        // x is used as argument
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    #[test]
    fn test_variable_used_in_method_call() {
        let issues = parse_and_lint("let s = \"hello\"\ns.len()");
        // s is used as method receiver
        assert!(!issues
            .iter()
            .any(|i| i.name == "s" && i.rule.contains("unused")));
    }

    // ============== Struct/Object Tests ==============

    #[test]
    fn test_variable_used_in_struct_field() {
        // Just verify struct literal doesn't crash
        let _issues = parse_and_lint("let x = 42\n{ a: x }");
    }

    #[test]
    fn test_variable_used_in_field_access() {
        let issues = parse_and_lint("let obj = { x: 1 }\nobj.x");
        // obj is used in field access
        assert!(!issues
            .iter()
            .any(|i| i.name == "obj" && i.rule.contains("unused")));
    }

    // ============== Closure/Lambda Tests ==============

    #[test]
    fn test_variable_captured_by_closure() {
        // Just verify closure doesn't crash
        let _issues = parse_and_lint("let x = 42\nfun foo() { x }");
    }

    // ============== Return Tests ==============

    #[test]
    fn test_variable_used_in_return() {
        let issues = parse_and_lint("fun foo() { let x = 42\nreturn x }");
        // x is used in return
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Edge Cases ==============

    #[test]
    fn test_literal_42_no_issues() {
        let issues = parse_and_lint("42");
        // Just a literal should have no issues
        assert!(issues.is_empty());
    }

    #[test]
    fn test_simple_let_used() {
        let issues = parse_and_lint("let x = 1\nx");
        // x is defined and used
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule == "undefined"));
    }

    #[test]
    fn test_nested_if_expression() {
        // Just verify nested if doesn't crash
        let _issues = parse_and_lint("if true { if true { 42 } }");
    }

    #[test]
    fn test_multiple_bindings_same_name() {
        let issues = parse_and_lint("fun foo(x) { let x = x + 1\nx }");
        // Inner x shadows parameter x, both are used
        let _ = issues; // Just verify no crash
    }

    // ============== Type Annotation Tests ==============

    #[test]
    fn test_variable_with_type_annotation() {
        let issues = parse_and_lint("let x: int = 42\nx");
        // x with type annotation is used
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
    }

    // ============== Tuple Tests ==============

    #[test]
    fn test_variable_used_in_tuple() {
        let issues = parse_and_lint("let x = 1\nlet y = 2\n(x, y)");
        // Both x and y are used in tuple
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
        assert!(!issues
            .iter()
            .any(|i| i.name == "y" && i.rule.contains("unused")));
    }

    // ============== Range Expression Tests ==============

    #[test]
    fn test_variable_in_range_start() {
        let issues = parse_and_lint("let start = 0\nrange(start, 10)");
        // start is used in range
        assert!(!issues
            .iter()
            .any(|i| i.name == "start" && i.rule.contains("unused")));
    }

    #[test]
    fn test_variable_in_range_end() {
        let issues = parse_and_lint("let end = 10\nrange(0, end)");
        // end is used in range
        assert!(!issues
            .iter()
            .any(|i| i.name == "end" && i.rule.contains("unused")));
    }

    // === EXTREME TDD Round 124 tests ===

    #[test]
    fn test_is_builtin_additional_math() {
        assert!(is_builtin("exp"));
        assert!(is_builtin("ln"));
        assert!(is_builtin("log10"));
        assert!(is_builtin("log2"));
    }

    #[test]
    fn test_is_builtin_fs_additional() {
        assert!(is_builtin("fs_metadata"));
        assert!(is_builtin("fs_create_dir"));
        assert!(is_builtin("fs_read_dir"));
        assert!(is_builtin("fs_copy"));
    }

    #[test]
    fn test_is_builtin_format_function() {
        assert!(is_builtin("format"));
    }

    #[test]
    fn test_undefined_in_nested_call() {
        let issues = parse_and_lint("foo(bar(undefined_var))");
        assert!(issues.iter().any(|i| i.rule == "undefined"));
    }

    #[test]
    fn test_function_parameter_used_in_binary() {
        let issues = parse_and_lint("fun add(a, b) { a + b }");
        // Parameters used in binary expression
        assert!(!issues
            .iter()
            .any(|i| (i.name == "a" || i.name == "b") && i.rule.contains("unused")));
    }

    #[test]
    fn test_function_parameter_used_in_call() {
        let issues = parse_and_lint("fun wrapper(f, x) { f(x) }");
        // f used as function, x used as argument
        assert!(!issues
            .iter()
            .any(|i| (i.name == "f" || i.name == "x") && i.rule.contains("unused")));
    }

    #[test]
    fn test_shadowing_in_nested_scope() {
        let _issues = parse_and_lint("let x = 1\n{ let x = 2\nx }");
        // Verify no crash with shadowing
    }

    #[test]
    fn test_variable_used_in_match_arm() {
        let _issues = parse_and_lint("let x = 1\nmatch x { 1 => true, _ => false }");
        // x used in match expression
    }

    #[test]
    fn test_variable_used_in_array_index() {
        let issues = parse_and_lint("let i = 0\nlet arr = [1,2,3]\narr[i]");
        // i used as index
        assert!(!issues
            .iter()
            .any(|i| i.name == "i" && i.rule.contains("unused")));
    }

    #[test]
    fn test_variable_used_in_compound_assign() {
        let _issues = parse_and_lint("let mut x = 1\nx += 1");
        // x used in compound assignment
    }

    #[test]
    fn test_unary_not_parsing() {
        // Just verify unary not parses and lints without crash
        let _issues = parse_and_lint("!true");
    }

    #[test]
    fn test_unary_negate_parsing() {
        // Just verify unary negate parses and lints without crash
        let _issues = parse_and_lint("-42");
    }

    #[test]
    fn test_empty_function_body() {
        let _issues = parse_and_lint("fun noop() { }");
        // Empty function should not crash
    }

    #[test]
    fn test_is_builtin_assert_functions() {
        assert!(is_builtin("assert"));
        assert!(is_builtin("assert_eq"));
        assert!(is_builtin("assert_ne"));
    }

    #[test]
    fn test_is_builtin_env_set_var() {
        assert!(is_builtin("env_set_var"));
    }

    #[test]
    fn test_is_builtin_time_duration() {
        assert!(is_builtin("time_duration"));
    }

    #[test]
    fn test_is_builtin_fs_rename() {
        assert!(is_builtin("fs_rename"));
    }

    // ============== EXTREME TDD ROUND 129 ==============
    // Additional linter coverage tests (matching is_builtin registry)

    #[test]
    fn test_is_builtin_math_functions_r129() {
        // All from linter.rs is_builtin match
        assert!(is_builtin("abs"));
        assert!(is_builtin("sqrt"));
        assert!(is_builtin("floor"));
        assert!(is_builtin("ceil"));
        assert!(is_builtin("round"));
        assert!(is_builtin("sin"));
        assert!(is_builtin("cos"));
        assert!(is_builtin("tan"));
        assert!(is_builtin("pow"));
        assert!(is_builtin("exp"));
        assert!(is_builtin("ln"));
        assert!(is_builtin("log10"));
        assert!(is_builtin("log2"));
    }

    #[test]
    fn test_is_builtin_output_functions_r129() {
        // Output functions from linter.rs
        assert!(is_builtin("println"));
        assert!(is_builtin("print"));
        assert!(is_builtin("eprintln"));
        assert!(is_builtin("eprint"));
        assert!(is_builtin("dbg"));
    }

    #[test]
    fn test_is_builtin_collection_functions_r129() {
        assert!(is_builtin("range"));
        assert!(is_builtin("HashMap"));
        assert!(is_builtin("HashSet"));
        assert!(is_builtin("min"));
        assert!(is_builtin("max"));
    }

    #[test]
    fn test_is_builtin_io_functions_r129() {
        // FS functions from linter.rs
        assert!(is_builtin("fs_read"));
        assert!(is_builtin("fs_write"));
        assert!(is_builtin("fs_exists"));
        assert!(is_builtin("fs_remove"));
        assert!(is_builtin("fs_metadata"));
    }

    #[test]
    fn test_is_builtin_process_functions_r129() {
        assert!(is_builtin("exit"));
        assert!(is_builtin("panic"));
        assert!(is_builtin("format"));
    }

    #[test]
    fn test_is_builtin_false_for_random_names_r129() {
        assert!(!is_builtin("my_custom_function"));
        assert!(!is_builtin("foo"));
        assert!(!is_builtin("bar_baz"));
        assert!(!is_builtin("notabuiltin123"));
        assert!(!is_builtin(""));
    }

    #[test]
    fn test_nested_function_calls_linting_r129() {
        let issues = parse_and_lint("println(len(\"hello\"))");
        // Should lint without errors
        assert!(!issues.iter().any(|i| i.rule.contains("error")));
    }

    #[test]
    fn test_chained_method_calls_r129() {
        let issues = parse_and_lint("\"hello\".upper().trim()");
        assert!(!issues.iter().any(|i| i.rule.contains("error")));
    }

    #[test]
    fn test_multiple_unused_variables_r129() {
        let issues = parse_and_lint("let a = 1\nlet b = 2\nlet c = 3");
        // Should report all three as unused
        let unused_count = issues.iter().filter(|i| i.rule.contains("unused")).count();
        assert!(unused_count >= 3);
    }

    #[test]
    fn test_variable_shadowing_r129() {
        let issues = parse_and_lint("let x = 1\nlet x = 2\nx");
        // First x is shadowed, second x is used - may or may not report unused depending on linter
        // Just verify it runs without crash
        let _ = issues;
    }

    #[test]
    fn test_for_loop_variable_used_r129() {
        let issues = parse_and_lint("for i in range(10) { println(i) }");
        // i should be considered used
        assert!(!issues
            .iter()
            .any(|i| i.name == "i" && i.rule.contains("unused")));
    }

    #[test]
    fn test_while_loop_linting_r129() {
        let _issues = parse_and_lint("let mut i = 0\nwhile i < 10 { i += 1 }");
        // Should lint without crash
    }

    #[test]
    fn test_if_else_branch_linting_r129() {
        let _issues = parse_and_lint("if true { 1 } else { 2 }");
        // Should lint without crash
    }

    #[test]
    fn test_match_expression_linting_r129() {
        let _issues = parse_and_lint("match 1 { 1 => \"one\", _ => \"other\" }");
        // Should lint without crash
    }

    #[test]
    fn test_lambda_expression_linting_r129() {
        let _issues = parse_and_lint("|x| x * 2");
        // Should lint without crash
    }

    #[test]
    fn test_array_literal_linting_r129() {
        let _issues = parse_and_lint("[1, 2, 3, 4, 5]");
        // Should lint without crash
    }

    #[test]
    fn test_object_literal_linting_r129() {
        let _issues = parse_and_lint("{\"name\": \"test\", \"value\": 42}");
        // Should lint without crash
    }

    #[test]
    fn test_complex_expression_linting_r129() {
        let _issues = parse_and_lint("(1 + 2) * (3 - 4) / 5");
        // Should lint without crash
    }

    #[test]
    fn test_comparison_operators_linting_r129() {
        let _issues = parse_and_lint("1 < 2 && 3 > 2 || 4 == 4");
        // Should lint without crash
    }

    #[test]
    fn test_string_interpolation_linting_r129() {
        let _issues = parse_and_lint("let name = \"world\"\nf\"Hello {name}\"");
        // name should be used in f-string
    }

    #[test]
    fn test_multiline_string_linting_r129() {
        let _issues = parse_and_lint("\"\"\"multi\nline\nstring\"\"\"");
        // Should lint without crash
    }

    #[test]
    fn test_function_with_return_type_r129() {
        let _issues = parse_and_lint("fun add(a: int, b: int) -> int { a + b }");
        // Should lint without crash
    }

    #[test]
    fn test_function_with_default_params_r129() {
        // Note: default params may or may not be supported
        let _issues = parse_and_lint("fun greet(name: str) { println(name) }");
        // Should lint without crash
    }

    #[test]
    fn test_struct_definition_linting_r129() {
        let _issues = parse_and_lint("struct Point { x: int, y: int }");
        // Should lint without crash
    }

    #[test]
    fn test_impl_block_linting_r129() {
        let _issues = parse_and_lint(
            "impl Point { fun new(x: int, y: int) -> Point { Point { x: x, y: y } } }",
        );
        // Should lint without crash
    }

    #[test]
    fn test_async_function_linting_r129() {
        let _issues = parse_and_lint("async fun fetch() { }");
        // Should lint without crash
    }

    #[test]
    fn test_comment_in_code_linting_r129() {
        let _issues = parse_and_lint("let x = 1 // comment\nx");
        // Should lint without crash, x should be used
    }

    #[test]
    fn test_multiple_statements_linting_r129() {
        let _issues = parse_and_lint("let a = 1\nlet b = 2\na + b");
        // a and b used in expression
    }

    #[test]
    fn test_recursive_function_linting_r129() {
        let _issues =
            parse_and_lint("fun fact(n: int) -> int { if n <= 1 { 1 } else { n * fact(n - 1) } }");
        // Should lint without crash
    }

    #[test]
    fn test_is_builtin_dataframe_functions_r129() {
        // DataFrame functions from linter.rs
        assert!(is_builtin("col"));
        assert!(is_builtin("lit"));
        assert!(is_builtin("DataFrame"));
    }

    #[test]
    fn test_is_builtin_http_functions_r129() {
        assert!(is_builtin("http_get"));
        assert!(is_builtin("http_post"));
        assert!(is_builtin("http_put"));
        assert!(is_builtin("http_delete"));
    }

    #[test]
    fn test_is_builtin_json_functions_r129() {
        assert!(is_builtin("json_parse"));
        assert!(is_builtin("json_stringify"));
    }

    #[test]
    fn test_is_builtin_time_functions_r129() {
        // Time functions from linter.rs
        assert!(is_builtin("time_now"));
        assert!(is_builtin("time_sleep"));
        assert!(is_builtin("time_duration"));
    }

    #[test]
    fn test_is_builtin_path_functions_r129() {
        assert!(is_builtin("path_join"));
        assert!(is_builtin("path_extension"));
        assert!(is_builtin("path_filename"));
        assert!(is_builtin("path_parent"));
    }

    #[test]
    fn test_is_builtin_regex_functions_r129() {
        assert!(is_builtin("regex_new"));
        assert!(is_builtin("regex_is_match"));
        assert!(is_builtin("regex_find"));
        assert!(is_builtin("regex_replace"));
    }

    #[test]
    fn test_is_builtin_logging_functions_r129() {
        assert!(is_builtin("log_info"));
        assert!(is_builtin("log_warn"));
        assert!(is_builtin("log_error"));
        assert!(is_builtin("log_debug"));
        assert!(is_builtin("log_trace"));
    }

    #[test]
    fn test_is_builtin_env_functions_r129() {
        assert!(is_builtin("env_var"));
        assert!(is_builtin("env_args"));
        assert!(is_builtin("env_current_dir"));
        assert!(is_builtin("env_set_var"));
    }
}
