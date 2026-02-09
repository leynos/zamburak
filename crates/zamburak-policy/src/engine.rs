//! Runtime policy-engine construction from validated policy definitions.

use crate::policy_def::{PolicyDefinition, PolicyLoadError};

/// Runtime policy engine backed by a validated policy definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEngine {
    policy_definition: PolicyDefinition,
}

impl PolicyEngine {
    /// Build a policy engine from a YAML policy document.
    pub fn from_yaml_str(policy_yaml: &str) -> Result<Self, PolicyLoadError> {
        let policy_definition = PolicyDefinition::from_yaml_str(policy_yaml)?;
        Ok(Self { policy_definition })
    }

    /// Build a policy engine from a JSON policy document.
    pub fn from_json_str(policy_json: &str) -> Result<Self, PolicyLoadError> {
        let policy_definition = PolicyDefinition::from_json_str(policy_json)?;
        Ok(Self { policy_definition })
    }

    /// Return the validated policy definition in use by this engine.
    #[must_use]
    pub const fn policy_definition(&self) -> &PolicyDefinition {
        &self.policy_definition
    }
}
