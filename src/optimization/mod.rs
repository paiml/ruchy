//! Mechanical sympathy tuner - Hardware-aware optimization analysis (RUCHY-0816)

pub mod hardware;
pub mod cache;
pub mod vectorization;
pub mod abstraction;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::frontend::ast::Expr;

/// Hardware characteristics for optimization modeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// CPU architecture (`x86_64`, aarch64, etc.)
    pub architecture: String,
    
    /// Number of CPU cores
    pub cores: usize,
    
    /// L1 cache size in bytes
    pub l1_cache_size: usize,
    
    /// L2 cache size in bytes
    pub l2_cache_size: usize,
    
    /// L3 cache size in bytes
    pub l3_cache_size: Option<usize>,
    
    /// Cache line size in bytes
    pub cache_line_size: usize,
    
    /// Branch predictor characteristics
    pub branch_predictor: BranchPredictorProfile,
    
    /// Vectorization capabilities
    pub vector_units: VectorUnitProfile,
}

/// Branch predictor characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPredictorProfile {
    /// Branch history table size
    pub bht_size: usize,
    
    /// Pattern history table size
    pub pht_size: usize,
    
    /// Prediction accuracy for regular patterns
    pub regular_accuracy: f64,
    
    /// Prediction accuracy for irregular patterns
    pub irregular_accuracy: f64,
}

/// Vector unit capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorUnitProfile {
    /// SIMD instruction sets available (SSE, AVX, AVX2, AVX512, NEON)
    pub instruction_sets: Vec<String>,
    
    /// Vector register width in bits
    pub register_width: usize,
    
    /// Number of vector execution units
    pub execution_units: usize,
    
    /// Operations per cycle for different types
    pub throughput: HashMap<String, f64>,
}

/// Optimization analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAnalysis {
    /// Hardware profile used
    pub hardware: HardwareProfile,
    
    /// Cache analysis results
    pub cache_analysis: cache::CacheAnalysis,
    
    /// Branch prediction analysis
    pub branch_analysis: cache::BranchAnalysis,
    
    /// Vectorization opportunities
    pub vectorization_opportunities: Vec<vectorization::VectorizationOpportunity>,
    
    /// Zero-cost abstraction verification
    pub abstraction_analysis: abstraction::AbstractionAnalysis,
    
    /// Overall optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    
    /// Performance score (0.0-1.0)
    pub performance_score: f64,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    
    /// Priority level (Critical/High/Medium/Low)
    pub priority: Priority,
    
    /// Description of the issue
    pub description: String,
    
    /// Suggested fix
    pub suggestion: String,
    
    /// Estimated performance impact (0.0-1.0)
    pub impact: f64,
    
    /// Code location (file, line, column)
    pub location: Option<CodeLocation>,
}

/// Types of optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationType {
    CacheOptimization,
    BranchPrediction,
    Vectorization,
    MemoryLayout,
    AlgorithmicComplexity,
    ZeroCostAbstraction,
}

/// Priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Critical, // >20% performance impact
    High,     // 10-20% impact
    Medium,   // 5-10% impact
    Low,      // <5% impact
}

/// Code location reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub span_length: usize,
}

/// Main optimization analyzer
pub struct MechanicalSympathyTuner {
    hardware_profile: HardwareProfile,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            architecture: detect_architecture(),
            cores: num_cpus::get(),
            l1_cache_size: 32 * 1024,    // 32KB typical
            l2_cache_size: 256 * 1024,   // 256KB typical
            l3_cache_size: Some(8 * 1024 * 1024), // 8MB typical
            cache_line_size: 64,         // 64 bytes typical
            branch_predictor: BranchPredictorProfile {
                bht_size: 16384,
                pht_size: 16384,
                regular_accuracy: 0.95,
                irregular_accuracy: 0.7,
            },
            vector_units: VectorUnitProfile {
                instruction_sets: detect_vector_capabilities(),
                register_width: 256, // AVX2 typical
                execution_units: 2,
                throughput: create_default_throughput_map(),
            },
        }
    }
}

impl MechanicalSympathyTuner {
    pub fn new() -> Self {
        Self {
            hardware_profile: HardwareProfile::default(),
        }
    }
    
    pub fn with_hardware_profile(hardware_profile: HardwareProfile) -> Self {
        Self { hardware_profile }
    }
    
    /// Analyze code for mechanical sympathy optimizations
    pub fn analyze(&self, ast: &Expr, _source_file: &str) -> OptimizationAnalysis {
        let cache_analysis = cache::analyze_cache_behavior(ast, &self.hardware_profile);
        let branch_analysis = cache::analyze_branch_patterns(ast, &self.hardware_profile);
        let vectorization_opportunities = vectorization::find_vectorization_opportunities(ast);
        let abstraction_analysis = abstraction::analyze_abstractions(ast);
        
        let mut recommendations = Vec::new();
        
        // Generate cache recommendations
        recommendations.extend(self.generate_cache_recommendations(&cache_analysis));
        
        // Generate branch prediction recommendations
        recommendations.extend(self.generate_branch_recommendations(&branch_analysis));
        
        // Generate vectorization recommendations
        recommendations.extend(self.generate_vectorization_recommendations(&vectorization_opportunities));
        
        // Generate abstraction recommendations
        recommendations.extend(self.generate_abstraction_recommendations(&abstraction_analysis));
        
        // Calculate overall performance score
        let performance_score = self.calculate_performance_score(&cache_analysis, &branch_analysis, &vectorization_opportunities, &abstraction_analysis);
        
        OptimizationAnalysis {
            hardware: self.hardware_profile.clone(),
            cache_analysis,
            branch_analysis,
            vectorization_opportunities,
            abstraction_analysis,
            recommendations,
            performance_score,
        }
    }
    
