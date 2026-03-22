//! Property tests for IFC trust types: `IntegrityLabel`, `DataLabels`,
//! `AuthoritySet`.

use proptest::prelude::*;
use rstest::rstest;

use super::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};
use crate::AuthorityCapability;

// ---------------------------------------------------------------------------
// Arbitrary implementations
// ---------------------------------------------------------------------------

fn arb_integrity_label() -> impl Strategy<Value = IntegrityLabel> {
    prop_oneof![
        Just(IntegrityLabel::Untrusted),
        Just(IntegrityLabel::Trusted),
        Just(IntegrityLabel::Verified),
    ]
}

fn arb_data_label() -> impl Strategy<Value = DataLabel> {
    prop_oneof![
        Just(DataLabel::Pii),
        Just(DataLabel::AuthSecret),
        Just(DataLabel::PrivateEmailBody),
        Just(DataLabel::PaymentInstrument),
        Just(DataLabel::InternalPolicyNote),
    ]
}

fn arb_data_labels() -> impl Strategy<Value = DataLabels> {
    proptest::collection::btree_set(arb_data_label(), 0..=5).prop_map(|set| {
        let labels: Vec<DataLabel> = set.into_iter().collect();
        DataLabels::from_iter(labels)
    })
}

/// Small alphabet for authority capability names.
const CAP_NAMES: &[&str] = &["A", "B", "C", "D", "E", "F", "G", "H"];

fn cap_from_index(i: usize) -> Option<AuthorityCapability> {
    let name = CAP_NAMES.get(i)?;
    AuthorityCapability::try_from(*name).ok()
}

fn arb_authority_set() -> impl Strategy<Value = AuthoritySet> {
    proptest::collection::btree_set(0..CAP_NAMES.len(), 0..=CAP_NAMES.len()).prop_map(|indices| {
        let mut set = AuthoritySet::new();
        for cap in indices.into_iter().filter_map(cap_from_index) {
            set.insert(cap);
        }
        set
    })
}

// ---------------------------------------------------------------------------
// IntegrityLabel lattice laws (proptest)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn integrity_join_commutativity(
        a in arb_integrity_label(),
        b in arb_integrity_label(),
    ) {
        prop_assert_eq!(a.join(b), b.join(a));
    }

    #[test]
    fn integrity_join_associativity(
        a in arb_integrity_label(),
        b in arb_integrity_label(),
        c in arb_integrity_label(),
    ) {
        prop_assert_eq!(a.join(b).join(c), a.join(b.join(c)));
    }

    #[test]
    fn integrity_join_idempotency(a in arb_integrity_label()) {
        prop_assert_eq!(a.join(a), a);
    }

    #[test]
    fn integrity_join_monotonicity(
        a in arb_integrity_label(),
        b in arb_integrity_label(),
        c in arb_integrity_label(),
    ) {
        if a <= b {
            prop_assert!(a.join(c) <= b.join(c));
        }
    }
}

// ---------------------------------------------------------------------------
// IntegrityLabel exhaustive bounded tests (rstest)
// ---------------------------------------------------------------------------

const ALL_LABELS: [IntegrityLabel; 3] = [
    IntegrityLabel::Untrusted,
    IntegrityLabel::Trusted,
    IntegrityLabel::Verified,
];

#[rstest]
#[case(IntegrityLabel::Untrusted, IntegrityLabel::Untrusted)]
#[case(IntegrityLabel::Untrusted, IntegrityLabel::Trusted)]
#[case(IntegrityLabel::Untrusted, IntegrityLabel::Verified)]
#[case(IntegrityLabel::Trusted, IntegrityLabel::Untrusted)]
#[case(IntegrityLabel::Trusted, IntegrityLabel::Trusted)]
#[case(IntegrityLabel::Trusted, IntegrityLabel::Verified)]
#[case(IntegrityLabel::Verified, IntegrityLabel::Untrusted)]
#[case(IntegrityLabel::Verified, IntegrityLabel::Trusted)]
#[case(IntegrityLabel::Verified, IntegrityLabel::Verified)]
fn exhaustive_join_commutativity(#[case] a: IntegrityLabel, #[case] b: IntegrityLabel) {
    assert_eq!(a.join(b), b.join(a));
}

#[test]
fn exhaustive_join_associativity_all_27_triples() {
    for a in &ALL_LABELS {
        for b in &ALL_LABELS {
            for c in &ALL_LABELS {
                assert_eq!(
                    a.join(*b).join(*c),
                    a.join(b.join(*c)),
                    "associativity failed for {a:?}, {b:?}, {c:?}",
                );
            }
        }
    }
}

fn assert_join_monotone(a: &IntegrityLabel, b: &IntegrityLabel, c: &IntegrityLabel) {
    if a <= b {
        assert!(
            a.join(*c) <= b.join(*c),
            "monotonicity failed for {a:?}, {b:?}, {c:?}",
        );
    }
}

#[test]
fn exhaustive_join_monotonicity_all_27_triples() {
    for a in &ALL_LABELS {
        for b in &ALL_LABELS {
            for c in &ALL_LABELS {
                assert_join_monotone(a, b, c);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DataLabels lattice laws
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn data_labels_join_commutativity(
        a in arb_data_labels(),
        b in arb_data_labels(),
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn data_labels_join_associativity(
        a in arb_data_labels(),
        b in arb_data_labels(),
        c in arb_data_labels(),
    ) {
        prop_assert_eq!(a.join(&b).join(&c), a.join(&b.join(&c)));
    }

    #[test]
    fn data_labels_join_monotonicity(
        a in arb_data_labels(),
        b in arb_data_labels(),
    ) {
        // Union is monotone: a ⊆ a.join(b)
        prop_assert!(a.is_subset_of(&a.join(&b)));
    }

    #[test]
    fn data_labels_join_idempotency(a in arb_data_labels()) {
        prop_assert_eq!(a.join(&a), a);
    }
}

// ---------------------------------------------------------------------------
// AuthoritySet lattice laws
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn authority_set_join_commutativity(
        a in arb_authority_set(),
        b in arb_authority_set(),
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn authority_set_join_associativity(
        a in arb_authority_set(),
        b in arb_authority_set(),
        c in arb_authority_set(),
    ) {
        prop_assert_eq!(a.join(&b).join(&c), a.join(&b.join(&c)));
    }

    #[test]
    fn authority_set_join_monotonicity_narrows(
        a in arb_authority_set(),
        b in arb_authority_set(),
    ) {
        // Intersection narrows: a.join(b) ⊆ a
        let joined = a.join(&b);
        for cap in joined.iter() {
            prop_assert!(a.contains(cap));
        }
    }
}
