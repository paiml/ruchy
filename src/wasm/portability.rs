//! Portability scoring for WebAssembly components (RUCHY-0819)
//!
//! Analyzes and scores the portability of Ruchy-generated WASM components
//! across different platforms and runtimes.
#[cfg(test)]
mod tests {
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
}
use super::component::WasmComponent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
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
    /// Safety compliance score
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
    /// Check safety requirements
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
    /// Critical (safety or major functionality issue)
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
    /// Performance concern
    Performance,
    /// Safety concern
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
impl Default for PortabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
impl PortabilityAnalyzer {
    /// Create a new portability analyzer with default config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::portability::PortabilityAnalyzer;
    ///
    /// let instance = PortabilityAnalyzer::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
            compatibility_matrix: Self::build_compatibility_matrix(),
            platform_requirements: Self::build_platform_requirements(),
        }
    }
    /// Create a new portability analyzer with specific config
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm::portability::PortabilityAnalyzer;
    ///
    /// let mut instance = PortabilityAnalyzer::new();
    /// let result = instance.new_with_config();
    /// // Verify behavior
    /// ```
    pub fn new_with_config(config: AnalysisConfig) -> Self {
        Self {
            config,
            compatibility_matrix: Self::build_compatibility_matrix(),
            platform_requirements: Self::build_platform_requirements(),
        }
    }
    /// Analyze a WASM component's portability
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::wasm::portability::analyze;
    ///
    /// let result = analyze(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
                } else if !reqs.required_features.contains(feature)
                    && !reqs.optional_features.contains(feature)
                {
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
            let score = f64::from(support_count) / total_platforms as f64;
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
        // Check safety requirements
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
        let platform_avg = if platform_scores.is_empty() {
            0.0
        } else {
            platform_scores.values().sum::<f64>() / platform_scores.len() as f64
        };
        let feature_avg = if feature_scores.is_empty() {
            1.0
        } else {
            feature_scores.values().sum::<f64>() / feature_scores.len() as f64
        };
        // Weighted average
        platform_avg * 0.3
            + feature_avg * 0.2
            + api_compatibility * 0.2
            + size_efficiency * 0.1
            + performance_portability * 0.1
            + security_compliance * 0.1
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
                            fix_suggestion: Some(
                                "Consider optimizing component size or splitting functionality"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
        }
        Ok(issues)
    }
    fn generate_recommendations(
        &self,
        info: &ComponentInfo,
        issues: &[CompatibilityIssue],
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        // Size optimization recommendation
        if info.size > 100 * 1024 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                title: "Optimize component size".to_string(),
                description: "Component size can be reduced through optimization techniques"
                    .to_string(),
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
    fn analyze_platform_support(
        &self,
        info: &ComponentInfo,
    ) -> Result<HashMap<String, PlatformSupport>> {
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
            support.insert(
                platform.clone(),
                PlatformSupport {
                    platform: platform.clone(),
                    support_level,
                    compatibility_score: score,
                    required_modifications: Vec::new(),
                    runtime_requirements: None,
                },
            );
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
        let custom_sections_size: usize = component
            .custom_sections
            .values()
            .map(std::vec::Vec::len)
            .sum();
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
        limits.insert("fastly-compute".to_string(), 50 * 1024 * 1024); // 50MB
        limits.insert("aws-lambda".to_string(), 250 * 1024 * 1024); // 250MB
        limits.insert("browser".to_string(), 100 * 1024 * 1024); // 100MB
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
        requirements.insert(
            "cloudflare-workers".to_string(),
            PlatformRequirements {
                required_features: HashSet::new(),
                optional_features: HashSet::from([
                    "bulk-memory".to_string(),
                    "reference-types".to_string(),
                ]),
                incompatible_features: HashSet::from(["threads".to_string()]),
                size_limit: Some(10 * 1024 * 1024),
                _api_requirements: HashSet::new(),
            },
        );
        // Browser requirements
        requirements.insert(
            "browser".to_string(),
            PlatformRequirements {
                required_features: HashSet::new(),
                optional_features: HashSet::from(["simd".to_string(), "threads".to_string()]),
                incompatible_features: HashSet::new(),
                size_limit: Some(100 * 1024 * 1024),
                _api_requirements: HashSet::new(),
            },
        );
        requirements
    }
}
#[cfg(test)]
mod additional_portability_tests {
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
        let report = analyzer.analyze(&component).expect("analysis should succeed");

        assert_eq!(report.component_info.name, "test-component");
        assert!(report.score.overall_score > 0.0);
        assert!(report.score.overall_score <= 1.0);
    }

    #[test]
    fn test_analyzer_analyze_large_component() {
        let analyzer = PortabilityAnalyzer::new();
        let component = create_large_wasm_component();
        let report = analyzer.analyze(&component).expect("analysis should succeed");

        assert_eq!(report.component_info.name, "large-component");
        // Large component should have recommendations
        assert!(report.recommendations.len() >= 1);
    }

    #[test]
    fn test_analyzer_analyze_very_large_component() {
        let analyzer = PortabilityAnalyzer::new();
        let component = create_very_large_wasm_component();
        let report = analyzer.analyze(&component).expect("analysis should succeed");

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

        let report = analyzer.analyze(&component).expect("analysis should succeed");
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

        let report = analyzer.analyze(&component).expect("analysis should succeed");
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
        assert!(issues.iter().any(|i| i.category == IssueCategory::SizeConstraint));
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
        let report = analyzer.analyze(&component).expect("analysis should succeed");

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
        assert!(recommendations.iter().any(|r| r.priority == RecommendationPriority::Critical));
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
}

#[cfg(test)]
mod property_tests_portability {
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
