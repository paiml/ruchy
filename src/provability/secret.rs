//! Information-flow types (§14.10.1, SECRET-XXX).
//!
//! `Secret<T>` carries a value that came from a credential/key/password; the
//! static analyzer (future sprint) will forbid branching, matching, logging,
//! or indexing on it. `Public<T>` marks values that are safe to leak.
//!
//! Today these are newtype wrappers; in a future sprint a dedicated lint
//! pass will enforce the compile-time rules from §14.10.1.
//!
//! Reference: HACL*/F* `Lib.IntTypes.secret` pattern.

use std::marker::PhantomData;

/// A value classified as secret (credential / key / password / PII).
///
/// The runtime type is a pure wrapper today; the compile-time enforcement
/// (no branching, no matching, no logging, no secret-indexed access) is
/// delivered by the SECRET-002 lint pass in a future sprint.
///
/// # Example
///
/// ```
/// use ruchy::provability::{Secret, declassify};
/// let key: Secret<u64> = Secret::new(0xDEADBEEF);
/// // Future sprint: `if key == ... { ... }` will be a compile error.
/// let leaked: u64 = declassify(key, "test-vector");
/// assert_eq!(leaked, 0xDEADBEEF);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Secret<T> {
    value: T,
    _marker: PhantomData<()>,
}

impl<T> Secret<T> {
    /// Wrap a raw value as a `Secret<T>`.
    ///
    /// Callers should prefer constructing secrets at the I/O boundary
    /// (e.g. reading a key from a file) and never synthesize secrets from
    /// hard-coded literals in release code.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

/// A value explicitly classified as public.
///
/// `Public<T>` is the dual of `Secret<T>` — it's the type of values that
/// are safe to log, branch on, index arrays with, and return to callers.
/// Every `declassify` result is `Public<T>` by definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Public<T> {
    value: T,
}

impl<T> Public<T> {
    /// Wrap a raw value as `Public<T>`.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Extract the underlying value. Safe because the value is public.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Borrow the underlying value. Safe because the value is public.
    pub const fn get(&self) -> &T {
        &self.value
    }
}

/// The single canonical escape hatch from `Secret<T>` to `Public<T>`.
///
/// Per §14.10.1 the call site must be marked `#[contract_exempt]` with a
/// reason string; that enforcement is delivered by SECRET-002. For now
/// `declassify` is an ordinary function that takes a `reason: &'static str`
/// to force authors to justify the declassification in code.
///
/// # Example
///
/// ```
/// use ruchy::provability::{Secret, declassify};
/// let s = Secret::new(42u32);
/// let p = declassify(s, "unit-test: no leak of real credentials");
/// assert_eq!(p, 42);
/// ```
#[must_use]
pub fn declassify<T>(secret: Secret<T>, _reason: &'static str) -> T {
    secret.value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_construct_and_declassify() {
        let s = Secret::new(0xCAFEu32);
        let p = declassify(s, "test");
        assert_eq!(p, 0xCAFE);
    }

    #[test]
    fn test_public_into_inner() {
        let p = Public::new(7i32);
        assert_eq!(*p.get(), 7);
        assert_eq!(p.into_inner(), 7);
    }

    #[test]
    fn test_secret_is_copy() {
        // Being Copy is intentional: we do NOT want `Secret<T>` to be
        // linearly consumed — that would conflict with capability types,
        // which ARE linear. Information-flow is enforced at the lint
        // layer, not via ownership.
        let s = Secret::new(1u8);
        let s2 = s;
        let _ = declassify(s, "copy-a");
        let _ = declassify(s2, "copy-b");
    }

    #[test]
    fn test_public_equality() {
        assert_eq!(Public::new(5), Public::new(5));
        assert_ne!(Public::new(5), Public::new(6));
    }
}
