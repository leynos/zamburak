//! Runtime policy-engine construction from validated policy definitions.

use crate::migration::MigrationAuditRecord;
use crate::policy_def::{PolicyDefinition, PolicyLoadError, PolicyLoadOutcome};

/// Runtime policy engine backed by a validated policy definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEngine {
    policy_definition: PolicyDefinition,
}

/// Outcome of constructing a policy engine with migration evidence.
///
/// # Examples
///
/// ```rust
/// use zamburak_policy::{PolicyEngine, PolicyLoadError};
///
/// let policy_yaml = r#"
/// schema_version: 1
/// policy_name: minimal_policy
/// default_action: Deny
/// strict_mode: true
/// budgets:
///   max_values: 1
///   max_parents_per_value: 1
///   max_closure_steps: 1
///   max_witness_depth: 1
/// tools: []
/// "#;
///
/// let load_outcome = PolicyEngine::from_yaml_str_with_migration_audit(policy_yaml)?;
/// let (_policy_engine, migration_audit) = load_outcome.into_parts();
/// assert_eq!(migration_audit.source_schema_version.as_u64(), 1);
///
/// Ok::<(), PolicyLoadError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEngineLoadOutcome {
    policy_engine: PolicyEngine,
    migration_audit: MigrationAuditRecord,
}

impl PolicyEngineLoadOutcome {
    /// Return the built policy engine.
    #[must_use]
    pub const fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    /// Return migration audit evidence for this load operation.
    #[must_use]
    pub const fn migration_audit(&self) -> &MigrationAuditRecord {
        &self.migration_audit
    }

    /// Consume this outcome and return both engine and audit evidence.
    #[must_use]
    pub fn into_parts(self) -> (PolicyEngine, MigrationAuditRecord) {
        (self.policy_engine, self.migration_audit)
    }
}

impl PolicyEngine {
    /// Build a policy engine from a YAML policy document.
    pub fn from_yaml_str(policy_yaml: &str) -> Result<Self, PolicyLoadError> {
        let load_outcome = Self::from_yaml_str_with_migration_audit(policy_yaml)?;
        Ok(load_outcome.policy_engine)
    }

    /// Build a policy engine from a YAML policy document with migration audit evidence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak_policy::{PolicyEngine, PolicyLoadError};
    ///
    /// let policy_yaml = r#"
    /// schema_version: 1
    /// policy_name: minimal_policy
    /// default_action: Deny
    /// strict_mode: true
    /// budgets:
    ///   max_values: 1
    ///   max_parents_per_value: 1
    ///   max_closure_steps: 1
    ///   max_witness_depth: 1
    /// tools: []
    /// "#;
    ///
    /// let load_outcome = PolicyEngine::from_yaml_str_with_migration_audit(policy_yaml)?;
    /// assert!(!load_outcome.migration_audit().was_migrated());
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_yaml_str_with_migration_audit(
        policy_yaml: &str,
    ) -> Result<PolicyEngineLoadOutcome, PolicyLoadError> {
        let policy_load_outcome =
            PolicyDefinition::from_yaml_str_with_migration_audit(policy_yaml)?;
        Ok(build_engine_load_outcome(policy_load_outcome))
    }

    /// Build a policy engine from a JSON policy document.
    pub fn from_json_str(policy_json: &str) -> Result<Self, PolicyLoadError> {
        let load_outcome = Self::from_json_str_with_migration_audit(policy_json)?;
        Ok(load_outcome.policy_engine)
    }

    /// Build a policy engine from a JSON policy document with migration audit evidence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak_policy::{PolicyEngine, PolicyLoadError};
    ///
    /// let policy_json = r#"{
    ///   "schema_version": 1,
    ///   "policy_name": "minimal_policy",
    ///   "default_action": "Deny",
    ///   "strict_mode": true,
    ///   "budgets": {
    ///     "max_values": 1,
    ///     "max_parents_per_value": 1,
    ///     "max_closure_steps": 1,
    ///     "max_witness_depth": 1
    ///   },
    ///   "tools": []
    /// }"#;
    ///
    /// let load_outcome = PolicyEngine::from_json_str_with_migration_audit(policy_json)?;
    /// assert_eq!(load_outcome.migration_audit().target_schema_version.as_u64(), 1);
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    pub fn from_json_str_with_migration_audit(
        policy_json: &str,
    ) -> Result<PolicyEngineLoadOutcome, PolicyLoadError> {
        let policy_load_outcome =
            PolicyDefinition::from_json_str_with_migration_audit(policy_json)?;
        Ok(build_engine_load_outcome(policy_load_outcome))
    }

    /// Return the validated policy definition in use by this engine.
    #[must_use]
    pub const fn policy_definition(&self) -> &PolicyDefinition {
        &self.policy_definition
    }
}

fn build_engine_load_outcome(policy_load_outcome: PolicyLoadOutcome) -> PolicyEngineLoadOutcome {
    let (policy_definition, migration_audit) = policy_load_outcome.into_parts();
    let policy_engine = PolicyEngine { policy_definition };
    PolicyEngineLoadOutcome {
        policy_engine,
        migration_audit,
    }
}
