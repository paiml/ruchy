//! Documentation Generation Handler
//!
//! Handles generation of documentation from Ruchy source files.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use ruchy::frontend::ast::{CommentKind, Expr, ExprKind, Pattern};
use ruchy::frontend::parser::Parser;
use std::fs;
use std::path::Path;

/// Handle doc command - generate documentation from Ruchy files
///
/// # Arguments
/// * `path` - Path to file or directory to document
/// * `output` - Output directory for generated documentation
/// * `format` - Output format (html, markdown, json)
/// * `private` - Include private items in documentation
/// * `_open` - Open documentation in browser (not yet implemented)
/// * `_all` - Include all items (not yet implemented)
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if file cannot be read, parsed, or documentation cannot be generated
pub fn handle_doc_command(
    path: &Path,
    output: &Path,
    format: &str,
    private: bool,
    _open: bool,
    _all: bool,
    verbose: bool,
) -> Result<()> {
    // Validate format
    if !matches!(format, "html" | "markdown" | "json") {
        bail!(
            "Invalid format '{}'. Supported formats: html, markdown, json",
            format
        );
    }

    // Check if path exists
    if !path.exists() {
        bail!("File or directory not found: {}", path.display());
    }

    if verbose {
        println!("{} Parsing {}...", "→".bright_blue(), path.display());
    }

    // Read and parse the file
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut parser = Parser::new(&source);
    let ast = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    if verbose {
        println!("{} Extracting documentation...", "→".bright_blue());
    }

    // Extract documentation from AST
    let docs = extract_documentation(&ast, private);

    if verbose {
        println!(
            "{} Generating {} documentation...",
            "→".bright_blue(),
            format
        );
    }

    // Create output directory
    fs::create_dir_all(output)
        .with_context(|| format!("Failed to create output directory: {}", output.display()))?;

    // Generate documentation in requested format
    let content = match format {
        "markdown" => generate_markdown_docs(&docs, path),
        "json" => generate_json_docs(&docs, path)?,
        "html" => generate_html_docs(&docs, path),
        _ => unreachable!(),
    };

    // Determine output filename
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("docs");
    let extension = match format {
        "markdown" => "md",
        "json" => "json",
        "html" => "html",
        _ => unreachable!(),
    };
    let output_file = output.join(format!("{}.{}", file_stem, extension));

    // Write documentation
    fs::write(&output_file, content)
        .with_context(|| format!("Failed to write documentation: {}", output_file.display()))?;

    println!(
        "{} Generated documentation: {}",
        "✓".bright_green(),
        output_file.display()
    );

    Ok(())
}

/// Documentation item extracted from source code
#[derive(Debug)]
pub struct DocItem {
    pub kind: DocItemKind,
    pub name: String,
    pub params: Vec<String>,
    pub doc_comment: Option<String>,
}

#[derive(Debug)]
pub enum DocItemKind {
    Function,
}

/// Extract documentation from AST
pub fn extract_documentation(ast: &Expr, include_private: bool) -> Vec<DocItem> {
    let mut docs = Vec::new();
    extract_docs_recursive(ast, &mut docs, include_private);
    docs
}

/// Recursively extract documentation from AST nodes
fn extract_docs_recursive(expr: &Expr, docs: &mut Vec<DocItem>, include_private: bool) {
    match &expr.kind {
        ExprKind::Function { name, params, .. } => {
            // Extract leading doc comments from Comment structs
            let doc_comment = expr
                .leading_comments
                .iter()
                .map(|c| match &c.kind {
                    CommentKind::Line(text)
                    | CommentKind::Block(text)
                    | CommentKind::Doc(text) => text.clone(),
                })
                .collect::<Vec<_>>()
                .join("\n");
            let has_doc = !doc_comment.is_empty() || include_private;

            if has_doc {
                // Extract parameter names from patterns
                let param_names: Vec<String> = params
                    .iter()
                    .map(|p| match &p.pattern {
                        Pattern::Identifier(name) => name.clone(),
                        _ => "_".to_string(),
                    })
                    .collect();

                docs.push(DocItem {
                    kind: DocItemKind::Function,
                    name: name.clone(),
                    params: param_names,
                    doc_comment: if doc_comment.is_empty() {
                        None
                    } else {
                        Some(doc_comment)
                    },
                });
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                extract_docs_recursive(e, docs, include_private);
            }
        }
        _ => {}
    }
}

