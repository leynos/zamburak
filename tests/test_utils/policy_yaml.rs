//! Shared policy fixtures for compatibility, migration, and unit tests.

/// Return the canonical policy fixture YAML used across test suites.
#[must_use]
pub const fn canonical_policy_yaml() -> &'static str {
    include_str!("../../policies/default.yaml")
}

/// Return a schema-v0 legacy policy fixture used for migration tests.
#[must_use]
pub const fn legacy_policy_v0_yaml() -> &'static str {
    include_str!("policy-v0.yaml")
}

/// Return schema-v0 legacy policy JSON used for migration tests.
#[must_use]
pub const fn legacy_policy_v0_json() -> &'static str {
    include_str!("policy-v0.json")
}

/// Return canonical policy YAML with a substituted `schema_version`.
#[must_use]
pub fn policy_yaml_with_schema_version(schema_version: u64) -> String {
    let replacement = format!("schema_version: {schema_version}");
    canonical_policy_yaml().replacen("schema_version: 1", &replacement, 1)
}
