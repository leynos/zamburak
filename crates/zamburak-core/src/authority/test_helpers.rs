//! Shared constants, fixtures, and assertion helpers for authority tests.

use super::{
    AuthorityCapability, AuthorityLifecycleError, AuthorityScope, AuthoritySubject, AuthorityToken,
    AuthorityTokenId, DelegationRequest, IssuerTrust, MintRequest, RevocationIndex, ScopeResource,
    TokenTimestamp,
};

pub const TEST_ISSUER: &str = "policy-host";
pub const TEST_SUBJECT: &str = "assistant";
pub const TEST_CAPABILITY: &str = "EmailSendCap";
pub const TEST_DELEGATED_BY: &str = "policy-host";
pub const TOKEN_NAME_PARENT: &str = "parent";
pub const TOKEN_NAME_CHILD: &str = "child";
pub const TOKEN_NAME_VALID: &str = "valid";
pub const TOKEN_NAME_REVOKED: &str = "revoked";
pub const TOKEN_NAME_EXPIRED: &str = "expired";
pub const TOKEN_NAME_TOKEN: &str = "token";
pub const TOKEN_NAME_FUTURE: &str = "future";
pub const TOKEN_NAME_MINT_UNTRUSTED: &str = "mint-untrusted";
pub const TOKEN_NAME_MINT_INVALID: &str = "mint-invalid-lifetime";

lazy_static::lazy_static! {
    pub static ref TOKEN_ID_PARENT: AuthorityTokenId = token_id(TOKEN_NAME_PARENT);
    pub static ref TOKEN_ID_CHILD: AuthorityTokenId = token_id(TOKEN_NAME_CHILD);
    pub static ref TOKEN_ID_VALID: AuthorityTokenId = token_id(TOKEN_NAME_VALID);
    pub static ref TOKEN_ID_REVOKED: AuthorityTokenId = token_id(TOKEN_NAME_REVOKED);
    pub static ref TOKEN_ID_EXPIRED: AuthorityTokenId = token_id(TOKEN_NAME_EXPIRED);
    pub static ref TOKEN_ID_TOKEN: AuthorityTokenId = token_id(TOKEN_NAME_TOKEN);
    pub static ref TOKEN_ID_FUTURE: AuthorityTokenId = token_id(TOKEN_NAME_FUTURE);
    pub static ref SCOPE_SEND_EMAIL: AuthorityScope = scope(&["send_email"]);
    pub static ref SCOPE_EMAIL_AND_DRAFT: AuthorityScope = scope(&["send_email", "send_email_draft"]);
    pub static ref SUBJECT_ASSISTANT: AuthoritySubject = subject(TEST_SUBJECT);
    pub static ref SCOPE_WIDENED: AuthorityScope =
        scope(&["send_email", "send_email_draft", "calendar_write"]);
    pub static ref TOKEN_PARENT: AuthorityToken =
        mint_token(TOKEN_ID_PARENT.clone(), SCOPE_EMAIL_AND_DRAFT.clone(), 10, 200);
    pub static ref TOKEN_VALID: AuthorityToken =
        mint_token(TOKEN_ID_VALID.clone(), SCOPE_SEND_EMAIL.clone(), 10, 300);
    pub static ref TOKEN_REVOKED: AuthorityToken =
        mint_token(TOKEN_ID_REVOKED.clone(), SCOPE_SEND_EMAIL.clone(), 10, 300);
    pub static ref TOKEN_EXPIRED: AuthorityToken =
        mint_token(TOKEN_ID_EXPIRED.clone(), SCOPE_SEND_EMAIL.clone(), 10, 100);
    pub static ref TOKEN_FOR_RESTORE: AuthorityToken =
        mint_token(TOKEN_ID_TOKEN.clone(), SCOPE_SEND_EMAIL.clone(), 10, 100);
    pub static ref TOKEN_FUTURE: AuthorityToken =
        mint_token(TOKEN_ID_FUTURE.clone(), SCOPE_SEND_EMAIL.clone(), 500, 900);
}

pub struct MintRequestBuilder {
    token_name: String,
    issuer: String,
    issuer_trust: IssuerTrust,
    subject_name: String,
    capability_name: String,
    scope_resources: Vec<String>,
    issued_at: u64,
    expires_at: u64,
}

