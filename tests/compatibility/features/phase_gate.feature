Feature: Phase-gate CI enforcement

  Scenario: Phase zero target passes with no mandated suites
    Given a phase-gate target input "phase0"
    And an empty verification catalogue
    When the phase gate is evaluated
    Then the phase gate status is "Passed"

  Scenario: Phase one target blocks when mandated suites are missing
    Given a phase-gate target input "phase1"
    And the verification catalogue has schema and authority suites only
    When the phase gate is evaluated
    Then the phase gate status is "MissingSuites"
    And missing suites include "llm-sink-enforcement,localization-contract"

  Scenario: Phase one target blocks when a mandated suite fails
    Given a phase-gate target input "phase1"
    And the verification catalogue has all phase-one suites
    And the suite "authority-lifecycle" is marked failing
    When the phase gate is evaluated
    Then the phase gate status is "FailingSuites"
    And failing suites include "authority-lifecycle"

  Scenario: Unsupported phase target input is rejected
    Given a phase-gate target input "phase-99"
    When the phase-gate target is parsed
    Then the phase-gate target parse result is invalid
