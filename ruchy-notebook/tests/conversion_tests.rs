use ruchy_notebook::{DemoParser, NotebookConverter};
use ruchy_notebook::converter::{DemoCell, NotebookFormat};
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_demo_to_notebook_conversion() {
    let demo_content = r#"
// # Data Analysis Demo
// This demonstrates Ruchy's data capabilities

println("=== Loading Data ===")

// Load CSV data
let df = read_csv("data.csv")
println(df.shape())

// Basic statistics
let stats = df.describe()
println(stats)

// === Visualization ===

// Create a histogram
let hist = df.column("value").hist()
hist.show()

// Filter and group data
let filtered = df.filter(col("value") > 100)
let grouped = filtered.group_by("category").sum()
println(grouped)
"#;
    
    let mut parser = DemoParser::new();
    let cells = parser.parse_content(demo_content).unwrap();
    
    // Should have mixed cell types
    let has_markdown = cells.iter().any(|c| matches!(c, DemoCell::Markdown { .. }));
    let has_code = cells.iter().any(|c| matches!(c, DemoCell::Code { .. }));
    let has_section = cells.iter().any(|c| matches!(c, DemoCell::Section { .. }));
    
    assert!(has_markdown, "Should have markdown cells from comments");
    assert!(has_code, "Should have code cells");
    assert!(has_section, "Should have section headers");
    
    // Convert to notebook
    let notebook = NotebookConverter::to_jupyter(&cells);
    
    assert!(notebook.cells.len() >= 3);
    assert_eq!(notebook.metadata.kernelspec.language, "ruchy");
    assert_eq!(notebook.nbformat, 4);
}

