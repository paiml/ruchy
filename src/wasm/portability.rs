//! Portability scoring for WebAssembly components (RUCHY-0819)
//!
//! Analyzes and scores the portability of Ruchy-generated WASM components
//! across different platforms and runtimes.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use super::component::WasmComponent;

/// Portability analyzer for WASM components
pub struct PortabilityAnalyzer {
    /// Analysis configuration
    config: AnalysisConfig,
    
    /// Feature compatibility matrix
    compatibility_matrix: CompatibilityMatrix,
    
    /// Platform requirements
    platform_requirements: HashMap<String, PlatformRequirements>,
}

/// Portability score for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityScore {
    /// Overall portability score (0.0 - 1.0)
    pub overall_score: f64,
    
    /// Platform-specific scores
    pub platform_scores: HashMap<String, f64>,
    
    /// Feature compatibility scores
    pub feature_scores: HashMap<String, f64>,
    
    /// API compatibility score
    pub api_compatibility: f64,
    
    /// Size efficiency score
    pub size_efficiency: f64,
    
    /// Performance portability score
    pub performance_portability: f64,
    
    /// Security compliance score
    pub security_compliance: f64,
}

/// Detailed portability report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityReport {
    /// Component information
    pub component_info: ComponentInfo,
    
    /// Portability score
    pub score: PortabilityScore,
    
    /// Compatibility issues
    pub issues: Vec<CompatibilityIssue>,
    
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    
    /// Platform support matrix
    pub platform_support: HashMap<String, PlatformSupport>,
    
    /// Feature usage analysis
    pub feature_usage: FeatureUsage,
    
    /// Size analysis
    pub size_analysis: SizeAnalysis,
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Target platforms to analyze
    pub target_platforms: Vec<String>,
    
    /// Check API compatibility
    pub check_api_compatibility: bool,
    
    /// Check size constraints
    pub check_size_constraints: bool,
    
    /// Check performance characteristics
    pub check_performance: bool,
    
    /// Check security requirements
    pub check_security: bool,
    
    /// Strict mode (fail on any incompatibility)
    pub strict_mode: bool,
}

/// Component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Component name
    pub name: String,
    
    /// Component version
    pub version: String,
    
    /// Component size in bytes
    pub size: usize,
    
    /// Number of exports
    pub exports_count: usize,
    
    /// Number of imports
    pub imports_count: usize,
    
    /// Used features
    pub features: HashSet<String>,
}

/// Compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    
    /// Issue category
    pub category: IssueCategory,
    
    /// Affected platforms
    pub affected_platforms: Vec<String>,
    
    /// Issue description
    pub description: String,
    
    /// Suggested fix
    pub fix_suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error (blocks deployment)
    Error,
    /// Critical (security or major functionality issue)
    Critical,
}

/// Issue categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    /// API incompatibility
    ApiIncompatibility,
    /// Feature not supported
    FeatureNotSupported,
    /// Size constraint violation
    SizeConstraint,
    /// Performance issue
    Performance,
    /// Security concern
    Security,
    /// Configuration issue
    Configuration,
}

/// Recommendation for improving portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,
    
    /// Recommendation title
    pub title: String,
    
    /// Detailed description
    pub description: String,
    
    /// Expected impact on portability score
    pub impact: f64,
    
    /// Affected platforms
    pub platforms: Vec<String>,
}

impl fmt::Display for Recommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

/// Recommendation priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical (must fix)
    Critical,
}

/// Platform support information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSupport {
    /// Platform name
    pub platform: String,
    
    /// Support level
    pub support_level: SupportLevel,
    
    /// Compatibility score (0.0 - 1.0)
    pub compatibility_score: f64,
    
    /// Required modifications
    pub required_modifications: Vec<String>,
    
    /// Runtime version requirements
    pub runtime_requirements: Option<String>,
}

/// Support levels for platforms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    /// Full support
    Full,
    /// Partial support (some features missing)
    Partial,
    /// Limited support (major limitations)
    Limited,
    /// No support
    None,
}

/// Feature usage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsage {
    /// Core WASM features used
    pub core_features: HashSet<String>,
    
    /// Proposal features used
    pub proposal_features: HashSet<String>,
    
    /// Platform-specific features
    pub platform_specific: HashMap<String, HashSet<String>>,
    
    /// Feature compatibility matrix
    pub compatibility: HashMap<String, Vec<String>>,
}

