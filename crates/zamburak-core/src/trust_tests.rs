//! Unit tests for integrity, confidentiality, and authority label types.

use rstest::rstest;

use std::str::FromStr;

use crate::AuthorityCapability;

use super::{AuthoritySet, DataLabel, DataLabels, IntegrityLabel};

// ---------------------------------------------------------------------------
// IntegrityLabel join
// ---------------------------------------------------------------------------

#[rstest]
#[case(
    IntegrityLabel::Verified,
    IntegrityLabel::Verified,
    IntegrityLabel::Verified
)]
#[case(
    IntegrityLabel::Trusted,
    IntegrityLabel::Trusted,
    IntegrityLabel::Trusted
)]
#[case(
    IntegrityLabel::Untrusted,
    IntegrityLabel::Untrusted,
    IntegrityLabel::Untrusted
)]
#[case(
    IntegrityLabel::Verified,
    IntegrityLabel::Trusted,
    IntegrityLabel::Trusted
)]
#[case(
    IntegrityLabel::Trusted,
    IntegrityLabel::Verified,
    IntegrityLabel::Trusted
)]
#[case(
    IntegrityLabel::Verified,
    IntegrityLabel::Untrusted,
    IntegrityLabel::Untrusted
)]
#[case(
    IntegrityLabel::Trusted,
    IntegrityLabel::Untrusted,
    IntegrityLabel::Untrusted
)]
#[case(
    IntegrityLabel::Untrusted,
    IntegrityLabel::Verified,
    IntegrityLabel::Untrusted
)]
#[case(
    IntegrityLabel::Untrusted,
    IntegrityLabel::Trusted,
    IntegrityLabel::Untrusted
)]
fn integrity_join_cases(
    #[case] a: IntegrityLabel,
    #[case] b: IntegrityLabel,
    #[case] expected: IntegrityLabel,
) {
    assert_eq!(a.join(b), expected);
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

#[test]
fn data_labels_iter_visits_every_member() {
    let labels = DataLabels::from_iter([DataLabel::Pii, DataLabel::AuthSecret]);

    let mut seen: Vec<DataLabel> = labels.iter().copied().collect();
    seen.sort_by_key(|label| format!("{label:?}"));

    let mut expected = vec![DataLabel::Pii, DataLabel::AuthSecret];
    expected.sort_by_key(|label| format!("{label:?}"));

    assert_eq!(seen, expected);
}

// ---------------------------------------------------------------------------
// Parse error Display
// ---------------------------------------------------------------------------

#[test]
fn parse_integrity_label_error_display_reports_the_offending_input() {
    let error = IntegrityLabel::from_str("not-a-real-label").expect_err("must fail closed");

    assert_eq!(
        error.to_string(),
        "unrecognised integrity label 'not-a-real-label'"
    );
}

#[test]
fn parse_data_label_error_display_reports_the_offending_input() {
    let error = DataLabel::from_str("not-a-real-label").expect_err("must fail closed");

    assert_eq!(
        error.to_string(),
        "unrecognised data label 'not-a-real-label'"
    );
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
fn authority_set_full_is_identity_for_join() {
    let non_empty = AuthoritySet::from_iter([cap("A"), cap("B")]);
    let full = AuthoritySet::full();

    assert_eq!(non_empty.join(&full), non_empty);
    assert_eq!(full.join(&non_empty), non_empty);
}

#[test]
fn authority_set_full_contains_any_capability() {
    let full = AuthoritySet::full();
    assert!(full.is_full());
    assert!(!full.is_empty());
    assert!(full.contains(&cap("Anything")));
}

#[test]
fn authority_set_full_join_full_is_full() {
    let a = AuthoritySet::full();
    let b = AuthoritySet::full();
    let joined = a.join(&b);
    assert!(joined.is_full());
}

#[test]
fn authority_set_insert_and_contains() {
    let mut set = AuthoritySet::new();
    let capability = cap("TestCap");

    assert!(!set.contains(&capability));
    set.insert(capability.clone());
    assert!(set.contains(&capability));
}
