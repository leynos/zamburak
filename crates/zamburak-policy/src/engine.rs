//! Runtime policy-engine construction from validated policy definitions.

use crate::load_outcome::LoadOutcome;
use crate::migration::MigrationAuditRecord;
use crate::policy_def::{PolicyDefinition, PolicyLoadError, PolicyLoadOutcome};
use zamburak_core::authority::{
    AuthorityBoundaryValidation, AuthorityToken, RevocationIndex, TokenTimestamp,
    validate_tokens_at_policy_boundary,
};

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
    load_outcome: LoadOutcome<PolicyEngine>,
}

impl PolicyEngineLoadOutcome {
    /// Return the built policy engine.
    #[must_use]
    pub const fn policy_engine(&self) -> &PolicyEngine {
        self.load_outcome.value()
    }

    /// Return migration audit evidence for this load operation.
    #[must_use]
    pub const fn migration_audit(&self) -> &MigrationAuditRecord {
        self.load_outcome.migration_audit()
    }

    /// Consume this outcome and return both engine and audit evidence.
    #[must_use]
    pub fn into_parts(self) -> (PolicyEngine, MigrationAuditRecord) {
        self.load_outcome.into_parts()
    }
}

impl PolicyEngine {
    /// Build a policy engine from a YAML policy document.
    pub fn from_yaml_str(policy_yaml: &str) -> Result<Self, PolicyLoadError> {
        let load_outcome = Self::from_yaml_str_with_migration_audit(policy_yaml)?;
        let (policy_engine, _migration_audit) = load_outcome.into_parts();
        Ok(policy_engine)
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
        let (policy_engine, _migration_audit) = load_outcome.into_parts();
        Ok(policy_engine)
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

    /// Validate authority tokens at a policy-evaluation boundary.
    ///
    /// Tokens that are revoked, expired, or pre-issuance at
    /// `evaluation_time` are stripped from the effective authority set and
    /// recorded as invalid. The remaining effective tokens represent the
    /// authority available for downstream policy checks.
    ///
    /// This method delegates to the canonical lifecycle validation in
    /// `zamburak-core` so the policy engine consumes lifecycle verdicts
    /// rather than duplicating transition logic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zamburak_core::{RevocationIndex, TokenTimestamp};
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
    /// let engine = PolicyEngine::from_yaml_str(policy_yaml)?;
    /// let revocation_index = RevocationIndex::default();
    /// let validation = engine.validate_authority_tokens(
    ///     &[],
    ///     &revocation_index,
    ///     TokenTimestamp::new(100),
    /// );
    /// assert!(validation.effective_tokens().is_empty());
    ///
    /// Ok::<(), PolicyLoadError>(())
    /// ```
    #[must_use]
    pub fn validate_authority_tokens(
        &self,
        tokens: &[AuthorityToken],
        revocation_index: &RevocationIndex,
        evaluation_time: TokenTimestamp,
    ) -> AuthorityBoundaryValidation {
        validate_tokens_at_policy_boundary(tokens, revocation_index, evaluation_time)
    }
}

fn build_engine_load_outcome(policy_load_outcome: PolicyLoadOutcome) -> PolicyEngineLoadOutcome {
    let (policy_definition, migration_audit) = policy_load_outcome.into_parts();
    let policy_engine = PolicyEngine { policy_definition };
    PolicyEngineLoadOutcome {
        load_outcome: LoadOutcome::new(policy_engine, migration_audit),
    }
}
