/// Property-based tests module
///
/// Contains proptest-based property tests for various components
///
/// CERTEZA Phase 3: Property Testing Expansion
/// Target: 80%+ property test coverage for High-Risk modules
///
/// Modules:
/// - parser_properties: 30+ properties for parser correctness (P0 CRITICAL)
/// - typechecker_properties: 10+ properties for type inference/unification (P0 CRITICAL)
/// - class_properties: 10 properties for class/struct parsing
///
/// Total: 50+ properties covering High-Risk modules (parser, type checker)

pub mod class_properties;
pub mod parser_properties;
pub mod typechecker_properties;
