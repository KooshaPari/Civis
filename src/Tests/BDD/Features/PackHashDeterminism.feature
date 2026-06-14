Feature: Pack hash determinism
  Pack hashes should ignore signature artifacts and stay stable for identical content.

  Scenario: Signature artifacts do not change the pack hash
    Given a pack directory with content files
    When I compute the baseline pack hash
    And I add signature artifacts to the pack directory
    And I recompute the pack hash
    Then the pack hash should stay the same
