//! Downstream compile fixture for the public policy migration API.

use zamburak_policy::{MigrationAuditRecord, PolicyDefinition};

fn main() -> Result<(), zamburak_policy::PolicyLoadError> {
    let legacy_policy_yaml = r#"
schema_version: 0
policy_name: minimal_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 1
  max_parents_per_value: 1
  max_closure_steps: 1
  max_witness_depth: 1
tools: []
"#;

    let load_outcome = PolicyDefinition::from_yaml_str_with_migration_audit(legacy_policy_yaml)?;
    let _: &MigrationAuditRecord = load_outcome.migration_audit();

    Ok(())
}
