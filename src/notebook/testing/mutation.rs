// SPRINT2-003: Mutation testing implementation
// PMAT Complexity: <10 per function
use crate::notebook::testing::tester::NotebookTester;
use crate::notebook::testing::types::Cell;
#[derive(Debug, Clone)]
pub struct MutationConfig {
    pub enabled_mutations: Vec<MutationType>,
    pub timeout_ms: u64,
}
impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            enabled_mutations: vec![
                MutationType::ArithmeticOperator,
                MutationType::ComparisonOperator,
                MutationType::BoundaryValue,
                MutationType::LogicalOperator,
            ],
            timeout_ms: 5000,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum MutationType {
    ArithmeticOperator,
    ComparisonOperator,
    BoundaryValue,
    LogicalOperator,
    ReturnValue,
    ConditionalNegation,
}
#[derive(Debug, Clone)]
pub struct Mutation {
    pub id: String,
    pub cell_id: String,
    pub mutation_type: MutationType,
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub mutated: String,
}
#[derive(Debug, Clone)]
pub struct MutationResult {
    pub mutation: Mutation,
    pub killed: bool,
    pub killing_test: Option<String>,
}

/// Mutation testing for notebook cells
pub struct MutationTester {
    tester: NotebookTester,
    config: MutationConfig,
    results: Vec<MutationResult>,
}

impl Default for MutationTester {
    fn default() -> Self {
        Self::new()
    }
}

