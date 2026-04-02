Feature: Governed IFC contexts

  Scenario: Strict mode carries control context into a constant effect call
    Given a program whose constant effect call is control-dependent on an external result
    And strict IFC propagation is configured
    When governed execution starts and the first external call returns string "yes"
    Then the captured call context for "effect" has aggregate integrity "Untrusted"
    And the captured call context for "effect" has first argument integrity "Trusted"
    And the captured call context for "effect" has PC integrity "Untrusted"

  Scenario: Normal mode leaves the constant effect call data summary unchanged
    Given a program whose constant effect call is control-dependent on an external result
    And normal IFC propagation is configured
    When governed execution starts and the first external call returns string "yes"
    Then the captured call context for "effect" has aggregate integrity "Trusted"
    And the captured call context for "effect" has first argument integrity "Trusted"
    And the captured call context for "effect" has PC integrity "Untrusted"

  Scenario: Returned host values retain provenance for a following effect
    Given a program whose second effect consumes a resumed external return value
    And strict IFC propagation is configured
    When governed execution starts and the first external call returns string "payload"
    Then the captured call context for "sink" has aggregate integrity "Untrusted"
    And the captured call context for "sink" has first argument integrity "Untrusted"
    And the captured call context for "sink" has aggregate origin-count at least 2
    And the captured call context for "sink" has first argument origin-count at least 2
