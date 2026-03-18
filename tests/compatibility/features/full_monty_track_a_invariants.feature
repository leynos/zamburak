Feature: Full-monty Track A invariants compatibility probe

  Scenario: Full-monty Track A invariants BDD suite succeeds from the superproject
    Given a full-monty Track A invariants BDD probe command
    When the Track A invariants probe command is executed
    Then the Track A invariants probe command succeeds
    And the probe output mentions Track A invariants coverage