/// Size analysis for portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeAnalysis {
    /// Total component size
    pub total_size: usize,
    
    /// Code size
    pub code_size: usize,
    
    /// Data size
    pub data_size: usize,
    
    /// Custom sections size
    pub custom_sections_size: usize,
    
    /// Size by section
    pub section_sizes: HashMap<String, usize>,
    
    /// Platform size limits
    pub platform_limits: HashMap<String, usize>,
}

/// Platform requirements
#[derive(Debug, Clone)]
struct PlatformRequirements {
    /// Required features
    required_features: HashSet<String>,
    
    /// Optional features
    optional_features: HashSet<String>,
    
    /// Incompatible features
    incompatible_features: HashSet<String>,
    
    /// Size limit in bytes
    size_limit: Option<usize>,
    
    /// API requirements
    _api_requirements: HashSet<String>,
}

/// Feature compatibility matrix
struct CompatibilityMatrix {
    /// Feature support by platform
    support: HashMap<String, HashMap<String, bool>>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            target_platforms: vec![
                "cloudflare-workers".to_string(),
                "fastly-compute".to_string(),
                "aws-lambda".to_string(),
                "browser".to_string(),
                "nodejs".to_string(),
                "wasmtime".to_string(),
            ],
            check_api_compatibility: true,
            check_size_constraints: true,
            check_performance: true,
            check_security: true,
            strict_mode: false,
        }
    }
}

