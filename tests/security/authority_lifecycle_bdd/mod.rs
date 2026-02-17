//! Behavioural tests validating authority token lifecycle conformance.

mod helpers;

use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use zamburak_core::{
    AuthorityBoundaryValidation, AuthorityLifecycleError, AuthorityToken, AuthorityTokenId,
    DelegationRequest, InvalidAuthorityReason, MintRequest, RevocationIndex, TokenTimestamp,
    revalidate_tokens_on_restore, validate_tokens_at_policy_boundary,
};

use helpers::{
    IssuerConfig, assert_delegation_rejected_expired_parent, assert_delegation_rejected_lifetime,
    assert_delegation_rejected_revoked_parent, assert_delegation_rejected_scope,
    assert_invalid_token_stripped, assert_mint_rejected_lifetime, assert_mint_rejected_untrusted,
    assert_no_effective_tokens, assert_valid_token_remains, create_delegation_request,
    create_delegation_request_at, create_mint_request, create_token_set_with_expired,
    create_token_set_with_revoked, make_scope, mint_fixture, require_delegation_result,
    require_mint_result,
};

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

#[given("a host-trusted minting request for subject {subject} with capability {capability}")]
fn host_trusted_mint_request(world: &mut LifecycleWorld, subject: String, capability: String) {
    create_mint_request(world, &subject, &capability, IssuerConfig::HostTrusted);
}

#[given("an untrusted minting request for subject {subject} with capability {capability}")]
fn untrusted_mint_request(world: &mut LifecycleWorld, subject: String, capability: String) {
    create_mint_request(world, &subject, &capability, IssuerConfig::Untrusted);
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

#[when("the host mints the authority token")]
fn do_mint(world: &mut LifecycleWorld) {
    let Some(request) = world.mint_request.take() else {
        panic!("mint request must be set before minting");
    };
    world.mint_result = Some(AuthorityToken::mint(request));
}

#[then("the mint succeeds")]
fn mint_succeeds(world: &LifecycleWorld) {
    let result = require_mint_result(world);
    assert!(result.is_ok(), "expected mint success, got: {result:?}");
}

#[then("the mint is rejected as untrusted")]
fn mint_rejected_untrusted(world: &LifecycleWorld) {
    assert_mint_rejected_untrusted(world);
}

#[then("the mint is rejected for invalid lifetime")]
fn mint_rejected_lifetime(world: &LifecycleWorld) {
    assert_mint_rejected_lifetime(world);
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

#[given("a minted parent token with scope {res_a} and {res_b} expiring at {expires_at:u64}")]
fn minted_parent(world: &mut LifecycleWorld, res_a: String, res_b: String, expires_at: u64) {
    world.parent_token = Some(mint_fixture("parent", &[&res_a, &res_b], 10, expires_at));
}

#[given("a delegation request narrowing scope to {res} expiring at {expires_at:u64}")]
fn narrowed_delegation(world: &mut LifecycleWorld, res: String, expires_at: u64) {
    create_delegation_request(world, &[&res], expires_at);
}

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
    assert_delegation_rejected_scope(world);
}

#[then("the delegation is rejected for non-strict lifetime")]
fn delegation_rejected_lifetime(world: &LifecycleWorld) {
    assert_delegation_rejected_lifetime(world);
}

#[then("the delegation is rejected because the parent is revoked")]
fn delegation_rejected_revoked_parent(world: &LifecycleWorld) {
    assert_delegation_rejected_revoked_parent(world);
}

#[then("the delegation is rejected because the parent is expired")]
fn delegation_rejected_expired_parent(world: &LifecycleWorld) {
    assert_delegation_rejected_expired_parent(world);
}

#[given("a set of authority tokens including a revoked token")]
fn token_set_with_revoked(world: &mut LifecycleWorld) {
    create_token_set_with_revoked(world);
}

#[given("a set of authority tokens including an expired token")]
fn token_set_with_expired(world: &mut LifecycleWorld) {
    create_token_set_with_expired(world);
}

#[given("a set of authority tokens that are all expired before time 200")]
fn token_set_all_expired(world: &mut LifecycleWorld) {
    let early = mint_fixture("early-tok", &["send_email"], 10, 50);
    let late = mint_fixture("late-tok", &["send_email"], 10, 150);
    world.token_set = vec![early, late];
}

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
    assert_valid_token_remains(world);
}

#[then("no tokens remain in the effective set")]
fn no_effective_tokens(world: &LifecycleWorld) {
    assert_no_effective_tokens(world);
}

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
