Feature: Policy schema loader

  Scenario: Load canonical schema version 1 policy
    Given a canonical schema v1 policy document
    When the runtime loads the policy
    Then the policy loads successfully

  Scenario: Reject unknown policy schema version
    Given a policy document with unknown schema version 2
    When the runtime loads the policy
    Then the runtime rejects the policy as unsupported schema version
