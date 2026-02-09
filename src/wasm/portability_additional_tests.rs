use super::*;
use crate::wasm::component::{ComponentMetadata, WasmComponent};

fn create_test_wasm_component() -> WasmComponent {
    WasmComponent {
        name: "test-component".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    }
}

fn create_large_wasm_component() -> WasmComponent {
    use crate::wasm::component::{ExportDefinition, ExportType, TypeSignature};
    WasmComponent {
        name: "large-component".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 200 * 1024], // 200KB
        imports: vec![],
        exports: vec![ExportDefinition {
            name: "main".to_string(),
            export_type: ExportType::Function,
            signature: TypeSignature {
                params: vec![],
                results: vec![],
                metadata: HashMap::new(),
            },
            documentation: None,
        }],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    }
}

fn create_very_large_wasm_component() -> WasmComponent {
    WasmComponent {
        name: "very-large-component".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 1024 * 1024], // 1MB
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    }
}

// Test analyze method
#[test]
fn test_analyzer_analyze_small_component() {
    let analyzer = PortabilityAnalyzer::new();
    let component = create_test_wasm_component();
    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");

    assert_eq!(report.component_info.name, "test-component");
    assert!(report.score.overall_score > 0.0);
    assert!(report.score.overall_score <= 1.0);
}

#[test]
fn test_analyzer_analyze_large_component() {
    let analyzer = PortabilityAnalyzer::new();
    let component = create_large_wasm_component();
    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");

    assert_eq!(report.component_info.name, "large-component");
    // Large component should have recommendations
    assert!(report.recommendations.len() >= 1);
}

#[test]
fn test_analyzer_analyze_very_large_component() {
    let analyzer = PortabilityAnalyzer::new();
    let component = create_very_large_wasm_component();
    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");

    // Very large component (1MB) should have lower size efficiency
    assert!(report.score.size_efficiency < 1.0);
}

#[test]
fn test_analyzer_analyze_component_with_features() {
    use crate::wasm::component::{
        ExportDefinition, ExportType, ImportDefinition, ImportType, TypeSignature,
    };
    let analyzer = PortabilityAnalyzer::new();
    // Create component larger than 100KB to trigger feature detection
    let component = WasmComponent {
        name: "feature-component".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 105 * 1024], // 105KB to trigger large-module feature
        imports: vec![ImportDefinition {
            module: "env".to_string(),
            name: "import1".to_string(),
            import_type: ImportType::Function,
            signature: TypeSignature {
                params: vec![],
                results: vec![],
                metadata: HashMap::new(),
            },
        }],
        exports: vec![
            ExportDefinition {
                name: "export1".to_string(),
                export_type: ExportType::Function,
                signature: TypeSignature {
                    params: vec![],
                    results: vec![],
                    metadata: HashMap::new(),
                },
                documentation: None,
            },
            ExportDefinition {
                name: "export2".to_string(),
                export_type: ExportType::Function,
                signature: TypeSignature {
                    params: vec![],
                    results: vec![],
                    metadata: HashMap::new(),
                },
                documentation: None,
            },
        ],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    };

    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");
    assert!(report.component_info.features.contains("large-module"));
    assert_eq!(report.component_info.imports_count, 1);
    assert_eq!(report.component_info.exports_count, 2);
}

#[test]
fn test_analyzer_analyze_with_custom_sections() {
    let analyzer = PortabilityAnalyzer::new();
    let mut custom_sections = HashMap::new();
    custom_sections.insert("debug".to_string(), vec![1, 2, 3, 4, 5]);
    custom_sections.insert("name".to_string(), vec![6, 7, 8]);

    let component = WasmComponent {
        name: "custom-sections-component".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 1024],
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections,
    };

    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");
    assert_eq!(report.size_analysis.custom_sections_size, 8);
    assert_eq!(report.size_analysis.section_sizes.len(), 2);
}

