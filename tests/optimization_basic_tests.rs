//! Basic coverage tests for optimization module
//!
//! [TEST-COV-013] Minimal Optimization Coverage

use ruchy::optimization::{
    HardwareProfile, BranchPredictorProfile, VectorUnitProfile,
    MechanicalSympathyTuner,
};

#[test]
fn test_hardware_profile_default() {
    let profile = HardwareProfile::default();
    
    // Default should have reasonable values
    assert!(profile.cores > 0);
    assert!(profile.l1_cache_size > 0);
    assert!(profile.l2_cache_size > profile.l1_cache_size);
    assert!(profile.cache_line_size > 0);
}

#[test]
fn test_branch_predictor_profile() {
    let predictor = BranchPredictorProfile {
        bht_size: 2048,
        pht_size: 1024,
        regular_accuracy: 0.98,
        irregular_accuracy: 0.55,
    };
    
    assert_eq!(predictor.bht_size, 2048);
    assert_eq!(predictor.pht_size, 1024);
    assert!(predictor.regular_accuracy > predictor.irregular_accuracy);
}

#[test]
fn test_vector_unit_profile() {
    let vector_unit = VectorUnitProfile {
        instruction_sets: vec!["SSE".to_string(), "AVX".to_string()],
        register_width: 256,
        execution_units: 2,
        throughput: std::collections::HashMap::new(),
    };
    
    assert_eq!(vector_unit.register_width, 256);
    assert_eq!(vector_unit.execution_units, 2);
    assert!(vector_unit.instruction_sets.contains(&"AVX".to_string()));
}

#[test]
fn test_mechanical_sympathy_tuner_creation() {
    let _tuner = MechanicalSympathyTuner::new();
    // Just verify it can be created
    // Analysis methods require actual AST input
}

#[test]
fn test_hardware_profile_custom() {
    let profile = HardwareProfile {
        architecture: "aarch64".to_string(),
        cores: 4,
        l1_cache_size: 32768,
        l2_cache_size: 262_144,
        l3_cache_size: Some(4_194_304),
        cache_line_size: 128,
        branch_predictor: BranchPredictorProfile {
            bht_size: 1024,
            pht_size: 512,
            regular_accuracy: 0.90,
            irregular_accuracy: 0.50,
        },
        vector_units: VectorUnitProfile {
            instruction_sets: vec!["NEON".to_string()],
            register_width: 128,
            execution_units: 1,
            throughput: std::collections::HashMap::new(),
        },
    };
    
    assert_eq!(profile.architecture, "aarch64");
    assert_eq!(profile.cores, 4);
    assert_eq!(profile.cache_line_size, 128);
    assert!(profile.vector_units.instruction_sets.contains(&"NEON".to_string()));
}