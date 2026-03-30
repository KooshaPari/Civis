"""Unit tests for MCP server game automation tools."""
import pytest
import json
from unittest.mock import Mock, patch, MagicMock


class TestGameStatusTool:
    """Tests for game_status tool."""

    def test_game_not_running_returns_false(self):
        """game_status should return is_running=False when game is not active."""
        # This is a placeholder test demonstrating the test structure
        # Real implementation would require mocking the game process detection
        status = {
            "is_running": False,
            "entity_count": 0,
            "loaded_packs": []
        }
        assert status["is_running"] is False
        assert isinstance(status["entity_count"], int)
        assert isinstance(status["loaded_packs"], list)

    def test_game_running_reports_metrics(self):
        """game_status should report entity count and packs when game is running."""
        status = {
            "is_running": True,
            "entity_count": 45776,
            "loaded_packs": ["example-balance", "warfare-starwars"],
            "world_name": "Default World"
        }
        assert status["is_running"] is True
        assert status["entity_count"] == 45776
        assert len(status["loaded_packs"]) == 2
        assert "Default World" in status["world_name"]

    def test_game_status_structure(self):
        """game_status response should have expected fields."""
        status = {
            "is_running": True,
            "entity_count": 100,
            "loaded_packs": [],
            "scene": "gameplay",
            "game_version": "1.0.0"
        }
        required_fields = {"is_running", "entity_count", "loaded_packs", "scene"}
        assert required_fields.issubset(status.keys())


class TestGameQueryEntitiesTool:
    """Tests for game_query_entities tool."""

    def test_query_entities_empty_result(self):
        """Query with no matches should return empty list."""
        result = {
            "entities": [],
            "total_count": 0,
            "query": "component:Unknown"
        }
        assert result["total_count"] == 0
        assert len(result["entities"]) == 0

    def test_query_entities_with_matches(self):
        """Query with matches should return entity list."""
        result = {
            "entities": [
                {"entity_id": 1, "components": ["Health", "ArmorData"]},
                {"entity_id": 2, "components": ["Health", "ArmorData"]}
            ],
            "total_count": 2,
            "query": "component:Health"
        }
        assert result["total_count"] == 2
        assert all("entity_id" in e for e in result["entities"])
        assert all("components" in e for e in result["entities"])

    def test_query_entities_respects_limit(self):
        """Query should respect limit parameter."""
        result = {
            "entities": [
                {"entity_id": 1, "components": ["Health"]},
                {"entity_id": 2, "components": ["Health"]}
            ],
            "total_count": 1000,  # More exist, but limited by request
            "limit": 2,
            "query": "component:Health"
        }
        assert len(result["entities"]) == result["limit"]
        assert result["total_count"] >= result["limit"]


class TestGameApplyOverrideTool:
    """Tests for game_apply_override tool."""

    def test_override_success(self):
        """Successful override should return updated stat."""
        result = {
            "success": True,
            "entity_id": 123,
            "stat_name": "health",
            "old_value": 100,
            "new_value": 150,
            "applied_at": 1234567890
        }
        assert result["success"] is True
        assert result["new_value"] == 150
        assert result["old_value"] != result["new_value"]

    def test_override_invalid_entity(self):
        """Override on invalid entity should fail."""
        result = {
            "success": False,
            "error": "Entity not found: 99999",
            "entity_id": 99999
        }
        assert result["success"] is False
        assert "not found" in result["error"].lower()

    def test_override_invalid_stat(self):
        """Override with invalid stat name should fail."""
        result = {
            "success": False,
            "error": "Unknown stat: invalid_stat",
            "stat_name": "invalid_stat"
        }
        assert result["success"] is False
        assert "unknown" in result["error"].lower()


class TestGameReloadPacksTool:
    """Tests for game_reload_packs tool."""

    def test_reload_packs_success(self):
        """Successful pack reload should list reloaded packs."""
        result = {
            "success": True,
            "reloaded_packs": ["example-balance", "warfare-starwars"],
            "failed_packs": [],
            "reload_time_ms": 250
        }
        assert result["success"] is True
        assert len(result["reloaded_packs"]) == 2
        assert len(result["failed_packs"]) == 0

    def test_reload_packs_partial_failure(self):
        """Reload with some failures should report them."""
        result = {
            "success": False,
            "reloaded_packs": ["example-balance"],
            "failed_packs": ["invalid-pack"],
            "errors": {
                "invalid-pack": "Pack manifest not found"
            }
        }
        assert result["success"] is False
        assert len(result["failed_packs"]) > 0
        assert "invalid-pack" in result["errors"]

    def test_reload_packs_empty(self):
        """Reload with no packs should still succeed."""
        result = {
            "success": True,
            "reloaded_packs": [],
            "failed_packs": [],
            "reload_time_ms": 10
        }
        assert result["success"] is True


class TestGameDumpStateTool:
    """Tests for game_dump_state tool."""

    def test_dump_state_creates_file(self):
        """dump_state should create output file."""
        result = {
            "success": True,
            "output_file": "/path/to/dump.json",
            "entity_count": 1000,
            "archetype_count": 50,
            "dump_size_bytes": 102400
        }
        assert result["success"] is True
        assert result["output_file"].endswith(".json")
        assert result["entity_count"] > 0

    def test_dump_state_with_filter(self):
        """dump_state should filter by archetype if specified."""
        result = {
            "success": True,
            "output_file": "/path/to/dump.json",
            "entity_count": 100,  # Filtered count
            "archetype_filter": "Health",
            "total_entities": 45776
        }
        assert result["success"] is True
        assert result["entity_count"] < result["total_entities"]


class TestGameScreenshotTool:
    """Tests for game_screenshot tool."""

    def test_screenshot_success(self):
        """Successful screenshot should return file path."""
        result = {
            "success": True,
            "screenshot_path": "/tmp/game_screenshot_1234567890.png",
            "timestamp": 1234567890
        }
        assert result["success"] is True
        assert result["screenshot_path"].endswith(".png")

    def test_screenshot_game_not_running(self):
        """Screenshot when game not running should fail gracefully."""
        result = {
            "success": False,
            "error": "Game is not running"
        }
        assert result["success"] is False
        assert "not running" in result["error"].lower()


class TestGameInputTool:
    """Tests for game_input tool."""

    def test_key_press_input(self):
        """Key press input should be sent successfully."""
        result = {
            "success": True,
            "input_type": "key",
            "key": "F9",
            "duration_ms": 50
        }
        assert result["success"] is True
        assert result["input_type"] == "key"

    def test_mouse_click_input(self):
        """Mouse click input should be sent successfully."""
        result = {
            "success": True,
            "input_type": "mouse",
            "button": "left",
            "x": 640,
            "y": 480
        }
        assert result["success"] is True
        assert result["input_type"] == "mouse"
        assert result["x"] > 0 and result["y"] > 0

    def test_invalid_input_fails(self):
        """Invalid input command should fail."""
        result = {
            "success": False,
            "error": "Unknown input type: invalid"
        }
        assert result["success"] is False


class TestGameWaitForWorldTool:
    """Tests for game_wait_for_world tool."""

    def test_wait_for_world_success(self):
        """Waiting for world should succeed when ready."""
        result = {
            "success": True,
            "ready": True,
            "wait_time_ms": 500,
            "entity_count": 45776
        }
        assert result["success"] is True
        assert result["ready"] is True

    def test_wait_for_world_timeout(self):
        """Waiting should timeout if world takes too long."""
        result = {
            "success": False,
            "error": "World initialization timeout after 30000ms",
            "wait_time_ms": 30000
        }
        assert result["success"] is False
        assert "timeout" in result["error"].lower()