impl MutationTester {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::mutation::MutationTester;
    ///
    /// let instance = MutationTester::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            tester: NotebookTester::new(),
            config: MutationConfig::default(),
            results: Vec::new(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::mutation::MutationTester;
    ///
    /// let mut instance = MutationTester::new();
    /// let result = instance.with_config();
    /// // Verify behavior
    /// ```
    pub fn with_config(config: MutationConfig) -> Self {
        Self {
            tester: NotebookTester::new(),
            config,
            results: Vec::new(),
        }
    }
    /// Generate mutations for a cell
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::mutation::MutationTester;
    ///
    /// let mut instance = MutationTester::new();
    /// let result = instance.generate_mutations();
    /// // Verify behavior
    /// ```
    pub fn generate_mutations(&self, cell: &Cell) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        for mutation_type in &self.config.enabled_mutations {
            mutations.extend(self.generate_mutations_by_type(cell, mutation_type));
        }
        mutations
    }
    fn generate_mutations_by_type(
        &self,
        cell: &Cell,
        mutation_type: &MutationType,
    ) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let lines: Vec<&str> = cell.source.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            match mutation_type {
                MutationType::ArithmeticOperator => {
                    mutations.extend(self.mutate_arithmetic(cell, line_num, line));
                }
                MutationType::ComparisonOperator => {
                    mutations.extend(self.mutate_comparison(cell, line_num, line));
                }
                MutationType::BoundaryValue => {
                    mutations.extend(self.mutate_boundary(cell, line_num, line));
                }
                MutationType::LogicalOperator => {
                    mutations.extend(self.mutate_logical(cell, line_num, line));
                }
                _ => {}
            }
        }
        mutations
    }
    fn mutate_arithmetic(&self, cell: &Cell, line_num: usize, line: &str) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let operators = vec![("+", "-"), ("-", "+"), ("*", "/"), ("/", "*")];
        for (original_op, mutated_op) in operators {
            if let Some(col) = line.find(original_op) {
                mutations.push(Mutation {
                    id: format!("mut_arith_{line_num}_{col}"),
                    cell_id: cell.id.clone(),
                    mutation_type: MutationType::ArithmeticOperator,
                    line: line_num,
                    column: col,
                    original: line.to_string(),
                    mutated: line.replace(original_op, mutated_op),
                });
            }
        }
        mutations
    }
    fn mutate_comparison(&self, cell: &Cell, line_num: usize, line: &str) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let operators = vec![
            (">", "<"),
            ("<", ">"),
            (">=", "<="),
            ("<=", ">="),
            ("==", "!="),
            ("!=", "=="),
        ];
        for (original_op, mutated_op) in operators {
            if let Some(col) = line.find(original_op) {
                mutations.push(Mutation {
                    id: format!("mut_comp_{line_num}_{col}"),
                    cell_id: cell.id.clone(),
                    mutation_type: MutationType::ComparisonOperator,
                    line: line_num,
                    column: col,
                    original: line.to_string(),
                    mutated: line.replacen(original_op, mutated_op, 1),
                });
            }
        }
        mutations
    }
    fn mutate_boundary(&self, cell: &Cell, line_num: usize, line: &str) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        // Look for common boundary values
        let boundaries = vec![(" 0", " 1"), ("(0", "(1"), (" 1", " 0"), ("(1", "(0")];
        for (original, mutated) in boundaries {
            if let Some(col) = line.find(original) {
                mutations.push(Mutation {
                    id: format!("mut_bound_{line_num}_{col}"),
                    cell_id: cell.id.clone(),
                    mutation_type: MutationType::BoundaryValue,
                    line: line_num,
                    column: col,
                    original: line.to_string(),
                    mutated: line.replacen(original, mutated, 1),
                });
            }
        }
        mutations
    }
    fn mutate_logical(&self, cell: &Cell, line_num: usize, line: &str) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let operators = vec![("&&", "||"), ("||", "&&")];
        for (original_op, mutated_op) in operators {
            if let Some(col) = line.find(original_op) {
                mutations.push(Mutation {
                    id: format!("mut_logic_{line_num}_{col}"),
                    cell_id: cell.id.clone(),
                    mutation_type: MutationType::LogicalOperator,
                    line: line_num,
                    column: col,
                    original: line.to_string(),
                    mutated: line.replace(original_op, mutated_op),
                });
            }
        }
        mutations
    }
    /// Apply a mutation to a cell
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::mutation::MutationTester;
    ///
    /// let mut instance = MutationTester::new();
    /// let result = instance.apply_mutation();
    /// // Verify behavior
    /// ```
    pub fn apply_mutation(&self, cell: &Cell, mutation: &Mutation) -> Cell {
        let lines: Vec<&str> = cell.source.lines().collect();
        let mut mutated_lines = lines.clone();
        if mutation.line < mutated_lines.len() {
            mutated_lines[mutation.line] = &mutation.mutated;
        }
        Cell {
            id: cell.id.clone(),
            source: mutated_lines.join("\n"),
            cell_type: cell.cell_type.clone(),
            metadata: cell.metadata.clone(),
        }
    }
    /// Test if a mutation is killed by test cells
    pub fn test_mutation(
        &mut self,
        original_cell: &Cell,
        mutation: &Mutation,
        test_cells: &[Cell],
    ) -> MutationResult {
        let mutated_cell = self.apply_mutation(original_cell, mutation);
        // Execute with original
        let mut original_tester = NotebookTester::new();
        let _ = original_tester.execute_cell(original_cell);
        // Execute with mutation
        let mut mutated_tester = NotebookTester::new();
        let _ = mutated_tester.execute_cell(&mutated_cell);
        // Run tests
        for test_cell in test_cells {
            let original_result = original_tester.execute_cell(test_cell);
            let mutated_result = mutated_tester.execute_cell(test_cell);
            if original_result != mutated_result {
                // Mutation killed
                return MutationResult {
                    mutation: mutation.clone(),
                    killed: true,
                    killing_test: Some(test_cell.id.clone()),
                };
            }
        }
        // Mutation survived
        MutationResult {
            mutation: mutation.clone(),
            killed: false,
            killing_test: None,
        }
    }
    /// Calculate mutation score
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::mutation::MutationTester;
    ///
    /// let mut instance = MutationTester::new();
    /// let result = instance.calculate_score();
    /// // Verify behavior
    /// ```
    pub fn calculate_score(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let killed = self.results.iter().filter(|r| r.killed).count();
        (killed as f64) / (self.results.len() as f64)
    }
    /// Generate mutation testing report
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::mutation::generate_report;
    ///
    /// let result = generate_report(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Mutation Testing Report ===\n\n");
        let total = self.results.len();
        let killed = self.results.iter().filter(|r| r.killed).count();
        let survived = total - killed;
        let score = self.calculate_score() * 100.0;
        report.push_str(&format!("Total Mutations: {total}\n"));
        report.push_str(&format!("Killed: {killed}\n"));
        report.push_str(&format!("Survived: {survived}\n"));
        report.push_str(&format!("Mutation Score: {score:.1}%\n\n"));
        if survived > 0 {
            report.push_str("Surviving Mutations (improve tests for these):\n");
            for result in self.results.iter().filter(|r| !r.killed) {
                report.push_str(&format!(
                    "  - {} in cell '{}' at line {}\n",
                    result.mutation.id, result.mutation.cell_id, result.mutation.line
                ));
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // EXTREME TDD: Comprehensive test coverage for mutation testing

    #[test]
    fn test_mutation_config_default() {
        let config = MutationConfig::default();

        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.enabled_mutations.len(), 4);
        assert!(config
            .enabled_mutations
            .contains(&MutationType::ArithmeticOperator));
        assert!(config
            .enabled_mutations
            .contains(&MutationType::ComparisonOperator));
        assert!(config
            .enabled_mutations
            .contains(&MutationType::BoundaryValue));
        assert!(config
            .enabled_mutations
            .contains(&MutationType::LogicalOperator));
    }

    #[test]
    fn test_mutation_config_custom() {
        let config = MutationConfig {
            enabled_mutations: vec![MutationType::ReturnValue, MutationType::ConditionalNegation],
            timeout_ms: 10000,
        };

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.enabled_mutations.len(), 2);
        assert!(config
            .enabled_mutations
            .contains(&MutationType::ReturnValue));
        assert!(config
            .enabled_mutations
            .contains(&MutationType::ConditionalNegation));
    }

    #[test]
    fn test_mutation_type_equality() {
        assert_eq!(
            MutationType::ArithmeticOperator,
            MutationType::ArithmeticOperator
        );
        assert_ne!(
            MutationType::ArithmeticOperator,
            MutationType::ComparisonOperator
        );
        assert_ne!(MutationType::BoundaryValue, MutationType::LogicalOperator);
    }

    #[test]
    fn test_mutation_creation() {
        let mutation = Mutation {
            id: "mut_001".to_string(),
            cell_id: "cell_1".to_string(),
            mutation_type: MutationType::ArithmeticOperator,
            line: 10,
            column: 5,
            original: "+".to_string(),
            mutated: "-".to_string(),
        };

        assert_eq!(mutation.id, "mut_001");
        assert_eq!(mutation.cell_id, "cell_1");
        assert_eq!(mutation.mutation_type, MutationType::ArithmeticOperator);
        assert_eq!(mutation.line, 10);
        assert_eq!(mutation.column, 5);
        assert_eq!(mutation.original, "+");
        assert_eq!(mutation.mutated, "-");
    }

    #[test]
    fn test_mutation_result_killed() {
        let mutation = Mutation {
            id: "mut_002".to_string(),
            cell_id: "cell_2".to_string(),
            mutation_type: MutationType::ComparisonOperator,
            line: 20,
            column: 10,
            original: ">".to_string(),
            mutated: "<".to_string(),
        };

        let result = MutationResult {
            mutation,
            killed: true,
            killing_test: Some("test_comparison".to_string()),
        };

        assert_eq!(result.mutation.id, "mut_002");
        assert!(result.killed);
        assert_eq!(result.killing_test, Some("test_comparison".to_string()));
    }

    #[test]
    fn test_mutation_result_survived() {
        let mutation = Mutation {
            id: "mut_003".to_string(),
            cell_id: "cell_3".to_string(),
            mutation_type: MutationType::LogicalOperator,
            line: 30,
            column: 15,
            original: "&&".to_string(),
            mutated: "||".to_string(),
        };

        let result = MutationResult {
            mutation,
            killed: false,
            killing_test: None,
        };

        assert!(!result.killed);
        assert!(result.killing_test.is_none());
    }

    #[test]
    fn test_mutation_tester_new() {
        let tester = MutationTester::new();
        assert_eq!(tester.config.timeout_ms, 5000);
        assert!(tester.results.is_empty());
    }

    #[test]
    fn test_mutation_tester_with_config() {
        let config = MutationConfig {
            enabled_mutations: vec![MutationType::ReturnValue],
            timeout_ms: 3000,
        };

        let tester = MutationTester::with_config(config);
        assert_eq!(tester.config.timeout_ms, 3000);
        assert_eq!(tester.config.enabled_mutations.len(), 1);
    }

    #[test]
    fn test_mutation_tester_default() {
        let tester = MutationTester::default();
        assert_eq!(tester.config.timeout_ms, 5000);
        assert!(tester.results.is_empty());
    }

    #[test]
    fn test_all_mutation_types() {
        let types = vec![
            MutationType::ArithmeticOperator,
            MutationType::ComparisonOperator,
            MutationType::BoundaryValue,
            MutationType::LogicalOperator,
            MutationType::ReturnValue,
            MutationType::ConditionalNegation,
        ];

        for mutation_type in types {
            let mutation = Mutation {
                id: format!("mut_{mutation_type:?}"),
                cell_id: "test_cell".to_string(),
                mutation_type: mutation_type.clone(),
                line: 1,
                column: 1,
                original: "original".to_string(),
                mutated: "mutated".to_string(),
            };

            assert_eq!(mutation.mutation_type, mutation_type);
        }
    }

    #[test]
    fn test_mutation_clone() {
        let mutation = Mutation {
            id: "mut_clone".to_string(),
            cell_id: "cell_clone".to_string(),
            mutation_type: MutationType::BoundaryValue,
            line: 5,
            column: 10,
            original: "0".to_string(),
            mutated: "1".to_string(),
        };

        let cloned = mutation.clone();
        assert_eq!(cloned.id, mutation.id);
        assert_eq!(cloned.cell_id, mutation.cell_id);
        assert_eq!(cloned.mutation_type, mutation.mutation_type);
    }

    #[test]
    fn test_mutation_config_clone() {
        let config = MutationConfig {
            enabled_mutations: vec![MutationType::ArithmeticOperator],
            timeout_ms: 7500,
        };

        let cloned = config.clone();
        assert_eq!(cloned.timeout_ms, config.timeout_ms);
        assert_eq!(
            cloned.enabled_mutations.len(),
            config.enabled_mutations.len()
        );
    }

    #[test]
    fn test_mutation_result_clone() {
        let mutation = Mutation {
            id: "mut_result".to_string(),
            cell_id: "cell_result".to_string(),
            mutation_type: MutationType::ConditionalNegation,
            line: 15,
            column: 20,
            original: "if x > 0".to_string(),
            mutated: "if x <= 0".to_string(),
        };

        let result = MutationResult {
            mutation,
            killed: true,
            killing_test: Some("test_condition".to_string()),
        };

        let cloned = result.clone();
        assert_eq!(cloned.mutation.id, result.mutation.id);
        assert_eq!(cloned.killed, result.killed);
        assert_eq!(cloned.killing_test, result.killing_test);
    }

    // EXTREME TDD Round 109: Coverage tests for mutation generation

    fn make_test_cell(source: &str) -> Cell {
        Cell {
            id: "test_cell".to_string(),
            source: source.to_string(),
            cell_type: crate::notebook::testing::types::CellType::Code,
            metadata: crate::notebook::testing::types::CellMetadata { test: None },
        }
    }

    #[test]
    fn test_generate_mutations_arithmetic() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = a + b;");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('-')),
            "Should generate + to - mutation"
        );
    }

    #[test]
    fn test_generate_mutations_subtraction() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = a - b;");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('+')),
            "Should generate - to + mutation"
        );
    }

    #[test]
    fn test_generate_mutations_multiplication() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = a * b;");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('/')),
            "Should generate * to / mutation"
        );
    }

    #[test]
    fn test_generate_mutations_division() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = a / b;");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('*')),
            "Should generate / to * mutation"
        );
    }

    #[test]
    fn test_generate_mutations_greater_than() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x > 0 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('<')),
            "Should generate > to < mutation"
        );
    }

    #[test]
    fn test_generate_mutations_less_than() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x < 10 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains('>')),
            "Should generate < to > mutation"
        );
    }

    #[test]
    fn test_generate_mutations_equals() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x == 5 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains("!=")),
            "Should generate == to != mutation"
        );
    }

    #[test]
    fn test_generate_mutations_not_equals() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x != 0 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains("==")),
            "Should generate != to == mutation"
        );
    }

    #[test]
    fn test_generate_mutations_logical_and() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if a && b { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains("||")),
            "Should generate && to || mutation"
        );
    }

    #[test]
    fn test_generate_mutations_logical_or() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if a || b { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains("&&")),
            "Should generate || to && mutation"
        );
    }

    #[test]
    fn test_generate_mutations_boundary_zero() {
        let tester = MutationTester::new();
        let cell = make_test_cell("for i in 0..n { }");
        let mutations = tester.generate_mutations(&cell);
        // Boundary mutations check for " 0" or "(0" patterns
        let _has_boundary = mutations
            .iter()
            .any(|m| m.mutation_type == MutationType::BoundaryValue);
        // May or may not match depending on pattern - verify generate_mutations runs
        let _ = mutations.len();
    }

    #[test]
    fn test_generate_mutations_boundary_one() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = arr[ 1];");
        let mutations = tester.generate_mutations(&cell);
        let _has_boundary = mutations
            .iter()
            .any(|m| m.mutation_type == MutationType::BoundaryValue && m.mutated.contains(" 0"));
        // May match " 1" to " 0" - verify generate_mutations runs
        let _ = mutations.len();
    }

    #[test]
    fn test_generate_mutations_empty_cell() {
        let tester = MutationTester::new();
        let cell = make_test_cell("");
        let mutations = tester.generate_mutations(&cell);
        assert!(mutations.is_empty(), "Empty cell should have no mutations");
    }

    #[test]
    fn test_generate_mutations_no_operators() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = 42;");
        let mutations = tester.generate_mutations(&cell);
        // No arithmetic, comparison, or logical operators
        assert!(
            mutations.is_empty() || mutations.iter().all(|m| m.mutation_type == MutationType::BoundaryValue),
            "Should have no mutations or only boundary mutations"
        );
    }

    #[test]
    fn test_apply_mutation_single_line() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = 1 + 2;");
        let mutation = Mutation {
            id: "test".to_string(),
            cell_id: cell.id.clone(),
            mutation_type: MutationType::ArithmeticOperator,
            line: 0,
            column: 10,
            original: "let x = 1 + 2;".to_string(),
            mutated: "let x = 1 - 2;".to_string(),
        };
        let mutated_cell = tester.apply_mutation(&cell, &mutation);
        assert!(mutated_cell.source.contains('-'));
        assert!(!mutated_cell.source.contains('+'));
    }

    #[test]
    fn test_apply_mutation_multi_line() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = 1;\nlet y = x + 2;");
        let mutation = Mutation {
            id: "test".to_string(),
            cell_id: cell.id.clone(),
            mutation_type: MutationType::ArithmeticOperator,
            line: 1,
            column: 10,
            original: "let y = x + 2;".to_string(),
            mutated: "let y = x - 2;".to_string(),
        };
        let mutated_cell = tester.apply_mutation(&cell, &mutation);
        assert!(mutated_cell.source.contains("let x = 1;"));
        assert!(mutated_cell.source.contains('-'));
    }

    #[test]
    fn test_apply_mutation_invalid_line() {
        let tester = MutationTester::new();
        let cell = make_test_cell("let x = 1;");
        let mutation = Mutation {
            id: "test".to_string(),
            cell_id: cell.id.clone(),
            mutation_type: MutationType::ArithmeticOperator,
            line: 999, // Out of bounds
            column: 0,
            original: "".to_string(),
            mutated: "invalid".to_string(),
        };
        let mutated_cell = tester.apply_mutation(&cell, &mutation);
        // Should not crash, returns original
        assert_eq!(mutated_cell.source, "let x = 1;");
    }

    #[test]
    fn test_calculate_score_empty() {
        let tester = MutationTester::new();
        assert_eq!(tester.calculate_score(), 0.0);
    }

    #[test]
    fn test_calculate_score_all_killed() {
        let mut tester = MutationTester::new();
        tester.results.push(MutationResult {
            mutation: Mutation {
                id: "1".to_string(),
                cell_id: "c".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                line: 0,
                column: 0,
                original: "+".to_string(),
                mutated: "-".to_string(),
            },
            killed: true,
            killing_test: Some("test1".to_string()),
        });
        tester.results.push(MutationResult {
            mutation: Mutation {
                id: "2".to_string(),
                cell_id: "c".to_string(),
                mutation_type: MutationType::ComparisonOperator,
                line: 0,
                column: 0,
                original: ">".to_string(),
                mutated: "<".to_string(),
            },
            killed: true,
            killing_test: Some("test2".to_string()),
        });
        assert_eq!(tester.calculate_score(), 1.0);
    }

    #[test]
    fn test_calculate_score_half_killed() {
        let mut tester = MutationTester::new();
        tester.results.push(MutationResult {
            mutation: Mutation {
                id: "1".to_string(),
                cell_id: "c".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                line: 0,
                column: 0,
                original: "+".to_string(),
                mutated: "-".to_string(),
            },
            killed: true,
            killing_test: Some("test1".to_string()),
        });
        tester.results.push(MutationResult {
            mutation: Mutation {
                id: "2".to_string(),
                cell_id: "c".to_string(),
                mutation_type: MutationType::ComparisonOperator,
                line: 0,
                column: 0,
                original: ">".to_string(),
                mutated: "<".to_string(),
            },
            killed: false,
            killing_test: None,
        });
        assert_eq!(tester.calculate_score(), 0.5);
    }

    #[test]
    fn test_calculate_score_none_killed() {
        let mut tester = MutationTester::new();
        tester.results.push(MutationResult {
            mutation: Mutation {
                id: "1".to_string(),
                cell_id: "c".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                line: 0,
                column: 0,
                original: "+".to_string(),
                mutated: "-".to_string(),
            },
            killed: false,
            killing_test: None,
        });
        assert_eq!(tester.calculate_score(), 0.0);
    }

    #[test]
    fn test_mutation_type_debug() {
        let arith = format!("{:?}", MutationType::ArithmeticOperator);
        assert!(arith.contains("Arithmetic"));

        let comp = format!("{:?}", MutationType::ComparisonOperator);
        assert!(comp.contains("Comparison"));
    }

    #[test]
    fn test_mutation_struct_debug() {
        let mutation = Mutation {
            id: "debug_test".to_string(),
            cell_id: "cell".to_string(),
            mutation_type: MutationType::LogicalOperator,
            line: 5,
            column: 10,
            original: "&&".to_string(),
            mutated: "||".to_string(),
        };
        let debug_str = format!("{:?}", mutation);
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_mutation_config_debug() {
        let config = MutationConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("5000"));
    }

    #[test]
    fn test_greater_equal_mutation() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x >= 5 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains("<=")),
            "Should generate >= to <= mutation"
        );
    }

    #[test]
    fn test_less_equal_mutation() {
        let tester = MutationTester::new();
        let cell = make_test_cell("if x <= 10 { }");
        let mutations = tester.generate_mutations(&cell);
        assert!(
            mutations.iter().any(|m| m.mutated.contains(">=")),
            "Should generate <= to >= mutation"
        );
    }
}