#[test]
fn test_notebook_round_trip() {
    let original_demo = r#"
// This is a test
// Multi-line comment

let x = 42
let y = x * 2
println(f"Result: {y}")

// Another section
fun factorial(n) {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}

println(factorial(5))
"#;
    
    let mut parser = DemoParser::new();
    let original_cells = parser.parse_content(original_demo).unwrap();
    
    // Round trip: cells -> notebook -> script -> cells
    let notebook = NotebookConverter::to_jupyter(&original_cells);
    let regenerated_script = NotebookConverter::from_jupyter(&notebook);
    let final_cells = parser.parse_content(&regenerated_script).unwrap();
    
    // Should preserve essential structure
    assert!(!original_cells.is_empty());
    assert!(!final_cells.is_empty());
    
    // Check content preservation
    let original_code: String = original_cells.iter()
        .filter_map(|c| match c {
            DemoCell::Code { source, .. } => Some(source.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    let final_code: String = final_cells.iter()
        .filter_map(|c| match c {
            DemoCell::Code { source, .. } => Some(source.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Core code should be preserved
    assert!(final_code.contains("let x = 42"));
    assert!(final_code.contains("factorial"));
    assert!(final_code.contains("println(factorial(5))"));
}

#[test]
fn test_save_and_load_notebook() {
    let cells = vec![
        DemoCell::Section {
            title: "Test Notebook".to_string(),
            level: 1,
        },
        DemoCell::Markdown {
            content: "This is a test of save/load functionality".to_string(),
            metadata: Default::default(),
        },
        DemoCell::Code {
            source: r#"
// Test function
fun greet(name) {
    println(f"Hello, {name}!")
}

greet("World")
println("Test complete")
"#.trim().to_string(),
            metadata: Default::default(),
        },
    ];
    
    let notebook = NotebookConverter::to_jupyter(&cells);
    
    // Save to temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    
    NotebookConverter::save_notebook(&notebook, path).unwrap();
    
    // Load back
    let loaded_notebook = NotebookConverter::load_notebook(path).unwrap();
    
    // Verify structure preserved
    assert_eq!(loaded_notebook.cells.len(), notebook.cells.len());
    assert_eq!(loaded_notebook.nbformat, notebook.nbformat);
    assert_eq!(loaded_notebook.metadata.kernelspec.name, notebook.metadata.kernelspec.name);
}

#[test]
fn test_complex_demo_parsing() {
    let complex_demo = r#"
// # Advanced Ruchy Features Demo
// This demonstrates advanced language features

println("=== Pattern Matching ===")

// Enum definition
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Pattern matching function
fun handle_result<T>(result: Result<T, String>) -> String {
    match result {
        Ok(value) => f"Success: {value}",
        Err(error) => f"Error: {error}",
    }
}

// Test pattern matching
let good_result = Ok(42)
let bad_result = Err("Something went wrong")

println(handle_result(good_result))
println(handle_result(bad_result))

// --- Async Programming ---

// Async function
async fun fetch_data(url: String) -> Result<String, String> {
    // Simulate network call
    if url.starts_with("https://") {
        Ok(f"Data from {url}")
    } else {
        Err("Invalid URL")
    }
}

// Await example
async fun main() {
    let data = await fetch_data("https://api.example.com")
    match data {
        Ok(content) => println(content),
        Err(error) => println(f"Failed: {error}"),
    }
}

// ### DataFrames and Analysis

// Create sample data
let df = df! {
    "name" => ["Alice", "Bob", "Charlie", "Diana"],
    "age" => [25, 30, 35, 28],
    "salary" => [50000, 75000, 65000, 58000],
}

// Statistical operations
println("DataFrame shape:", df.shape())
println("Average salary:", df.column("salary").mean())

// Filtering and aggregation
let seniors = df.filter(col("age") >= 30)
let avg_senior_salary = seniors.column("salary").mean()
println(f"Average senior salary: ${avg_senior_salary}")
"#;
    
    let mut parser = DemoParser::new();
    let cells = parser.parse_content(complex_demo).unwrap();
    
    parser.group_related_code();
    let grouped_cells = parser.cells();
    
    // Should have meaningful structure
    assert!(grouped_cells.len() >= 5);
    
    // Check for different cell types
    let sections = grouped_cells.iter().filter(|c| matches!(c, DemoCell::Section { .. })).count();
    let markdown = grouped_cells.iter().filter(|c| matches!(c, DemoCell::Markdown { .. })).count();
    let code = grouped_cells.iter().filter(|c| matches!(c, DemoCell::Code { .. })).count();
    
    assert!(sections >= 2, "Should have multiple sections");
    assert!(markdown >= 1, "Should have markdown cells");
    assert!(code >= 2, "Should have multiple code cells");
    
    // Convert to notebook and verify
    let notebook = NotebookConverter::to_jupyter(grouped_cells);
    assert!(notebook.cells.len() >= 5);
    
    // Should preserve all major code sections
    let regenerated = NotebookConverter::from_jupyter(&notebook);
    assert!(regenerated.contains("Pattern Matching"));
    assert!(regenerated.contains("enum Result"));
    assert!(regenerated.contains("async fun"));
    assert!(regenerated.contains("DataFrames"));
}

#[test]
fn test_malformed_input_handling() {
    let malformed_inputs = vec![
        "", // Empty
        "// Only comments\n// No code",
        "let x = // incomplete",
        "println(\"unclosed string",
    ];
    
    let mut parser = DemoParser::new();
    
    for input in malformed_inputs {
        // Should not panic, even on malformed input
        let result = parser.parse_content(input);
        assert!(result.is_ok(), "Should handle malformed input gracefully");
        
        let cells = result.unwrap();
        // Should produce some result, even if empty
        // Don't assert on specific content as malformed input behavior may vary
    }
}

#[test] 
fn test_preserve_code_structure() {
    let structured_code = r#"
// Function definitions
fun add(a, b) {
    return a + b
}

fun multiply(a, b) {
    return a * b
}

// Main logic
let x = 10
let y = 20
let sum = add(x, y)
let product = multiply(x, y)

println(f"Sum: {sum}")
println(f"Product: {product}")

// Test with different values
for i in [1, 2, 3, 4, 5] {
    println(f"{i} + {i} = {add(i, i)}")
    println(f"{i} * {i} = {multiply(i, i)}")
}
"#;
    
    let mut parser = DemoParser::new();
    let cells = parser.parse_content(structured_code).unwrap();
    
    // Convert and back
    let notebook = NotebookConverter::to_jupyter(&cells);
    let restored = NotebookConverter::from_jupyter(&notebook);
    
    // Check that function structure is preserved
    assert!(restored.contains("fun add"));
    assert!(restored.contains("fun multiply"));
    assert!(restored.contains("return a + b"));
    assert!(restored.contains("for i in [1, 2, 3, 4, 5]"));
    
    // Verify notebook has reasonable cell count
    assert!(notebook.cells.len() >= 2);
    assert!(notebook.cells.len() <= 10); // Not too fragmented
}