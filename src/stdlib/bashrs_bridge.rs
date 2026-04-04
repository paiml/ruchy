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

/// Supported target shells for transpilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// GNU Bash (default)
    Bash,
    /// Z Shell
    Zsh,
    /// Debian Almquist Shell (POSIX)
    Dash,
    /// Friendly Interactive Shell
    Fish,
    /// Almquist Shell
    Ash,
    /// BusyBox sh
    BusyBox,
}

impl Shell {
    /// Parse from string, case-insensitive.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "dash" => Some(Self::Dash),
            "fish" => Some(Self::Fish),
            "ash" => Some(Self::Ash),
            "busybox" | "busybox-sh" => Some(Self::BusyBox),
            _ => None,
        }
    }

    /// Get the shebang line for this shell.
    pub fn shebang(&self) -> &'static str {
        match self {
            Self::Bash => "#!/usr/bin/env bash",
            Self::Zsh => "#!/usr/bin/env zsh",
            Self::Dash => "#!/usr/bin/env dash",
            Self::Fish => "#!/usr/bin/env fish",
            Self::Ash => "#!/usr/bin/env ash",
            Self::BusyBox => "#!/bin/sh",
        }
    }

    /// Whether this shell supports POSIX `set -euo pipefail`.
    pub fn supports_pipefail(&self) -> bool {
        matches!(self, Self::Bash | Self::Zsh)
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Zsh => write!(f, "zsh"),
            Self::Dash => write!(f, "dash"),
            Self::Fish => write!(f, "fish"),
            Self::Ash => write!(f, "ash"),
            Self::BusyBox => write!(f, "busybox"),
        }
    }
}

/// Shell transpilation target configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellTarget {
    /// Target shell
    pub shell: Shell,
    /// Enable strict mode (set -euo pipefail where supported)
    pub strict: bool,
    /// Enable shellcheck-compatible output
    pub shellcheck: bool,
}

impl Default for ShellTarget {
    fn default() -> Self {
        Self {
            shell: Shell::Bash,
            strict: true,
            shellcheck: true,
        }
    }
}

impl ShellTarget {
    /// Create a target for a specific shell.
    pub fn for_shell(shell: Shell) -> Self {
        Self {
            shell,
            ..Default::default()
        }
    }

    /// Generate the preamble (shebang + strict mode).
    pub fn preamble(&self) -> String {
        let mut lines = vec![self.shell.shebang().to_string()];
        if self.strict {
            if self.shell.supports_pipefail() {
                lines.push("set -euo pipefail".to_string());
            } else {
                lines.push("set -eu".to_string());
            }
        }
        if self.shellcheck {
            lines.push(format!("# shellcheck shell={}", self.shell));
        }
        lines.join("\n")
    }
}

/// Severity of a shell script issue found by `ruchy purify`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational suggestion
    Info,
    /// Style/convention issue
    Style,
    /// Potential bug or portability issue
    Warning,
    /// Definite bug or security issue
    Error,
}

/// A single issue found during shell script analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct PurifyIssue {
    /// Line number (1-based)
    pub line: usize,
    /// Column (1-based)
    pub column: usize,
    /// Severity level
    pub severity: IssueSeverity,
    /// Issue code (e.g., "SC2086")
    pub code: String,
    /// Human-readable description
    pub message: String,
    /// Whether this issue can be auto-fixed
    pub fixable: bool,
}

/// Result of shell script analysis from `ruchy purify`.
#[derive(Debug, Clone)]
pub struct PurifyResult {
    /// All issues found
    pub issues: Vec<PurifyIssue>,
    /// Number of issues auto-fixed (if fix mode was enabled)
    pub fixed: usize,
}

impl PurifyResult {
    /// Create an empty (clean) result.
    pub fn clean() -> Self {
        Self {
            issues: Vec::new(),
            fixed: 0,
        }
    }

    /// Count of issues by severity.
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Total number of issues found.
    pub fn total_issues(&self) -> usize {
        self.issues.len()
    }