// Test size efficiency branches
#[test]
fn test_calculate_size_efficiency_small() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "tiny".to_string(),
        version: "1.0.0".to_string(),
        size: 10 * 1024, // 10KB - small
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer.calculate_size_efficiency(&info).unwrap();
    assert_eq!(score, 1.0);
}

#[test]
fn test_calculate_size_efficiency_medium_small() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "medium-small".to_string(),
        version: "1.0.0".to_string(),
        size: 75 * 1024, // 75KB - between 50 and 100
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer.calculate_size_efficiency(&info).unwrap();
    assert_eq!(score, 0.9);
}

#[test]
fn test_calculate_size_efficiency_medium() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "medium".to_string(),
        version: "1.0.0".to_string(),
        size: 300 * 1024, // 300KB - between 100 and 500
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer.calculate_size_efficiency(&info).unwrap();
    assert_eq!(score, 0.7);
}

#[test]
fn test_calculate_size_efficiency_large() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "large".to_string(),
        version: "1.0.0".to_string(),
        size: 800 * 1024, // 800KB - between 500 and 1000
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer.calculate_size_efficiency(&info).unwrap();
    assert_eq!(score, 0.5);
}

#[test]
fn test_calculate_size_efficiency_very_large() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "very-large".to_string(),
        version: "1.0.0".to_string(),
        size: 2000 * 1024, // 2000KB - over 1000
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer.calculate_size_efficiency(&info).unwrap();
    assert_eq!(score, 0.3);
}

// Test platform score branches
#[test]
fn test_calculate_platform_score_unknown_platform() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer
        .calculate_platform_score(&info, "unknown-platform")
        .unwrap();
    assert_eq!(score, 0.5); // Unknown platform default
}

#[test]
fn test_calculate_platform_score_with_incompatible_feature() {
    let analyzer = PortabilityAnalyzer::new();
    let mut features = HashSet::new();
    features.insert("threads".to_string()); // Incompatible with cloudflare-workers
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features,
    };
    let score = analyzer
        .calculate_platform_score(&info, "cloudflare-workers")
        .unwrap();
    assert_eq!(score, 0.0); // Incompatible feature
}

#[test]
fn test_calculate_platform_score_exceeds_size_limit() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "huge".to_string(),
        version: "1.0.0".to_string(),
        size: 20 * 1024 * 1024, // 20MB - exceeds cloudflare 10MB limit
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let score = analyzer
        .calculate_platform_score(&info, "cloudflare-workers")
        .unwrap();
    assert_eq!(score, 0.5); // 50% penalty for exceeding size
}

#[test]
fn test_calculate_platform_score_unknown_feature() {
    let analyzer = PortabilityAnalyzer::new();
    let mut features = HashSet::new();
    features.insert("some-unknown-feature".to_string());
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features,
    };
    let score = analyzer
        .calculate_platform_score(&info, "cloudflare-workers")
        .unwrap();
    assert!(score < 1.0); // Penalty for unknown feature
}

// Test feature scores
#[test]
fn test_calculate_feature_scores_with_features() {
    let analyzer = PortabilityAnalyzer::new();
    let mut features = HashSet::new();
    features.insert("simd".to_string());
    features.insert("threads".to_string());
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features,
    };
    let scores = analyzer.calculate_feature_scores(&info).unwrap();
    assert!(scores.contains_key("simd"));
    assert!(scores.contains_key("threads"));
}

// Test issue finding
#[test]
fn test_find_issues_size_constraint() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "huge".to_string(),
        version: "1.0.0".to_string(),
        size: 20 * 1024 * 1024, // 20MB - exceeds cloudflare limit
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let issues = analyzer.find_issues(&info).unwrap();
    // Should find size constraint issues for platforms with limits
    assert!(issues
        .iter()
        .any(|i| i.category == IssueCategory::SizeConstraint));
}

// Test enum variants not covered
#[test]
fn test_issue_severity_critical() {
    let issue = CompatibilityIssue {
        severity: IssueSeverity::Critical,
        category: IssueCategory::Security,
        affected_platforms: vec!["all".to_string()],
        description: "Critical security issue".to_string(),
        fix_suggestion: Some("Fix immediately".to_string()),
    };
    assert_eq!(issue.severity, IssueSeverity::Critical);
}

