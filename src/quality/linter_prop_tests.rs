    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Linter creation never panics
        #[test]
        fn test_linter_creation_never_panics(_input: String) {
            let _ = Linter::new();
            let _ = Linter::default();
        }

        /// Property: Rule setting never panics on any string input
        #[test]
        fn test_set_rules_never_panics(rule_string in "\\PC*") {
            let mut linter = Linter::new();
            linter.set_rules(&rule_string);
        }

        /// Property: Strict mode setting always succeeds
        #[test]
        fn test_strict_mode_setting_invariant(strict_flag: bool) {
            let mut linter = Linter::new();
            linter.set_strict_mode(strict_flag);
            assert_eq!(linter.strict_mode, strict_flag);
        }

        /// Property: Rule count is always non-negative after any operation
        #[test]
        fn test_rule_count_invariant(rule_string in "\\PC*") {
            let mut linter = Linter::new();
            linter.set_rules(&rule_string);
            // Rules length is always >= 0 for usize, no need to check
        }

        /// Property: Auto-fix never produces longer strings for simple cases
        #[test]
        fn test_auto_fix_length_property(input in "[a-zA-Z0-9 ]{0,50}") {
            let linter = Linter::new();
            let issues = vec![LintIssue {
                line: 1,
                column: 1,
                severity: "warning".to_string(),
                rule: "style".to_string(),
                message: "spacing".to_string(),
                suggestion: "fix spacing".to_string(),
                issue_type: "style".to_string(),
                name: "spacing".to_string(),
            }];
            if let Ok(fixed) = linter.auto_fix(&input, &issues) {
                // Style fixes should not increase length significantly
                assert!(fixed.len() <= input.len() + 10);
            }
        }
    }
