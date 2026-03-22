Feature: IFC dependency graph and propagation

  Scenario: Transitive label propagation through dependency chain
    Given a default dependency graph
    And an untrusted PII-labelled value 1
    And a verified clean value 2 depending on value 1
    When the summary is computed for value 2
    Then the summary integrity is Untrusted
    And the summary confidentiality includes PII
    And the summary is not truncated

  Scenario: Budget exhaustion yields conservative summary
    Given a dependency graph with max_closure_steps 0
    And an untrusted PII-labelled value 1
    And a verified clean value 2 depending on value 1
    When the summary is computed for value 2
    Then the summary equals unknown_top

  Scenario: Normal mode excludes control context
    Given a default dependency graph
    And a trusted clean value 3
    And a fresh execution context
    And an untrusted condition pushed into the context
    When labels are propagated in Normal mode for value 3
    Then the propagation result integrity is Trusted
    And the propagation result does not include PII

  Scenario: Strict mode includes control context
    Given a default dependency graph
    And a trusted clean value 3
    And a fresh execution context
    And an untrusted condition pushed into the context
    When labels are propagated in Strict mode for value 3
    Then the propagation result integrity is Untrusted
    And the propagation result includes PII
