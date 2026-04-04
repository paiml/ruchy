//! Bashrs Bridge Module (Pillar 4: Shell Transpilation)
//!
//! Thin wrappers around bashrs for Ruchy stdlib.
//! Per ruchy-5.0-sovereign-platform.md Section 2: `--target shell` compilation
//! produces POSIX-compliant shell scripts with injection-proof quoting.
//!
//! # Design
//! - POSIX script output from Ruchy source
//! - Injection-proof quoting via bashrs
//! - `ruchy purify` command for legacy bash cleanup
//! - Cross-shell matrix testing (bash, zsh, dash, fish, ash, busybox)
//!
//! # Feature Gate
//! Requires `--features shell-target` to enable.

#[cfg(feature = "shell-target")]
mod inner {
    pub use bashrs::*;
}

#[cfg(feature = "shell-target")]
pub use inner::*;

/// Shell transpilation target configuration.
#[derive(Debug, Clone)]
pub struct ShellTarget {
    /// Target shell (bash, zsh, dash, fish, ash, busybox)
    pub shell: String,
    /// Enable strict mode (set -euo pipefail)
    pub strict: bool,
    /// Enable shellcheck-compatible output
    pub shellcheck: bool,
}

impl Default for ShellTarget {
    fn default() -> Self {
        Self {
            shell: "bash".to_string(),
            strict: true,
            shellcheck: true,
        }
    }
}

/// Result of shell script analysis (for `ruchy purify`).
#[derive(Debug)]
pub struct PurifyResult {
    /// Number of issues found
    pub issues: usize,
    /// Number of issues auto-fixed
    pub fixed: usize,
    /// Remaining issues requiring manual review
    pub remaining: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_target_default() {
        let target = ShellTarget::default();
        assert_eq!(target.shell, "bash");
        assert!(target.strict);
        assert!(target.shellcheck);
    }
}