impl MintRequestBuilder {
    pub fn new(token_name: &str) -> Self {
        Self {
            token_name: token_name.to_owned(),
            issuer: TEST_ISSUER.to_owned(),
            issuer_trust: IssuerTrust::HostTrusted,
            subject_name: TEST_SUBJECT.to_owned(),
            capability_name: TEST_CAPABILITY.to_owned(),
            scope_resources: vec!["send_email".to_owned()],
            issued_at: 0,
            expires_at: 100,
        }
    }

    pub fn issuer(mut self, issuer: &str, trust: IssuerTrust) -> Self {
        self.issuer = issuer.to_owned();
        self.issuer_trust = trust;
        self
    }

    pub fn lifetime(mut self, issued_at: u64, expires_at: u64) -> Self {
        self.issued_at = issued_at;
        self.expires_at = expires_at;
        self
    }

    pub fn build(self) -> MintRequest {
        let scope_refs: Vec<&str> = self.scope_resources.iter().map(String::as_str).collect();
        MintRequest {
            token_id: token_id(&self.token_name),
            issuer: self.issuer,
            issuer_trust: self.issuer_trust,
            subject: subject(&self.subject_name),
            capability: capability(&self.capability_name),
            scope: scope(&scope_refs),
            issued_at: TokenTimestamp::new(self.issued_at),
            expires_at: TokenTimestamp::new(self.expires_at),
        }
    }
}

pub fn token_id(value: &str) -> AuthorityTokenId {
    AuthorityTokenId::try_from(value).expect("token ids used in tests are valid")
}

pub fn subject(value: &str) -> AuthoritySubject {
    AuthoritySubject::try_from(value).expect("subjects used in tests are valid")
}

pub fn capability(value: &str) -> AuthorityCapability {
    AuthorityCapability::try_from(value).expect("capabilities used in tests are valid")
}

pub fn scope(resources: &[&str]) -> AuthorityScope {
    let parsed_resources = resources
        .iter()
        .map(|resource| ScopeResource::try_from(*resource).expect("scope resources are valid"));
    AuthorityScope::new(parsed_resources).expect("scope fixtures are valid")
}

pub fn mint_token(
    token_id: AuthorityTokenId,
    scope: AuthorityScope,
    issued_at: u64,
    expires_at: u64,
) -> AuthorityToken {
    AuthorityToken::mint(MintRequest {
        token_id,
        issuer: TEST_ISSUER.to_owned(),
        issuer_trust: IssuerTrust::HostTrusted,
        subject: subject(TEST_SUBJECT),
        capability: capability(TEST_CAPABILITY),
        scope,
        issued_at: TokenTimestamp::new(issued_at),
        expires_at: TokenTimestamp::new(expires_at),
    })
    .expect("mint fixtures are valid")
}

pub fn assert_mint_fails<F: Fn(&AuthorityLifecycleError) -> bool>(
    request: MintRequest,
    predicate: F,
) {
    let result = AuthorityToken::mint(request);
    match result {
        Err(ref err) if predicate(err) => {}
        _ => panic!("mint request did not fail as expected, got: {result:?}"),
    }
}

pub fn delegation_request_with_scope(
    child_id: &AuthorityTokenId,
    scope: &AuthorityScope,
    delegated_at: u64,
    expires_at: u64,
) -> DelegationRequest {
    DelegationRequest {
        token_id: child_id.clone(),
        delegated_by: TEST_DELEGATED_BY.to_owned(),
        subject: SUBJECT_ASSISTANT.clone(),
        scope: scope.clone(),
        delegated_at: TokenTimestamp::new(delegated_at),
        expires_at: TokenTimestamp::new(expires_at),
    }
}

pub fn assert_delegation_fails<F: Fn(&AuthorityLifecycleError) -> bool>(
    parent: &AuthorityToken,
    request: DelegationRequest,
    revocation_index: &RevocationIndex,
    predicate: F,
) {
    let result = AuthorityToken::delegate(parent, request, revocation_index);
    match result {
        Err(ref err) if predicate(err) => {}
        _ => panic!("delegation request did not fail as expected, got: {result:?}"),
    }
}
