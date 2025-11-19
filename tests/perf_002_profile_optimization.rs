// PERF-002: Optimized Binary Speed & Size Specification
// Tests for cargo profile optimization (Phase 2: Fix release-dist profile)
// EXTREME TDD: RED phase - Tests verify profile configuration
//
// Reference: docs/specifications/optimized-binary-speed-size-spec.md
// Issue: release-dist uses opt-level="z" (size) instead of opt-level=3 (speed)
// Fix: Change to opt-level=3 for 15x speedup (vs 2x with size optimization)

#![allow(missing_docs)]

use std::fs;
use std::path::PathBuf;

/// Helper: Read Cargo.toml and parse as TOML Table
fn read_cargo_toml() -> toml::Table {
    let cargo_toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");
    content
        .parse::<toml::Table>()
        .expect("Failed to parse Cargo.toml as TOML table")
}

/// Helper: Get profile configuration from Cargo.toml
fn get_profile_config(profile_name: &str) -> toml::Table {
    let cargo_toml = read_cargo_toml();
    cargo_toml
        .get("profile")
        .and_then(|p| p.as_table())
        .and_then(|profiles| profiles.get(profile_name))
        .and_then(|p| p.as_table())
        .cloned()
        .unwrap_or_else(|| panic!("{profile_name} profile not found in Cargo.toml"))
}

// ============================================================================
// RED PHASE: Profile Configuration Tests (WILL FAIL INITIALLY)
// ============================================================================

/// Test 1: release-dist profile uses opt-level=3 (speed), not "z" (size)
/// Expected: opt-level = 3 for maximum speed (15x speedup)
/// Current: opt-level = "z" for size (only 2x speedup) ❌
#[test]
fn test_perf_002_01_release_dist_uses_speed_optimization() {
    let profile = get_profile_config("release-dist");

    let opt_level = profile
        .get("opt-level")
        .expect("opt-level not found in release-dist profile");

    // CRITICAL: Must be 3 (speed), not "z" (size)
    assert_eq!(
        opt_level.as_integer().unwrap_or(0),
        3,
        "release-dist profile must use opt-level=3 (speed), not '{opt_level}' (found: {opt_level:?})"
    );
}

/// Test 2: release-dist profile inherits from release
/// Expected: inherits = "release" for consistency
#[test]
fn test_perf_002_02_release_dist_inherits_from_release() {
    let profile = get_profile_config("release-dist");

    let inherits = profile
        .get("inherits")
        .expect("inherits not found in release-dist profile");

    assert_eq!(
        inherits.as_str().unwrap(),
        "release",
        "release-dist must inherit from release profile"
    );
}

/// Test 3: release-dist profile uses fat LTO
/// Expected: lto = "fat" for maximum optimization
#[test]
fn test_perf_002_03_release_dist_uses_fat_lto() {
    let profile = get_profile_config("release-dist");

    let lto = profile
        .get("lto")
        .expect("lto not found in release-dist profile");

    assert_eq!(
        lto.as_str().unwrap(),
        "fat",
        "release-dist must use lto='fat' for maximum optimization"
    );
}

/// Test 4: release-dist profile uses single codegen unit
/// Expected: codegen-units = 1 for best optimization
#[test]
fn test_perf_002_04_release_dist_uses_single_codegen_unit() {
    let profile = get_profile_config("release-dist");

    let codegen_units = profile
        .get("codegen-units")
        .expect("codegen-units not found in release-dist profile");

    assert_eq!(
        codegen_units.as_integer().unwrap(),
        1,
        "release-dist must use codegen-units=1 for best optimization"
    );
}

/// Test 5: release-dist profile disables overflow checks
/// Expected: overflow-checks = false for maximum speed
#[test]
fn test_perf_002_05_release_dist_disables_overflow_checks() {
    let profile = get_profile_config("release-dist");

    let overflow_checks = profile
        .get("overflow-checks")
        .expect("overflow-checks not found in release-dist profile");

    assert!(
        !overflow_checks.as_bool().unwrap(),
        "release-dist must disable overflow-checks for speed"
    );
}

/// Test 6: release-dist profile disables debug assertions
/// Expected: debug-assertions = false for speed
#[test]
fn test_perf_002_06_release_dist_disables_debug_assertions() {
    let profile = get_profile_config("release-dist");

    let debug_assertions = profile
        .get("debug-assertions")
        .expect("debug-assertions not found in release-dist profile");

    assert!(
        !debug_assertions.as_bool().unwrap(),
        "release-dist must disable debug-assertions for speed"
    );
}

/// Test 7: release-dist profile disables incremental compilation
/// Expected: incremental = false for better optimization
#[test]
fn test_perf_002_07_release_dist_disables_incremental() {
    let profile = get_profile_config("release-dist");

    let incremental = profile
        .get("incremental")
        .expect("incremental not found in release-dist profile");

    assert!(
        !incremental.as_bool().unwrap(),
        "release-dist must disable incremental for better optimization"
    );
}

// ============================================================================
// Verification Tests: Other Profiles
// ============================================================================

