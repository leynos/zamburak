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

  Scenario: Missing uv metadata fails baseline checks
    Given a roadmap script missing uv metadata
    When I run the script baseline checker
    Then the checker exits with an error
    And the output mentions missing uv metadata

  Scenario: Incorrect requires-python fails baseline checks
    Given a roadmap script with incorrect requires-python
    When I run the script baseline checker
    Then the checker exits with an error
    And the output mentions invalid requires-python

  Scenario: Forbidden command imports fail baseline checks
    Given a roadmap script with forbidden command imports
    When I run the script baseline checker
    Then the checker exits with an error
    And the output mentions forbidden command imports
