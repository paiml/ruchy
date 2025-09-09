use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Represents a cell in a demo file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemoCell {
    /// Markdown documentation cell
    Markdown {
        content: String,
        metadata: CellMetadata,
    },
    /// Code cell with Ruchy code
    Code {
        source: String,
        metadata: CellMetadata,
    },
    /// Section header derived from println patterns
    Section {
        title: String,
        level: usize,
    },
}

/// Metadata for cells
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellMetadata {
    pub tags: Vec<String>,
    pub collapsed: bool,
    pub deletable: bool,
    pub editable: bool,
}

/// Parser for converting Ruchy demo files to notebook cells
pub struct DemoParser {
    cells: Vec<DemoCell>,
    current_code: Vec<String>,
    current_comments: Vec<String>,
}

impl DemoParser {
    /// Create a new demo parser
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            current_code: Vec::new(),
            current_comments: Vec::new(),
        }
    }
    
    /// Parse a demo file from path
    pub fn parse_file(&mut self, path: &Path) -> Result<Vec<DemoCell>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read demo file: {}", path.display()))?;
        
        self.parse_content(&content)
    }
    
    /// Parse demo content
    pub fn parse_content(&mut self, content: &str) -> Result<Vec<DemoCell>> {
        self.cells.clear();
        self.current_code.clear();
        self.current_comments.clear();
        
        for line in content.lines() {
            self.process_line(line);
        }
        
        // Flush any remaining content
        self.flush_current_cell();
        
        Ok(self.cells.clone())
    }
    
    /// Process a single line
    fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();
        
        // Check for section markers (common patterns in demos)
        if self.is_section_marker(trimmed) {
            self.flush_current_cell();
            self.add_section(trimmed);
        }
        // Check for comment lines
        else if trimmed.starts_with("//") {
            let comment = trimmed.trim_start_matches("//").trim();
            
            // Check if this is a documentation comment
            if comment.starts_with('/') || comment.starts_with('!') {
                self.current_comments.push(comment.trim_start_matches(['/', '!']).trim().to_string());
            } else {
                self.current_comments.push(comment.to_string());
            }
        }
        // Check for println that might be section headers
        else if trimmed.starts_with("println(") && self.looks_like_header(trimmed) {
            self.flush_current_cell();
            if let Some(header) = self.extract_header(trimmed) {
                self.add_section(&header);
            }
        }
        // Regular code line
        else if !trimmed.is_empty() {
            // If we have accumulated comments, flush them as markdown
            if !self.current_comments.is_empty() && self.current_code.is_empty() {
                self.flush_comments_as_markdown();
            }
            self.current_code.push(line.to_string());
        }
        // Empty line - might signal cell boundary
        else if !self.current_code.is_empty() {
            self.current_code.push(line.to_string());
        }
    }
    
    /// Check if line looks like a section marker
    fn is_section_marker(&self, line: &str) -> bool {
        line.starts_with("// ===") || 
        line.starts_with("// ---") ||
        line.starts_with("// ###") ||
        line.starts_with("// ##") ||
        line.starts_with("// #")
    }
    
    /// Check if println looks like a header
    fn looks_like_header(&self, line: &str) -> bool {
        line.contains("===") || 
        line.contains("---") ||
        line.contains("###") ||
        line.contains("Example") ||
        line.contains("Demo") ||
        line.contains("Section")
    }
    
    /// Extract header text from println
    fn extract_header(&self, line: &str) -> Option<String> {
        // Extract string from println("...")
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if start < end {
                    return Some(line[start+1..end].to_string());
                }
            }
        }
        None
    }
    
    /// Add a section header
    fn add_section(&mut self, title: &str) {
        let clean_title = title
            .trim_start_matches(['/', '=', '-', '#', ' '])
            .trim_end_matches(['=', '-', '#', ' '])
            .trim();
        
        let level = if title.contains("###") { 3 }
        else if title.contains("##") { 2 }
        else if title.contains('#') { 1 }
        else if title.contains("===") { 1 }
        else { 2 };
        
        self.cells.push(DemoCell::Section {
            title: clean_title.to_string(),
            level,
        });
    }
    
    /// Flush accumulated comments as markdown
    fn flush_comments_as_markdown(&mut self) {
        if !self.current_comments.is_empty() {
            let content = self.current_comments.join("\n");
            self.cells.push(DemoCell::Markdown {
                content,
                metadata: CellMetadata::default(),
            });
            self.current_comments.clear();
        }
    }
    
    /// Flush current cell
    fn flush_current_cell(&mut self) {
        // First flush any comments
        self.flush_comments_as_markdown();
        
        // Then flush code
        if !self.current_code.is_empty() {
            // Remove trailing empty lines
            while self.current_code.last() == Some(&String::new()) {
                self.current_code.pop();
            }
            
            if !self.current_code.is_empty() {
                let source = self.current_code.join("\n");
                self.cells.push(DemoCell::Code {
                    source,
                    metadata: CellMetadata::default(),
                });
                self.current_code.clear();
            }
        }
    }
    
    /// Get current cells
    pub fn cells(&self) -> &[DemoCell] {
        &self.cells
    }
    
    /// Group cells by heuristics (code that belongs together)
    pub fn group_related_code(&mut self) {
        let mut grouped = Vec::new();
        let mut current_group = Vec::new();
        
        for cell in &self.cells {
            match cell {
                DemoCell::Section { .. } | DemoCell::Markdown { .. } => {
                    if !current_group.is_empty() {
                        // Merge code cells in group
                        let merged = self.merge_code_cells(&current_group);
                        grouped.push(merged);
                        current_group.clear();
                    }
                    grouped.push(cell.clone());
                }
                DemoCell::Code { .. } => {
                    current_group.push(cell.clone());
                    
                    // Check if this looks like a complete unit
                    if self.looks_complete(&current_group) {
                        let merged = self.merge_code_cells(&current_group);
                        grouped.push(merged);
                        current_group.clear();
                    }
                }
            }
        }
        
        // Flush remaining group
        if !current_group.is_empty() {
            let merged = self.merge_code_cells(&current_group);
            grouped.push(merged);
        }
        
        self.cells = grouped;
    }
    
    /// Check if code cells look like a complete unit
    fn looks_complete(&self, cells: &[DemoCell]) -> bool {
        if cells.is_empty() { return false; }
        
        let total_lines: usize = cells.iter()
            .filter_map(|c| match c {
                DemoCell::Code { source, .. } => Some(source.lines().count()),
                _ => None,
            })
            .sum();
        
        // Simple heuristic: if we have >10 lines, probably complete
        total_lines > 10
    }
    
    /// Merge multiple code cells into one
    fn merge_code_cells(&self, cells: &[DemoCell]) -> DemoCell {
        let mut sources = Vec::new();
        
        for cell in cells {
            if let DemoCell::Code { source, .. } = cell {
                sources.push(source.clone());
            }
        }
        
        DemoCell::Code {
            source: sources.join("\n\n"),
            metadata: CellMetadata::default(),
        }
    }
}

