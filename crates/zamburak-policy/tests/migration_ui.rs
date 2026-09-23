//! Compile a downstream consumer of the policy migration API.

#[test]
fn migration_loader_compiles_with_sha2_011() {
    trybuild::TestCases::new().pass("tests/ui/migration_api.rs");
}
