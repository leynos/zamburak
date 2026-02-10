# Zamburak user's guide

## Policy loader contract

The runtime policy loader enforces canonical policy schema v1 and supports an
explicit legacy migration path.

- accepted without migration: `schema_version: 1`,
- accepted with explicit migration: `schema_version: 0` (migrated to v1),
- rejected fail-closed: unknown schema versions and unknown schema families.

Unknown schema versions are never defaulted, partially loaded, or heuristically
migrated.

## Runtime API

Use `PolicyEngine` or `PolicyDefinition` from `zamburak-policy`:

- `PolicyEngine::from_yaml_str(...)`,
- `PolicyEngine::from_json_str(...)`,
- `PolicyDefinition::from_yaml_str(...)`,
- `PolicyDefinition::from_json_str(...)`.

These entrypoints return canonical policy objects and hide migration evidence.

For auditable migration evidence, use:

- `PolicyEngine::from_yaml_str_with_migration_audit(...)`,
- `PolicyEngine::from_json_str_with_migration_audit(...)`,
- `PolicyDefinition::from_yaml_str_with_migration_audit(...)`,
- `PolicyDefinition::from_json_str_with_migration_audit(...)`.

Audit outcomes include:

- source and target schema versions,
- source and target canonicalized SHA-256 document hashes,
- ordered per-step transform records (`policy_schema_v0_to_v1`).

## Example: canonical policy (schema v1)

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

## Example: migrated legacy policy (schema v0)

```yaml
schema_version: 0
policy_name: personal_assistant_default
default_action: Deny
strict_mode: true
budgets:
  max_values: 100000
  max_parents_per_value: 64
  max_closure_steps: 10000
  max_witness_depth: 32
tools:
  - name: send_email
    side_effect: ExternalWrite
    authority: [EmailSendCap]
    args:
      - name: body
        forbid_confidentiality: [AUTH_SECRET]
    context:
      deny_if_pc_integrity_contains: [Untrusted]
    default_decision: RequireConfirmation
```

The loader migrates this policy to canonical v1 and exposes migration audit
metadata through the audit-bearing API variants.

## Example: rejected policy

A policy document using an unsupported schema version such as
`schema_version: 2` is rejected with
`PolicyLoadError::UnsupportedSchemaVersion`.

This fail-closed behaviour is intentional and required by the security
contracts.
