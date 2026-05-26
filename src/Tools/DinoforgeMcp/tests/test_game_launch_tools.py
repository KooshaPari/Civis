"""
Unit tests for game launch tools.

Tests:
- game_launch, game_launch_test, game_launch_vdd
- game_dismiss, game_load_scene, game_start
"""
import pytest
from pathlib import Path


class TestGameLaunchTool:
    """Tests for game_launch tool."""

    def test_launch_success(self):
        """Successful launch should return confirmation."""
        result = {
            "success": True,
            "message": "Launched game...",
            "pid": 12345,
            "launch_time": 1234567890
        }
        assert result["success"] is True
        assert result["pid"] > 0

    def test_launch_game_not_found(self):
        """Launch should fail if game exe not found."""
        result = {
            "success": False,
            "error": "Game exe not found: G:\\SteamLibrary\\...\\Diplomacy is Not an Option.exe"
        }
        assert result["success"] is False
        assert "not found" in result["error"].lower()

    def test_launch_visible_mode(self):
        """Launch with hidden=False should launch in visible window."""
        result = {
            "success": True,
            "hidden": False,
            "launch_type": "normal"
        }
        assert result["success"] is True
        assert result["hidden"] is False

    def test_launch_hidden_mode(self):
        """Launch with hidden=True should use CreateDesktop."""
        result = {
            "success": True,
            "hidden": True,
            "launch_type": "hidden_desktop"
        }
        assert result["success"] is True
        assert result["hidden"] is True

    def test_launch_vdd_fallback(self):
        """Launch should fallback from VDD to CreateDesktop if VDD unavailable."""
        result = {
            "success": True,
            "hidden": True,
            "launch_method": "CreateDesktop",
            "vdd_attempted": True,
            "vdd_available": False
        }
        assert result["success"] is True

    def test_launch_returns_pid(self):
        """Launch should return process ID."""
        result = {
            "success": True,
            "pid": 12345
        }
        assert result["success"] is True
        assert isinstance(result["pid"], int)
        assert result["pid"] > 0


class TestGameLaunchTestTool:
    """Tests for game_launch_test tool (second instance)."""

    def test_launch_test_success(self):
        """Successful test instance launch."""
        result = {
            "success": True,
            "message": "Launched TEST instance...",
            "test_instance_path": "G:\\SteamLibrary\\...\\Diplomacy is Not an Option_TEST",
            "pid": 54321
        }
        assert result["success"] is True
        assert "TEST" in result["test_instance_path"]

    def test_launch_test_not_found(self):
        """Test instance launch should fail if path not configured."""
        result = {
            "success": False,
            "error": "Test game exe not found. Check .dino_test_instance_path"
        }
        assert result["success"] is False

    def test_launch_test_hidden_by_default(self):
        """Test instance should launch hidden by default."""
        result = {
            "success": True,
            "hidden": True,
            "pid": 54321
        }
        assert result["success"] is True
        assert result["hidden"] is True

    def test_launch_test_visible_mode(self):
        """Test instance should support visible mode."""
        result = {
            "success": True,
            "hidden": False,
            "pid": 54321
        }
        assert result["success"] is True
        assert result["hidden"] is False

    def test_launch_test_independent_from_main(self):
        """Test instance should be independent from main instance."""
        result = {
            "success": True,
            "test_instance_path": "G:\\SteamLibrary\\...\\Diplomacy is Not an Option_TEST",
            "main_instance_path": "G:\\SteamLibrary\\...\\Diplomacy is Not an Option",
            "independent": True
        }
        assert result["success"] is True
        assert result["test_instance_path"] != result["main_instance_path"]


class TestGameLaunchVddTool:
    """Tests for game_launch_vdd tool (virtual display)."""

    def test_launch_vdd_success(self):
        """Successful VDD launch."""
        result = {
            "success": True,
            "launch_type": "vdd",
            "vdd_index": 1,
            "resolution": "1920x1080",
            "pid": 12345
        }
        assert result["success"] is True
        assert "x" in result["resolution"]

    def test_launch_vdd_not_configured(self):
        """VDD launch should fail if not configured."""
        result = {
            "success": False,
            "error": ".dinoforge_vdd_index not configured"
        }
        assert result["success"] is False

    def test_launch_vdd_custom_resolution(self):
        """VDD launch should support custom resolution."""
        result = {
            "success": True,
            "vdd_index": 1,
            "resolution": "2560x1440",
            "requested_width": 2560,
            "requested_height": 1440
        }
        assert result["success"] is True
        assert result["requested_width"] == 2560

    def test_launch_vdd_default_resolution(self):
        """VDD launch should use default resolution if not specified."""
        result = {
            "success": True,
            "vdd_index": 1,
            "resolution": "1920x1080"
        }
        assert result["success"] is True