    fn generate_cache_recommendations(&self, analysis: &cache::CacheAnalysis) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if analysis.cache_miss_rate > 0.1 {
            recommendations.push(OptimizationRecommendation {
                optimization_type: OptimizationType::CacheOptimization,
                priority: if analysis.cache_miss_rate > 0.3 { Priority::Critical } else { Priority::High },
                description: format!("High cache miss rate: {:.1}%", analysis.cache_miss_rate * 100.0),
                suggestion: "Consider data structure reorganization for better cache locality".to_string(),
                impact: analysis.cache_miss_rate.min(0.5),
                location: None,
            });
        }
        
        if analysis.false_sharing_risk > 0.5 {
            recommendations.push(OptimizationRecommendation {
                optimization_type: OptimizationType::CacheOptimization,
                priority: Priority::High,
                description: "Potential false sharing detected".to_string(),
                suggestion: "Align data structures to cache line boundaries or use padding".to_string(),
                impact: 0.15,
                location: None,
            });
        }
        
        recommendations
    }
    
    fn generate_branch_recommendations(&self, analysis: &cache::BranchAnalysis) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if analysis.unpredictable_branches > 5 {
            recommendations.push(OptimizationRecommendation {
                optimization_type: OptimizationType::BranchPrediction,
                priority: Priority::Medium,
                description: format!("{} unpredictable branch patterns detected", analysis.unpredictable_branches),
                suggestion: "Consider using branchless techniques or profile-guided optimization".to_string(),
                impact: 0.1,
                location: None,
            });
        }
        
        recommendations
    }
    
    fn generate_vectorization_recommendations(&self, opportunities: &[vectorization::VectorizationOpportunity]) -> Vec<OptimizationRecommendation> {
        opportunities.iter().map(|opp| {
            OptimizationRecommendation {
                optimization_type: OptimizationType::Vectorization,
                priority: match opp.speedup_potential {
                    x if x > 4.0 => Priority::Critical,
                    x if x > 2.0 => Priority::High,
                    _ => Priority::Medium,
                },
                description: format!("Vectorization opportunity: {}", opp.description),
                suggestion: format!("Apply {:?} vectorization for {:.1}x speedup", opp.vector_type, opp.speedup_potential),
                impact: (opp.speedup_potential - 1.0) / opp.speedup_potential,
                location: opp.location.clone(),
            }
        }).collect()
    }
    
    fn generate_abstraction_recommendations(&self, analysis: &abstraction::AbstractionAnalysis) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if analysis.runtime_overhead > 0.05 {
            recommendations.push(OptimizationRecommendation {
                optimization_type: OptimizationType::ZeroCostAbstraction,
                priority: if analysis.runtime_overhead > 0.2 { Priority::Critical } else { Priority::High },
                description: format!("Non-zero-cost abstractions detected: {:.1}% overhead", analysis.runtime_overhead * 100.0),
                suggestion: "Replace with zero-cost alternatives or optimize hot paths".to_string(),
                impact: analysis.runtime_overhead,
                location: None,
            });
        }
        
        recommendations
    }
    
    fn calculate_performance_score(
        &self,
        cache: &cache::CacheAnalysis,
        branch: &cache::BranchAnalysis,
        vectorization: &[vectorization::VectorizationOpportunity],
        abstraction: &abstraction::AbstractionAnalysis,
    ) -> f64 {
        let cache_score = 1.0 - cache.cache_miss_rate;
        let branch_score = 1.0 - (branch.unpredictable_branches as f64 * 0.02).min(0.3);
        let vector_score = if vectorization.is_empty() { 0.8 } else { 0.9 };
        let abstraction_score = 1.0 - abstraction.runtime_overhead;
        
        // Weighted average
        (cache_score * 0.4 + branch_score * 0.2 + vector_score * 0.2 + abstraction_score * 0.2).clamp(0.0, 1.0)
    }
}

impl Default for MechanicalSympathyTuner {
    fn default() -> Self {
        Self::new()
    }
}

// Hardware detection functions
fn detect_architecture() -> String {
    #[cfg(target_arch = "x86_64")]
    return "x86_64".to_string();
    
    #[cfg(target_arch = "aarch64")]
    return "aarch64".to_string();
    
    #[cfg(target_arch = "x86")]
    return "x86".to_string();
    
    #[allow(unreachable_code)]
    "unknown".to_string()
}

fn detect_vector_capabilities() -> Vec<String> {
    let mut capabilities = Vec::new();
    
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse") {
            capabilities.push("SSE".to_string());
        }
        if is_x86_feature_detected!("sse2") {
            capabilities.push("SSE2".to_string());
        }
        if is_x86_feature_detected!("sse3") {
            capabilities.push("SSE3".to_string());
        }
        if is_x86_feature_detected!("sse4.1") {
            capabilities.push("SSE4.1".to_string());
        }
        if is_x86_feature_detected!("sse4.2") {
            capabilities.push("SSE4.2".to_string());
        }
        if is_x86_feature_detected!("avx") {
            capabilities.push("AVX".to_string());
        }
        if is_x86_feature_detected!("avx2") {
            capabilities.push("AVX2".to_string());
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        capabilities.push("NEON".to_string());
    }
    
    if capabilities.is_empty() {
        capabilities.push("None".to_string());
    }
    
    capabilities
}

fn create_default_throughput_map() -> HashMap<String, f64> {
    let mut throughput = HashMap::new();
    throughput.insert("add".to_string(), 2.0);
    throughput.insert("mul".to_string(), 1.0);
    throughput.insert("div".to_string(), 0.25);
    throughput.insert("fma".to_string(), 2.0);
    throughput
}