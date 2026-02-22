Feature: Localization contract

  Scenario: Explicit localizer injection with no-op fallback
    Given a no-op localizer
    When a message is requested with id "zamburak-deny-reason" and fallback "Access denied"
    Then the rendered message is "Access denied"

  Scenario: Localizer lookup returns None for no-op localizer
    Given a no-op localizer
    When a lookup is performed for id "zamburak-deny-reason"
    Then the lookup result is absent

  Scenario: Localizer trait is object-safe for dynamic dispatch
    Given a no-op localizer behind a trait object
    When a message is requested through the trait object with fallback "fallback text"
    Then the rendered message is "fallback text"

  Scenario: Localized diagnostic renders through injected localizer
    Given a diagnostic with message id "zamburak-test-diag" and fallback "test fallback"
    And a no-op localizer
    When the diagnostic is rendered with the localizer
    Then the rendered diagnostic text is "test fallback"

  Scenario: Message request with non-empty interpolation arguments
    Given a no-op localizer
    When a message with interpolation arguments is requested for id "greeting" and fallback "Hello, {name}!"
    Then the rendered message is "Hello, {name}!"

  Scenario: No global mutable localizer state exists
    Given two independent no-op localizer instances
    When messages are requested from both localizers independently
    Then both produce deterministic fallback results without shared state
