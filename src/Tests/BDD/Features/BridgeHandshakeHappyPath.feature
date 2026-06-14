Feature: Bridge handshake happy path
  The bridge client should complete the connect handshake and accept a signed follow-up response.

  Scenario: Connect and verify a signed ping
    Given a bridge server that mints a fresh session and signs ping responses
    When I connect the bridge client
    And I request a ping
    Then the bridge client should remain connected
    And the ping should succeed