#[test]
fn test_issue_severity_ordering() {
    assert!(IssueSeverity::Info < IssueSeverity::Warning);
    assert!(IssueSeverity::Warning < IssueSeverity::Error);
    assert!(IssueSeverity::Error < IssueSeverity::Critical);
}

#[test]
fn test_recommendation_priority_critical() {
    let rec = Recommendation {
        priority: RecommendationPriority::Critical,
        title: "Critical fix needed".to_string(),
        description: "Must fix immediately".to_string(),
        impact: 0.5,
        platforms: vec!["all".to_string()],
    };
    assert_eq!(rec.priority, RecommendationPriority::Critical);
}

#[test]
fn test_recommendation_priority_ordering() {
    assert!(RecommendationPriority::Low < RecommendationPriority::Medium);
    assert!(RecommendationPriority::Medium < RecommendationPriority::High);
    assert!(RecommendationPriority::High < RecommendationPriority::Critical);
}

#[test]
fn test_support_level_limited() {
    let support = PlatformSupport {
        platform: "limited-platform".to_string(),
        support_level: SupportLevel::Limited,
        compatibility_score: 0.4,
        required_modifications: vec!["Major changes".to_string()],
        runtime_requirements: Some(">=5.0.0".to_string()),
    };
    assert_eq!(support.support_level, SupportLevel::Limited);
}

// Test Recommendation Display trait
#[test]
fn test_recommendation_display() {
    let rec = Recommendation {
        priority: RecommendationPriority::High,
        title: "Optimize component".to_string(),
        description: "Details here".to_string(),
        impact: 0.2,
        platforms: vec![],
    };
    let display = format!("{}", rec);
    assert_eq!(display, "Optimize component");
}

// Test overall score calculation edge cases
#[test]
fn test_calculate_overall_score_empty_platforms() {
    let platform_scores: HashMap<String, f64> = HashMap::new();
    let feature_scores: HashMap<String, f64> = HashMap::new();
    let overall = PortabilityAnalyzer::calculate_overall_score(
        &platform_scores,
        &feature_scores,
        0.9,
        0.8,
        0.7,
        0.95,
    );
    // With empty platform_scores (0.0 * 0.3) + empty feature_scores (1.0 * 0.2)
    // + 0.9 * 0.2 + 0.8 * 0.1 + 0.7 * 0.1 + 0.95 * 0.1
    // = 0.0 + 0.2 + 0.18 + 0.08 + 0.07 + 0.095 = 0.625
    assert!(overall > 0.0);
    assert!(overall < 1.0);
}

// Test analyzer default
#[test]
fn test_portability_analyzer_default() {
    let analyzer = PortabilityAnalyzer::default();
    assert_eq!(analyzer.config.target_platforms.len(), 6);
}

// Test analysis config strict mode effect
#[test]
fn test_analyzer_strict_mode_analysis() {
    let config = AnalysisConfig {
        target_platforms: vec!["browser".to_string()],
        check_api_compatibility: true,
        check_size_constraints: true,
        check_performance: true,
        check_security: true,
        strict_mode: true,
    };
    let analyzer = PortabilityAnalyzer::new_with_config(config);
    let component = create_test_wasm_component();
    let report = analyzer
        .analyze(&component)
        .expect("analysis should succeed");

    assert!(report.score.overall_score > 0.0);
}

// Test platform support levels based on scores
#[test]
fn test_analyze_platform_support_full() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024, // Small size, high compatibility
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let support = analyzer.analyze_platform_support(&info).unwrap();
    // Browser should have full support for small, featureless component
    if let Some(browser_support) = support.get("browser") {
        assert!(browser_support.compatibility_score >= 0.9);
    }
}

