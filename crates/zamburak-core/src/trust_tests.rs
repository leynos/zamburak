//! Unit tests for integrity, confidentiality, and authority label types.

use crate::AuthorityCapability;

use super::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};

// ---------------------------------------------------------------------------
// IntegrityLabel join
// ---------------------------------------------------------------------------

#[test]
fn integrity_join_untrusted_dominates() {
    assert_eq!(
        IntegrityLabel::Verified.join(IntegrityLabel::Untrusted),
        IntegrityLabel::Untrusted,
    );
    assert_eq!(
        IntegrityLabel::Trusted.join(IntegrityLabel::Untrusted),
        IntegrityLabel::Untrusted,
    );
    assert_eq!(
        IntegrityLabel::Untrusted.join(IntegrityLabel::Untrusted),
        IntegrityLabel::Untrusted,
    );
}

#[test]
fn integrity_join_trusted_and_verified() {
    assert_eq!(
        IntegrityLabel::Verified.join(IntegrityLabel::Trusted),
        IntegrityLabel::Trusted,
    );
    assert_eq!(
        IntegrityLabel::Trusted.join(IntegrityLabel::Verified),
        IntegrityLabel::Trusted,
    );
}

#[test]
fn integrity_join_same_returns_self() {
    assert_eq!(
        IntegrityLabel::Verified.join(IntegrityLabel::Verified),
        IntegrityLabel::Verified,
    );
    assert_eq!(
        IntegrityLabel::Trusted.join(IntegrityLabel::Trusted),
        IntegrityLabel::Trusted,
    );
}

#[test]
fn integrity_join_is_commutative() {
    let labels = [
        IntegrityLabel::Untrusted,
        IntegrityLabel::Trusted,
        IntegrityLabel::Verified,
    ];
    for a in &labels {
        for b in &labels {
            assert_eq!(
                a.join(*b),
                b.join(*a),
                "commutativity failed for {a:?}, {b:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// DataLabels join
// ---------------------------------------------------------------------------

#[test]
fn data_labels_join_is_union() {
    let a = DataLabels::from_iter([DataLabel::Pii]);
    let b = DataLabels::from_iter([DataLabel::AuthSecret]);
    let joined = a.join(&b);

    assert!(joined.contains(&DataLabel::Pii));
    assert!(joined.contains(&DataLabel::AuthSecret));
    assert_eq!(joined.len(), 2);
}

#[test]
fn data_labels_join_is_commutative() {
    let a = DataLabels::from_iter([DataLabel::Pii, DataLabel::PaymentInstrument]);
    let b = DataLabels::from_iter([DataLabel::AuthSecret]);

    assert_eq!(a.join(&b), b.join(&a));
}

#[test]
fn data_labels_join_is_idempotent() {
    let a = DataLabels::from_iter([DataLabel::Pii, DataLabel::AuthSecret]);
    assert_eq!(a.join(&a), a);
}

#[test]
fn data_labels_all_contains_every_variant() {
    let all = DataLabels::all();
    for variant in DataLabel::all_variants() {
        assert!(all.contains(&variant), "all() missing {variant:?}");
    }
    assert_eq!(all.len(), 5);
}

#[test]
fn data_labels_empty_is_identity_for_join() {
    let empty = DataLabels::new();
    let non_empty = DataLabels::from_iter([DataLabel::Pii]);

    assert_eq!(non_empty.join(&empty), non_empty);
    assert_eq!(empty.join(&non_empty), non_empty);
}

#[test]
fn data_labels_subset_check() {
    let small = DataLabels::from_iter([DataLabel::Pii]);
    let large = DataLabels::from_iter([DataLabel::Pii, DataLabel::AuthSecret]);

    assert!(small.is_subset_of(&large));
    assert!(!large.is_subset_of(&small));
}

// ---------------------------------------------------------------------------
// AuthoritySet join
// ---------------------------------------------------------------------------

fn cap(name: &str) -> AuthorityCapability {
    AuthorityCapability::try_from(name).expect("invalid capability name in test")
}

#[test]
fn authority_set_join_is_intersection() {
    let email = cap("EmailSendCap");
    let calendar = cap("CalendarWriteCap");

    let a = AuthoritySet::from_iter([email.clone(), calendar.clone()]);
    let b = AuthoritySet::from_iter([email.clone()]);
    let joined = a.join(&b);

    assert!(joined.contains(&email));
    assert!(!joined.contains(&calendar));
}

#[test]
fn authority_set_join_is_commutative() {
    let a = AuthoritySet::from_iter([cap("A"), cap("B")]);
    let b = AuthoritySet::from_iter([cap("B"), cap("C")]);

    assert_eq!(a.join(&b), b.join(&a));
}

#[test]
fn authority_set_join_with_empty_yields_empty() {
    let non_empty = AuthoritySet::from_iter([cap("A")]);
    let empty = AuthoritySet::new();

    assert!(non_empty.join(&empty).is_empty());
    assert!(empty.join(&non_empty).is_empty());
}

#[test]
fn authority_set_insert_and_contains() {
    let mut set = AuthoritySet::new();
    let capability = cap("TestCap");

    assert!(!set.contains(&capability));
    set.insert(capability.clone());
    assert!(set.contains(&capability));
}
