#![allow(missing_docs)]
//! Integration tests for §14.10 HARD REQUIREMENTS runtime skeletons.
//!
//! Ticket: SPEC-HARDREQ-002 (first delivery slice of §14.10).

use ruchy::provability::{
    classify, declassify, ClockCap, EnvCap, FsCap, NetCap, Public, RandomCap, RootCapability,
    Secret, Tier, TierInputs, Totality,
};
use ruchy::provability::capabilities::FsMode;

// ============================================================================
// §14.10.1 Secret<T> / Public<T> round-trip (SECRET-XXX)
// ============================================================================

#[test]
fn test_secret_001_secret_wraps_and_declassifies() {
    let key: Secret<u64> = Secret::new(0xDEADBEEF_CAFEBABE);
    let leaked = declassify(key, "integration-test: fake key only");
    assert_eq!(leaked, 0xDEADBEEF_CAFEBABE);
}

#[test]
fn test_secret_001_public_explicit_construction() {
    let p: Public<i32> = Public::new(42);
    assert_eq!(*p.get(), 42);
    assert_eq!(p.into_inner(), 42);
}

#[test]
fn test_secret_001_secret_of_different_t() {
    // Secret<T> must work for arbitrary T, not just integers.
    let s_str: Secret<&'static str> = Secret::new("password123");
    let p_str = declassify(s_str, "test");
    assert_eq!(p_str, "password123");

    let s_vec: Secret<u8> = Secret::new(0xAB);
    let p_vec = declassify(s_vec, "test");
    assert_eq!(p_vec, 0xAB);
}

// ============================================================================
// §14.10.2 Capability derivation from RootCapability (CAP-XXX)
// ============================================================================

#[test]
fn test_cap_001_root_derives_all_five_capability_kinds() {
    let root = RootCapability::__acquire_for_main();
    let _fs: FsCap = root.fs_scope("/tmp", FsMode::Read);
    let _net: NetCap = root.net_scope("localhost");
    let _env: EnvCap = root.env_scope(&["HOME"]);
    let _clock: ClockCap = root.clock();
    let _rand: RandomCap = root.random();
}

#[test]
fn test_cap_001_fs_scope_preserves_mode_and_root() {
    let root = RootCapability::__acquire_for_main();
    let fs = root.fs_scope("/etc", FsMode::ReadWrite);
    assert_eq!(fs.root(), "/etc");
    assert_eq!(fs.mode(), FsMode::ReadWrite);
}

#[test]
fn test_cap_001_net_scope_preserves_host() {
    let root = RootCapability::__acquire_for_main();
    let net = root.net_scope("api.example.com");
    assert_eq!(net.host(), "api.example.com");
}

#[test]
fn test_cap_001_env_scope_preserves_var_list() {
    let root = RootCapability::__acquire_for_main();
    let env = root.env_scope(&["PATH", "HOME", "USER"]);
    assert_eq!(env.vars(), &["PATH", "HOME", "USER"]);
}

// ============================================================================
// §14.10.3 Totality markers (TOTAL-XXX)
// ============================================================================

#[test]
fn test_total_001_totality_enum_states() {
    assert!(Totality::Total.is_total());
    assert!(Totality::Total.is_gold_eligible());
    assert!(!Totality::Partial.is_total());
    assert!(!Totality::Partial.is_gold_eligible());
    assert!(Totality::Corecursive("server event loop").is_gold_eligible());
    assert!(!Totality::Corecursive("server event loop").is_total());
}

#[test]
fn test_total_001_totality_parses_from_decorator_names() {
    assert_eq!(Totality::from_decorator("total"), Some(Totality::Total));
    assert_eq!(Totality::from_decorator("partial"), Some(Totality::Partial));
    // @corecursive requires a justification, so the bare-name lookup returns None
    assert_eq!(Totality::from_decorator("corecursive"), None);
    // Non-totality decorators return None
    assert_eq!(Totality::from_decorator("verified"), None);
    assert_eq!(Totality::from_decorator("bronze"), None);
    assert_eq!(Totality::from_decorator("gold"), None);
    assert_eq!(Totality::from_decorator("platinum"), None);
}

#[test]
fn test_total_001_default_totality_is_unknown() {
    let t = Totality::default();
    assert_eq!(t, Totality::Unknown);
    // Unknown is NOT gold-eligible (warning tier).
    assert!(!t.is_gold_eligible());
}

// ============================================================================
// §14.2 Tier classification pipeline (TIER-XXX)
// ============================================================================

#[test]
fn test_tier_001_classify_via_full_api() {
    // Bare function -> Bronze
    let bronze = TierInputs::default();
    assert_eq!(classify(&bronze), Tier::Bronze);

    // With contracts -> Silver
    let silver = TierInputs {
        decorators: vec![],
        has_requires: true,
        has_ensures: true,
        has_yaml_contract: false,
        has_lean_proof: false,
    };
    assert_eq!(classify(&silver), Tier::Silver);

    // With @gold + contracts -> Gold
    let gold = TierInputs {
        decorators: vec!["gold"],
        has_requires: true,
        has_ensures: true,
        has_yaml_contract: false,
        has_lean_proof: false,
    };
    assert_eq!(classify(&gold), Tier::Gold);

    // Full Platinum stack
    let platinum = TierInputs {
        decorators: vec!["platinum"],
        has_requires: true,
        has_ensures: true,
        has_yaml_contract: true,
        has_lean_proof: true,
    };
    assert_eq!(classify(&platinum), Tier::Platinum);
}

#[test]
fn test_tier_001_tier_obligations_align_with_spec() {
    // §14.2 + §14.6: Bronze is banned in stdlib after 5.2
    assert!(!Tier::Bronze.is_stdlib_eligible_at_52());
    assert!(Tier::Silver.is_stdlib_eligible_at_52());

    // §14.2 + §14.10.6: Gold+ requires Kani
    assert!(!Tier::Silver.requires_kani());
    assert!(Tier::Gold.requires_kani());
    assert!(Tier::Platinum.requires_kani());

    // §14.10.5: Only Platinum requires Lean refinement proof
    assert!(!Tier::Gold.requires_lean_refinement());
    assert!(Tier::Platinum.requires_lean_refinement());
}
