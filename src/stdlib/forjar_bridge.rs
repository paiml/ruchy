//! Forjar Bridge Module (Pillar 3: Infrastructure as Code)
//!
//! Thin wrappers around Forjar for Ruchy stdlib.
//! Per ruchy-5.0-sovereign-platform.md Section 2: `infra {}` blocks transpile to
//! `forjar::InfraSpec` builders.
//!
//! # Design
//! - Declarative infrastructure syntax
//! - DAG-ordered resource resolution
//! - BLAKE3 state locks
//! - Copia delta sync for files >1MB
//!
//! # Feature Gate
//! Requires `--features infra` to enable.

#[cfg(feature = "infra")]
mod inner {
    pub use forjar::*;
}

#[cfg(feature = "infra")]
pub use inner::*;

/// Machine resource specification for infrastructure blocks.
///
/// Maps to `forjar::MachineSpec` when the `infra` feature is enabled.
#[derive(Debug, Clone)]
pub struct MachineSpec {
    /// Machine name/identifier
    pub name: String,
    /// Machine type (e.g., "web", "db", "worker")
    pub machine_type: String,
    /// Region/location
    pub region: Option<String>,
}

impl MachineSpec {
    /// Create a new machine specification.
    pub fn new(name: &str, machine_type: &str) -> Self {
        Self {
            name: name.to_string(),
            machine_type: machine_type.to_string(),
            region: None,
        }
    }
}

/// Infrastructure plan result.
#[derive(Debug)]
pub struct InfraPlan {
    /// Resources to create
    pub creates: Vec<String>,
    /// Resources to update
    pub updates: Vec<String>,
    /// Resources to destroy
    pub destroys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_spec_new() {
        let spec = MachineSpec::new("web-1", "web");
        assert_eq!(spec.name, "web-1");
        assert_eq!(spec.machine_type, "web");
        assert!(spec.region.is_none());
    }
}