/// Generate Markdown documentation
pub fn generate_markdown_docs(docs: &[DocItem], source_path: &Path) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "# Documentation for {}\n\n",
        source_path.display()
    ));

    for doc in docs {
        match doc.kind {
            DocItemKind::Function => {
                output.push_str(&format!("## `{}({})`\n\n", doc.name, doc.params.join(", ")));
                if let Some(comment) = &doc.doc_comment {
                    let clean_comment = comment
                        .lines()
                        .map(|line| line.trim_start_matches("///").trim())
                        .collect::<Vec<_>>()
                        .join("\n");
                    output.push_str(&format!("{}\n\n", clean_comment));
                } else {
                    output.push_str("*No documentation available*\n\n");
                }
            }
        }
    }

    output
}

/// Generate JSON documentation
pub fn generate_json_docs(docs: &[DocItem], source_path: &Path) -> Result<String> {
    let mut json_docs = Vec::new();

    for doc in docs {
        let mut obj = serde_json::Map::new();
        obj.insert("kind".to_string(), serde_json::json!("function"));
        obj.insert("name".to_string(), serde_json::json!(doc.name));
        obj.insert("params".to_string(), serde_json::json!(doc.params));
        if let Some(comment) = &doc.doc_comment {
            let clean_comment = comment
                .lines()
                .map(|line| line.trim_start_matches("///").trim())
                .collect::<Vec<_>>()
                .join("\n");
            obj.insert("doc_comment".to_string(), serde_json::json!(clean_comment));
        }
        json_docs.push(serde_json::Value::Object(obj));
    }

    let result = serde_json::json!({
        "source": source_path.display().to_string(),
        "items": json_docs
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

/// Generate HTML documentation
pub fn generate_html_docs(docs: &[DocItem], source_path: &Path) -> String {
    let mut output = String::new();
    output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    output.push_str(&format!(
        "<title>Documentation for {}</title>\n",
        source_path.display()
    ));
    output.push_str("<style>\n");
    output.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
    output.push_str("h1 { color: #333; }\n");
    output.push_str("h2 { color: #666; border-bottom: 1px solid #ddd; padding-bottom: 5px; }\n");
    output.push_str("code { background: #f4f4f4; padding: 2px 5px; border-radius: 3px; }\n");
    output.push_str("</style>\n</head>\n<body>\n");
    output.push_str(&format!(
        "<h1>Documentation for {}</h1>\n",
        source_path.display()
    ));

    for doc in docs {
        match doc.kind {
            DocItemKind::Function => {
                output.push_str(&format!(
                    "<h2><code>{}({})</code></h2>\n",
                    doc.name,
                    doc.params.join(", ")
                ));
                if let Some(comment) = &doc.doc_comment {
                    let clean_comment = comment
                        .lines()
                        .map(|line| line.trim_start_matches("///").trim())
                        .collect::<Vec<_>>()
                        .join("<br>\n");
                    output.push_str(&format!("<p>{}</p>\n", clean_comment));
                } else {
                    output.push_str("<p><em>No documentation available</em></p>\n");
                }
            }
        }
    }

    output.push_str("</body>\n</html>\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_markdown_empty() {
        let docs: Vec<DocItem> = vec![];
        let result = generate_markdown_docs(&docs, Path::new("test.ruchy"));
        assert!(result.contains("# Documentation for"));
    }

    #[test]
    fn test_generate_markdown_with_function() {
        let docs = vec![DocItem {
            kind: DocItemKind::Function,
            name: "foo".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            doc_comment: Some("/// Does something".to_string()),
        }];
        let result = generate_markdown_docs(&docs, Path::new("test.ruchy"));
        assert!(result.contains("## `foo(x, y)`"));
        assert!(result.contains("Does something"));
    }

    #[test]
    fn test_generate_json_empty() {
        let docs: Vec<DocItem> = vec![];
        let result = generate_json_docs(&docs, Path::new("test.ruchy")).unwrap();
        assert!(result.contains("\"items\": []"));
    }

    #[test]
    fn test_generate_json_with_function() {
        let docs = vec![DocItem {
            kind: DocItemKind::Function,
            name: "bar".to_string(),
            params: vec!["a".to_string()],
            doc_comment: None,
        }];
        let result = generate_json_docs(&docs, Path::new("test.ruchy")).unwrap();
        assert!(result.contains("\"name\": \"bar\""));
    }

    #[test]
    fn test_generate_html_empty() {
        let docs: Vec<DocItem> = vec![];
        let result = generate_html_docs(&docs, Path::new("test.ruchy"));
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_generate_html_with_function() {
        let docs = vec![DocItem {
            kind: DocItemKind::Function,
            name: "baz".to_string(),
            params: vec![],
            doc_comment: Some("/// Test function".to_string()),
        }];
        let result = generate_html_docs(&docs, Path::new("test.ruchy"));
        assert!(result.contains("<code>baz()</code>"));
        assert!(result.contains("Test function"));
    }
}
