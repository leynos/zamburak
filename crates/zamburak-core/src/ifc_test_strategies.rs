//! Shared proptest strategies for IFC types.
//!
//! Provides reusable `proptest` generators for `IntegrityLabel`,
//! `DataLabel`, `DataLabels`, and `AuthoritySet`. These strategies are
//! used across property tests in `trust_proptests.rs`,
//! `summary_proptests.rs`, and `propagation_proptests.rs`.

use proptest::prelude::*;

use crate::AuthorityCapability;
use crate::trust::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};

/// Generate an arbitrary `IntegrityLabel`.
pub fn arb_integrity_label() -> impl Strategy<Value = IntegrityLabel> {
    prop_oneof![
        Just(IntegrityLabel::Untrusted),
        Just(IntegrityLabel::Trusted),
        Just(IntegrityLabel::Verified),
    ]
}

/// Generate an arbitrary `DataLabel`.
pub fn arb_data_label() -> impl Strategy<Value = DataLabel> {
    prop_oneof![
        Just(DataLabel::Pii),
        Just(DataLabel::AuthSecret),
        Just(DataLabel::PrivateEmailBody),
        Just(DataLabel::PaymentInstrument),
        Just(DataLabel::InternalPolicyNote),
    ]
}

/// Generate an arbitrary `DataLabels` set.
pub fn arb_data_labels() -> impl Strategy<Value = DataLabels> {
    proptest::collection::btree_set(arb_data_label(), 0..=5).prop_map(|set| {
        let labels: Vec<DataLabel> = set.into_iter().collect();
        DataLabels::from_iter(labels)
    })
}

/// Small alphabet for authority capability names in tests.
pub const CAP_NAMES: &[&str] = &["A", "B", "C", "D", "E"];

/// Convert an index into a capability.
///
/// # Errors
///
/// Returns a description of the problem if the index is out of bounds for
/// `CAP_NAMES` or if the capability name fails to parse. The strategies
/// reject such draws, and the `cap_names_all_parse` test guards against
/// drift in `CAP_NAMES`, so a broken alphabet fails the suite instead of
/// silently weakening the generator.
pub fn cap_from_index(i: usize) -> Result<AuthorityCapability, String> {
    let name = CAP_NAMES
        .get(i)
        .ok_or_else(|| format!("cap_from_index: index {i} out of bounds for CAP_NAMES"))?;
    AuthorityCapability::try_from(*name).map_err(|_| {
        format!("cap_from_index: failed to parse CAP_NAMES[{i}] = '{name}' as AuthorityCapability")
    })
}

/// Generate an arbitrary `AuthoritySet`, including the full (universe)
/// set with low probability.
pub fn arb_authority_set() -> impl Strategy<Value = AuthoritySet> {
    prop_oneof![
        9 => proptest::collection::btree_set(
            0..CAP_NAMES.len(), 0..=CAP_NAMES.len(),
        ).prop_filter_map("capability index must resolve", |indices| {
            let mut set = AuthoritySet::new();
            for i in indices {
                set.insert(cap_from_index(i).ok()?);
            }
            Some(set)
        }),
        1 => Just(AuthoritySet::full()),
    ]
}

/// Generate an arbitrary `DependencySummary`.
pub fn arb_summary() -> impl Strategy<Value = crate::summary::DependencySummary> {
    use crate::summary::DependencySummary;
    (
        arb_integrity_label(),
        arb_data_labels(),
        arb_authority_set(),
        0..100u32,
        any::<bool>(),
    )
        .prop_map(
            |(integrity, confidentiality, authority, count, trunc)| DependencySummary {
                integrity_join: integrity,
                confidentiality_join: confidentiality,
                authority_join: authority,
                origin_count: count,
                truncated: trunc,
            },
        )
}

#[cfg(test)]
mod tests {
    //! Guards the capability alphabet used by the strategies.

    use super::{CAP_NAMES, cap_from_index};

    #[test]
    fn cap_names_all_parse() {
        for i in 0..CAP_NAMES.len() {
            assert!(
                cap_from_index(i).is_ok(),
                "CAP_NAMES[{i}] must parse as an AuthorityCapability",
            );
        }
    }

    #[test]
    fn cap_from_index_rejects_out_of_bounds() {
        assert!(cap_from_index(CAP_NAMES.len()).is_err());
    }
}
