// SPRINT2-003: Mutation testing implementation
// PMAT Complexity: <10 per function

use crate::notebook::testing::types::*;
use crate::notebook::testing::NotebookTester;

/// Mutation testing for notebook code
pub struct MutationTester {
    config: MutationConfig,
    results: Vec<MutationResult>,
}

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

impl MutationTester {
    pub fn new() -> Self {
        Self {
            config: MutationConfig::default(),
            results: Vec::new(),
        }
    }
    
    pub fn with_config(config: MutationConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }
    
    /// Generate mutations for a cell
    pub fn generate_mutations(&self, cell: &Cell) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        
        for mutation_type in &self.config.enabled_mutations {
            mutations.extend(self.generate_mutations_by_type(cell, mutation_type));
        }
        
        mutations
    }
    
    fn generate_mutations_by_type(&self, cell: &Cell, mutation_type: &MutationType) -> Vec<Mutation> {
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
        
        let operators = vec![
            ("+", "-"),
            ("-", "+"),
            ("*", "/"),
            ("/", "*"),
        ];
        
        for (original_op, mutated_op) in operators {
            if let Some(col) = line.find(original_op) {
                mutations.push(Mutation {
                    id: format!("mut_arith_{}_{}", line_num, col),
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
                    id: format!("mut_comp_{}_{}", line_num, col),
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
        let boundaries = vec![
            (" 0", " 1"),
            ("(0", "(1"),
            (" 1", " 0"),
            ("(1", "(0"),
        ];
        
        for (original, mutated) in boundaries {
            if let Some(col) = line.find(original) {
                mutations.push(Mutation {
                    id: format!("mut_bound_{}_{}", line_num, col),
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
        
        let operators = vec![
            ("&&", "||"),
            ("||", "&&"),
        ];
        
        for (original_op, mutated_op) in operators {
            if let Some(col) = line.find(original_op) {
                mutations.push(Mutation {
                    id: format!("mut_logic_{}_{}", line_num, col),
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
    pub fn calculate_score(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        
        let killed = self.results.iter().filter(|r| r.killed).count();
        (killed as f64) / (self.results.len() as f64)
    }
    
    /// Generate mutation testing report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Mutation Testing Report ===\n\n");
        
        let total = self.results.len();
        let killed = self.results.iter().filter(|r| r.killed).count();
        let survived = total - killed;
        let score = self.calculate_score() * 100.0;
        
        report.push_str(&format!("Total Mutations: {}\n", total));
        report.push_str(&format!("Killed: {}\n", killed));
        report.push_str(&format!("Survived: {}\n", survived));
        report.push_str(&format!("Mutation Score: {:.1}%\n\n", score));
        
        if survived > 0 {
            report.push_str("Surviving Mutations (improve tests for these):\n");
            for result in self.results.iter().filter(|r| !r.killed) {
                report.push_str(&format!(
                    "  - {} in cell '{}' at line {}\n",
                    result.mutation.id,
                    result.mutation.cell_id,
                    result.mutation.line
                ));
            }
        }
        
        report
    }
}