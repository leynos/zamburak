Feature: Full-monty fork policy enforcement

  Scenario: Generic observer API additions are accepted
    Given a generic observer hook patch
    When the fork policy checker evaluates the patch
    Then the violation count is 0

  Scenario: Zamburak semantics in public API are rejected
    Given a patch with Zamburak semantics in public API
    When the fork policy checker evaluates the patch
    Then the violation count is 1
    And a violation token includes "zamburak"

  Scenario: Forbidden terms in non-public additions are ignored
    Given a patch with forbidden term in non-public code
    When the fork policy checker evaluates the patch
    Then the violation count is 0

  Scenario: Mixed patch rejects only public API semantic violations
    Given a patch with mixed public and non-public forbidden terms
    When the fork policy checker evaluates the patch
    Then the violation count is 1
    And a violation token includes "policy"
