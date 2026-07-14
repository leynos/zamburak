//! Integrity, confidentiality, and authority label types for IFC.
//!
//! This module defines the lattice-structured label types used by the IFC
//! dependency graph. `IntegrityLabel` forms a three-level lattice where
//! `Untrusted < Trusted < Verified`. `DataLabels` tracks confidentiality
//! classifications as an additive set. `AuthoritySet` tracks capability
//! grants as a narrowing intersection.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::authority::AuthorityCapability;

/// Integrity level assigned to a runtime value.
///
/// The ordering `Untrusted < Trusted < Verified` defines a lattice where
/// the join operation (`join`) returns the greatest lower bound (meet):
/// when a value depends on both `Trusted` and `Untrusted` sources, its
/// integrity is `Untrusted`.
///
/// # Examples
///
/// ```
/// use zamburak_core::IntegrityLabel;
///
/// let result = IntegrityLabel::Trusted.join(IntegrityLabel::Untrusted);
/// assert_eq!(result, IntegrityLabel::Untrusted);
/// ```
///
/// ## Parsing from strings
///
/// `FromStr` accepts both CamelCase (`Untrusted`) and canonical uppercase
/// (`UNTRUSTED`) spellings, so policy YAML files can use either convention.
///
/// ```
/// use zamburak_core::IntegrityLabel;
///
/// assert_eq!("Untrusted".parse::<IntegrityLabel>().unwrap(), IntegrityLabel::Untrusted);
/// assert_eq!("TRUSTED".parse::<IntegrityLabel>().unwrap(), IntegrityLabel::Trusted);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IntegrityLabel {
    /// Value originates from an untrusted source.
    Untrusted,
    /// Value originates from a trusted source.
    Trusted,
    /// Value has been formally verified.
    Verified,
}

impl IntegrityLabel {
    /// Compute the IFC join (greatest lower bound) of two integrity labels.
    ///
    /// The result is the minimum of the two labels under the ordering
    /// `Untrusted < Trusted < Verified`. This reflects the principle that
    /// a derived value's integrity cannot exceed its weakest input.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        std::cmp::min(self, other)
    }
}

/// Error returned when an unrecognized integrity label string is parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseIntegrityLabelError {
    /// The unrecognized input string.
    pub label: String,
}

impl fmt::Display for ParseIntegrityLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognised integrity label '{}'", self.label)
    }
}

impl std::error::Error for ParseIntegrityLabelError {}

impl FromStr for IntegrityLabel {
    type Err = ParseIntegrityLabelError;

    /// Parse an integrity label from CamelCase or canonical uppercase.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Untrusted" | "UNTRUSTED" => Ok(Self::Untrusted),
            "Trusted" | "TRUSTED" => Ok(Self::Trusted),
            "Verified" | "VERIFIED" => Ok(Self::Verified),
            _ => Err(ParseIntegrityLabelError {
                label: s.to_owned(),
            }),
        }
    }
}

/// Confidentiality classification tag for a runtime value.
///
/// Each variant identifies a distinct class of sensitive data. Labels are
/// combined via set union in `DataLabels::join`: once a value acquires a
/// confidentiality tag, it cannot be removed by further computation.
///
/// ## Parsing from strings
///
/// `FromStr` accepts both CamelCase (`AuthSecret`) and canonical uppercase
/// (`AUTH_SECRET`) spellings, so policy YAML files can use either convention.
///
/// ```
/// use zamburak_core::DataLabel;
///
/// assert_eq!("AuthSecret".parse::<DataLabel>().unwrap(), DataLabel::AuthSecret);
/// assert_eq!("AUTH_SECRET".parse::<DataLabel>().unwrap(), DataLabel::AuthSecret);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DataLabel {
    /// Personally identifiable information.
    Pii,
    /// Authentication secrets (passwords, tokens, keys).
    AuthSecret,
    /// Private email body content.
    PrivateEmailBody,
    /// Payment instrument data (card numbers, account details).
    PaymentInstrument,
    /// Internal policy notes not for external disclosure.
    InternalPolicyNote,
}

impl DataLabel {
    /// Return an array of all `DataLabel` variants.
    #[must_use]
    pub const fn all_variants() -> [Self; 5] {
        [
            Self::Pii,
            Self::AuthSecret,
            Self::PrivateEmailBody,
            Self::PaymentInstrument,
            Self::InternalPolicyNote,
        ]
    }
}

/// Error returned when an unrecognized data label string is parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseDataLabelError {
    /// The unrecognized input string.
    pub label: String,
}

impl fmt::Display for ParseDataLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognised data label '{}'", self.label)
    }
}

impl std::error::Error for ParseDataLabelError {}

impl FromStr for DataLabel {
    type Err = ParseDataLabelError;

    /// Parse a confidentiality label from CamelCase or canonical uppercase.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pii" | "PII" => Ok(Self::Pii),
            "AuthSecret" | "AUTH_SECRET" => Ok(Self::AuthSecret),
            "PrivateEmailBody" | "PRIVATE_EMAIL_BODY" => Ok(Self::PrivateEmailBody),
            "PaymentInstrument" | "PAYMENT_INSTRUMENT" => Ok(Self::PaymentInstrument),
            "InternalPolicyNote" | "INTERNAL_POLICY_NOTE" => Ok(Self::InternalPolicyNote),
            _ => Err(ParseDataLabelError {
                label: s.to_owned(),
            }),
        }
    }
}

