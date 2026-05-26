"""E2E tests for game status and connection via MCP bridge."""
import pytest


@pytest.mark.e2e
def test_game_status_returns_dict(game_status, mcp_server):
    """Test that game_status() returns a dictionary."""
    result = mcp_server.game_status()
    assert isinstance(result, dict), "game_status() should return a dict"


@pytest.mark.e2e
def test_game_status_running(game_status, mcp_server):
    """Test that game_status() indicates game is running."""
    result = mcp_server.game_status()
    assert result.get("running") is True, "Game should be running"


@pytest.mark.e2e
def test_game_status_entity_count(game_status, mcp_server):
    """Test that game_status() returns reasonable entity count."""
    result = mcp_server.game_status()
    entity_count = result.get("entity_count", 0)
    assert entity_count > 100, f"Expected > 100 entities, got {entity_count}"


@pytest.mark.e2e
def test_game_resources_available(game_status, mcp_server):
    """Test that game_resources() returns valid resource dict."""
    result = mcp_server.game_resources()
    assert isinstance(result, dict), "game_resources() should return a dict"
    assert "gold" in result and "wood" in result, "Resources missing standard keys"
