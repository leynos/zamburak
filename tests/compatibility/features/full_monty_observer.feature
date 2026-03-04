Feature: Full-monty observer compatibility probe

  Scenario: Full-monty observer BDD suite succeeds from the superproject
    Given a full-monty observer BDD probe command
    When the probe command is executed
    Then the probe command succeeds
    And the probe output mentions runtime observer events
