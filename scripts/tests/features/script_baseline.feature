Feature: Script baseline validation
  Scenario: Compliant script passes baseline checks
    Given a compliant roadmap script tree
    When I run the script baseline checker
    Then the checker exits successfully

  Scenario: Missing matching tests fails baseline checks
    Given a roadmap script without matching tests
    When I run the script baseline checker
    Then the checker exits with an error
    And the output mentions missing matching test
