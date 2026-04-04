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
/// Used by `ruchy infra plan/apply/status/destroy` subcommands.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineSpec {
    /// Machine name/identifier
    pub name: String,
    /// Machine type (e.g., "web", "db", "worker")
    pub machine_type: String,
    /// Region/location
    pub region: Option<String>,
    /// CPU count
    pub cpus: u32,
    /// Memory in MB
    pub memory_mb: u64,
    /// Disk in GB
    pub disk_gb: u64,
}

impl MachineSpec {
    /// Create a new machine specification with sensible defaults.
    pub fn new(name: &str, machine_type: &str) -> Self {
        Self {
            name: name.to_string(),
            machine_type: machine_type.to_string(),
            region: None,
            cpus: 1,
            memory_mb: 512,
            disk_gb: 10,
        }
    }

    /// Set the region for this machine.
    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    /// Set CPU count.
    pub fn with_cpus(mut self, cpus: u32) -> Self {
        self.cpus = cpus;
        self
    }

    /// Set memory in MB.
    pub fn with_memory_mb(mut self, mb: u64) -> Self {
        self.memory_mb = mb;
        self
    }

    /// Set disk in GB.
    pub fn with_disk_gb(mut self, gb: u64) -> Self {
        self.disk_gb = gb;
        self
    }
}

/// Network specification for infrastructure blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSpec {
    /// Network name
    pub name: String,
    /// CIDR block (e.g., "10.0.0.0/16")
    pub cidr: String,
    /// Whether this network is public-facing
    pub public: bool,
}

impl NetworkSpec {
    /// Create a new network specification.
    pub fn new(name: &str, cidr: &str) -> Self {
        Self {
            name: name.to_string(),
            cidr: cidr.to_string(),
            public: false,
        }
    }
}

/// A single resource change in an infrastructure plan.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceChange {
    /// Resource will be created
    Create(String),
    /// Resource will be updated (old → new description)
    Update { name: String, detail: String },
    /// Resource will be destroyed
    Destroy(String),
    /// Resource is unchanged
    NoOp(String),
}

/// Infrastructure plan result from `ruchy infra plan`.
#[derive(Debug, Clone)]
pub struct InfraPlan {
    /// All planned resource changes
    pub changes: Vec<ResourceChange>,
}

impl InfraPlan {
    /// Create an empty plan.
    pub fn empty() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Count of resources to create.
    pub fn creates(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ResourceChange::Create(_)))
            .count()
    }

    /// Count of resources to update.
    pub fn updates(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ResourceChange::Update { .. }))
            .count()
    }

    /// Count of resources to destroy.
    pub fn destroys(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| matches!(c, ResourceChange::Destroy(_)))
            .count()
    }

    /// Whether this plan has any changes.
    pub fn has_changes(&self) -> bool {
        self.changes
            .iter()
            .any(|c| !matches!(c, ResourceChange::NoOp(_)))
    }

    /// Format the plan as a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Plan: {} to create, {} to update, {} to destroy",
            self.creates(),
            self.updates(),
            self.destroys()
        )
    }
}

/// Infrastructure state snapshot for drift detection.
#[derive(Debug, Clone, PartialEq)]
pub struct InfraState {
    /// Known resources and their current state hashes
    pub resources: Vec<(String, String)>,
}

impl InfraState {
    /// Create an empty state.
    pub fn empty() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Number of tracked resources.
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
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
        assert_eq!(spec.cpus, 1);
        assert_eq!(spec.memory_mb, 512);
    }

    #[test]
    fn test_machine_spec_builder() {
        let spec = MachineSpec::new("db-1", "db")
            .with_region("us-east-1")
            .with_cpus(4)
            .with_memory_mb(8192)
            .with_disk_gb(500);
        assert_eq!(spec.region, Some("us-east-1".to_string()));
        assert_eq!(spec.cpus, 4);
        assert_eq!(spec.memory_mb, 8192);
        assert_eq!(spec.disk_gb, 500);
    }

    #[test]
    fn test_network_spec_new() {
        let net = NetworkSpec::new("vpc-main", "10.0.0.0/16");
        assert_eq!(net.name, "vpc-main");
        assert_eq!(net.cidr, "10.0.0.0/16");
        assert!(!net.public);
    }

    #[test]
    fn test_infra_plan_empty() {
        let plan = InfraPlan::empty();
        assert_eq!(plan.creates(), 0);
        assert_eq!(plan.updates(), 0);
        assert_eq!(plan.destroys(), 0);
        assert!(!plan.has_changes());
    }

    #[test]
    fn test_infra_plan_with_changes() {
        let plan = InfraPlan {
            changes: vec![
                ResourceChange::Create("web-1".to_string()),
                ResourceChange::Create("web-2".to_string()),
                ResourceChange::Update {
                    name: "db-1".to_string(),
                    detail: "resize 4→8 CPU".to_string(),
                },
                ResourceChange::Destroy("old-cache".to_string()),
                ResourceChange::NoOp("vpc-main".to_string()),
            ],
        };
        assert_eq!(plan.creates(), 2);
        assert_eq!(plan.updates(), 1);
        assert_eq!(plan.destroys(), 1);
        assert!(plan.has_changes());
        assert_eq!(
            plan.summary(),
            "Plan: 2 to create, 1 to update, 1 to destroy"
        );
    }

    #[test]
    fn test_infra_state_empty() {
        let state = InfraState::empty();
        assert_eq!(state.resource_count(), 0);
    }

    #[test]
    fn test_infra_state_with_resources() {
        let state = InfraState {
            resources: vec![
                ("web-1".to_string(), "abc123".to_string()),
                ("db-1".to_string(), "def456".to_string()),
            ],
        };
        assert_eq!(state.resource_count(), 2);
    }
}