/// Test 8: release profile uses opt-level=3 (baseline verification)
/// Expected: Already correct (changed in PERF-001 v3.174.0)
#[test]
fn test_perf_002_08_release_profile_uses_speed_optimization() {
    let profile = get_profile_config("release");

    let opt_level = profile
        .get("opt-level")
        .expect("opt-level not found in release profile");

    assert_eq!(
        opt_level.as_integer().unwrap(),
        3,
        "release profile must use opt-level=3 (changed in PERF-001)"
    );
}

/// Test 9: release-tiny profile uses opt-level="z" (size optimization)
/// Expected: Preserves size-optimized behavior for embedded systems
#[test]
fn test_perf_002_09_release_tiny_uses_size_optimization() {
    let profile = get_profile_config("release-tiny");

    let opt_level = profile
        .get("opt-level")
        .expect("opt-level not found in release-tiny profile");

    assert_eq!(
        opt_level.as_str().unwrap(),
        "z",
        "release-tiny must use opt-level='z' for size optimization"
    );
}

/// Test 10: release-ultra profile uses opt-level=3 (maximum performance)
/// Expected: Speed optimization for PGO builds
#[test]
fn test_perf_002_10_release_ultra_uses_speed_optimization() {
    let profile = get_profile_config("release-ultra");

    let opt_level = profile
        .get("opt-level")
        .expect("opt-level not found in release-ultra profile");

    assert_eq!(
        opt_level.as_integer().unwrap(),
        3,
        "release-ultra must use opt-level=3 for maximum performance"
    );
}

// ============================================================================
// Consistency Tests: Profile Hierarchy
// ============================================================================

/// Test 11: All release variants inherit from release or use consistent settings
/// Expected: Consistency across profile family
#[test]
fn test_perf_002_11_profile_consistency() {
    let release = get_profile_config("release");
    let release_dist = get_profile_config("release-dist");
    let release_ultra = get_profile_config("release-ultra");

    // All should use fat LTO
    assert_eq!(release.get("lto").unwrap().as_str().unwrap(), "fat");
    assert_eq!(release_dist.get("lto").unwrap().as_str().unwrap(), "fat");
    assert_eq!(release_ultra.get("lto").unwrap().as_str().unwrap(), "fat");

    // All should use single codegen unit
    assert_eq!(
        release.get("codegen-units").unwrap().as_integer().unwrap(),
        1
    );
    assert_eq!(
        release_dist
            .get("codegen-units")
            .unwrap()
            .as_integer()
            .unwrap(),
        1
    );
    assert_eq!(
        release_ultra
            .get("codegen-units")
            .unwrap()
            .as_integer()
            .unwrap(),
        1
    );
}

/// Test 12: Profile documentation comment exists
/// Expected: Cargo.toml has comments explaining profile choices
#[test]
fn test_perf_002_12_profile_documentation() {
    let cargo_toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");

    // Check for documentation comments about profiles
    assert!(
        content.contains("PERF-001") || content.contains("PERF-002"),
        "Cargo.toml must have comments documenting profile choices"
    );
}

// ============================================================================
// Integration Tests: Property-Based Verification
// ============================================================================

/// Property Test 1: All speed-optimized profiles use opt-level=3
/// Expected: release, release-dist, release-ultra all use opt-level=3
#[test]
fn test_perf_002_property_speed_profiles_use_opt_level_3() {
    let speed_profiles = vec!["release", "release-dist", "release-ultra"];

    for profile_name in speed_profiles {
        let profile = get_profile_config(profile_name);

        let opt_level = profile
            .get("opt-level")
            .unwrap_or_else(|| panic!("opt-level not found in {profile_name}"));

        assert_eq!(
            opt_level.as_integer().unwrap_or(0),
            3,
            "{profile_name} must use opt-level=3 for speed"
        );
    }
}

/// Property Test 2: All release profiles use fat LTO
/// Expected: LTO is essential for both speed AND size benefits
#[test]
fn test_perf_002_property_all_release_profiles_use_fat_lto() {
    let profiles = vec!["release", "release-dist", "release-ultra", "release-tiny"];

    for profile_name in profiles {
        let profile = get_profile_config(profile_name);

        let lto = profile
            .get("lto")
            .unwrap_or_else(|| panic!("lto not found in {profile_name}"));

        assert_eq!(
            lto.as_str().unwrap(),
            "fat",
            "{profile_name} must use lto='fat' per PERF-002 findings"
        );
    }
}

/// Property Test 3: Size-optimized profiles use opt-level="z" or "s"
/// Expected: Only release-tiny uses size optimization
#[test]
fn test_perf_002_property_size_profiles_use_z_or_s() {
    let profile = get_profile_config("release-tiny");

    let opt_level = profile
        .get("opt-level")
        .expect("opt-level not found in release-tiny");

    let opt_level_str = opt_level.as_str().unwrap();
    assert!(
        opt_level_str == "z" || opt_level_str == "s",
        "release-tiny must use opt-level='z' or 's' for size optimization"
    );
}