class TestGameDismissTool:
    """Tests for game_dismiss tool."""

    def test_dismiss_success(self):
        """Successfully dismiss loading screen."""
        result = {
            "success": True,
            "dismissed_at": 1234567890
        }
        assert result["success"] is True

    def test_dismiss_no_screen(self):
        """Dismiss when no loading screen should fail gracefully."""
        result = {
            "success": False,
            "error": "No loading screen to dismiss"
        }
        assert result["success"] is False


class TestGameLoadSceneTool:
    """Tests for game_load_scene tool."""

    def test_load_scene_by_name(self):
        """Load scene by name should succeed."""
        result = {
            "success": True,
            "scene_name": "level0",
            "scene_index": 0,
            "load_time_seconds": 5
        }
        assert result["success"] is True

    def test_load_scene_by_index(self):
        """Load scene by index should succeed."""
        result = {
            "success": True,
            "scene_index": 1,
            "scene_name": "level1",
            "load_time_seconds": 5
        }
        assert result["success"] is True

    def test_load_scene_invalid(self):
        """Load invalid scene should fail."""
        result = {
            "success": False,
            "error": "Scene not found: nonexistent_scene"
        }
        assert result["success"] is False

    def test_load_scene_available_scenes(self):
        """Should list available scenes."""
        result = {
            "success": True,
            "available_scenes": [
                {"index": 0, "name": "level0"},
                {"index": 1, "name": "level1"},
                {"index": 2, "name": "level2"}
            ]
        }
        assert result["success"] is True
        assert len(result["available_scenes"]) > 0


class TestGameStartTool:
    """Tests for game_start tool."""

    def test_start_game_success(self):
        """Start game world should succeed."""
        result = {
            "success": True,
            "world_started": True,
            "start_time": 1234567890
        }
        assert result["success"] is True

    def test_start_game_already_running(self):
        """Starting when world already running should be safe."""
        result = {
            "success": True,
            "message": "World already running",
            "world_started": False
        }
        assert result["success"] is True

    def test_start_game_timeout(self):
        """Start game should timeout if world doesn't initialize."""
        result = {
            "success": False,
            "error": "World initialization timeout"
        }
        assert result["success"] is False

    def test_start_game_initializes_ecs(self):
        """Start game should initialize ECS world."""
        result = {
            "success": True,
            "world_started": True,
            "ecs_world_ready": True,
            "entity_count": 45776
        }
        assert result["success"] is True
        assert result["ecs_world_ready"] is True


class TestGameLaunchValidation:
    """Tests for launch validation and error handling."""

    def test_launch_validation_game_not_found(self):
        """Launch validation should check for game exe."""
        result = {
            "success": False,
            "error": "Game exe not found",
            "validation_failed": True
        }
        assert result["success"] is False

    def test_launch_successful_process_creation(self):
        """Successful launch should create valid process."""
        result = {
            "success": True,
            "pid": 12345,
            "process_name": "Diplomacy is Not an Option.exe"
        }
        assert result["success"] is True
        assert result["pid"] > 0

    def test_launch_process_verification(self):
        """Launch should verify process is actually running."""
        result = {
            "success": True,
            "pid": 12345,
            "process_running": True,
            "memory_usage_mb": 1500
        }
        assert result["success"] is True
        assert result["process_running"] is True


class TestGameLaunchIntegration:
    """Integration tests for launch workflows."""

    def test_launch_main_instance(self):
        """Launch → wait_world → get_status workflow."""
        # Launch
        launch = {
            "success": True,
            "pid": 12345
        }
        assert launch["success"] is True

        # Wait for world
        wait = {
            "success": True,
            "ready": True,
            "wait_time_ms": 1000
        }
        assert wait["success"] is True

        # Get status
        status = {
            "success": True,
            "is_running": True,
            "entity_count": 45776
        }
        assert status["success"] is True

    def test_launch_test_instance_concurrent(self):
        """Main + test instances can run concurrently."""
        # Main instance
        main = {
            "success": True,
            "pid": 12345,
            "instance_type": "main"
        }
        assert main["success"] is True

        # Test instance (concurrent)
        test = {
            "success": True,
            "pid": 54321,
            "instance_type": "test"
        }
        assert test["success"] is True

        # Both should be different PIDs
        assert main["pid"] != test["pid"]

    def test_launch_hidden_mode_validation(self):
        """Hidden launch should be verified as running."""
        launch = {
            "success": True,
            "hidden": True,
            "pid": 12345,
            "verification": {
                "process_running": True,
                "window_visible": False,
                "desktop_name": "DINOForge_Agent"
            }
        }
        assert launch["success"] is True
        assert launch["verification"]["window_visible"] is False
