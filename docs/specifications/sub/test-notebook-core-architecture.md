# Sub-spec: Test Notebook Framework — Core Testing Architecture

**Parent:** [ruchy-test-notebook-framework.md](../ruchy-test-notebook-framework.md) Sections 1-4

---

## Command Architecture

`ruchy test` executes notebook cells as test cases, validating outputs against expectations with property-based and deterministic verification.

```bash
# Test all notebooks in project
ruchy test

# Test specific notebook
ruchy test notebooks/analysis.ruchynb

# Test with coverage
ruchy test --coverage

# Test with mutation
ruchy test --mutate

# Regenerate golden outputs
ruchy test --update-golden
```

## Cell Test Annotations

Notebooks contain test metadata inline with cells.

```rust
// Cell metadata in .ruchynb format
{
  "cell_type": "code",
  "source": "let df = read_csv('data.csv')\ndf.describe()",
  "metadata": {
    "test": {
      "type": "deterministic",
      "golden": "outputs/df_describe.golden",
      "tolerance": 1e-6
    }
  }
}
```

### Test Types

```rust
enum CellTestType {
    Deterministic {
        golden: PathBuf,
        tolerance: Option<f64>,
    },
    Property {
        invariants: Vec<String>,
        generators: HashMap<String, Generator>,
    },
    Regression {
        baseline: PathBuf,
        max_time_factor: f64,
        max_memory_factor: f64,
    },
    Differential {
        backends: Vec<Backend>,
        equivalence: EquivalenceType,
    },
    Skip,
}
```

## Output Validation Engine

### Deterministic Testing (nbval-style)

```rust
impl NotebookTester {
    fn test_deterministic(&self, cell: &Cell) -> TestResult {
        let actual = self.execute_cell(cell)?;
        let expected = self.load_golden(&cell.metadata.golden)?;
        
        match (&actual, &expected) {
            (Output::DataFrame(a), Output::DataFrame(e)) => {
                self.compare_dataframes(a, e, cell.metadata.tolerance)
            }
            (Output::Plot(a), Output::Plot(e)) => {
                self.compare_plots(a, e)  // Perceptual hashing
            }
            (Output::Value(a), Output::Value(e)) => {
                self.compare_values(a, e, cell.metadata.tolerance)
            }
            _ => TestResult::TypeMismatch
        }
    }
    
    fn compare_dataframes(&self, actual: &DataFrame, expected: &DataFrame, tol: f64) -> TestResult {
        // Shape check
        if actual.shape() != expected.shape() {
            return TestResult::ShapeMismatch;
        }
        
        // Numeric columns: use tolerance
        for col in actual.numeric_columns() {
            if !actual[col].approx_eq(&expected[col], tol) {
                return TestResult::NumericDivergence { col, max_delta };
            }
        }
        
        // Categorical columns: exact match
        for col in actual.categorical_columns() {
            if actual[col] != expected[col] {
                return TestResult::CategoricalMismatch { col };
            }
        }
        
        TestResult::Pass
    }
}
```

### Property-Based Cell Testing

```rust
// In-notebook property specification
#[test_cell(property)]
fn test_transformation_properties() {
    // Generator configuration
    let df = generate_dataframe(rows: 100..1000, cols: 5..20);
    
    // Properties that must hold
    assert_property!(df.normalize().mean() ~= 0.0);
    assert_property!(df.normalize().std() ~= 1.0);
    assert_property!(df.filter(|r| r.valid()).len() <= df.len());
}

impl PropertyTester {
    fn test_property_cell(&self, cell: &PropertyCell) -> TestResult {
        let mut failures = vec![];
        
        for _ in 0..self.num_iterations {
            let inputs = self.generate_inputs(&cell.generators);
            let env = self.create_environment(inputs);
            
            match self.execute_with_env(cell, env) {
                Ok(_) => continue,
                Err(PropertyViolation { property, counterexample }) => {
                    failures.push((property, self.shrink(counterexample)));
                }
            }
        }
        
        if failures.is_empty() {
            TestResult::Pass
        } else {
            TestResult::PropertyFailures(failures)
        }
    }
}
```

## State Management Across Cells

Notebooks maintain state between cells. Test framework must handle this correctly.

```rust
struct NotebookTestSession {
    state: NotebookState,
    cell_outputs: Vec<CellOutput>,
    checkpoints: HashMap<CellId, NotebookState>,
}

impl NotebookTestSession {
    fn run_notebook_test(&mut self, notebook: &Notebook) -> TestReport {
        let mut results = vec![];
        
        for cell in &notebook.cells {
            // Checkpoint before potentially failing cell
            if cell.has_test() {
                self.checkpoints.insert(cell.id, self.state.clone());
            }
            
            // Execute cell with accumulated state
            let output = self.execute_with_state(cell, &mut self.state)?;
            
            // Validate if test metadata present
            if let Some(test) = &cell.metadata.test {
                let result = self.validate_output(&output, test);
                
                if result.is_failure() && test.stop_on_failure {
                    // Restore checkpoint for debugging
                    self.state = self.checkpoints[&cell.id].clone();
                    break;
                }
                
                results.push(result);
            }
            
            self.cell_outputs.push(output);
        }
        
        TestReport { results, coverage: self.compute_coverage() }
    }
}
```

