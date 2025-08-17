//! Compilation Provenance Tracking
//!
//! Based on docs/ruchy-transpiler-docs.md Section 7: Compilation Provenance Tracking
//! Complete audit trail of compilation decisions

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Complete trace of a compilation
#[derive(Debug, Serialize, Deserialize)]
pub struct CompilationTrace {
    /// SHA256 hash of the source code
    pub source_hash: String,
    /// All transformations applied
    pub transformations: Vec<Transformation>,
    /// Total compilation time
    pub total_duration_ns: u64,
    /// Metadata about the compilation
    pub metadata: CompilationMetadata,
}

/// Metadata about the compilation environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationMetadata {
    pub ruchy_version: String,
    pub rustc_version: String,
    pub timestamp: String,
    pub deterministic_seed: u64,
    pub optimization_level: String,
}

/// A single transformation pass
#[derive(Debug, Serialize, Deserialize)]
pub struct Transformation {
    /// Name of the transformation pass
    pub pass: String,
    /// Hash of input to this pass
    pub input_hash: String,
    /// Hash of output from this pass
    pub output_hash: String,
    /// Rules applied during this transformation
    pub rules_applied: Vec<Rule>,
    /// Time taken for this pass
    pub duration_ns: u64,
}

/// A single rule application
#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    /// Name of the rule
    pub name: String,
    /// Source location where rule was applied
    pub location: SourceSpan,
    /// Code before transformation
    pub before: String,
    /// Code after transformation
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

/// Provenance tracker that records all compilation decisions
pub struct ProvenanceTracker {
    source_hash: String,
    transformations: Vec<Transformation>,
    current_transformation: Option<TransformationBuilder>,
    start_time: Instant,
}

