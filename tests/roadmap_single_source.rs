use std::fs;
use std::process::Command;

/// Test PMAT-098: Exactly one roadmap.yaml exists outside docs/archive/
/// This enforces the single source of truth policy for roadmap tracking.
#[test]
fn test_pmat_098_single_roadmap_file_enforced() {
    // Get all tracked files via git
    let output = Command::new("git")
        .arg("ls-files")
        .output()
        .expect("Failed to run 'git ls-files'");

    let git_files =
        String::from_utf8(output.stdout).expect("git ls-files output is not valid UTF-8");

    // Find all roadmap.yaml files (not .lock) outside docs/archive/
    let roadmap_files: Vec<&str> = git_files
        .lines()
        .filter(|path| path.ends_with("roadmap.yaml") && !path.contains("docs/archive/"))
        .collect();

    // Assert exactly one roadmap.yaml exists outside archive
    assert_eq!(
        roadmap_files.len(),
        1,
        "Expected exactly one roadmap.yaml outside docs/archive/, found: {:?}",
        roadmap_files
    );

    // Assert it is docs/roadmaps/roadmap.yaml
    let expected_path = "docs/roadmaps/roadmap.yaml";
    assert_eq!(
        roadmap_files[0], expected_path,
        "Expected the single roadmap to be at {}, but found at {}",
        expected_path, roadmap_files[0]
    );
}

/// Test PMAT-098: CLAUDE.md does not reference archived roadmap path
#[test]
fn test_pmat_098_claude_md_no_archived_roadmap_reference() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        !claude_md.contains("docs/execution/roadmap.yaml"),
        "CLAUDE.md should not contain reference to docs/execution/roadmap.yaml (archived path)"
    );
}

/// Test PMAT-098: CLAUDE.md references the single source of truth roadmap
#[test]
fn test_pmat_098_claude_md_references_single_sot() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        claude_md.contains("docs/roadmaps/roadmap.yaml"),
        "CLAUDE.md should contain reference to docs/roadmaps/roadmap.yaml (single source of truth)"
    );
}
