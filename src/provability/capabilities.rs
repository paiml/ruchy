//! Capability types for effect isolation (§14.10.2, CAP-XXX).
//!
//! Every I/O operation in Ruchy 5.2+ requires an explicit capability
//! argument: `fs::read(&fs_cap, path)` replaces `fs::read(path)`.
//! `RootCapability` is passed to `main()` by the runtime and is the ONLY
//! source of authority in the program.
//!
//! This module ships the **runtime skeletons**. The compile-time
//! enforcement (reject ambient authority in stdlib `pub fn`) is delivered
//! by the CAP-002 lint pass in a future sprint.
//!
//! Reference: Austral capability-based security.

/// The root capability — passed to `main` by the runtime. The ONLY source
/// of authority in a Ruchy program.
///
/// Derived capabilities are obtained via `root.fs_scope()`, `root.net_scope()`,
/// etc. `RootCapability` itself is NOT `Clone`/`Copy` — it must be borrowed
/// or passed linearly.
///
/// # Security
///
/// - Stdlib `pub fn`s CANNOT accept `RootCapability` directly (anti-privilege-
///   escalation). They must accept a scoped subcapability.
/// - Library code that needs I/O declares the specific capability it needs
///   in its signature, making the permission surface visible to callers.
#[derive(Debug)]
pub struct RootCapability {
    _private: (),
}

impl RootCapability {
    /// Obtain the root capability. Currently unrestricted, but future
    /// sprints (CAP-003) will require an explicit opt-in in `main`.
    #[doc(hidden)]
    #[must_use]
    pub const fn __acquire_for_main() -> Self {
        Self { _private: () }
    }

    /// Derive a filesystem capability scoped to a specific root path with
    /// specified access mode.
    #[must_use]
    pub fn fs_scope(&self, root: &'static str, mode: FsMode) -> FsCap {
        FsCap { root, mode }
    }

    /// Derive a network capability scoped to a specific host.
    #[must_use]
    pub fn net_scope(&self, host: &'static str) -> NetCap {
        NetCap { host }
    }

    /// Derive an environment capability for a fixed set of variable names.
    #[must_use]
    pub fn env_scope(&self, vars: &'static [&'static str]) -> EnvCap {
        EnvCap { vars }
    }

    /// Obtain a clock capability (read wall-clock time).
    #[must_use]
    pub fn clock(&self) -> ClockCap {
        ClockCap { _private: () }
    }

    /// Obtain a random-number capability.
    #[must_use]
    pub fn random(&self) -> RandomCap {
        RandomCap { _private: () }
    }
}

/// Filesystem access mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsMode {
    Read,
    Write,
    ReadWrite,
}

/// Scoped filesystem capability.
#[derive(Debug)]
pub struct FsCap {
    root: &'static str,
    mode: FsMode,
}

impl FsCap {
    /// The filesystem root this capability is scoped to.
    #[must_use]
    pub const fn root(&self) -> &'static str {
        self.root
    }

    /// The access mode granted by this capability.
    #[must_use]
    pub const fn mode(&self) -> FsMode {
        self.mode
    }
}

/// Scoped network capability.
#[derive(Debug)]
pub struct NetCap {
    host: &'static str,
}

impl NetCap {
    /// The host this capability is scoped to.
    #[must_use]
    pub const fn host(&self) -> &'static str {
        self.host
    }
}

/// Scoped environment-variable capability.
#[derive(Debug)]
pub struct EnvCap {
    vars: &'static [&'static str],
}

impl EnvCap {
    /// The set of environment variables this capability grants access to.
    #[must_use]
    pub const fn vars(&self) -> &'static [&'static str] {
        self.vars
    }
}

/// Wall-clock capability.
#[derive(Debug)]
pub struct ClockCap {
    _private: (),
}

/// Random-number-generation capability.
#[derive(Debug)]
pub struct RandomCap {
    _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_derives_fs_capability() {
        let root = RootCapability::__acquire_for_main();
        let fs = root.fs_scope("/tmp", FsMode::Read);
        assert_eq!(fs.root(), "/tmp");
        assert_eq!(fs.mode(), FsMode::Read);
    }

    #[test]
    fn test_root_derives_net_capability() {
        let root = RootCapability::__acquire_for_main();
        let net = root.net_scope("api.example.com");
        assert_eq!(net.host(), "api.example.com");
    }

    #[test]
    fn test_root_derives_env_capability() {
        let root = RootCapability::__acquire_for_main();
        let env = root.env_scope(&["PATH", "HOME"]);
        assert_eq!(env.vars(), &["PATH", "HOME"]);
    }

    #[test]
    fn test_capabilities_are_distinct_types() {
        // This is a compile-time check: trying to pass FsCap where NetCap
        // is expected must NOT compile. (Verified by type system; we just
        // verify the types exist and are separate.)
        let root = RootCapability::__acquire_for_main();
        let _fs: FsCap = root.fs_scope("/", FsMode::ReadWrite);
        let _net: NetCap = root.net_scope("localhost");
        let _env: EnvCap = root.env_scope(&[]);
        let _clock: ClockCap = root.clock();
        let _rand: RandomCap = root.random();
    }

    #[test]
    fn test_fs_mode_equality() {
        assert_eq!(FsMode::Read, FsMode::Read);
        assert_ne!(FsMode::Read, FsMode::Write);
        assert_ne!(FsMode::Read, FsMode::ReadWrite);
    }
}
