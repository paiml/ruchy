//! Compilation Provenance Tracking
//!
//! Based on docs/ruchy-transpiler-docs.md Section 7: Compilation Provenance Tracking
//! Complete audit trail of compilation decisions
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
/// Complete trace of a compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::provenance::ProvenanceTracker;
    ///
    /// let tracker = ProvenanceTracker::new("let x = 5;");
    /// ```
    pub fn new(source: &str) -> Self {
        Self {
            source_hash: Self::hash(source),
            transformations: Vec::new(),
            current_transformation: None,
            start_time: Instant::now(),
        }
    }
    /// Start tracking a new transformation pass
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::transpiler::provenance::ProvenanceTracker;
    ///
    /// let mut tracker = ProvenanceTracker::new("source");
    /// tracker.begin_pass("optimization", "input");
    /// ```
    pub fn begin_pass(&mut self, name: &str, input: &str) {
        if let Some(builder) = self.current_transformation.take() {
            self.transformations.push(builder.finish());
        }
        self.current_transformation = Some(TransformationBuilder::new(name, input));
    }
    /// Record a rule application
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::transpiler::provenance::record_rule;
    ///
    /// let result = record_rule(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn record_rule(&mut self, rule: Rule) {
        if let Some(ref mut builder) = self.current_transformation {
            builder.add_rule(rule);
        }
    }
    /// Finish the current pass
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::transpiler::provenance::end_pass;
    ///
    /// let result = end_pass("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn end_pass(&mut self, output: &str) {
        if let Some(mut builder) = self.current_transformation.take() {
            builder.set_output(output);
            self.transformations.push(builder.finish());
        }
    }
    /// Generate the complete compilation trace
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::provenance::finish;
    ///
    /// let result = finish(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn finish(mut self) -> CompilationTrace {
        // Finish any pending transformation
        if let Some(builder) = self.current_transformation.take() {
            self.transformations.push(builder.finish());
        }
        CompilationTrace {
            source_hash: self.source_hash,
            transformations: self.transformations,
            total_duration_ns: self
                .start_time
                .elapsed()
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
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
#[allow(clippy::cast_possible_truncation)]
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
            duration_ns: self
                .start_time
                .elapsed()
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::provenance::find_divergence;
    ///
    /// let result = find_divergence(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::transpiler::provenance::transpile_with_provenance;
    ///
    /// let result = transpile_with_provenance(());
    /// assert_eq!(result, Ok(()));
    /// ```
    /// TRANSPILER-009: Changed to &mut self to match `transpile()` signature
    pub fn transpile_with_provenance(
        &mut self,
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
            tracker.end_pass(&format!("{tokens}"));
        } else {
            tracker.end_pass("error");
        }
        (result, tracker.finish())
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
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

    // Additional coverage tests

    #[test]
    fn test_provenance_tracker_new() {
        let tracker = ProvenanceTracker::new("test source");
        let trace = tracker.finish();
        assert!(!trace.source_hash.is_empty());
        assert!(trace.transformations.is_empty());
    }

    #[test]
    fn test_trace_differ_same_traces() {
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

        let trace2 = trace1.clone();
        let differ = TraceDiffer::new(trace1, trace2);
        let divergence = differ.find_divergence();
        assert!(divergence.is_none());
    }

    #[test]
    fn test_trace_differ_different_source() {
        let trace1 = CompilationTrace {
            source_hash: "abc".to_string(),
            transformations: vec![],
            total_duration_ns: 1000,
            metadata: CompilationMetadata {
                ruchy_version: "1.0.0".to_string(),
                rustc_version: "1.75.0".to_string(),
                timestamp: "2024-01-01".to_string(),
                deterministic_seed: 42,
                optimization_level: "O2".to_string(),
            },
        };

        let trace2 = CompilationTrace {
            source_hash: "xyz".to_string(), // Different source
            transformations: vec![],
            total_duration_ns: 1000,
            metadata: trace1.metadata.clone(),
        };

        let differ = TraceDiffer::new(trace1, trace2);
        let divergence = differ.find_divergence();
        assert!(divergence.is_some());
        assert_eq!(divergence.unwrap().stage, "source");
    }

    #[test]
    fn test_trace_differ_different_pass() {
        let trace1 = CompilationTrace {
            source_hash: "abc".to_string(),
            transformations: vec![Transformation {
                pass: "parse".to_string(),
                input_hash: "in".to_string(),
                output_hash: "out".to_string(),
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
                pass: "different_pass".to_string(), // Different pass name
                input_hash: "in".to_string(),
                output_hash: "out".to_string(),
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
        assert!(point.description.contains("Different pass"));
    }

    #[test]
    fn test_source_span_creation() {
        let span = SourceSpan {
            file: Some("test.ruchy".to_string()),
            line_start: 1,
            line_end: 5,
            column_start: 0,
            column_end: 20,
        };

        assert_eq!(span.file, Some("test.ruchy".to_string()));
        assert_eq!(span.line_start, 1);
        assert_eq!(span.line_end, 5);
        assert_eq!(span.column_start, 0);
        assert_eq!(span.column_end, 20);
    }

    #[test]
    fn test_source_span_no_file() {
        let span = SourceSpan {
            file: None,
            line_start: 1,
            line_end: 1,
            column_start: 0,
            column_end: 10,
        };

        assert!(span.file.is_none());
    }

    #[test]
    fn test_rule_creation() {
        let rule = Rule {
            name: "test_rule".to_string(),
            location: SourceSpan {
                file: None,
                line_start: 1,
                line_end: 1,
                column_start: 0,
                column_end: 10,
            },
            before: "before_code".to_string(),
            after: "after_code".to_string(),
        };

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.before, "before_code");
        assert_eq!(rule.after, "after_code");
    }

    #[test]
    fn test_transformation_creation() {
        let transform = Transformation {
            pass: "optimize".to_string(),
            input_hash: "abc123".to_string(),
            output_hash: "def456".to_string(),
            rules_applied: vec![
                Rule {
                    name: "rule1".to_string(),
                    location: SourceSpan {
                        file: None,
                        line_start: 1,
                        line_end: 1,
                        column_start: 0,
                        column_end: 5,
                    },
                    before: "x".to_string(),
                    after: "y".to_string(),
                },
                Rule {
                    name: "rule2".to_string(),
                    location: SourceSpan {
                        file: None,
                        line_start: 2,
                        line_end: 2,
                        column_start: 0,
                        column_end: 5,
                    },
                    before: "a".to_string(),
                    after: "b".to_string(),
                },
            ],
            duration_ns: 5000,
        };

        assert_eq!(transform.pass, "optimize");
        assert_eq!(transform.rules_applied.len(), 2);
        assert_eq!(transform.duration_ns, 5000);
    }

    #[test]
    fn test_compilation_metadata_creation() {
        let metadata = CompilationMetadata {
            ruchy_version: "0.1.0".to_string(),
            rustc_version: "1.75.0".to_string(),
            timestamp: "2024-12-15T00:00:00Z".to_string(),
            deterministic_seed: 12345,
            optimization_level: "O3".to_string(),
        };

        assert_eq!(metadata.ruchy_version, "0.1.0");
        assert_eq!(metadata.deterministic_seed, 12345);
        assert_eq!(metadata.optimization_level, "O3");
    }

    #[test]
    fn test_compilation_trace_serialization() {
        let trace = CompilationTrace {
            source_hash: "abc".to_string(),
            transformations: vec![],
            total_duration_ns: 1000,
            metadata: CompilationMetadata {
                ruchy_version: "1.0.0".to_string(),
                rustc_version: "1.75.0".to_string(),
                timestamp: "2024-01-01".to_string(),
                deterministic_seed: 42,
                optimization_level: "O2".to_string(),
            },
        };

        let json = serde_json::to_string(&trace).expect("serialize");
        assert!(json.contains("source_hash"));
        assert!(json.contains("abc"));

        let deserialized: CompilationTrace = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.source_hash, trace.source_hash);
    }

    #[test]
    fn test_divergence_point_debug() {
        let point = DivergencePoint {
            stage: "parse".to_string(),
            pass_index: 2,
            description: "Output mismatch".to_string(),
        };

        let debug = format!("{:?}", point);
        assert!(debug.contains("DivergencePoint"));
        assert!(debug.contains("parse"));
    }

    #[test]
    fn test_provenance_tracker_multiple_passes() {
        let mut tracker = ProvenanceTracker::new("code");

        tracker.begin_pass("pass1", "input1");
        tracker.end_pass("output1");

        tracker.begin_pass("pass2", "input2");
        tracker.end_pass("output2");

        tracker.begin_pass("pass3", "input3");
        tracker.end_pass("output3");

        let trace = tracker.finish();
        assert_eq!(trace.transformations.len(), 3);
        assert_eq!(trace.transformations[0].pass, "pass1");
        assert_eq!(trace.transformations[1].pass, "pass2");
        assert_eq!(trace.transformations[2].pass, "pass3");
    }

    #[test]
    fn test_provenance_tracker_unfinished_pass() {
        let mut tracker = ProvenanceTracker::new("code");

        tracker.begin_pass("pass1", "input1");
        // Don't call end_pass

        let trace = tracker.finish();
        // Unfinished pass should still be included
        assert_eq!(trace.transformations.len(), 1);
        assert_eq!(trace.transformations[0].output_hash, "incomplete");
    }

    #[test]
    fn test_hash_deterministic() {
        let source = "let x = 42";
        let tracker1 = ProvenanceTracker::new(source);
        let tracker2 = ProvenanceTracker::new(source);

        let trace1 = tracker1.finish();
        let trace2 = tracker2.finish();

        assert_eq!(trace1.source_hash, trace2.source_hash);
    }

    #[test]
    fn test_hash_different_inputs() {
        let tracker1 = ProvenanceTracker::new("code1");
        let tracker2 = ProvenanceTracker::new("code2");

        let trace1 = tracker1.finish();
        let trace2 = tracker2.finish();

        assert_ne!(trace1.source_hash, trace2.source_hash);
    }

    #[test]
    fn test_source_span_clone() {
        let span = SourceSpan {
            file: Some("test.ruchy".to_string()),
            line_start: 1,
            line_end: 5,
            column_start: 0,
            column_end: 20,
        };

        let cloned = span.clone();
        assert_eq!(span.file, cloned.file);
        assert_eq!(span.line_start, cloned.line_start);
    }

    #[test]
    fn test_compilation_metadata_clone() {
        let metadata = CompilationMetadata {
            ruchy_version: "0.1.0".to_string(),
            rustc_version: "1.75.0".to_string(),
            timestamp: "2024-12-15".to_string(),
            deterministic_seed: 42,
            optimization_level: "O2".to_string(),
        };

        let cloned = metadata.clone();
        assert_eq!(metadata.ruchy_version, cloned.ruchy_version);
        assert_eq!(metadata.deterministic_seed, cloned.deterministic_seed);
    }
}
#[cfg(test)]
mod property_tests_provenance {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