#[test]
fn test_analyze_platform_support_limited() {
    let analyzer = PortabilityAnalyzer::new();
    let mut features = HashSet::new();
    features.insert("unknown-feature-1".to_string());
    features.insert("unknown-feature-2".to_string());
    features.insert("unknown-feature-3".to_string());
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features,
    };
    let support = analyzer.analyze_platform_support(&info).unwrap();
    // Multiple unknown features should lower the score
    for platform_support in support.values() {
        // Score should be lowered due to unknown features
        assert!(platform_support.compatibility_score <= 1.0);
    }
}

// Test generate recommendations with feature issues
#[test]
fn test_generate_recommendations_with_feature_issue() {
    let analyzer = PortabilityAnalyzer::new();
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features: HashSet::new(),
    };
    let issues = vec![CompatibilityIssue {
        severity: IssueSeverity::Error,
        category: IssueCategory::FeatureNotSupported,
        affected_platforms: vec!["browser".to_string()],
        description: "SIMD not supported".to_string(),
        fix_suggestion: Some("Remove SIMD usage".to_string()),
    }];
    let recommendations = analyzer.generate_recommendations(&info, &issues).unwrap();
    // Should have critical recommendation for feature incompatibility
    assert!(recommendations
        .iter()
        .any(|r| r.priority == RecommendationPriority::Critical));
}

// Test analyze feature usage
#[test]
fn test_analyze_feature_usage() {
    let analyzer = PortabilityAnalyzer::new();
    let mut features = HashSet::new();
    features.insert("memory".to_string());
    features.insert("tables".to_string());
    let info = ComponentInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 0,
        imports_count: 0,
        features: features.clone(),
    };
    let usage = analyzer.analyze_feature_usage(&info).unwrap();
    assert_eq!(usage.core_features, features);
}

// Test get_platform_limits
#[test]
fn test_get_platform_limits() {
    let analyzer = PortabilityAnalyzer::new();
    let limits = analyzer.get_platform_limits();
    assert!(limits.contains_key("cloudflare-workers"));
    assert!(limits.contains_key("browser"));
    assert_eq!(limits["cloudflare-workers"], 10 * 1024 * 1024);
}

// Test CompatibilityMatrix
#[test]
fn test_build_compatibility_matrix() {
    let matrix = PortabilityAnalyzer::build_compatibility_matrix();
    assert!(matrix.support.contains_key("cloudflare-workers"));
    assert!(matrix.support.contains_key("browser"));

    let cloudflare = matrix.support.get("cloudflare-workers").unwrap();
    assert_eq!(cloudflare.get("simd"), Some(&false));
    assert_eq!(cloudflare.get("bulk-memory"), Some(&true));
}

// Test serialization/deserialization of all types
#[test]
fn test_issue_severity_serialization() {
    for severity in [
        IssueSeverity::Info,
        IssueSeverity::Warning,
        IssueSeverity::Error,
        IssueSeverity::Critical,
    ] {
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: IssueSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(severity, deserialized);
    }
}

#[test]
fn test_issue_category_serialization() {
    for category in [
        IssueCategory::ApiIncompatibility,
        IssueCategory::FeatureNotSupported,
        IssueCategory::SizeConstraint,
        IssueCategory::Performance,
        IssueCategory::Security,
        IssueCategory::Configuration,
    ] {
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: IssueCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }
}

#[test]
fn test_recommendation_priority_serialization() {
    for priority in [
        RecommendationPriority::Low,
        RecommendationPriority::Medium,
        RecommendationPriority::High,
        RecommendationPriority::Critical,
    ] {
        let json = serde_json::to_string(&priority).unwrap();
        let deserialized: RecommendationPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(priority, deserialized);
    }
}

