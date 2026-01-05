//! Oracle Command Handler
//!
//! Handles ML-based compilation error classification using aprender.

use anyhow::Result;

/// Handle oracle command - classify compilation errors using ML
///
/// Uses aprender `RandomForestClassifier` to categorize rustc errors
/// and suggest fixes from pattern database.
///
/// # Arguments
/// * `error_message` - The compilation error message
/// * `code` - Optional error code (e.g., "E0308")
/// * `format` - Output format ("text" or "json")
/// * `verbose` - Show confidence scores and details
///
/// # Returns
/// * Classification result with category and suggestions
pub fn handle_oracle_command(
    error_message: &str,
    code: Option<&str>,
    format: &str,
    verbose: bool,
) -> Result<()> {
    use ruchy::oracle::{CompilationError, ModelPaths, RuchyOracle, SerializedModel};

    if verbose {
        eprintln!("Classifying error: {}", error_message);
        if let Some(c) = code {
            eprintln!("Error code: {}", c);
        }
    }

    // Try to load persisted model first, then fall back to training
    let mut oracle = RuchyOracle::new();
    let paths = ModelPaths::default();
    if let Some(model_path) = paths.find_existing() {
        if let Ok(model) = SerializedModel::load(&model_path) {
            if verbose {
                eprintln!("Loaded model from: {}", model_path.display());
            }
            oracle.load_from_serialized(&model)?;
        } else {
            oracle.train_from_examples()?;
        }
    } else {
        oracle.train_from_examples()?;
    }

    // Create compilation error
    let mut error = CompilationError::new(error_message);
    if let Some(c) = code {
        error = error.with_code(c);
    }

    // Classify
    let classification = oracle.classify(&error);

    // Output result
    if format == "json" {
        let json = serde_json::json!({
            "category": format!("{:?}", classification.category),
            "confidence": classification.confidence,
            "suggestions": classification.suggestions.iter().map(|s| {
                serde_json::json!({
                    "pattern_id": s.pattern_id,
                    "description": s.description,
                    "success_rate": s.success_rate,
                })
            }).collect::<Vec<_>>(),
            "should_auto_fix": classification.should_auto_fix,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("Category: {:?}", classification.category);
        println!("Confidence: {:.2}%", classification.confidence * 100.0);

        if !classification.suggestions.is_empty() {
            println!("\nSuggested fixes:");
            for (i, suggestion) in classification.suggestions.iter().enumerate() {
                println!(
                    "  {}. {} (success rate: {:.0}%)",
                    i + 1,
                    suggestion.description,
                    suggestion.success_rate * 100.0
                );
            }
        }

        if classification.should_auto_fix {
            println!("\n✓ Auto-fix recommended");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_oracle_command_basic() {
        // Just verify the function signature is correct
        let result = handle_oracle_command("test error", None, "text", false);
        // May succeed or fail depending on model availability
        let _ = result;
    }
}
