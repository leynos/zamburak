Feature: Policy schema loader

  Scenario: Load canonical schema version 1 policy
    Given a canonical schema v1 policy document
    When the runtime loads the policy
    Then the policy loads successfully

  Scenario: Reject unknown policy schema version
    Given a policy document with unknown schema version 2
    When the runtime loads the policy
    Then the runtime rejects the policy as unsupported schema version

  Scenario: Migrate supported legacy policy schema version
    Given a legacy schema v0 policy document
    When the runtime loads the policy
    Then the policy loads successfully
    And migration audit records source schema version 0 and target schema version 1
    And migration audit records 1 applied migration step

  Scenario: Keep canonical policy load unmigrated in migration audit
    Given a canonical schema v1 policy document
    When the runtime loads the policy
    Then migration audit records source schema version 1 and target schema version 1
    And migration audit records 0 applied migration step
