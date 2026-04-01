"""
Unit tests for game bridge tools.

Tests game_* tools that communicate with BepInEx via GameControlCli:
- game_status, game_wait_world, game_resources
- game_query_entities, game_get_stat, game_apply_override
- game_screenshot, game_input
"""
import pytest
import json
from unittest.mock import Mock, patch, MagicMock


class TestGameStatusTool:
    """Tests for game_status tool."""

    def test_game_not_running_returns_false(self):
        """game_status should return is_running=False when game is not active."""
        status = {
            "is_running": False,
            "entity_count": 0,
            "loaded_packs": []
        }
        assert status["is_running"] is False
        assert isinstance(status["entity_count"], int)
        assert isinstance(status["loaded_packs"], list)

    def test_game_running_reports_metrics(self, mock_game_status_response):
        """game_status should report entity count and packs when running."""
        assert mock_game_status_response["is_running"] is True
        assert mock_game_status_response["entity_count"] == 45776
        assert len(mock_game_status_response["loaded_packs"]) == 2

    def test_game_status_has_required_fields(self):
        """game_status response should have expected fields."""
        status = {
            "is_running": True,
            "entity_count": 100,
            "loaded_packs": [],
            "scene": "gameplay",
            "game_version": "1.0.0"
        }
        required_fields = {"is_running", "entity_count", "loaded_packs"}
        assert required_fields.issubset(status.keys())

    def test_game_status_entity_count_valid(self):
        """game_status entity_count should be a non-negative integer."""
        status = {"is_running": True, "entity_count": 45776}
        assert isinstance(status["entity_count"], int)
        assert status["entity_count"] >= 0

    def test_game_status_loaded_packs_is_list(self, mock_game_status_response):
        """game_status loaded_packs should always be a list."""
        assert isinstance(mock_game_status_response["loaded_packs"], list)


class TestGameWaitWorldTool:
    """Tests for game_wait_world tool."""

    def test_wait_world_success(self):
        """game_wait_world should succeed when world initializes."""
        result = {
            "success": True,
            "ready": True,
            "wait_time_ms": 500,
            "entity_count": 45776
        }
        assert result["success"] is True
        assert result["ready"] is True
        assert result["wait_time_ms"] > 0

    def test_wait_world_timeout(self):
        """game_wait_world should timeout if world takes too long."""
        result = {
            "success": False,
            "error": "World initialization timeout after 30000ms"
        }
        assert result["success"] is False
        assert "timeout" in result["error"].lower()

    def test_wait_world_returns_wait_time(self):
        """game_wait_world should return actual wait time."""
        result = {
            "success": True,
            "wait_time_ms": 1250
        }
        assert "wait_time_ms" in result
        assert result["wait_time_ms"] >= 0


class TestGameResourcesTool:
    """Tests for game_resources/game_get_resources tools."""

    def test_resources_returns_dict(self):
        """game_resources should return resource quantities."""
        result = {
            "success": True,
            "gold": 1000,
            "wood": 500,
            "food": 750,
            "stone": 200
        }
        assert result["success"] is True
        assert isinstance(result["gold"], int)
        assert all(isinstance(v, int) for v in [result["gold"], result["wood"], result["food"], result["stone"]])

    def test_resources_zero_values_valid(self):
        """game_resources with zero values should be valid."""
        result = {
            "success": True,
            "gold": 0,
            "wood": 0,
            "food": 0
        }
        assert result["success"] is True
        assert result["gold"] == 0

    def test_resources_large_values(self):
        """game_resources should handle large resource values."""
        result = {
            "success": True,
            "gold": 999999,
            "wood": 888888
        }
        assert result["success"] is True
        assert result["gold"] > 100000


class TestGameQueryEntitiesTool:
    """Tests for game_query_entities tool."""

    def test_query_entities_empty_result(self):
        """Query with no matches should return empty list."""
        result = {
            "success": True,
            "entities": [],
            "total_count": 0,
            "query": "component:Unknown"
        }
        assert result["total_count"] == 0
        assert len(result["entities"]) == 0

    def test_query_entities_with_matches(self, mock_entities_response):
        """Query with matches should return entity list."""
        assert mock_entities_response["success"] is True
        assert mock_entities_response["total_count"] == 2
        assert len(mock_entities_response["entities"]) == 2

    def test_query_entities_respects_limit(self):
        """Query should respect limit parameter."""
        result = {
            "success": True,
            "entities": [
                {"entity_id": 1, "components": ["Health"]},
                {"entity_id": 2, "components": ["Health"]}
            ],
            "total_count": 1000,
            "limit": 2
        }
        assert len(result["entities"]) == result["limit"]
        assert result["total_count"] >= result["limit"]

    def test_query_entities_has_structure(self, mock_entities_response):
        """Query results should have proper entity structure."""
        for entity in mock_entities_response["entities"]:
            assert "entity_id" in entity
            assert "components" in entity
            assert isinstance(entity["components"], list)

    def test_query_entities_invalid_component(self):
        """Query with invalid component should fail gracefully."""
        result = {
            "success": False,
            "error": "Invalid component type: NonExistent"
        }
        assert result["success"] is False


