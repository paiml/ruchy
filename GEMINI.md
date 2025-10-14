## Gemini Agent Instructions for the Ruchy Project

This document provides instructions for the Gemini agent working on the Ruchy programming language project. The instructions are derived from the project's `roadmap.yaml` and reflect its development philosophy.

### Core Principles

*   **All work must be tied to a ticket:** Before starting any work, identify the corresponding ticket in the `roadmap.yaml`. If a ticket doesn't exist, one should be created.
*   **Extreme Test-Driven Development (TDD):** Follow the RED-GREEN-REFACTOR cycle for all changes.
    1.  **RED:** Write a failing test that reproduces the bug or defines the new feature.
    2.  **GREEN:** Write the simplest code to make the test pass.
    3.  **REFACTOR:** Improve the code quality while keeping the tests passing.
*   **Stop the Line Policy:** If any defect is found, stop all other work and fix it immediately. This includes any test failures, build errors, or quality gate violations.
*   **Atomic Commits:** Each commit should correspond to a single ticket and be a self-contained, atomic change.

### Quality Gates

All code changes must meet the following quality gates before being committed:

*   **Maximum Cyclomatic Complexity:** 10
*   **Minimum Test Coverage:** 80% (aim for 95%+)
*   **Minimum Mutation Score:** 75% (aim for 80%+)
*   **Zero Self-Admitted Technical Debt (SATD):** No new SATD should be introduced.

### Testing and Verification

*   **Run tests:** Use `cargo test` and `cargo nextest` to run the test suite.
*   **Property-based testing:** Use `proptest` for property-based testing.
*   **Mutation testing:** Use `cargo-mutants` to check the quality of the tests.
*   **Code coverage:** Use `cargo-llvm-cov` to measure test coverage.
*   **Linting:** Use `clippy` to check for code quality issues.
*   **Quality Gates Check:** Use `pmat` to verify quality gates.

### Current Sprints

The project is currently focused on the following sprints:

*   **Runtime Implementation:** Implementing runtime execution for features that currently only parse (structs, classes, actors, async/await).
*   **Ruchy-Book Compatibility:** Reaching 100% compatibility with the `ruchy-book`.

By following these instructions, the Gemini agent can contribute effectively to the Ruchy project while adhering to its high-quality standards.