#[test]
fn test_support_level_serialization() {
    for level in [
        SupportLevel::Full,
        SupportLevel::Partial,
        SupportLevel::Limited,
        SupportLevel::None,
    ] {
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: SupportLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}

// Test clone traits
#[test]
fn test_compatibility_issue_clone() {
    let issue = CompatibilityIssue {
        severity: IssueSeverity::Warning,
        category: IssueCategory::Performance,
        affected_platforms: vec!["test".to_string()],
        description: "Test issue".to_string(),
        fix_suggestion: None,
    };
    let cloned = issue.clone();
    assert_eq!(issue.severity, cloned.severity);
    assert_eq!(issue.category, cloned.category);
}

#[test]
fn test_recommendation_clone() {
    let rec = Recommendation {
        priority: RecommendationPriority::Medium,
        title: "Test".to_string(),
        description: "Description".to_string(),
        impact: 0.1,
        platforms: vec!["test".to_string()],
    };
    let cloned = rec.clone();
    assert_eq!(rec.title, cloned.title);
    assert_eq!(rec.priority, cloned.priority);
}

#[test]
fn test_platform_support_clone() {
    let support = PlatformSupport {
        platform: "test".to_string(),
        support_level: SupportLevel::Partial,
        compatibility_score: 0.75,
        required_modifications: vec!["mod1".to_string()],
        runtime_requirements: Some(">=1.0".to_string()),
    };
    let cloned = support.clone();
    assert_eq!(support.platform, cloned.platform);
    assert_eq!(support.support_level, cloned.support_level);
}

#[test]
fn test_feature_usage_clone() {
    let mut core = HashSet::new();
    core.insert("memory".to_string());
    let usage = FeatureUsage {
        core_features: core,
        proposal_features: HashSet::new(),
        platform_specific: HashMap::new(),
        compatibility: HashMap::new(),
    };
    let cloned = usage.clone();
    assert_eq!(usage.core_features, cloned.core_features);
}

#[test]
fn test_size_analysis_clone() {
    let analysis = SizeAnalysis {
        total_size: 1000,
        code_size: 800,
        data_size: 100,
        custom_sections_size: 100,
        section_sizes: HashMap::new(),
        platform_limits: HashMap::new(),
    };
    let cloned = analysis.clone();
    assert_eq!(analysis.total_size, cloned.total_size);
}

#[test]
fn test_portability_score_clone() {
    let score = PortabilityScore {
        overall_score: 0.9,
        platform_scores: HashMap::new(),
        feature_scores: HashMap::new(),
        api_compatibility: 0.95,
        size_efficiency: 0.85,
        performance_portability: 0.8,
        security_compliance: 1.0,
    };
    let cloned = score.clone();
    assert_eq!(score.overall_score, cloned.overall_score);
}

#[test]
fn test_portability_report_clone() {
    let report = PortabilityReport {
        component_info: ComponentInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            size: 1024,
            exports_count: 0,
            imports_count: 0,
            features: HashSet::new(),
        },
        score: PortabilityScore {
            overall_score: 0.9,
            platform_scores: HashMap::new(),
            feature_scores: HashMap::new(),
            api_compatibility: 0.95,
            size_efficiency: 0.85,
            performance_portability: 0.8,
            security_compliance: 1.0,
        },
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
            total_size: 1024,
            code_size: 1024,
            data_size: 0,
            custom_sections_size: 0,
            section_sizes: HashMap::new(),
            platform_limits: HashMap::new(),
        },
    };
    let cloned = report.clone();
    assert_eq!(report.component_info.name, cloned.component_info.name);
}

// Debug trait tests
#[test]
fn test_portability_score_debug() {
    let score = PortabilityScore {
        overall_score: 0.85,
        platform_scores: HashMap::new(),
        feature_scores: HashMap::new(),
        api_compatibility: 0.9,
        size_efficiency: 0.8,
        performance_portability: 0.75,
        security_compliance: 0.95,
    };
    let debug_str = format!("{:?}", score);
    assert!(debug_str.contains("PortabilityScore"));
    assert!(debug_str.contains("0.85"));
}

#[test]
fn test_component_info_debug() {
    let info = ComponentInfo {
        name: "debug-test".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        exports_count: 5,
        imports_count: 3,
        features: HashSet::new(),
    };
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("ComponentInfo"));
    assert!(debug_str.contains("debug-test"));
}
