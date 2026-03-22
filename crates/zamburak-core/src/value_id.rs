//! Stable, opaque identifier for runtime values in the IFC dependency graph.
//!
//! `ValueId` wraps a `u64`, mirroring the backing type of `RuntimeValueId`
//! in `full-monty`. Values are host-only identifiers: they are not
//! guest-writable and carry no policy meaning on their own.

#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "newt-hype macro expansion emits explicit Clone for Copy wrappers"
)]
mod value_id_newtype {
    //! Newtype wrapper for `ValueId` generated via `newt-hype`.

    use newt_hype::{base_newtype, newtype};

    base_newtype!(ValueIdBase);
    newtype!(ValueId, ValueIdBase, u64);
}

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
pub type ValueId = value_id_newtype::ValueId;

#[cfg(test)]
#[path = "value_id_tests.rs"]
mod tests;
