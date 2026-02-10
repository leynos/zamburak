# Zamburak user's guide

## Policy loader contract (schema v1 freeze)

The runtime policy loader now enforces a canonical schema contract:

- accepted schema: `schema_version: 1`,
- rejected schema: any other version.

The loader is fail-closed. Unknown schema versions are rejected and are not
silently migrated, defaulted, or partially loaded.

## Runtime API

Use `PolicyEngine` or `PolicyDefinition` from `zamburak-policy`:

- `PolicyEngine::from_yaml_str(...)`,
- `PolicyEngine::from_json_str(...)`,
- `PolicyDefinition::from_yaml_str(...)`,
- `PolicyDefinition::from_json_str(...)`.

All entrypoints enforce the same schema-version check before returning a loaded
policy.

## Example: accepted policy

```yaml
schema_version: 1
policy_name: personal_assistant_default
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools: []
```

## Example: rejected policy

A policy document using `schema_version: 2` is rejected with
`PolicyLoadError::UnsupportedSchemaVersion`.

This behaviour is intentional and required by the fail-closed standards.
