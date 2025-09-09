use std::collections::HashMap;

/// Engine for generating helpful error suggestions
pub struct SuggestionEngine {
    symbols: HashMap<String, SymbolInfo>,
}

/// Information about a symbol for suggestions
#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    kind: SymbolKind,
    usage_count: usize,
}

/// Types of symbols
#[derive(Debug, Clone, PartialEq)]
enum SymbolKind {
    Variable,
    Function,
    Type,
    Module,
    Keyword,
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new() -> Self {
        let mut engine = Self {
            symbols: HashMap::new(),
        };
        
        // Add common keywords and built-ins
        engine.add_builtins();
        engine
    }
    
    /// Add built-in symbols
    fn add_builtins(&mut self) {
        let keywords = vec![
            "let", "mut", "fun", "if", "else", "while", "for", "in", 
            "match", "return", "break", "continue", "true", "false",
            "null", "async", "await", "import", "export", "struct",
            "enum", "trait", "impl", "pub", "use", "mod",
        ];
        
        for keyword in keywords {
            self.add_symbol(keyword.to_string(), SymbolKind::Keyword);
        }
        
        let builtins = vec![
            "println", "print", "len", "push", "pop", "get", "set",
            "map", "filter", "reduce", "sort", "reverse", "join",
            "split", "trim", "replace", "contains", "starts_with",
            "ends_with", "to_string", "to_int", "to_float", "type_of",
        ];
        
        for builtin in builtins {
            self.add_symbol(builtin.to_string(), SymbolKind::Function);
        }
    }
    
    /// Add a symbol to the engine
    pub fn add_symbol(&mut self, name: String, kind: SymbolKind) {
        let entry = self.symbols.entry(name.clone()).or_insert(SymbolInfo {
            name,
            kind,
            usage_count: 0,
        });
        entry.usage_count += 1;
    }
    
    /// Add a variable
    pub fn add_variable(&mut self, name: String) {
        self.add_symbol(name, SymbolKind::Variable);
    }
    
    /// Add a function
    pub fn add_function(&mut self, name: String) {
        self.add_symbol(name, SymbolKind::Function);
    }
    
    /// Get suggestions for an undefined symbol
    pub fn suggest_for_undefined(&self, undefined_name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let mut scored_matches = Vec::new();
        
        // Calculate similarity scores
        for symbol in self.symbols.values() {
            let distance = levenshtein_distance(&symbol.name, undefined_name);
            let max_len = symbol.name.len().max(undefined_name.len());
            
            if max_len == 0 {
                continue;
            }
            
            let similarity = 1.0 - (distance as f64 / max_len as f64);
            
            // Consider suggestions if similarity is high enough
            if similarity >= 0.6 {
                let score = similarity * (1.0 + (symbol.usage_count as f64 / 100.0));
                scored_matches.push((symbol.name.clone(), score, symbol.kind.clone()));
            }
        }
        
        // Sort by score (higher is better)
        scored_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Generate suggestions
        for (name, _score, kind) in scored_matches.into_iter().take(3) {
            let suggestion = match kind {
                SymbolKind::Variable => format!("Did you mean the variable '{}'?", name),
                SymbolKind::Function => format!("Did you mean the function '{}'?", name),
                SymbolKind::Type => format!("Did you mean the type '{}'?", name),
                SymbolKind::Module => format!("Did you mean the module '{}'?", name),
                SymbolKind::Keyword => format!("Did you mean the keyword '{}'?", name),
            };
            suggestions.push(suggestion);
        }
        
        // Add general advice if no good matches
        if suggestions.is_empty() {
            suggestions.push("Check the spelling of the identifier".to_string());
            suggestions.push("Make sure the variable is defined before use".to_string());
            suggestions.push("Check if you need to import a module".to_string());
        }
        
        suggestions
    }
    
    /// Get suggestions for syntax errors
    pub fn suggest_for_syntax_error(&self, error_msg: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if error_msg.contains("expected ';'") {
            suggestions.push("Add a semicolon at the end of the statement".to_string());
            suggestions.push("Check if you forgot a semicolon on the previous line".to_string());
        } else if error_msg.contains("expected '}'") {
            suggestions.push("Add a closing brace '}'".to_string());
            suggestions.push("Check for unmatched opening braces '{'".to_string());
        } else if error_msg.contains("expected ')'") {
            suggestions.push("Add a closing parenthesis ')'".to_string());
            suggestions.push("Check for unmatched opening parentheses '('".to_string());
        } else if error_msg.contains("expected identifier") {
            suggestions.push("Add a valid identifier (variable or function name)".to_string());
            suggestions.push("Identifiers must start with a letter or underscore".to_string());
        } else if error_msg.contains("unexpected token") {
            suggestions.push("Remove the unexpected token".to_string());
            suggestions.push("Check for typos in keywords or operators".to_string());
        }
        
        suggestions
    }
    
    /// Get suggestions for type errors
    pub fn suggest_for_type_error(&self, expected: &str, found: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if expected == "int" && found == "string" {
            suggestions.push("Use .to_int() to convert string to integer".to_string());
            suggestions.push("Parse the string: int.parse(value)".to_string());
        } else if expected == "string" && found == "int" {
            suggestions.push("Use .to_string() to convert integer to string".to_string());
            suggestions.push("Use string formatting: f\"{value}\"".to_string());
        } else if expected == "bool" && (found == "int" || found == "string") {
            suggestions.push("Use comparison operators to get a boolean".to_string());
            suggestions.push("Convert explicitly: value != 0 or !value.is_empty()".to_string());
        } else {
            suggestions.push(format!("Expected type '{}', but got '{}'", expected, found));
            suggestions.push("Check the types of your variables and expressions".to_string());
        }
        
        suggestions
    }
    
    /// Clear all user-defined symbols (keep built-ins)
    pub fn clear_user_symbols(&mut self) {
        self.symbols.retain(|_, info| {
            matches!(info.kind, SymbolKind::Keyword | SymbolKind::Function) && info.usage_count == 1
        });
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    
    if len1 == 0 { return len2; }
    if len2 == 0 { return len1; }
    
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i-1][j] + 1,      // deletion
                    matrix[i][j-1] + 1,      // insertion
                ),
                matrix[i-1][j-1] + cost,     // substitution
            );
        }
    }
    
    matrix[len1][len2]
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }
    
    #[test]
    fn test_undefined_suggestions() {
        let mut engine = SuggestionEngine::new();
        engine.add_variable("my_variable".to_string());
        engine.add_function("calculate".to_string());
        
        let suggestions = engine.suggest_for_undefined("my_variabel"); // typo
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("my_variable"));
        
        let suggestions = engine.suggest_for_undefined("calc");
        assert!(!suggestions.is_empty());
        // Should suggest "calculate" due to prefix match
    }
    
    #[test]
    fn test_syntax_suggestions() {
        let engine = SuggestionEngine::new();
        
        let suggestions = engine.suggest_for_syntax_error("expected ';'");
        assert!(suggestions.iter().any(|s| s.contains("semicolon")));
        
        let suggestions = engine.suggest_for_syntax_error("expected '}'");
        assert!(suggestions.iter().any(|s| s.contains("brace")));
    }
    
    #[test]
    fn test_type_suggestions() {
        let engine = SuggestionEngine::new();
        
        let suggestions = engine.suggest_for_type_error("int", "string");
        assert!(suggestions.iter().any(|s| s.contains("to_int")));
        
        let suggestions = engine.suggest_for_type_error("string", "int");
        assert!(suggestions.iter().any(|s| s.contains("to_string")));
    }
}