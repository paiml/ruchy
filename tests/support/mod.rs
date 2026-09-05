//! Shared helpers for CLI contract tests.
//!
//! Each `tests/*.rs` file is its own crate; a test file pulls this in with
//! `mod support;` so every helper has exactly one definition.

/// The CLI must have accepted its arguments.
///
/// clap reports a usage error (unknown flag, bad value) with exit code 2; any other
/// outcome means the arguments were parsed and the command ran (or was killed by a
/// timeout, in which case there is no exit code at all).
pub fn assert_args_accepted(assert: assert_cmd::assert::Assert) {
    let code = assert.get_output().status.code();
    assert_ne!(
        code,
        Some(2),
        "CLI rejected its arguments (clap usage error): {:?}",
        assert.get_output()
    );
}
