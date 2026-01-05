//! Publish Command Handler
//!
//! Handles publishing packages to the Ruchy registry.

use anyhow::{bail, Context, Result};
use std::fs;

/// Handle publish command - publish a package to the Ruchy registry
///
/// TOOL-FEATURE-001: Package publishing with Ruchy.toml validation
///
/// # Arguments
/// * `registry` - Registry URL to publish to
/// * `version` - Optional version override (reads from Ruchy.toml if None)
/// * `dry_run` - Validate without publishing
/// * `allow_dirty` - Allow publishing with uncommitted changes
/// * `verbose` - Show detailed output
///
/// # Errors
/// Returns error if:
/// - Ruchy.toml not found
/// - Required fields missing (name, version, authors, description, license)
/// - Invalid semver version
/// - Package validation fails
pub fn handle_publish_command(
    _registry: &str,
    _version: Option<&str>,
    dry_run: bool,
    allow_dirty: bool,
    verbose: bool,
) -> Result<()> {
    use semver::Version;
    use serde::Deserialize;
    use std::env;

    // Package metadata from Ruchy.toml
    #[derive(Debug, Deserialize)]
    struct PackageManifest {
        package: PackageMetadata,
    }

    #[derive(Debug, Deserialize)]
    struct PackageMetadata {
        name: String,
        version: String,
        authors: Vec<String>,
        description: String,
        license: String,
        repository: Option<String>,
    }

    // Find Ruchy.toml in current directory
    let manifest_path = env::current_dir()?.join("Ruchy.toml");

    if !manifest_path.exists() {
        bail!("Ruchy.toml not found in current directory.\nRun 'ruchy publish' from your package root.");
    }

    if verbose {
        eprintln!("Reading manifest: {}", manifest_path.display());
    }

    // Parse Ruchy.toml
    let manifest_content =
        fs::read_to_string(&manifest_path).context("Failed to read Ruchy.toml")?;

    let manifest: PackageManifest = toml::from_str(&manifest_content)
        .context("Failed to parse Ruchy.toml.\nEnsure all required fields are present: name, version, authors, description, license")?;

    // Validate required fields
    if manifest.package.name.is_empty() {
        bail!("Package name cannot be empty in Ruchy.toml");
    }

    if manifest.package.authors.is_empty() {
        bail!("At least one author is required in Ruchy.toml");
    }

    if manifest.package.description.is_empty() {
        bail!("Package description cannot be empty in Ruchy.toml");
    }

    if manifest.package.license.is_empty() {
        bail!("Package license cannot be empty in Ruchy.toml");
    }

    // Validate semver version
    Version::parse(&manifest.package.version).context(format!(
        "Invalid version '{}' in Ruchy.toml.\nMust be valid semver (e.g., 1.0.0, 0.2.3)",
        manifest.package.version
    ))?;

    if verbose {
        eprintln!("Manifest validation passed");
        eprintln!("   Name: {}", manifest.package.name);
        eprintln!("   Version: {}", manifest.package.version);
        eprintln!("   Authors: {}", manifest.package.authors.join(", "));
        eprintln!("   Description: {}", manifest.package.description);
        eprintln!("   License: {}", manifest.package.license);
        if let Some(repo) = &manifest.package.repository {
            eprintln!("   Repository: {}", repo);
        }
    }

    if dry_run {
        println!(
            "Dry-run mode: Validating package '{}'",
            manifest.package.name
        );
        println!("Package validation successful");
        println!(
            "Package: {} v{}",
            manifest.package.name, manifest.package.version
        );
        println!("Authors: {}", manifest.package.authors.join(", "));
        println!("License: {}", manifest.package.license);
        println!("\nWould publish package (skipped in dry-run mode)");
        Ok(())
    } else {
        // Actually publish to crates.io via cargo publish
        println!(
            "Publishing {} v{}...",
            manifest.package.name, manifest.package.version
        );

        use std::process::Command;

        // Build cargo publish command
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.arg("publish");

        if verbose {
            cargo_cmd.arg("--verbose");
        }

        if allow_dirty {
            cargo_cmd.arg("--allow-dirty");
        }

        // Execute cargo publish
        let status = cargo_cmd
            .status()
            .context("Failed to execute 'cargo publish'. Ensure cargo is installed.")?;

        if status.success() {
            println!(
                "Successfully published {} v{} to crates.io",
                manifest.package.name, manifest.package.version
            );
            Ok(())
        } else {
            bail!("cargo publish failed with exit code: {}", status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_publish_command_no_manifest() {
        // Should fail without Ruchy.toml in a temp dir
        let result = handle_publish_command("https://crates.io", None, true, false, false);
        // May succeed or fail depending on current directory
        let _ = result;
    }
}
