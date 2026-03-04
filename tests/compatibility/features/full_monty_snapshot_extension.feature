Feature: Full-monty snapshot extension compatibility probe

  Scenario: Full-monty snapshot extension BDD suite succeeds from the superproject
    Given a full-monty snapshot extension BDD probe command
    When the snapshot extension probe command is executed
    Then the snapshot extension probe command succeeds
    And the probe output mentions snapshot extension coverage
