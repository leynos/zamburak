//! Unit tests for `PolicyEngine` construction and authority-token validation.

use rstest::rstest;
use zamburak_core::{
    AuthorityCapability, AuthorityIssuer, AuthorityLifecycleError, AuthorityScope,
    AuthoritySubject, AuthorityToken, AuthorityTokenId, IssuerTrust, MintRequest, RevocationIndex,
    ScopeResource, TokenTimestamp,
};

use super::PolicyEngine;
use crate::PolicyLoadError;

const CANONICAL_POLICY_JSON: &str = r#"
    {
      "schema_version": 1,
      "policy_name": "engine_tests_policy",
      "default_action": "Deny",
      "strict_mode": true,
      "budgets": {
        "max_values": 1,
        "max_parents_per_value": 1,
        "max_closure_steps": 1,
        "max_witness_depth": 1
      },
      "tools": []
    }
"#;

const CANONICAL_POLICY_YAML: &str = r#"
schema_version: 1
policy_name: engine_tests_policy
default_action: Deny
strict_mode: true
budgets:
  max_values: 1
  max_parents_per_value: 1
  max_closure_steps: 1
  max_witness_depth: 1
tools: []
"#;

#[rstest]
fn from_json_str_builds_a_usable_engine() -> Result<(), PolicyLoadError> {
    let engine = PolicyEngine::from_json_str(CANONICAL_POLICY_JSON)?;

    assert_eq!(
        engine.policy_definition().policy_name,
        "engine_tests_policy"
    );
    Ok(())
}

#[rstest]
fn load_outcome_policy_engine_accessor_matches_into_parts() -> Result<(), PolicyLoadError> {
    let load_outcome = PolicyEngine::from_yaml_str_with_migration_audit(CANONICAL_POLICY_YAML)?;

    let via_accessor = load_outcome
        .policy_engine()
        .policy_definition()
        .policy_name
        .clone();
    let (engine, _migration_audit) = load_outcome.into_parts();

    assert_eq!(via_accessor, engine.policy_definition().policy_name);
    Ok(())
}

fn minted_token(
    token_id: &str,
    issued_at: u64,
    expires_at: u64,
) -> Result<AuthorityToken, AuthorityLifecycleError> {
    AuthorityToken::mint(MintRequest {
        token_id: AuthorityTokenId::try_from(token_id)?,
        issuer: AuthorityIssuer::try_from("policy-host")?,
        issuer_trust: IssuerTrust::HostTrusted,
        subject: AuthoritySubject::try_from("assistant")?,
        capability: AuthorityCapability::try_from("EmailSendCap")?,
        scope: AuthorityScope::new(vec![ScopeResource::try_from("send_email")?])?,
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    })
}

#[rstest]
fn validate_authority_tokens_strips_revoked_tokens_at_the_policy_boundary()
-> Result<(), AuthorityLifecycleError> {
    let engine = PolicyEngine::from_yaml_str(CANONICAL_POLICY_YAML)
        .expect("canonical policy yaml should load");

    let token = minted_token("revoked-at-boundary", 0, 1000)?;
    let mut revocation_index = RevocationIndex::default();
    revocation_index.revoke(token.token_id().clone());

    let validation =
        engine.validate_authority_tokens(&[token], &revocation_index, TokenTimestamp::new(500));

    assert!(validation.effective_tokens().is_empty());
    assert_eq!(validation.invalid_tokens().len(), 1);
    Ok(())
}

#[rstest]
fn validate_authority_tokens_keeps_tokens_that_are_still_valid()
-> Result<(), AuthorityLifecycleError> {
    let engine = PolicyEngine::from_yaml_str(CANONICAL_POLICY_YAML)
        .expect("canonical policy yaml should load");

    let token = minted_token("still-valid", 0, 1000)?;
    let revocation_index = RevocationIndex::default();

    let validation =
        engine.validate_authority_tokens(&[token], &revocation_index, TokenTimestamp::new(500));

    assert_eq!(validation.effective_tokens().len(), 1);
    assert!(validation.invalid_tokens().is_empty());
    Ok(())
}
