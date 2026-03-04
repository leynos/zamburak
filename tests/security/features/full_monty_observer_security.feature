Feature: Full-monty observer security regression probe

  Scenario: Full-monty observer error-path regression succeeds from the superproject
    Given a full-monty observer error-path probe command
    When the security probe command is executed
    Then the security probe command succeeds
    And the security probe output mentions error return coverage