    /// Number of fixable issues remaining.
    pub fn fixable_remaining(&self) -> usize {
        self.issues.iter().filter(|i| i.fixable).count()
    }

    /// Format as a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Purify: {} issues ({} errors, {} warnings), {} auto-fixed",
            self.total_issues(),
            self.count_by_severity(IssueSeverity::Error),
            self.count_by_severity(IssueSeverity::Warning),
            self.fixed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_from_str() {
        assert_eq!(Shell::from_str_loose("bash"), Some(Shell::Bash));
        assert_eq!(Shell::from_str_loose("ZSH"), Some(Shell::Zsh));
        assert_eq!(Shell::from_str_loose("DASH"), Some(Shell::Dash));
        assert_eq!(Shell::from_str_loose("fish"), Some(Shell::Fish));
        assert_eq!(Shell::from_str_loose("busybox"), Some(Shell::BusyBox));
        assert_eq!(Shell::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_shell_shebang() {
        assert_eq!(Shell::Bash.shebang(), "#!/usr/bin/env bash");
        assert_eq!(Shell::BusyBox.shebang(), "#!/bin/sh");
    }

    #[test]
    fn test_shell_pipefail_support() {
        assert!(Shell::Bash.supports_pipefail());
        assert!(Shell::Zsh.supports_pipefail());
        assert!(!Shell::Dash.supports_pipefail());
        assert!(!Shell::Fish.supports_pipefail());
    }

    #[test]
    fn test_shell_target_default() {
        let target = ShellTarget::default();
        assert_eq!(target.shell, Shell::Bash);
        assert!(target.strict);
        assert!(target.shellcheck);
    }

    #[test]
    fn test_shell_target_preamble_bash() {
        let target = ShellTarget::default();
        let preamble = target.preamble();
        assert!(preamble.contains("#!/usr/bin/env bash"));
        assert!(preamble.contains("set -euo pipefail"));
        assert!(preamble.contains("# shellcheck shell=bash"));
    }

    #[test]
    fn test_shell_target_preamble_dash() {
        let target = ShellTarget::for_shell(Shell::Dash);
        let preamble = target.preamble();
        assert!(preamble.contains("#!/usr/bin/env dash"));
        assert!(preamble.contains("set -eu"));
        assert!(!preamble.contains("pipefail"));
    }

    #[test]
    fn test_purify_result_clean() {
        let result = PurifyResult::clean();
        assert_eq!(result.total_issues(), 0);
        assert_eq!(result.fixed, 0);
        assert_eq!(result.fixable_remaining(), 0);
    }

    #[test]
    fn test_purify_result_with_issues() {
        let result = PurifyResult {
            issues: vec![
                PurifyIssue {
                    line: 5,
                    column: 1,
                    severity: IssueSeverity::Error,
                    code: "SC2086".to_string(),
                    message: "Double quote to prevent globbing".to_string(),
                    fixable: true,
                },
                PurifyIssue {
                    line: 12,
                    column: 3,
                    severity: IssueSeverity::Warning,
                    code: "SC2034".to_string(),
                    message: "Variable appears unused".to_string(),
                    fixable: false,
                },
                PurifyIssue {
                    line: 20,
                    column: 1,
                    severity: IssueSeverity::Style,
                    code: "SC2148".to_string(),
                    message: "Tips depend on target shell".to_string(),
                    fixable: true,
                },
            ],
            fixed: 0,
        };
        assert_eq!(result.total_issues(), 3);
        assert_eq!(result.count_by_severity(IssueSeverity::Error), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Warning), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Style), 1);
        assert_eq!(result.fixable_remaining(), 2);
        assert!(result.summary().contains("3 issues"));
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Info < IssueSeverity::Style);
        assert!(IssueSeverity::Style < IssueSeverity::Warning);
        assert!(IssueSeverity::Warning < IssueSeverity::Error);
    }

    #[test]
    fn test_shell_display() {
        assert_eq!(format!("{}", Shell::Bash), "bash");
        assert_eq!(format!("{}", Shell::Zsh), "zsh");
        assert_eq!(format!("{}", Shell::Fish), "fish");
    }
}
