Feature: Monty sync orchestration
  Scenario: Successful sync refreshes fork state and runs verification gates
    Given a monty sync happy-path command sequence
    When I run the monty sync workflow
    Then monty sync succeeds
    And the output mentions completion

  Scenario: Dirty superproject fails before submodule operations
    Given a dirty superproject command sequence
    When I run the monty sync workflow
    Then monty sync fails
    And the failure mentions superproject cleanliness

  Scenario: Missing fork remote fails with actionable error
    Given a missing fork remote command sequence
    When I run the monty sync workflow
    Then monty sync fails
    And the failure mentions missing fork remote

  Scenario: Verification gate failure fails the workflow
    Given a verification gate failure command sequence
    When I run the monty sync workflow
    Then monty sync fails
    And the failure mentions the lint gate