/// Set of confidentiality labels attached to a runtime value.
///
/// Confidentiality is additive: the `join` of two label sets is their union.
/// Once a label is present, no computation can remove it.
///
/// # Examples
///
/// ```
/// use zamburak_core::{DataLabel, DataLabels};
///
/// let a = DataLabels::from_iter([DataLabel::Pii]);
/// let b = DataLabels::from_iter([DataLabel::AuthSecret]);
/// let joined = a.join(&b);
///
/// assert!(joined.contains(&DataLabel::Pii));
/// assert!(joined.contains(&DataLabel::AuthSecret));
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataLabels {
    labels: BTreeSet<DataLabel>,
}

impl DataLabels {
    /// Create an empty label set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a label set containing all known confidentiality variants.
    ///
    /// Used for the unknown-top conservative summary when budget overflow
    /// forces fail-closed behaviour.
    #[must_use]
    pub fn all() -> Self {
        Self {
            labels: DataLabel::all_variants().into_iter().collect(),
        }
    }

    /// Compute the IFC join (union) of two confidentiality label sets.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        Self {
            labels: self.labels.union(&other.labels).copied().collect(),
        }
    }

    /// Check whether this set contains a specific label.
    #[must_use]
    pub fn contains(&self, label: &DataLabel) -> bool {
        self.labels.contains(label)
    }

    /// Return `true` if no confidentiality labels are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Return the number of confidentiality labels in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Iterate over the labels in this set.
    pub fn iter(&self) -> impl Iterator<Item = &DataLabel> {
        self.labels.iter()
    }

    /// Check whether this set is a subset of another.
    #[must_use]
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.labels.is_subset(&other.labels)
    }
}

impl FromIterator<DataLabel> for DataLabels {
    fn from_iter<I: IntoIterator<Item = DataLabel>>(iter: I) -> Self {
        Self {
            labels: iter.into_iter().collect(),
        }
    }
}

/// Set of authority capabilities that a runtime value may exercise.
///
/// Authority is narrowing: the `join` of two authority sets is their
/// intersection. A derived value can only exercise capabilities that all
/// its sources possess.
///
/// The special "full" set (`AuthoritySet::full()`) represents the
/// universe of all possible capabilities and acts as the identity
/// element for intersection-based `join`. Since `AuthorityCapability`
/// wraps an open string type, the universe cannot be enumerated; instead
/// a flag marks the set as containing every capability.
///
/// # Examples
///
/// ```
/// use zamburak_core::{AuthorityCapability, AuthoritySet};
///
/// let cap_a = AuthorityCapability::try_from("EmailSendCap")
///     .expect("invalid capability in doctest");
/// let cap_b = AuthorityCapability::try_from("CalendarWriteCap")
///     .expect("invalid capability in doctest");
///
/// let set_a = AuthoritySet::from_iter([cap_a.clone(), cap_b.clone()]);
/// let set_b = AuthoritySet::from_iter([cap_a.clone()]);
/// let joined = set_a.join(&set_b);
///
/// assert!(joined.contains(&cap_a));
/// assert!(!joined.contains(&cap_b));
///
/// // Full set is the identity for join (intersection).
/// let full = AuthoritySet::full();
/// assert_eq!(set_a.join(&full), set_a);
/// assert_eq!(full.join(&set_a), set_a);
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthoritySet {
    capabilities: BTreeSet<AuthorityCapability>,
    /// When `true`, the set represents the universe of all capabilities
    /// and acts as the identity element for intersection-based `join`.
    is_full: bool,
}

impl AuthoritySet {
    /// Create an empty authority set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return an `AuthoritySet` representing the universe of all
    /// capabilities — the identity element for the intersection-based
    /// `join` operation.
    ///
    /// `full().join(x) == x` and `x.join(full()) == x` for any `x`.
    #[must_use]
    pub fn full() -> Self {
        Self {
            capabilities: BTreeSet::new(),
            is_full: true,
        }
    }

    /// Return `true` if this set represents the full universe of
    /// capabilities.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.is_full
    }

    /// Compute the IFC join (intersection) of two authority sets.
    ///
    /// Authority narrows on dependency: a value derived from multiple
    /// sources can only exercise capabilities common to all sources.
    ///
    /// The full (universe) set is the identity element: joining with
    /// `full()` returns the other operand unchanged.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        if self.is_full {
            return other.clone();
        }
        if other.is_full {
            return self.clone();
        }
        Self {
            capabilities: self
                .capabilities
                .intersection(&other.capabilities)
                .cloned()
                .collect(),
            is_full: false,
        }
    }

    /// Check whether the set includes a specific capability.
    ///
    /// A full (universe) set contains every capability.
    #[must_use]
    pub fn contains(&self, cap: &AuthorityCapability) -> bool {
        self.is_full || self.capabilities.contains(cap)
    }

    /// Return `true` if no authority capabilities are present.
    ///
    /// A full (universe) set is never empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_full && self.capabilities.is_empty()
    }

    /// Add a capability to this authority set.
    pub fn insert(&mut self, cap: AuthorityCapability) {
        self.capabilities.insert(cap);
    }

    /// Iterate over the explicitly stored capabilities in this set.
    ///
    /// Note: for a full (universe) set this iterator yields no items,
    /// even though `contains` returns `true` for every capability.
    /// Use `is_full()` to detect universe sets.
    pub fn iter(&self) -> impl Iterator<Item = &AuthorityCapability> {
        self.capabilities.iter()
    }
}

impl FromIterator<AuthorityCapability> for AuthoritySet {
    fn from_iter<I: IntoIterator<Item = AuthorityCapability>>(iter: I) -> Self {
        Self {
            capabilities: iter.into_iter().collect(),
            is_full: false,
        }
    }
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "trust_proptests.rs"]
mod proptests;
