//! Compile a downstream consumer of the policy migration API.

/// Ensures a downstream crate can compile against the migration API with sha2 0.11.
#[test]
fn migration_loader_compiles_with_sha2_011() {
    trybuild::TestCases::new().pass("tests/ui/migration_api.rs");
}
