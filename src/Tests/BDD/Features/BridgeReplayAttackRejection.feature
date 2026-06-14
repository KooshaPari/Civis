Feature: Bridge replay attack rejection
  A replayed bridge receipt should be rejected when the world frame does not advance.

  Scenario: Reject a repeated ping frame
    Given a bridge server that replays the same signed ping frame
    When I connect the bridge client
    And I request a ping
    And I replay the same ping
    Then the replay should be rejected as a frame regression
