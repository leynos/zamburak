//! Shared migration-audit load-outcome helper used by policy and engine loaders.

use crate::migration::MigrationAuditRecord;

/// Generic container for a loaded value plus migration audit evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoadOutcome<T> {
    value: T,
    migration_audit: MigrationAuditRecord,
}

impl<T> LoadOutcome<T> {
    /// Build a load outcome from a value and migration audit evidence.
    #[must_use]
    pub(crate) fn new(value: T, migration_audit: MigrationAuditRecord) -> Self {
        Self {
            value,
            migration_audit,
        }
    }

    /// Return the loaded value.
    #[must_use]
    pub(crate) const fn value(&self) -> &T {
        &self.value
    }

    /// Return migration audit evidence.
    #[must_use]
    pub(crate) const fn migration_audit(&self) -> &MigrationAuditRecord {
        &self.migration_audit
    }

    /// Consume this outcome and return both value and audit evidence.
    #[must_use]
    pub(crate) fn into_parts(self) -> (T, MigrationAuditRecord) {
        (self.value, self.migration_audit)
    }
}
