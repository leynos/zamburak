//! Behavioural tests validating authority token lifecycle conformance.

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_core::{
    AuthorityBoundaryValidation, AuthorityCapability, AuthorityLifecycleError, AuthorityScope,
    AuthoritySubject, AuthorityToken, AuthorityTokenId, DelegationRequest, InvalidAuthorityReason,
    IssuerTrust, MintRequest, RevocationIndex, ScopeResource, TokenTimestamp,
    revalidate_tokens_on_restore, validate_tokens_at_policy_boundary,
};

// ── World ──────────────────────────────────────────────────────────

#[derive(Default)]
struct LifecycleWorld {
    mint_request: Option<MintRequest>,
    mint_result: Option<Result<AuthorityToken, AuthorityLifecycleError>>,
    parent_token: Option<AuthorityToken>,
    delegation_request: Option<DelegationRequest>,
    delegation_result: Option<Result<AuthorityToken, AuthorityLifecycleError>>,
    revocation_index: RevocationIndex,
    token_set: Vec<AuthorityToken>,
    boundary_result: Option<AuthorityBoundaryValidation>,
    /// Token id of the token that is expected to be invalid.
    expected_invalid_id: Option<AuthorityTokenId>,
    /// Token id of the token that is expected to be valid.
    expected_valid_id: Option<AuthorityTokenId>,
}

#[fixture]
fn world() -> LifecycleWorld {
    LifecycleWorld::default()
}

// ── Helpers ────────────────────────────────────────────────────────

fn make_token_id(name: &str) -> AuthorityTokenId {
    let Ok(id) = AuthorityTokenId::try_from(name) else {
        panic!("test token id '{name}' is invalid");
    };
    id
}

fn make_subject(name: &str) -> AuthoritySubject {
    let Ok(subject) = AuthoritySubject::try_from(name) else {
        panic!("test subject '{name}' is invalid");
    };
    subject
}

fn make_capability(name: &str) -> AuthorityCapability {
    let Ok(capability) = AuthorityCapability::try_from(name) else {
        panic!("test capability '{name}' is invalid");
    };
    capability
}

fn make_scope(resources: &[&str]) -> AuthorityScope {
    let parsed: Vec<ScopeResource> = resources
        .iter()
        .map(|r| {
            let Ok(resource) = ScopeResource::try_from(*r) else {
                panic!("test scope resource '{r}' is invalid");
            };
            resource
        })
        .collect();
    let Ok(scope) = AuthorityScope::new(parsed) else {
        panic!("test scope is invalid");
    };
    scope
}

fn mint_fixture(
    token_name: &str,
    scope_resources: &[&str],
    issued_at: u64,
    expires_at: u64,
) -> AuthorityToken {
    let Ok(token) = AuthorityToken::mint(MintRequest {
        token_id: make_token_id(token_name),
        issuer: "policy-host".to_owned(),
        issuer_trust: IssuerTrust::HostTrusted,
        subject: make_subject("assistant"),
        capability: make_capability("EmailSendCap"),
        scope: make_scope(scope_resources),
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    }) else {
        panic!("mint fixture '{token_name}' is invalid");
    };
    token
}

fn require_mint_result(world: &LifecycleWorld) -> &Result<AuthorityToken, AuthorityLifecycleError> {
    let Some(result) = world.mint_result.as_ref() else {
        panic!("mint step must run before assertion");
    };
    result
}

fn require_delegation_result(
    world: &LifecycleWorld,
) -> &Result<AuthorityToken, AuthorityLifecycleError> {
    let Some(result) = world.delegation_result.as_ref() else {
        panic!("delegation step must run before assertion");
    };
    result
}

fn require_boundary_result(world: &LifecycleWorld) -> &AuthorityBoundaryValidation {
    let Some(result) = world.boundary_result.as_ref() else {
        panic!("boundary validation step must run before assertion");
    };
    result
}

