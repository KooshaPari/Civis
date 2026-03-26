"""E2E tests for game screenshot capture via MCP bridge."""
import pytest
from pathlib import Path


@pytest.mark.e2e
def test_game_screenshot_returns_dict(game_status, mcp_server):
    """Test that game_screenshot() returns a dictionary."""
    result = mcp_server.game_screenshot()
    assert isinstance(result, dict), "game_screenshot() should return a dict"


@pytest.mark.e2e
def test_game_screenshot_successful(game_status, mcp_server):
    """Test that game_screenshot() succeeds."""
    result = mcp_server.game_screenshot()
    assert result.get("success") is True, f"Screenshot failed: {result.get('error')}"


@pytest.mark.e2e
def test_game_screenshot_file_exists(game_status, mcp_server):
    """Test that screenshot file is created."""
    result = mcp_server.game_screenshot()
    if not result.get("success"):
        pytest.skip(f"Screenshot failed: {result.get('error')}")
    screenshot_path = Path(result.get("path"))
    assert screenshot_path.exists(), f"Screenshot file should exist"


@pytest.mark.e2e
def test_game_screenshot_is_png(game_status, mcp_server):
    """Test that screenshot is a valid PNG file."""
    result = mcp_server.game_screenshot()
    if not result.get("success"):
        pytest.skip(f"Screenshot failed: {result.get('error')}")
    screenshot_path = Path(result.get("path"))
    magic_bytes = screenshot_path.read_bytes()[:8]
    png_signature = b"\x89PNG\r\n\x1a\n"
    assert magic_bytes == png_signature, "Screenshot should be valid PNG"


@pytest.mark.e2e
def test_game_screenshot_file_size(game_status, mcp_server):
    """Test that screenshot file is reasonably sized."""
    result = mcp_server.game_screenshot()
    if not result.get("success"):
        pytest.skip(f"Screenshot failed: {result.get('error')}")
    screenshot_path = Path(result.get("path"))
    file_size = screenshot_path.stat().st_size
    assert file_size > 1024, f"Screenshot should be > 1KB"