impl ProvenanceTracker {
    /// Create a new provenance tracker for the given source
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            source_hash: Self::hash(source),
            transformations: Vec::new(),
            current_transformation: None,
            start_time: Instant::now(),
        }
    }

    /// Start tracking a new transformation pass
    pub fn begin_pass(&mut self, name: &str, input: &str) {
        if let Some(builder) = self.current_transformation.take() {
            self.transformations.push(builder.finish());
        }

        self.current_transformation = Some(TransformationBuilder::new(name, input));
    }

    /// Record a rule application
    pub fn record_rule(&mut self, rule: Rule) {
        if let Some(ref mut builder) = self.current_transformation {
            builder.add_rule(rule);
        }
    }

    /// Finish the current pass
    pub fn end_pass(&mut self, output: &str) {
        if let Some(mut builder) = self.current_transformation.take() {
            builder.set_output(output);
            self.transformations.push(builder.finish());
        }
    }

    /// Generate the complete compilation trace
    #[must_use]
    pub fn finish(mut self) -> CompilationTrace {
        // Finish any pending transformation
        if let Some(builder) = self.current_transformation.take() {
            self.transformations.push(builder.finish());
        }

        CompilationTrace {
            source_hash: self.source_hash,
            transformations: self.transformations,
            total_duration_ns: self.start_time.elapsed().as_nanos() as u64,
            metadata: CompilationMetadata {
                ruchy_version: env!("CARGO_PKG_VERSION").to_string(),
                rustc_version: "1.75.0".to_string(), // Would get from rustc --version
                timestamp: chrono::Utc::now().to_rfc3339(),
                deterministic_seed: 42, // Would be configurable
                optimization_level: "O2".to_string(),
            },
        }
    }

    /// Calculate SHA256 hash
    fn hash(s: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Builder for a transformation
struct TransformationBuilder {
    pass: String,
    input_hash: String,
    output_hash: Option<String>,
    rules_applied: Vec<Rule>,
    start_time: Instant,
}

impl TransformationBuilder {
    fn new(pass: &str, input: &str) -> Self {
        Self {
            pass: pass.to_string(),
            input_hash: ProvenanceTracker::hash(input),
            output_hash: None,
            rules_applied: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn add_rule(&mut self, rule: Rule) {
        self.rules_applied.push(rule);
    }

    fn set_output(&mut self, output: &str) {
        self.output_hash = Some(ProvenanceTracker::hash(output));
    }

    fn finish(self) -> Transformation {
        Transformation {
            pass: self.pass,
            input_hash: self.input_hash,
            output_hash: self.output_hash.unwrap_or_else(|| "incomplete".to_string()),
            rules_applied: self.rules_applied,
            duration_ns: self.start_time.elapsed().as_nanos() as u64,
        }
    }
}

/// Compare two compilation traces to find divergence
pub struct TraceDiffer {
    trace1: CompilationTrace,
    trace2: CompilationTrace,
}

impl TraceDiffer {
    #[must_use]
    pub fn new(trace1: CompilationTrace, trace2: CompilationTrace) -> Self {
        Self { trace1, trace2 }
    }

    /// Find the first point where the traces diverge
    #[must_use]
    pub fn find_divergence(&self) -> Option<DivergencePoint> {
        // Check source hash
        if self.trace1.source_hash != self.trace2.source_hash {
            return Some(DivergencePoint {
                stage: "source".to_string(),
                pass_index: 0,
                description: format!(
                    "Different source files: {} vs {}",
                    self.trace1.source_hash, self.trace2.source_hash
                ),
            });
        }

        // Check each transformation
        for (i, (t1, t2)) in self
            .trace1
            .transformations
            .iter()
            .zip(self.trace2.transformations.iter())
            .enumerate()
        {
            if t1.pass != t2.pass {
                return Some(DivergencePoint {
                    stage: "transformation".to_string(),
                    pass_index: i,
                    description: format!(
                        "Different pass at index {}: {} vs {}",
                        i, t1.pass, t2.pass
                    ),
                });
            }

            if t1.output_hash != t2.output_hash {
                return Some(DivergencePoint {
                    stage: "transformation".to_string(),
                    pass_index: i,
                    description: format!(
                        "Different output in pass '{}': {} vs {}",
                        t1.pass, t1.output_hash, t2.output_hash
                    ),
                });
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct DivergencePoint {
    pub stage: String,
    pub pass_index: usize,
    pub description: String,
}

/// Integration with the transpiler
impl crate::Transpiler {
    /// Transpile with provenance tracking
    pub fn transpile_with_provenance(
        &self,
        expr: &crate::Expr,
    ) -> (
        Result<proc_macro2::TokenStream, anyhow::Error>,
        CompilationTrace,
    ) {
        let source = format!("{expr:?}"); // Simplified - would serialize properly
        let mut tracker = ProvenanceTracker::new(&source);

        // Track the transpilation
        tracker.begin_pass("transpile", &source);

        let result = self.transpile(expr);

        if let Ok(ref tokens) = result {
            tracker.end_pass(&tokens.to_string());
        } else {
            tracker.end_pass("error");
        }

        (result, tracker.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_tracking() {
        let mut tracker = ProvenanceTracker::new("let x = 10");

        tracker.begin_pass("parse", "let x = 10");
        tracker.record_rule(Rule {
            name: "let_statement".to_string(),
            location: SourceSpan {
                file: None,
                line_start: 1,
                line_end: 1,
                column_start: 0,
                column_end: 10,
            },
            before: "let x = 10".to_string(),
            after: "Let { name: \"x\", value: 10 }".to_string(),
        });
        tracker.end_pass("Let { name: \"x\", value: 10 }");

        tracker.begin_pass("normalize", "Let { name: \"x\", value: 10 }");
        tracker.end_pass("Let { name: \"x\", value: Literal(10), body: Unit }");

        let trace = tracker.finish();

        assert_eq!(trace.transformations.len(), 2);
        assert_eq!(trace.transformations[0].pass, "parse");
        assert_eq!(trace.transformations[1].pass, "normalize");
    }

    #[test]
    fn test_trace_differ() {
        let trace1 = CompilationTrace {
            source_hash: "abc".to_string(),
            transformations: vec![Transformation {
                pass: "parse".to_string(),
                input_hash: "in1".to_string(),
                output_hash: "out1".to_string(),
                rules_applied: vec![],
                duration_ns: 1000,
            }],
            total_duration_ns: 2000,
            metadata: CompilationMetadata {
                ruchy_version: "1.0.0".to_string(),
                rustc_version: "1.75.0".to_string(),
                timestamp: "2024-01-01".to_string(),
                deterministic_seed: 42,
                optimization_level: "O2".to_string(),
            },
        };

        let trace2 = CompilationTrace {
            source_hash: "abc".to_string(),
            transformations: vec![Transformation {
                pass: "parse".to_string(),
                input_hash: "in1".to_string(),
                output_hash: "out2".to_string(), // Different output
                rules_applied: vec![],
                duration_ns: 1000,
            }],
            total_duration_ns: 2000,
            metadata: trace1.metadata.clone(),
        };

        let differ = TraceDiffer::new(trace1, trace2);
        let divergence = differ.find_divergence();

        assert!(divergence.is_some());
        let point = divergence.unwrap();
        assert_eq!(point.stage, "transformation");
        assert_eq!(point.pass_index, 0);
    }
}
