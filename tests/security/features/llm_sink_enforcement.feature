Feature: LLM sink enforcement

  Scenario: Pre-dispatch allows planner LLM call with redaction applied
    Given a planner LLM sink call request with redaction applied
    When the pre-dispatch policy check is evaluated
    Then the pre-dispatch decision is Allow

  Scenario: Pre-dispatch denies planner LLM call without redaction
    Given a planner LLM sink call request without redaction applied
    When the pre-dispatch policy check is evaluated
    Then the pre-dispatch decision is Deny

  Scenario: Transport guard passes when redaction is applied
    Given a transport guard check with redaction applied
    When the transport guard is evaluated
    Then the transport guard outcome is Passed

  Scenario: Transport guard blocks when redaction is missing
    Given a transport guard check without redaction applied
    When the transport guard is evaluated
    Then the transport guard outcome is Blocked

  Scenario: Post-dispatch audit record links execution and call identifiers
    Given a planner LLM sink call request with execution id "exec_7f2c" and call id "call_0192"
    And redaction is applied
    When the pre-dispatch policy check is evaluated
    And an audit record is emitted
    Then the audit record execution id is "exec_7f2c"
    And the audit record call id is "call_0192"
    And the audit record decision is Allow

  Scenario: Quarantined LLM call emits linked audit record
    Given a quarantined LLM sink call request with execution id "exec_ab01" and call id "call_0500"
    And redaction is applied
    When the pre-dispatch policy check is evaluated
    And an audit record is emitted
    Then the audit record call path is Quarantined
    And the audit record execution id is "exec_ab01"
