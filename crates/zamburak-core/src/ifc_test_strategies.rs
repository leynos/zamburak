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

/// Convert an index into a capability name.
pub fn cap_from_index(i: usize) -> Option<AuthorityCapability> {
    let name = CAP_NAMES.get(i)?;
    AuthorityCapability::try_from(*name).ok()
}

/// Generate an arbitrary `AuthoritySet`.
pub fn arb_authority_set() -> impl Strategy<Value = AuthoritySet> {
    proptest::collection::btree_set(0..CAP_NAMES.len(), 0..=CAP_NAMES.len()).prop_map(|indices| {
        let mut set = AuthoritySet::new();
        for cap in indices.into_iter().filter_map(cap_from_index) {
            set.insert(cap);
        }
        set
    })
}
