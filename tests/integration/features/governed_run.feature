Feature: Governed execution with zamburak-monty

  Scenario: Simple program completes without external calls
    Given a simple arithmetic Monty program
    And an AllowAll mediator
    When the governed runner executes the program
    Then the result is Complete with integer value 3

  Scenario: External function call is denied by DenyAll mediator
    Given a Monty program that calls an external function "foo"
    And a DenyAll mediator
    When the governed runner executes the program
    Then the result is Denied for function "foo"
    And the denial reason mentions "DenyAllMediator"

  Scenario: Conditional execution completes under governance
    Given a Monty program with conditional branching
    And an AllowAll mediator
    When the governed runner executes the program
    Then the result is Complete with string value "big"

  Scenario: Program with inputs completes correctly
    Given a Monty program with two numeric inputs
    And an AllowAll mediator
    When the governed runner executes the program with integer inputs 10 and 32
    Then the result is Complete with integer value 42

  Scenario: Observer receives Track A events during governed execution
    Given a Monty program with conditional branching
    And an AllowAll mediator
    When the governed runner executes the program
    Then the result is Complete
    And the observer recorded value_created events
    And the observer recorded op_result events
    And the observer recorded control_condition events