impl PortabilityAnalyzer {
    /// Create a new portability analyzer with default config
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
            compatibility_matrix: Self::build_compatibility_matrix(),
            platform_requirements: Self::build_platform_requirements(),
        }
    }
    
    /// Create a new portability analyzer with specific config
    pub fn new_with_config(config: AnalysisConfig) -> Self {
        Self {
            config,
            compatibility_matrix: Self::build_compatibility_matrix(),
            platform_requirements: Self::build_platform_requirements(),
        }
    }
    
    /// Analyze a WASM component's portability
    pub fn analyze(&self, component: &WasmComponent) -> Result<PortabilityReport> {
        // Extract component information
        let component_info = self.extract_component_info(component)?;
        
        // Calculate portability scores
        let score = self.calculate_scores(&component_info)?;
        
        // Find compatibility issues
        let issues = self.find_issues(&component_info)?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&component_info, &issues)?;
        
        // Analyze platform support
        let platform_support = self.analyze_platform_support(&component_info)?;
        
        // Analyze feature usage
        let feature_usage = self.analyze_feature_usage(&component_info)?;
        
        // Analyze size
        let size_analysis = self.analyze_size(component)?;
        
        Ok(PortabilityReport {
            component_info,
            score,
            issues,
            recommendations,
            platform_support,
            feature_usage,
            size_analysis,
        })
    }
    
    fn extract_component_info(&self, component: &WasmComponent) -> Result<ComponentInfo> {
        let mut features = HashSet::new();
        
        // Analyze bytecode to detect used features
        // In a real implementation, this would parse the WASM module
        if component.bytecode.len() > 1024 * 100 {
            features.insert("large-module".to_string());
        }
        
        Ok(ComponentInfo {
            name: component.name.clone(),
            version: component.version.clone(),
            size: component.bytecode.len(),
            exports_count: component.exports.len(),
            imports_count: component.imports.len(),
            features,
        })
    }
    
    fn calculate_scores(&self, info: &ComponentInfo) -> Result<PortabilityScore> {
        let mut platform_scores = HashMap::new();
        
        // Calculate scores for each platform
        for platform in &self.config.target_platforms {
            let score = self.calculate_platform_score(info, platform)?;
            platform_scores.insert(platform.clone(), score);
        }
        
        // Calculate feature scores
        let feature_scores = self.calculate_feature_scores(info)?;
        
        // Calculate other scores
        let api_compatibility = self.calculate_api_compatibility(info)?;
        let size_efficiency = self.calculate_size_efficiency(info)?;
        let performance_portability = self.calculate_performance_portability(info)?;
        let security_compliance = self.calculate_security_compliance(info)?;
        
        // Calculate overall score
        let overall_score = Self::calculate_overall_score(
            &platform_scores,
            &feature_scores,
            api_compatibility,
            size_efficiency,
            performance_portability,
            security_compliance,
        );
        
        Ok(PortabilityScore {
            overall_score,
            platform_scores,
            feature_scores,
            api_compatibility,
            size_efficiency,
            performance_portability,
            security_compliance,
        })
    }
    
    fn calculate_platform_score(&self, info: &ComponentInfo, platform: &str) -> Result<f64> {
        let requirements = self.platform_requirements.get(platform);
        
        if let Some(reqs) = requirements {
            let mut score = 1.0;
            
            // Check size constraints
            if let Some(limit) = reqs.size_limit {
                if info.size > limit {
                    score *= 0.5; // Penalty for exceeding size limit
                }
            }
            
            // Check feature compatibility
            for feature in &info.features {
                if reqs.incompatible_features.contains(feature) {
                    score *= 0.0; // Incompatible feature
                } else if !reqs.required_features.contains(feature) && !reqs.optional_features.contains(feature) {
                    score *= 0.8; // Unknown feature
                }
            }
            
            Ok(score)
        } else {
            Ok(0.5) // Unknown platform
        }
    }
    
    fn calculate_feature_scores(&self, info: &ComponentInfo) -> Result<HashMap<String, f64>> {
        let mut scores = HashMap::new();
        
        // Score each feature based on platform support
        for feature in &info.features {
            let mut support_count = 0;
            let total_platforms = self.config.target_platforms.len();
            
            for platform in &self.config.target_platforms {
                if let Some(platform_features) = self.compatibility_matrix.support.get(platform) {
                    if platform_features.get(feature).copied().unwrap_or(false) {
                        support_count += 1;
                    }
                }
            }
            
            let score = support_count as f64 / total_platforms as f64;
            scores.insert(feature.clone(), score);
        }
        
        Ok(scores)
    }
    
    fn calculate_api_compatibility(&self, _info: &ComponentInfo) -> Result<f64> {
        // Check if APIs used are compatible across platforms
        // Simplified implementation
        Ok(0.9)
    }
    
    fn calculate_size_efficiency(&self, info: &ComponentInfo) -> Result<f64> {
        // Score based on component size
        let size_kb = info.size as f64 / 1024.0;
        
        if size_kb < 50.0 {
            Ok(1.0)
        } else if size_kb < 100.0 {
            Ok(0.9)
        } else if size_kb < 500.0 {
            Ok(0.7)
        } else if size_kb < 1000.0 {
            Ok(0.5)
        } else {
            Ok(0.3)
        }
    }
    
    fn calculate_performance_portability(&self, _info: &ComponentInfo) -> Result<f64> {
        // Analyze performance characteristics
        // Simplified implementation
        Ok(0.85)
    }
    
    fn calculate_security_compliance(&self, _info: &ComponentInfo) -> Result<f64> {
        // Check security requirements
        // Simplified implementation
        Ok(0.95)
    }
    
    fn calculate_overall_score(
        platform_scores: &HashMap<String, f64>,
        feature_scores: &HashMap<String, f64>,
        api_compatibility: f64,
        size_efficiency: f64,
        performance_portability: f64,
        security_compliance: f64,
    ) -> f64 {
        let platform_avg = if !platform_scores.is_empty() {
            platform_scores.values().sum::<f64>() / platform_scores.len() as f64
        } else {
            0.0
        };
        
        let feature_avg = if !feature_scores.is_empty() {
            feature_scores.values().sum::<f64>() / feature_scores.len() as f64
        } else {
            1.0
        };
        
        // Weighted average
        platform_avg * 0.3 +
         feature_avg * 0.2 +
         api_compatibility * 0.2 +
         size_efficiency * 0.1 +
         performance_portability * 0.1 +
         security_compliance * 0.1
    }
    
    fn find_issues(&self, info: &ComponentInfo) -> Result<Vec<CompatibilityIssue>> {
        let mut issues = Vec::new();
        
        // Check size constraints
        for platform in &self.config.target_platforms {
            if let Some(reqs) = self.platform_requirements.get(platform) {
                if let Some(limit) = reqs.size_limit {
                    if info.size > limit {
                        issues.push(CompatibilityIssue {
                            severity: IssueSeverity::Warning,
                            category: IssueCategory::SizeConstraint,
                            affected_platforms: vec![platform.clone()],
                            description: format!(
                                "Component size ({} KB) exceeds {} platform limit ({} KB)",
                                info.size / 1024,
                                platform,
                                limit / 1024
                            ),
                            fix_suggestion: Some("Consider optimizing component size or splitting functionality".to_string()),
                        });
                    }
                }
            }
        }
        
        Ok(issues)
    }
    
    fn generate_recommendations(&self, info: &ComponentInfo, issues: &[CompatibilityIssue]) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        // Size optimization recommendation
        if info.size > 100 * 1024 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                title: "Optimize component size".to_string(),
                description: "Component size can be reduced through optimization techniques".to_string(),
                impact: 0.2,
                platforms: self.config.target_platforms.clone(),
            });
        }
        
        // Feature compatibility recommendations
        for issue in issues {
            if issue.category == IssueCategory::FeatureNotSupported {
                recommendations.push(Recommendation {
                    priority: RecommendationPriority::Critical,
                    title: "Remove incompatible features".to_string(),
                    description: issue.description.clone(),
                    impact: 0.3,
                    platforms: issue.affected_platforms.clone(),
                });
            }
        }
        
        Ok(recommendations)
    }
    
    fn analyze_platform_support(&self, info: &ComponentInfo) -> Result<HashMap<String, PlatformSupport>> {
        let mut support = HashMap::new();
        
        for platform in &self.config.target_platforms {
            let score = self.calculate_platform_score(info, platform)?;
            
            let support_level = if score >= 0.9 {
                SupportLevel::Full
            } else if score >= 0.7 {
                SupportLevel::Partial
            } else if score >= 0.3 {
                SupportLevel::Limited
            } else {
                SupportLevel::None
            };
            
            support.insert(platform.clone(), PlatformSupport {
                platform: platform.clone(),
                support_level,
                compatibility_score: score,
                required_modifications: Vec::new(),
                runtime_requirements: None,
            });
        }
        
        Ok(support)
    }
    
    fn analyze_feature_usage(&self, info: &ComponentInfo) -> Result<FeatureUsage> {
        Ok(FeatureUsage {
            core_features: info.features.clone(),
            proposal_features: HashSet::new(),
            platform_specific: HashMap::new(),
            compatibility: HashMap::new(),
        })
    }
    
    fn analyze_size(&self, component: &WasmComponent) -> Result<SizeAnalysis> {
        let mut section_sizes = HashMap::new();
        
        // Add custom sections
        for (name, data) in &component.custom_sections {
            section_sizes.insert(name.clone(), data.len());
        }
        
        let custom_sections_size: usize = component.custom_sections.values().map(|v| v.len()).sum();
        
        Ok(SizeAnalysis {
            total_size: component.bytecode.len(),
            code_size: component.bytecode.len() - custom_sections_size,
            data_size: 0,
            custom_sections_size,
            section_sizes,
            platform_limits: self.get_platform_limits(),
        })
    }
    
    fn get_platform_limits(&self) -> HashMap<String, usize> {
        let mut limits = HashMap::new();
        limits.insert("cloudflare-workers".to_string(), 10 * 1024 * 1024); // 10MB
        limits.insert("fastly-compute".to_string(), 50 * 1024 * 1024);    // 50MB
        limits.insert("aws-lambda".to_string(), 250 * 1024 * 1024);       // 250MB
        limits.insert("browser".to_string(), 100 * 1024 * 1024);          // 100MB
        limits
    }
    
    fn build_compatibility_matrix() -> CompatibilityMatrix {
        let mut support = HashMap::new();
        
        // Cloudflare Workers feature support
        let mut cloudflare = HashMap::new();
        cloudflare.insert("simd".to_string(), false);
        cloudflare.insert("threads".to_string(), false);
        cloudflare.insert("bulk-memory".to_string(), true);
        cloudflare.insert("reference-types".to_string(), true);
        support.insert("cloudflare-workers".to_string(), cloudflare);
        
        // Browser feature support
        let mut browser = HashMap::new();
        browser.insert("simd".to_string(), true);
        browser.insert("threads".to_string(), true);
        browser.insert("bulk-memory".to_string(), true);
        browser.insert("reference-types".to_string(), true);
        support.insert("browser".to_string(), browser);
        
        CompatibilityMatrix { support }
    }
    
    fn build_platform_requirements() -> HashMap<String, PlatformRequirements> {
        let mut requirements = HashMap::new();
        
        // Cloudflare Workers requirements
        requirements.insert("cloudflare-workers".to_string(), PlatformRequirements {
            required_features: HashSet::new(),
            optional_features: HashSet::from(["bulk-memory".to_string(), "reference-types".to_string()]),
            incompatible_features: HashSet::from(["threads".to_string()]),
            size_limit: Some(10 * 1024 * 1024),
            _api_requirements: HashSet::new(),
        });
        
        // Browser requirements
        requirements.insert("browser".to_string(), PlatformRequirements {
            required_features: HashSet::new(),
            optional_features: HashSet::from(["simd".to_string(), "threads".to_string()]),
            incompatible_features: HashSet::new(),
            size_limit: Some(100 * 1024 * 1024),
            _api_requirements: HashSet::new(),
        });
        
        requirements
    }
}