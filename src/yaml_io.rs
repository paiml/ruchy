//! SEC-2 (#259): the one YAML reader and writer ruchy uses.
//!
//! ruchy used `serde_yaml` 0.9 and `serde_yaml_ng` 0.10. Both parse through
//! `unsafe-libyaml`, which is unmaintained (RUSTSEC-2024-0449, no patched
//! release), and ruchy's normal and build edges carried it into every
//! consumer. The reader (`yaml_read.rs`) is `serde_yaml_ng`'s deserializer
//! ported onto serde-saphyr's parser; the writer (`yaml_emit.rs`) is libyaml's
//! emitter as `serde_yaml_ng` drove it. Every call site keeps its behaviour:
//! the differential test in `yaml_io_tests.rs` holds that claim against
//! `serde_yaml_ng` itself (a dev-only oracle) over every YAML file the repo
//! commits.
//!
//! There is no YAML `Value` type here on purpose: a typeless read goes into
//! [`serde_json::Value`], which every call site already knew.

use serde::de::DeserializeOwned;
use serde::Serialize;

pub use emit::WriteError;
pub use read::{Error, Untagged};

#[path = "yaml_emit.rs"]
mod emit;
#[path = "yaml_read.rs"]
mod read;

/// Read one YAML document as `T`, as `serde_yaml_ng::from_str` did.
///
/// # Errors
///
/// Returns [`Error`] when the text is not YAML or not `T`'s shape.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    read::from_str(text)
}

/// [`from_str`] with every enum read as a one-key map (or a string for a
/// unit variant), as `serde_yaml_ng::with::singleton_map_recursive` did.
///
/// # Errors
///
/// As [`from_str`].
pub fn from_str_singleton<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    read::from_str_singleton(text)
}

/// `Ok` when `text` holds no local (`!x`) tag and no repeated key: what
/// reading it into `serde_yaml_ng::Value` and searching for `Value::Tagged`
/// decided.
///
/// # Errors
///
/// Returns why not.
pub fn untagged(text: &str) -> Result<(), Untagged> {
    read::untagged(text)
}

/// Write `value` as one YAML document, byte for byte as
/// `serde_yaml_ng::to_string` wrote it (see `yaml_emit.rs`: serde-saphyr's
/// writer always double-quotes, so the writer is in-tree).
///
/// # Errors
///
/// Returns [`WriteError`] when `value` has no YAML form (bytes).
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String, WriteError> {
    emit::to_string(value)
}

/// The 1-based (line, column) of a read error, or `(0, 0)` when it has none.
#[must_use]
pub fn line_col(e: &Error) -> (usize, usize) {
    e.location().unwrap_or((0, 0))
}

#[cfg(test)]
#[path = "yaml_io_tests.rs"]
mod yaml_io_tests;
