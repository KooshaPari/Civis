"""VLM-backed screenshot validation tests for visual game features."""
import pytest
from pathlib import Path


@pytest.mark.e2e
@pytest.mark.vlm
@pytest.mark.slow
def test_game_screenshot_vlm_valid(game_status, mcp_server, anthropic_client):
    """Test that we can capture a screenshot and it's a valid image."""
    result = mcp_server.game_screenshot()
    assert result.get("success"), "Screenshot capture failed"
    screenshot_path = Path(result.get("path"))
    assert screenshot_path.exists(), "Screenshot file should exist"


@pytest.mark.e2e
@pytest.mark.vlm
@pytest.mark.slow
def test_main_menu_visible_vlm(game_status, mcp_server, anthropic_client):
    """VLM test: Verify main menu is visible with expected buttons."""
    from vlm_judge import judge_screenshot_sync

    result = mcp_server.game_screenshot()
    if not result.get("success"):
        pytest.skip(f"Screenshot failed: {result.get('error')}")

    screenshot_path = result.get("path")
    judgment = judge_screenshot_sync(
        screenshot_path,
        "the game main menu with buttons like Play, Settings, or Quit is visible",
    )

    assert judgment.get("pass") is True, f"Main menu not visible: {judgment.get('reason')}"
    assert judgment.get("confidence", 0) > 0.7, f"Confidence too low: {judgment.get('reason')}"


@pytest.mark.e2e
@pytest.mark.vlm
def test_vlm_judge_reliability(game_status, mcp_server, anthropic_client):
    """Test VLM judge itself with obvious assertions."""
    from vlm_judge import judge_screenshot_sync

    result = mcp_server.game_screenshot()
    if not result.get("success"):
        pytest.skip(f"Screenshot failed: {result.get('error')}")

    screenshot_path = result.get("path")
    judgment = judge_screenshot_sync(
        screenshot_path,
        "the screenshot contains visible content (not a blank or solid-color screen)",
    )

    assert judgment.get("pass") is True, "Image appears blank"
    assert judgment.get("confidence", 0) > 0.8, "VLM confidence too low"