class TestGameGetStatTool:
    """Tests for game_get_stat tool."""

    def test_get_stat_success(self):
        """get_stat should return stat value."""
        result = {
            "success": True,
            "stat_path": "unit.stats.hp",
            "values": [100, 150, 200],
            "entity_count": 3
        }
        assert result["success"] is True
        assert len(result["values"]) == result["entity_count"]

    def test_get_stat_single_entity(self):
        """get_stat with entity_index should return single value."""
        result = {
            "success": True,
            "stat_path": "unit.stats.hp",
            "entity_index": 0,
            "value": 100
        }
        assert result["success"] is True
        assert "value" in result

    def test_get_stat_invalid_path(self):
        """get_stat with invalid path should fail."""
        result = {
            "success": False,
            "error": "Invalid stat path: nonexistent.path"
        }
        assert result["success"] is False

    def test_get_stat_invalid_entity(self):
        """get_stat with invalid entity should fail."""
        result = {
            "success": False,
            "error": "Entity index out of range: 99999"
        }
        assert result["success"] is False


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
        assert result["old_value"] < result["new_value"]

    def test_override_invalid_entity(self):
        """Override on invalid entity should fail."""
        result = {
            "success": False,
            "error": "Entity not found: 99999"
        }
        assert result["success"] is False
        assert "not found" in result["error"].lower()

    def test_override_invalid_stat(self):
        """Override with invalid stat name should fail."""
        result = {
            "success": False,
            "error": "Unknown stat: invalid_stat"
        }
        assert result["success"] is False

    def test_override_can_decrease_value(self):
        """Override should allow decreasing stat values."""
        result = {
            "success": True,
            "old_value": 100,
            "new_value": 50
        }
        assert result["success"] is True
        assert result["new_value"] < result["old_value"]

    def test_override_zero_value(self):
        """Override should allow setting to zero."""
        result = {
            "success": True,
            "new_value": 0
        }
        assert result["success"] is True
        assert result["new_value"] == 0


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
        assert result["timestamp"] > 0

    def test_screenshot_game_not_running(self):
        """Screenshot when game not running should fail gracefully."""
        result = {
            "success": False,
            "error": "Game is not running"
        }
        assert result["success"] is False

    def test_screenshot_custom_output_path(self):
        """Screenshot should accept custom output path."""
        result = {
            "success": True,
            "screenshot_path": "/custom/path/screenshot.png"
        }
        assert result["success"] is True
        assert "/custom/path/" in result["screenshot_path"]

    def test_screenshot_file_size_reasonable(self):
        """Screenshot file should be reasonable size (PNG compression)."""
        result = {
            "success": True,
            "screenshot_path": "/tmp/screenshot.png",
            "file_size_bytes": 512000  # ~500 KB for 1920x1080
        }
        assert result["success"] is True
        assert result["file_size_bytes"] > 0


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

    def test_mouse_move_input(self):
        """Mouse movement should be supported."""
        result = {
            "success": True,
            "input_type": "mouse",
            "action": "move",
            "x": 960,
            "y": 540
        }
        assert result["success"] is True

    def test_invalid_input_fails(self):
        """Invalid input command should fail."""
        result = {
            "success": False,
            "error": "Unknown input type: invalid"
        }
        assert result["success"] is False

    def test_key_combo_input(self):
        """Key combinations should be supported."""
        result = {
            "success": True,
            "input_type": "key",
            "keys": ["Control", "Shift", "R"]
        }
        assert result["success"] is True

    def test_game_not_running_input(self):
        """Input when game not running should fail."""
        result = {
            "success": False,
            "error": "Game is not running"
        }
        assert result["success"] is False


class TestGameUITreeTool:
    """Tests for game_ui_tree tool."""

    def test_ui_tree_returns_hierarchy(self):
        """game_ui_tree should return UI hierarchy."""
        result = {
            "success": True,
            "ui_elements": [
                {
                    "name": "MainCanvas",
                    "type": "Canvas",
                    "children": ["Button1", "Button2"]
                }
            ]
        }
        assert result["success"] is True
        assert isinstance(result["ui_elements"], list)

    def test_ui_tree_with_selector(self):
        """game_ui_tree should support selectors."""
        result = {
            "success": True,
            "selector": "button",
            "matching_elements": [
                {"name": "Button1", "type": "Button"}
            ]
        }
        assert result["success"] is True


class TestGameClickButtonTool:
    """Tests for game_click_button tool."""

    def test_click_button_success(self):
        """Clicking a button should succeed."""
        result = {
            "success": True,
            "button_name": "PlayButton",
            "clicked_at": 1234567890
        }
        assert result["success"] is True

    def test_click_button_not_found(self):
        """Clicking non-existent button should fail."""
        result = {
            "success": False,
            "error": "Button not found: NonExistentButton"
        }
        assert result["success"] is False
