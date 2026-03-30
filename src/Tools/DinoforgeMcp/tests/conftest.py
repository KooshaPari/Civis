"""Pytest configuration and shared fixtures for MCP server tests."""
import os
import sys
from pathlib import Path
import pytest

# Add parent directories to path for imports
mcp_dir = Path(__file__).parent.parent
sys.path.insert(0, str(mcp_dir))


@pytest.fixture
def mock_game_process():
    """Fixture for mocking a game process."""
    class MockGameProcess:
        def __init__(self):
            self.pid = 12345
            self.returncode = None

        def poll(self):
            return self.returncode

        def terminate(self):
            self.returncode = 0

        def kill(self):
            self.returncode = 1

    return MockGameProcess()


@pytest.fixture
def temp_game_state():
    """Fixture for temporary game state data."""
    return {
        "is_running": False,
        "entity_count": 0,
        "loaded_packs": [],
        "scene": "unknown"
    }
