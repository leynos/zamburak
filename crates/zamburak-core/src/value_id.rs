//! Stable, opaque identifier for runtime values in the IFC dependency graph.
//!
//! `ValueId` wraps a `u64`, mirroring the backing type of `RuntimeValueId`
//! in `full-monty`. Values are host-only identifiers: they are not
//! guest-writable and carry no policy meaning on their own.

use std::fmt;

/// Opaque identifier for a runtime value in the IFC dependency graph.
///
/// `ValueId` is a transparent wrapper around `u64`, providing type safety
/// and preventing accidental confusion with other integer quantities.
///
/// # Examples
///
/// ```
/// use zamburak_core::ValueId;
///
/// let id = ValueId::new(42);
/// assert_eq!(id.inner(), &42);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct ValueId(u64);

impl ValueId {
    /// Create a new `ValueId` from a raw `u64`.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Return a reference to the inner `u64`.
    #[must_use]
    pub fn inner(&self) -> &u64 {
        &self.0
    }
}

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "value_id_tests.rs"]
mod tests;
