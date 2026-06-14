Feature: Total conversion manifest validation
  A total conversion manifest should reject an invalid pack type.

  Scenario: Rejecting a non-total-conversion type
    Given a total conversion manifest with type "content"
    When I validate the manifest
    Then validation should fail with a type error
