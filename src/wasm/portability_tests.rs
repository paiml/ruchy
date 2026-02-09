use super::*;
use std::collections::{HashMap, HashSet};
// Helper functions for consistent test setup
fn create_test_config() -> AnalysisConfig {
    AnalysisConfig {
        target_platforms: vec![
            "wasmtime".to_string(),
            "wasmer".to_string(),
            "browser".to_string(),
        ],
        check_api_compatibility: true,
        check_size_constraints: true,
        check_performance: true,
        check_security: true,
        strict_mode: false,
    }
}
fn create_test_component_info() -> ComponentInfo {
    let mut features = HashSet::new();
    features.insert("memory".to_string());
    features.insert("tables".to_string());
    ComponentInfo {
        name: "test_component".to_string(),
        version: "1.0.0".to_string(),
        size: 1024 * 10, // 10KB
        exports_count: 5,
        imports_count: 3,
        features,
    }
}
fn create_test_portability_score() -> PortabilityScore {
    let mut platform_scores = HashMap::new();
    platform_scores.insert("wasmtime".to_string(), 0.95);
    platform_scores.insert("wasmer".to_string(), 0.92);
    platform_scores.insert("browser".to_string(), 0.88);
    let mut feature_scores = HashMap::new();
    feature_scores.insert("memory".to_string(), 0.9);
    feature_scores.insert("threading".to_string(), 0.7);
    feature_scores.insert("simd".to_string(), 0.6);
    PortabilityScore {
        overall_score: 0.85,
        platform_scores,
        feature_scores,
        api_compatibility: 0.90,
        size_efficiency: 0.88,
        performance_portability: 0.82,
        security_compliance: 0.95,
    }
}
fn create_test_analyzer() -> PortabilityAnalyzer {
    PortabilityAnalyzer::new()
}
fn create_test_analyzer_with_config() -> PortabilityAnalyzer {
    let config = create_test_config();
    PortabilityAnalyzer::new_with_config(config)
}
// ========== AnalysisConfig Tests ==========
#[test]
fn test_analysis_config_creation() {
    let config = create_test_config();
    assert_eq!(config.target_platforms.len(), 3);
    assert!(config.check_api_compatibility);
    assert!(config.check_size_constraints);
    assert!(config.check_performance);
    assert!(config.check_security);
    assert!(!config.strict_mode);
}
#[test]
fn test_analysis_config_default() {
    let config = AnalysisConfig::default();
    assert!(!config.target_platforms.is_empty());
    assert!(config.check_api_compatibility);
    assert!(!config.strict_mode);
}
#[test]
fn test_analysis_config_clone() {
    let config1 = create_test_config();
    let config2 = config1.clone();
    assert_eq!(config1.target_platforms, config2.target_platforms);
    assert_eq!(config1.strict_mode, config2.strict_mode);
}
#[test]
fn test_analysis_config_serialization() {
    let config = create_test_config();
    let json = serde_json::to_string(&config).expect("operation should succeed in test");
    let deserialized: AnalysisConfig =
        serde_json::from_str(&json).expect("operation should succeed in test");
    assert_eq!(config.target_platforms, deserialized.target_platforms);
}
// ========== ComponentInfo Tests ==========
#[test]
fn test_component_info_creation() {
    let info = create_test_component_info();
    assert_eq!(info.name, "test_component");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.size, 10240);
    assert_eq!(info.exports_count, 5);
    assert_eq!(info.imports_count, 3);
    assert_eq!(info.features.len(), 2);
}
#[test]
fn test_component_info_without_features() {
    let info = ComponentInfo {
        name: "minimal".to_string(),
        version: "0.1.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    assert_eq!(info.exports_count, 0);
    assert_eq!(info.imports_count, 0);
    assert!(info.features.is_empty());
}
#[test]
fn test_component_info_serialization() {
    let info = create_test_component_info();
    let json = serde_json::to_string(&info).expect("operation should succeed in test");
    let deserialized: ComponentInfo =
        serde_json::from_str(&json).expect("operation should succeed in test");
    assert_eq!(info.name, deserialized.name);
    assert_eq!(info.size, deserialized.size);
}
// ========== PortabilityScore Tests ==========
#[test]
fn test_portability_score_creation() {
    let score = create_test_portability_score();
    assert_eq!(score.overall_score, 0.85);
    assert_eq!(score.platform_scores.len(), 3);
    assert_eq!(score.feature_scores.len(), 3);
    assert_eq!(score.api_compatibility, 0.90);
    assert_eq!(score.security_compliance, 0.95);
}
#[test]
fn test_portability_score_platform_lookup() {
    let score = create_test_portability_score();
    assert_eq!(
        *score
            .platform_scores
            .get("wasmtime")
            .expect("operation should succeed in test"),
        0.95
    );
    assert_eq!(
        *score
            .platform_scores
            .get("wasmer")
            .expect("operation should succeed in test"),
        0.92
    );
    assert_eq!(
        *score
            .platform_scores
            .get("browser")
            .expect("operation should succeed in test"),
        0.88
    );
}
#[test]
fn test_portability_score_feature_lookup() {
    let score = create_test_portability_score();
    assert_eq!(
        *score
            .feature_scores
            .get("memory")
            .expect("operation should succeed in test"),
        0.9
    );
    assert_eq!(
        *score
            .feature_scores
            .get("threading")
            .expect("operation should succeed in test"),
        0.7
    );
    assert_eq!(
        *score
            .feature_scores
            .get("simd")
            .expect("operation should succeed in test"),
        0.6
    );
}
#[test]
fn test_portability_score_perfect() {
    let mut score = create_test_portability_score();
    score.overall_score = 1.0;
    score.api_compatibility = 1.0;
    score.size_efficiency = 1.0;
    score.performance_portability = 1.0;
    score.security_compliance = 1.0;
    assert_eq!(score.overall_score, 1.0);
    assert_eq!(score.api_compatibility, 1.0);
    assert_eq!(score.size_efficiency, 1.0);
    assert_eq!(score.performance_portability, 1.0);
    assert_eq!(score.security_compliance, 1.0);
}
#[test]
fn test_portability_score_failing() {
    let mut score = create_test_portability_score();
    score.overall_score = 0.4;
    assert!(score.overall_score < 0.5);
}
// ========== CompatibilityIssue Tests ==========
#[test]
fn test_compatibility_issue_creation() {
    let issue = CompatibilityIssue {
        severity: IssueSeverity::Warning,
        category: IssueCategory::ApiIncompatibility,
        affected_platforms: vec!["browser".to_string()],
        description: "Missing API support".to_string(),
        fix_suggestion: Some("Use polyfill or alternative API".to_string()),
    };
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.category, IssueCategory::ApiIncompatibility);
    assert_eq!(issue.affected_platforms.len(), 1);
    assert!(issue.fix_suggestion.is_some());
}
#[test]
fn test_compatibility_issue_severity_levels() {
    let severities = [
        IssueSeverity::Error,
        IssueSeverity::Warning,
        IssueSeverity::Info,
    ];
    assert_eq!(severities.len(), 3);
    assert_ne!(severities[0], severities[1]);
}
#[test]
fn test_compatibility_issue_categories() {
    let categories = [
        IssueCategory::ApiIncompatibility,
        IssueCategory::FeatureNotSupported,
        IssueCategory::Performance,
        IssueCategory::SizeConstraint,
        IssueCategory::Security,
        IssueCategory::Configuration,
    ];
    assert_eq!(categories.len(), 6);
}
// ========== Recommendation Tests ==========
#[test]
fn test_recommendation_creation() {
    let rec = Recommendation {
        priority: RecommendationPriority::High,
        title: "Optimize memory usage".to_string(),
        description: "Reduce memory allocation".to_string(),
        impact: 0.1,
        platforms: vec!["wasmtime".to_string(), "wasmer".to_string()],
    };
    assert_eq!(rec.priority, RecommendationPriority::High);
    assert_eq!(rec.title, "Optimize memory usage");
    assert_eq!(rec.impact, 0.1);
    assert_eq!(rec.platforms.len(), 2);
}
#[test]
fn test_recommendation_priorities() {
    let priorities = [
        RecommendationPriority::Low,
        RecommendationPriority::Medium,
        RecommendationPriority::High,
    ];
    assert_eq!(priorities.len(), 3);
}
#[test]
fn test_recommendation_serialization() {
    let rec = Recommendation {
        priority: RecommendationPriority::Medium,
        title: "Test recommendation".to_string(),
        description: "Test description".to_string(),
        impact: 0.05,
        platforms: vec!["browser".to_string()],
    };
    let json = serde_json::to_string(&rec).expect("operation should succeed in test");
    let deserialized: Recommendation =
        serde_json::from_str(&json).expect("operation should succeed in test");
    assert_eq!(rec.title, deserialized.title);
    assert_eq!(rec.impact, deserialized.impact);
}
// ========== PlatformSupport Tests ==========
#[test]
fn test_platform_support_creation() {
    let support = PlatformSupport {
        platform: "wasmtime".to_string(),
        support_level: SupportLevel::Full,
        compatibility_score: 0.9,
        required_modifications: vec![],
        runtime_requirements: Some("1.0-2.0".to_string()),
    };
    assert_eq!(support.platform, "wasmtime");
    assert_eq!(support.support_level, SupportLevel::Full);
    assert!(support.required_modifications.is_empty());
    assert_eq!(support.compatibility_score, 0.9);
}
#[test]
fn test_platform_support_partial() {
    let support = PlatformSupport {
        platform: "browser".to_string(),
        support_level: SupportLevel::Partial,
        compatibility_score: 0.7,
        required_modifications: vec!["Remove filesystem access".to_string()],
        runtime_requirements: None,
    };
    assert_eq!(support.support_level, SupportLevel::Partial);
    assert_eq!(support.required_modifications.len(), 1);
    assert_eq!(support.compatibility_score, 0.7);
    assert!(support.runtime_requirements.is_none());
}
// ========== FeatureUsage Tests ==========
#[test]
fn test_feature_usage_creation() {
    let mut core = HashSet::new();
    core.insert("memory".to_string());
    core.insert("tables".to_string());
    let mut proposal = HashSet::new();
    proposal.insert("simd".to_string());
    let usage = FeatureUsage {
        core_features: core,
        proposal_features: proposal,
        platform_specific: HashMap::new(),
        compatibility: HashMap::new(),
    };
    assert_eq!(usage.core_features.len(), 2);
    assert_eq!(usage.proposal_features.len(), 1);
    assert!(usage.platform_specific.is_empty());
}
#[test]
fn test_feature_usage_with_proposals() {
    let mut proposals = HashSet::new();
    proposals.insert("threads".to_string());
    proposals.insert("simd".to_string());
    let usage = FeatureUsage {
        core_features: HashSet::new(),
        proposal_features: proposals,
        platform_specific: HashMap::new(),
        compatibility: HashMap::new(),
    };
    assert_eq!(usage.proposal_features.len(), 2);
    assert!(usage.proposal_features.contains(&"threads".to_string()));
}
// ========== SizeAnalysis Tests ==========
#[test]
fn test_size_analysis_creation() {
    let analysis = SizeAnalysis {
        total_size: 10240,
        code_size: 6000,
        data_size: 2000,
        custom_sections_size: 1240,
        section_sizes: HashMap::new(),
        platform_limits: HashMap::new(),
    };
    assert_eq!(analysis.total_size, 10240);
    assert_eq!(analysis.code_size, 6000);
    assert_eq!(analysis.data_size, 2000);
    assert_eq!(analysis.custom_sections_size, 1240);
}
#[test]
fn test_size_analysis_with_sections() {
    let mut sections = HashMap::new();
    sections.insert("code".to_string(), 6000);
    sections.insert("data".to_string(), 2000);
    sections.insert("debug".to_string(), 500);
    let analysis = SizeAnalysis {
        total_size: 10240,
        code_size: 6000,
        data_size: 2000,
        custom_sections_size: 1500,
        section_sizes: sections,
        platform_limits: HashMap::new(),
    };
    assert_eq!(analysis.section_sizes.len(), 3);
    assert_eq!(
        *analysis
            .section_sizes
            .get("debug")
            .expect("operation should succeed in test"),
        500
    );
}
// ========== PortabilityAnalyzer Tests ==========
#[test]
fn test_analyzer_creation() {
    let analyzer = create_test_analyzer();
    // Default config has 6 platforms
    assert_eq!(analyzer.config.target_platforms.len(), 6);
}
#[test]
fn test_analyzer_with_custom_config() {
    let mut config = create_test_config();
    config.strict_mode = true;
    config.check_performance = false;
    let analyzer = PortabilityAnalyzer::new_with_config(config);
    assert!(analyzer.config.strict_mode);
    assert!(!analyzer.config.check_performance);
}
#[test]
fn test_analyzer_default_config() {
    let analyzer = create_test_analyzer();
    // Test that analyzer is created successfully
    assert!(analyzer.config.check_api_compatibility);
    assert!(analyzer.config.check_size_constraints);
    assert!(analyzer.config.check_performance);
    assert!(analyzer.config.check_security);
}
#[test]
fn test_analyzer_custom_config() {
    let analyzer = create_test_analyzer_with_config();
    // Verify custom config is applied
    assert_eq!(analyzer.config.target_platforms.len(), 3);
    assert!(analyzer
        .config
        .target_platforms
        .contains(&"wasmtime".to_string()));
    assert!(analyzer
        .config
        .target_platforms
        .contains(&"wasmer".to_string()));
    assert!(analyzer
        .config
        .target_platforms
        .contains(&"browser".to_string()));
}
#[test]
fn test_analyzer_strict_mode() {
    let mut config = create_test_config();
    config.strict_mode = true;
    let analyzer = PortabilityAnalyzer::new_with_config(config);
    assert!(analyzer.config.strict_mode);
    // Test non-strict mode
    let mut config2 = create_test_config();
    config2.strict_mode = false;
    let analyzer2 = PortabilityAnalyzer::new_with_config(config2);
    assert!(!analyzer2.config.strict_mode);
}
// ========== PortabilityReport Tests ==========
#[test]
fn test_portability_report_creation() {
    let report = PortabilityReport {
        component_info: create_test_component_info(),
        score: create_test_portability_score(),
        issues: vec![],
        recommendations: vec![],
        platform_support: HashMap::new(),
        feature_usage: FeatureUsage {
            core_features: HashSet::new(),
            proposal_features: HashSet::new(),
            platform_specific: HashMap::new(),
            compatibility: HashMap::new(),
        },
        size_analysis: SizeAnalysis {
            total_size: 0,
            code_size: 0,
            data_size: 0,
            custom_sections_size: 0,
            section_sizes: HashMap::new(),
            platform_limits: HashMap::new(),
        },
    };
    assert_eq!(report.component_info.name, "test_component");
    assert_eq!(report.score.overall_score, 0.85);
    assert!(report.issues.is_empty());
    assert!(report.recommendations.is_empty());
}
#[test]
fn test_portability_report_with_issues() {
    let issue = CompatibilityIssue {
        severity: IssueSeverity::Warning,
        category: IssueCategory::FeatureNotSupported,
        affected_platforms: vec!["browser".to_string()],
        description: "Threading not supported".to_string(),
        fix_suggestion: Some("Use web workers".to_string()),
    };
    let report = PortabilityReport {
        component_info: create_test_component_info(),
        score: create_test_portability_score(),
        issues: vec![issue],
        recommendations: vec![],
        platform_support: HashMap::new(),
        feature_usage: FeatureUsage {
            core_features: HashSet::new(),
            proposal_features: HashSet::new(),
            platform_specific: HashMap::new(),
            compatibility: HashMap::new(),
        },
        size_analysis: SizeAnalysis {
            total_size: 0,
            code_size: 0,
            data_size: 0,
            custom_sections_size: 0,
            section_sizes: HashMap::new(),
            platform_limits: HashMap::new(),
        },
    };
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].severity, IssueSeverity::Warning);
}
#[test]
fn test_portability_report_serialization() {
    let report = PortabilityReport {
        component_info: create_test_component_info(),
        score: create_test_portability_score(),
        issues: vec![],
        recommendations: vec![],
        platform_support: HashMap::new(),
        feature_usage: FeatureUsage {
            core_features: HashSet::new(),
            proposal_features: HashSet::new(),
            platform_specific: HashMap::new(),
            compatibility: HashMap::new(),
        },
        size_analysis: SizeAnalysis {
            total_size: 0,
            code_size: 0,
            data_size: 0,
            custom_sections_size: 0,
            section_sizes: HashMap::new(),
            platform_limits: HashMap::new(),
        },
    };
    let json = serde_json::to_string(&report).expect("operation should succeed in test");
    let deserialized: PortabilityReport =
        serde_json::from_str(&json).expect("operation should succeed in test");
    assert_eq!(report.component_info.name, deserialized.component_info.name);
    assert_eq!(report.score.overall_score, deserialized.score.overall_score);
}
// ========== Integration Tests ==========
#[test]
fn test_full_portability_analysis_workflow() {
    // Test creating complete portability report
    let report = PortabilityReport {
        component_info: create_test_component_info(),
        score: create_test_portability_score(),
        issues: vec![CompatibilityIssue {
            severity: IssueSeverity::Info,
            category: IssueCategory::Performance,
            affected_platforms: vec!["browser".to_string()],
            description: "Performance may vary".to_string(),
            fix_suggestion: None,
        }],
        recommendations: vec![Recommendation {
            priority: RecommendationPriority::Low,
            title: "Consider optimization".to_string(),
            description: "Optimize for browser platform".to_string(),
            impact: 0.05,
            platforms: vec!["browser".to_string()],
        }],
        platform_support: HashMap::new(),
        feature_usage: FeatureUsage {
            core_features: HashSet::new(),
            proposal_features: HashSet::new(),
            platform_specific: HashMap::new(),
            compatibility: HashMap::new(),
        },
        size_analysis: SizeAnalysis {
            total_size: 10240,
            code_size: 6000,
            data_size: 2000,
            custom_sections_size: 1240,
            section_sizes: HashMap::new(),
            platform_limits: HashMap::new(),
        },
    };
    // Check report completeness
    assert!(!report.component_info.name.is_empty());
    assert!(report.score.overall_score >= 0.0);
    assert!(report.score.overall_score <= 1.0);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.recommendations.len(), 1);
}
#[test]
fn test_enum_variants() {
    // Test all IssueSeverity variants
    let severities = vec![
        IssueSeverity::Error,
        IssueSeverity::Warning,
        IssueSeverity::Info,
    ];
    for s in &severities {
        assert!(matches!(
            s,
            IssueSeverity::Error | IssueSeverity::Warning | IssueSeverity::Info
        ));
    }
    // Test all IssueCategory variants
    let categories = [
        IssueCategory::ApiIncompatibility,
        IssueCategory::FeatureNotSupported,
        IssueCategory::SizeConstraint,
        IssueCategory::Performance,
        IssueCategory::Security,
        IssueCategory::Configuration,
    ];
    assert_eq!(categories.len(), 6);
    // Test RecommendationPriority variants
    let priorities = [
        RecommendationPriority::Low,
        RecommendationPriority::Medium,
        RecommendationPriority::High,
    ];
    assert_eq!(priorities.len(), 3);
    // Test SupportLevel variants
    let levels = [
        SupportLevel::Full,
        SupportLevel::Partial,
        SupportLevel::None,
    ];
    assert_eq!(levels.len(), 3);
}
#[test]
fn test_complex_portability_score() {
    let mut platform_scores = HashMap::new();
    platform_scores.insert("wasmtime".to_string(), 1.0);
    platform_scores.insert("wasmer".to_string(), 0.95);
    platform_scores.insert("browser".to_string(), 0.75);
    platform_scores.insert("node".to_string(), 0.85);
    let mut feature_scores = HashMap::new();
    feature_scores.insert("memory".to_string(), 1.0);
    feature_scores.insert("tables".to_string(), 1.0);
    feature_scores.insert("threading".to_string(), 0.5);
    feature_scores.insert("simd".to_string(), 0.8);
    feature_scores.insert("bulk-memory".to_string(), 0.9);
    let score = PortabilityScore {
        overall_score: 0.875,
        platform_scores,
        feature_scores,
        api_compatibility: 0.92,
        size_efficiency: 0.95,
        performance_portability: 0.78,
        security_compliance: 1.0,
    };
    assert_eq!(score.platform_scores.len(), 4);
    assert_eq!(score.feature_scores.len(), 5);
    assert_eq!(score.security_compliance, 1.0);
}
#[test]
fn test_edge_case_scores() {
    // Test minimum scores
    let min_score = PortabilityScore {
        overall_score: 0.0,
        platform_scores: HashMap::new(),
        feature_scores: HashMap::new(),
        api_compatibility: 0.0,
        size_efficiency: 0.0,
        performance_portability: 0.0,
        security_compliance: 0.0,
    };
    assert_eq!(min_score.overall_score, 0.0);
    assert!(min_score.platform_scores.is_empty());
    // Test maximum scores
    let mut perfect_platforms = HashMap::new();
    perfect_platforms.insert("all".to_string(), 1.0);
    let max_score = PortabilityScore {
        overall_score: 1.0,
        platform_scores: perfect_platforms,
        feature_scores: HashMap::new(),
        api_compatibility: 1.0,
        size_efficiency: 1.0,
        performance_portability: 1.0,
        security_compliance: 1.0,
    };
    assert_eq!(max_score.overall_score, 1.0);
    assert_eq!(max_score.api_compatibility, 1.0);
}
#[test]
fn test_component_info_edge_cases() {
    // Test component with maximum values
    let mut max_features = HashSet::new();
    for i in 0..20 {
        max_features.insert(format!("feature_{i}"));
    }
    let large_component = ComponentInfo {
        name: "large_component".to_string(),
        version: "99.99.99".to_string(),
        size: usize::MAX,
        exports_count: 1000,
        imports_count: 500,
        features: max_features,
    };
    assert_eq!(large_component.features.len(), 20);
    assert_eq!(large_component.exports_count, 1000);
    assert_eq!(large_component.size, usize::MAX);
    // Test component with minimum values
    let minimal_component = ComponentInfo {
        name: String::new(),
        version: String::new(),
        size: 0,
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    assert!(minimal_component.name.is_empty());
    assert_eq!(minimal_component.size, 0);
}
#[test]
fn test_platform_support_variations() {
    let support_variations = [
        PlatformSupport {
            platform: "wasmtime".to_string(),
            support_level: SupportLevel::Full,
            compatibility_score: 1.0,
            required_modifications: vec![],
            runtime_requirements: Some(">=1.0.0".to_string()),
        },
        PlatformSupport {
            platform: "wasmer".to_string(),
            support_level: SupportLevel::Partial,
            compatibility_score: 0.8,
            required_modifications: vec!["Remove SIMD".to_string()],
            runtime_requirements: Some(">=2.0.0".to_string()),
        },
        PlatformSupport {
            platform: "browser".to_string(),
            support_level: SupportLevel::None,
            compatibility_score: 0.0,
            required_modifications: vec!["Complete rewrite".to_string()],
            runtime_requirements: None,
        },
    ];
    assert_eq!(support_variations.len(), 3);
    assert_eq!(support_variations[0].support_level, SupportLevel::Full);
    assert_eq!(support_variations[1].support_level, SupportLevel::Partial);
    assert_eq!(support_variations[2].support_level, SupportLevel::None);
}
#[test]
fn test_feature_usage_complex() {
    let mut core_features = HashSet::new();
    core_features.insert("memory".to_string());
    core_features.insert("tables".to_string());
    core_features.insert("globals".to_string());
    let mut proposal_features = HashSet::new();
    proposal_features.insert("threads".to_string());
    proposal_features.insert("simd".to_string());
    proposal_features.insert("reference-types".to_string());
    let mut platform_specific = HashMap::new();
    let mut browser_features = HashSet::new();
    browser_features.insert("webgl".to_string());
    platform_specific.insert("browser".to_string(), browser_features);
    let usage = FeatureUsage {
        core_features: core_features.clone(),
        proposal_features: proposal_features.clone(),
        platform_specific,
        compatibility: HashMap::new(),
    };
    assert_eq!(usage.core_features.len(), 3);
    assert_eq!(usage.proposal_features.len(), 3);
    assert_eq!(usage.platform_specific.len(), 1);
    assert!(usage.core_features.contains("memory"));
    assert!(usage.proposal_features.contains("threads"));
}
#[test]
fn test_size_analysis_comprehensive() {
    let mut section_sizes = HashMap::new();
    section_sizes.insert("type".to_string(), 1024);
    section_sizes.insert("import".to_string(), 512);
    section_sizes.insert("function".to_string(), 256);
    section_sizes.insert("table".to_string(), 128);
    section_sizes.insert("memory".to_string(), 64);
    section_sizes.insert("global".to_string(), 32);
    section_sizes.insert("export".to_string(), 256);
    section_sizes.insert("start".to_string(), 8);
    section_sizes.insert("element".to_string(), 512);
    section_sizes.insert("code".to_string(), 8192);
    section_sizes.insert("data".to_string(), 4096);
    let mut platform_limits = HashMap::new();
    platform_limits.insert("browser".to_string(), 1024 * 1024 * 4); // 4MB
    platform_limits.insert("wasmtime".to_string(), 1024 * 1024 * 1024); // 1GB
    let analysis = SizeAnalysis {
        total_size: 15080,
        code_size: 8192,
        data_size: 4096,
        custom_sections_size: 0,
        section_sizes,
        platform_limits,
    };
    assert_eq!(analysis.section_sizes.len(), 11);
    assert_eq!(analysis.platform_limits.len(), 2);
    assert_eq!(
        *analysis
            .section_sizes
            .get("code")
            .expect("operation should succeed in test"),
        8192
    );
    assert_eq!(
        *analysis
            .platform_limits
            .get("browser")
            .expect("operation should succeed in test"),
        4_194_304
    );
}