/// Generic assertion for mint result error variants.
fn assert_mint_error<F>(world: &LifecycleWorld, predicate: F, error_desc: &str)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = require_mint_result(world);
    match result {
        Err(err) if predicate(err) => {} // success
        _ => panic!("expected {error_desc}, got: {result:?}"),
    }
}

/// Generic assertion for delegation result error variants.
fn assert_delegation_error<F>(world: &LifecycleWorld, predicate: F, error_desc: &str)
where
    F: Fn(&AuthorityLifecycleError) -> bool,
{
    let result = require_delegation_result(world);
    match result {
        Err(err) if predicate(err) => {} // success
        _ => panic!("expected {error_desc}, got: {result:?}"),
    }
}

/// Helper to assert that a token with the expected invalid ID is present in the
/// invalid tokens list with the specified reason.
fn assert_invalid_token_stripped(
    world: &LifecycleWorld,
    expected_reason: InvalidAuthorityReason,
    reason_desc: &str,
) {
    let validation = require_boundary_result(world);
    let Some(invalid_id) = world.expected_invalid_id.as_ref() else {
        panic!("expected invalid id must be set");
    };

    assert!(
        validation
            .invalid_tokens()
            .iter()
            .any(|t| t.token_id() == invalid_id && t.reason() == expected_reason),
        "expected {reason_desc} token to be stripped"
    );
}

/// Helper to create a mint request with configurable issuer trust.
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is a distinct domain field of the mint request"
)]
fn create_mint_request(
    world: &mut LifecycleWorld,
    subject: &str,
    capability: &str,
    issuer: &str,
    issuer_trust: IssuerTrust,
) {
    world.mint_request = Some(MintRequest {
        token_id: make_token_id("mint-token"),
        issuer: issuer.to_owned(),
        issuer_trust,
        subject: make_subject(subject),
        capability: make_capability(capability),
        scope: make_scope(&["placeholder"]),
        issued_at: TokenTimestamp::new(0),
        expires_at: TokenTimestamp::new(1),
    });
}

/// Helper to create a token set with one valid and one invalid token.
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter controls a distinct axis of the token-set fixture"
)]
fn create_token_set_with_invalid(
    world: &mut LifecycleWorld,
    valid_name: &str,
    invalid_name: &str,
    valid_expires: u64,
    invalid_expires: u64,
    revoke_invalid: bool,
) {
    let valid = mint_fixture(valid_name, &["send_email"], 10, valid_expires);
    let invalid = mint_fixture(invalid_name, &["send_email"], 10, invalid_expires);

    if revoke_invalid {
        world.revocation_index.revoke(invalid.token_id().clone());
    }

    world.expected_valid_id = Some(valid.token_id().clone());
    world.expected_invalid_id = Some(invalid.token_id().clone());
    world.token_set = vec![valid, invalid];
}

// ── Given: minting ─────────────────────────────────────────────────

#[given("a host-trusted minting request for subject {subject} with capability {capability}")]
fn host_trusted_mint_request(world: &mut LifecycleWorld, subject: String, capability: String) {
    create_mint_request(
        world,
        &subject,
        &capability,
        "policy-host",
        IssuerTrust::HostTrusted,
    );
}

#[given("an untrusted minting request for subject {subject} with capability {capability}")]
fn untrusted_mint_request(world: &mut LifecycleWorld, subject: String, capability: String) {
    create_mint_request(
        world,
        &subject,
        &capability,
        "remote-agent",
        IssuerTrust::Untrusted,
    );
}

#[given("the mint scope includes {res_a} and {res_b}")]
fn set_mint_scope_two(world: &mut LifecycleWorld, res_a: String, res_b: String) {
    if let Some(req) = world.mint_request.as_mut() {
        req.scope = make_scope(&[&res_a, &res_b]);
    }
}

#[given("the token lifetime is from {issued_at:u64} to {expires_at:u64}")]
fn set_mint_lifetime(world: &mut LifecycleWorld, issued_at: u64, expires_at: u64) {
    if let Some(req) = world.mint_request.as_mut() {
        req.issued_at = TokenTimestamp::new(issued_at);
        req.expires_at = TokenTimestamp::new(expires_at);
    }
}