impl Default for DemoParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_demo() {
        let content = r#"
// This is a demo
// It shows basic functionality

println("=== Example 1 ===")

let x = 42
println(x)

// Another section
println("--- Example 2 ---")

fun greet(name) {
    println(f"Hello, {name}!")
}

greet("World")
"#;
        
        let mut parser = DemoParser::new();
        let cells = parser.parse_content(content).unwrap();
        
        assert!(cells.len() > 0);
        
        // Check we have both markdown and code cells
        let has_markdown = cells.iter().any(|c| matches!(c, DemoCell::Markdown { .. }));
        let has_code = cells.iter().any(|c| matches!(c, DemoCell::Code { .. }));
        let has_section = cells.iter().any(|c| matches!(c, DemoCell::Section { .. }));
        
        assert!(has_markdown);
        assert!(has_code);
        assert!(has_section);
    }
    
    #[test]
    fn test_section_detection() {
        let mut parser = DemoParser::new();
        
        assert!(parser.is_section_marker("// === Section ==="));
        assert!(parser.is_section_marker("// --- Subsection ---"));
        assert!(parser.is_section_marker("// ### Header"));
        assert!(!parser.is_section_marker("// Regular comment"));
    }
    
    #[test]
    fn test_header_extraction() {
        let parser = DemoParser::new();
        
        assert_eq!(
            parser.extract_header(r#"println("=== Test ===")"#),
            Some("=== Test ===".to_string())
        );
        
        assert_eq!(
            parser.extract_header(r#"println("Hello World")"#),
            Some("Hello World".to_string())
        );
    }
}