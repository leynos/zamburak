Feature: Authority token lifecycle

  Scenario: Mint authority token with host-trusted issuer
    Given a host-trusted minting request for subject assistant with capability EmailSendCap
    And the mint scope includes send_email and read_contacts
    And the token lifetime is from 100 to 500
    When the host mints the authority token
    Then the mint succeeds
    And the minted token encodes the declared subject and capability
    And the minted token has no parent delegation

  Scenario: Reject minting from untrusted issuer
    Given an untrusted minting request for subject assistant with capability EmailSendCap
    And the mint scope includes send_email and read_contacts
    And the token lifetime is from 100 to 500
    When the host mints the authority token
    Then the mint is rejected as untrusted

  Scenario: Reject minting with zero-duration lifetime
    Given a host-trusted minting request for subject assistant with capability EmailSendCap
    And the mint scope includes send_email and read_contacts
    And the token lifetime is from 100 to 100
    When the host mints the authority token
    Then the mint is rejected for invalid lifetime

  Scenario: Delegate token with strictly narrowed scope and lifetime
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And a delegation request narrowing scope to send_email expiring at 400
    When the delegation is attempted
    Then the delegation succeeds
    And the delegated token retains parent lineage

  Scenario: Reject delegation that widens scope
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And a delegation request widening scope to send_email and read_contacts and calendar_write expiring at 400
    When the delegation is attempted
    Then the delegation is rejected for non-strict scope

  Scenario: Reject delegation with equal scope
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And a delegation request with equal scope send_email and read_contacts expiring at 400
    When the delegation is attempted
    Then the delegation is rejected for non-strict scope

  Scenario: Reject delegation with non-narrowed lifetime
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And a delegation request narrowing scope to send_email expiring at 500
    When the delegation is attempted
    Then the delegation is rejected for non-strict lifetime

  Scenario: Reject delegation from revoked parent
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And the parent token is revoked
    And a delegation request narrowing scope to send_email expiring at 400
    When the delegation is attempted
    Then the delegation is rejected because the parent is revoked

  Scenario: Reject delegation from expired parent
    Given a minted parent token with scope send_email and read_contacts expiring at 500
    And a delegation request narrowing scope to send_email expiring at 700 delegated at 600
    When the delegation is attempted
    Then the delegation is rejected because the parent is expired

  Scenario: Revoked token stripped at policy boundary
    Given a set of authority tokens including a revoked token
    When the tokens are validated at a policy boundary at time 200
    Then the revoked token is stripped from the effective set
    And the valid tokens remain in the effective set

  Scenario: Expired token stripped at policy boundary
    Given a set of authority tokens including an expired token
    When the tokens are validated at a policy boundary at time 200
    Then the expired token is stripped from the effective set
    And the valid tokens remain in the effective set

  Scenario: Snapshot restore revalidates tokens conservatively
    Given a set of authority tokens including a revoked token
    When the tokens are revalidated on snapshot restore at time 200
    Then the revoked token is stripped from the effective set
    And the valid tokens remain in the effective set

  Scenario: All tokens stripped when all are expired on restore
    Given a set of authority tokens that are all expired before time 200
    When the tokens are revalidated on snapshot restore at time 200
    Then no tokens remain in the effective set
