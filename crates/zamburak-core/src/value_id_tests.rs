//! Unit tests for `ValueId` identity and display semantics.

use super::ValueId;

#[test]
fn value_id_creation_and_equality() {
    let id_a = ValueId::new(1);
    let id_b = ValueId::new(1);
    let id_c = ValueId::new(2);

    assert_eq!(id_a, id_b);
    assert_ne!(id_a, id_c);
}

#[test]
fn value_id_ordering() {
    let small = ValueId::new(1);
    let large = ValueId::new(100);

    assert!(small < large);
    assert!(large > small);
}

#[test]
fn value_id_display() {
    let id = ValueId::new(42);
    let displayed = format!("{id}");

    // newt-hype derives Display forwarding to the inner type.
    assert_eq!(displayed, "42");
}

#[test]
fn value_id_inner_access() {
    let id = ValueId::new(999);
    assert_eq!(id.inner(), &999);
}

#[test]
fn value_id_clone_and_copy() {
    let id = ValueId::new(7);
    #[expect(
        clippy::clone_on_copy,
        reason = "Explicitly verifying Clone behaviour for ValueId"
    )]
    let cloned = id.clone();
    let copied = id;

    assert_eq!(id, cloned);
    assert_eq!(id, copied);
}