// ── When: minting ──────────────────────────────────────────────────

#[when("the host mints the authority token")]
fn do_mint(world: &mut LifecycleWorld) {
    let Some(request) = world.mint_request.take() else {
        panic!("mint request must be set before minting");
    };
    world.mint_result = Some(AuthorityToken::mint(request));
}

// ── Then: minting ──────────────────────────────────────────────────

#[then("the mint succeeds")]
fn mint_succeeds(world: &LifecycleWorld) {
    let result = require_mint_result(world);
    assert!(result.is_ok(), "expected mint success, got: {result:?}");
}

#[then("the mint is rejected as untrusted")]
fn mint_rejected_untrusted(world: &LifecycleWorld) {
    assert_mint_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::UntrustedMinter { .. }),
        "UntrustedMinter",
    );
}

#[then("the mint is rejected for invalid lifetime")]
fn mint_rejected_lifetime(world: &LifecycleWorld) {
    assert_mint_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::InvalidTokenLifetime { .. }),
        "InvalidTokenLifetime",
    );
}

#[then("the minted token encodes the declared subject and capability")]
fn minted_token_fields(world: &LifecycleWorld) {
    let result = require_mint_result(world);
    let Ok(token) = result else {
        panic!("expected successful mint, got: {result:?}");
    };
    assert_eq!(token.subject().as_str(), "assistant");
    assert_eq!(token.capability().as_str(), "EmailSendCap");
}

#[then("the minted token has no parent delegation")]
fn minted_no_parent(world: &LifecycleWorld) {
    let result = require_mint_result(world);
    let Ok(token) = result else {
        panic!("expected successful mint, got: {result:?}");
    };
    assert!(
        token.parent_token_id().is_none(),
        "freshly minted tokens must not have parent lineage"
    );
}

// ── Given: delegation ──────────────────────────────────────────────

#[given("a minted parent token with scope {res_a} and {res_b} expiring at {expires_at:u64}")]
fn minted_parent(world: &mut LifecycleWorld, res_a: String, res_b: String, expires_at: u64) {
    world.parent_token = Some(mint_fixture("parent", &[&res_a, &res_b], 10, expires_at));
}

#[given("a delegation request narrowing scope to {res} expiring at {expires_at:u64}")]
fn narrowed_delegation(world: &mut LifecycleWorld, res: String, expires_at: u64) {
    create_delegation_request(world, &[&res], expires_at);
}

/// Helper to create a delegation request with given scope resources.
fn create_delegation_request(world: &mut LifecycleWorld, resources: &[&str], expires_at: u64) {
    create_delegation_request_at(world, resources, 20, expires_at);
}

/// Helper to create a delegation request with custom delegation time.
fn create_delegation_request_at(
    world: &mut LifecycleWorld,
    resources: &[&str],
    delegated_at: u64,
    expires_at: u64,
) {
    world.delegation_request = Some(DelegationRequest {
        token_id: make_token_id("child"),
        delegated_by: "policy-host".to_owned(),
        subject: make_subject("assistant"),
        scope: make_scope(resources),
        delegated_at: TokenTimestamp::new(delegated_at),
        expires_at: TokenTimestamp::new(expires_at),
    });
}

/// Scope widening delegation step.
///
/// The feature file passes three resource names plus an expiry time. The
/// macro expands this into five parameters (world + 4 captures) which
/// exceeds the default Clippy argument limit. A tightly-scoped suppression
/// is appropriate here because the parameter count is driven by the Gherkin
/// step text, not by poor decomposition.
#[expect(
    clippy::too_many_arguments,
    reason = "parameter count driven by Gherkin step captures"
)]
#[given("a delegation request widening scope to {a} and {b} and {c} expiring at {expires_at:u64}")]
fn widened_delegation(
    world: &mut LifecycleWorld,
    a: String,
    b: String,
    c: String,
    expires_at: u64,
) {
    create_delegation_request(world, &[&a, &b, &c], expires_at);
}

#[given("a delegation request with equal scope {a} and {b} expiring at {expires_at:u64}")]
fn equal_scope_delegation(world: &mut LifecycleWorld, a: String, b: String, expires_at: u64) {
    create_delegation_request(world, &[&a, &b], expires_at);
}

#[given(
    "a delegation request narrowing scope to {res} expiring at {expires_at:u64} delegated at {delegated_at:u64}"
)]
fn narrowed_delegation_late(
    world: &mut LifecycleWorld,
    res: String,
    expires_at: u64,
    delegated_at: u64,
) {
    create_delegation_request_at(world, &[&res], delegated_at, expires_at);
}

#[given("the parent token is revoked")]
fn revoke_parent(world: &mut LifecycleWorld) {
    let Some(parent) = world.parent_token.as_ref() else {
        panic!("parent token must be set before revoking");
    };
    world.revocation_index.revoke(parent.token_id().clone());
}

// ── When: delegation ───────────────────────────────────────────────

#[when("the delegation is attempted")]
fn do_delegate(world: &mut LifecycleWorld) {
    let Some(parent) = world.parent_token.as_ref() else {
        panic!("parent token must be set before delegation");
    };
    let Some(request) = world.delegation_request.take() else {
        panic!("delegation request must be set before delegation");
    };
    world.delegation_result = Some(AuthorityToken::delegate(
        parent,
        request,
        &world.revocation_index,
    ));
}

// ── Then: delegation ───────────────────────────────────────────────

#[then("the delegation succeeds")]
fn delegation_succeeds(world: &LifecycleWorld) {
    let result = require_delegation_result(world);
    assert!(
        result.is_ok(),
        "expected delegation success, got: {result:?}"
    );
}

#[then("the delegated token retains parent lineage")]
fn delegation_lineage(world: &LifecycleWorld) {
    let result = require_delegation_result(world);
    let Ok(child) = result else {
        panic!("expected successful delegation, got: {result:?}");
    };
    let Some(parent) = world.parent_token.as_ref() else {
        panic!("parent token must be set");
    };
    assert_eq!(
        child.parent_token_id(),
        Some(parent.token_id()),
        "delegated token must retain parent lineage"
    );
}

#[then("the delegation is rejected for non-strict scope")]
fn delegation_rejected_scope(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| matches!(err, AuthorityLifecycleError::DelegationScopeNotStrictSubset),
        "DelegationScopeNotStrictSubset",
    );
}

#[then("the delegation is rejected for non-strict lifetime")]
fn delegation_rejected_lifetime(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::DelegationLifetimeNotStrictSubset { .. }
            )
        },
        "DelegationLifetimeNotStrictSubset",
    );
}

#[then("the delegation is rejected because the parent is revoked")]
fn delegation_rejected_revoked_parent(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidParentToken {
                    reason: InvalidAuthorityReason::Revoked,
                    ..
                }
            )
        },
        "InvalidParentToken(Revoked)",
    );
}

#[then("the delegation is rejected because the parent is expired")]
fn delegation_rejected_expired_parent(world: &LifecycleWorld) {
    assert_delegation_error(
        world,
        |err| {
            matches!(
                err,
                AuthorityLifecycleError::InvalidParentToken {
                    reason: InvalidAuthorityReason::Expired,
                    ..
                }
            )
        },
        "InvalidParentToken(Expired)",
    );
}

// ── Given: boundary / restore ──────────────────────────────────────

#[given("a set of authority tokens including a revoked token")]
fn token_set_with_revoked(world: &mut LifecycleWorld) {
    create_token_set_with_invalid(world, "valid-tok", "revoked-tok", 1000, 1000, true);
}

#[given("a set of authority tokens including an expired token")]
fn token_set_with_expired(world: &mut LifecycleWorld) {
    create_token_set_with_invalid(world, "valid-tok", "expired-tok", 1000, 100, false);
}

#[given("a set of authority tokens that are all expired before time 200")]
fn token_set_all_expired(world: &mut LifecycleWorld) {
    let early = mint_fixture("early-tok", &["send_email"], 10, 50);
    let late = mint_fixture("late-tok", &["send_email"], 10, 150);
    world.token_set = vec![early, late];
}

// ── When: boundary / restore ───────────────────────────────────────

#[when("the tokens are validated at a policy boundary at time {eval_time:u64}")]
fn validate_at_boundary(world: &mut LifecycleWorld, eval_time: u64) {
    world.boundary_result = Some(validate_tokens_at_policy_boundary(
        &world.token_set,
        &world.revocation_index,
        TokenTimestamp::new(eval_time),
    ));
}

#[when("the tokens are revalidated on snapshot restore at time {restore_time:u64}")]
fn revalidate_on_restore(world: &mut LifecycleWorld, restore_time: u64) {
    world.boundary_result = Some(revalidate_tokens_on_restore(
        &world.token_set,
        &world.revocation_index,
        TokenTimestamp::new(restore_time),
    ));
}

// ── Then: boundary / restore ───────────────────────────────────────

#[then("the revoked token is stripped from the effective set")]
fn revoked_stripped(world: &LifecycleWorld) {
    assert_invalid_token_stripped(world, InvalidAuthorityReason::Revoked, "revoked");
}

#[then("the expired token is stripped from the effective set")]
fn expired_stripped(world: &LifecycleWorld) {
    assert_invalid_token_stripped(world, InvalidAuthorityReason::Expired, "expired");
}

#[then("the valid tokens remain in the effective set")]
fn valid_tokens_remain(world: &LifecycleWorld) {
    let validation = require_boundary_result(world);
    let Some(valid_id) = world.expected_valid_id.as_ref() else {
        panic!("expected valid id must be set");
    };

    assert!(
        validation
            .effective_tokens()
            .iter()
            .any(|t| t.token_id() == valid_id),
        "expected valid token to remain in effective set"
    );
}

#[then("no tokens remain in the effective set")]
fn no_effective_tokens(world: &LifecycleWorld) {
    let validation = require_boundary_result(world);
    assert!(
        validation.effective_tokens().is_empty(),
        "expected no effective tokens, found: {:?}",
        validation.effective_tokens()
    );
}

// ── Scenarios ──────────────────────────────────────────────────────

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Mint authority token with host-trusted issuer"
)]
fn mint_host_trusted(world: LifecycleWorld) {
    assert!(world.mint_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject minting from untrusted issuer"
)]
fn reject_untrusted_mint(world: LifecycleWorld) {
    assert!(world.mint_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject minting with zero-duration lifetime"
)]
fn reject_zero_lifetime_mint(world: LifecycleWorld) {
    assert!(world.mint_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Delegate token with strictly narrowed scope and lifetime"
)]
fn delegate_strictly_narrowed(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject delegation that widens scope"
)]
fn reject_widened_delegation(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject delegation with equal scope"
)]
fn reject_equal_scope_delegation(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject delegation with non-narrowed lifetime"
)]
fn reject_non_narrowed_lifetime_delegation(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject delegation from revoked parent"
)]
fn reject_revoked_parent_delegation(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Reject delegation from expired parent"
)]
fn reject_expired_parent_delegation(world: LifecycleWorld) {
    assert!(world.delegation_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Revoked token stripped at policy boundary"
)]
fn revoked_stripped_at_boundary(world: LifecycleWorld) {
    assert!(world.boundary_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Expired token stripped at policy boundary"
)]
fn expired_stripped_at_boundary(world: LifecycleWorld) {
    assert!(world.boundary_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "Snapshot restore revalidates tokens conservatively"
)]
fn restore_revalidates_conservatively(world: LifecycleWorld) {
    assert!(world.boundary_result.is_some());
}

#[scenario(
    path = "tests/security/features/authority_lifecycle.feature",
    name = "All tokens stripped when all are expired on restore"
)]
fn all_stripped_on_restore(world: LifecycleWorld) {
    assert!(world.boundary_result.is_some());
}
