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


@pytest.fixture
def mock_game_status_response():
    """Fixture for mocking game status response."""
    return {
        "is_running": True,
        "entity_count": 45776,
        "loaded_packs": ["example-balance", "warfare-starwars"],
        "world_name": "Default World",
        "scene": "gameplay",
        "game_version": "1.0.0"
    }


@pytest.fixture
def mock_entities_response():
    """Fixture for mocking game entities response."""
    return {
        "success": True,
        "entities": [
            {"entity_id": 1, "components": ["Health", "ArmorData"]},
            {"entity_id": 2, "components": ["Health", "ArmorData"]}
        ],
        "total_count": 2,
        "query": "component:Health"
    }


@pytest.fixture
def mock_debug_log(tmp_path):
    """Fixture for mocking debug log file."""
    log_file = tmp_path / "dinoforge_debug.log"
    log_content = """[2026-04-05 10:15:30] DINOForge Runtime initialized
[2026-04-05 10:15:31] ECS bridge connected: 45776 entities
[2026-04-05 10:15:32] Loaded pack: example-balance (v0.1.0)
[2026-04-05 10:15:33] Loaded pack: warfare-starwars (v0.2.0)
[2026-04-05 10:15:34] Asset swap registry initialized
"""
    log_file.write_text(log_content)
    return log_file


@pytest.fixture
def mock_catalog_json(tmp_path):
    """Fixture for mocking addressables catalog."""
    catalog_file = tmp_path / "catalog.json"
    catalog_content = """{
        "m_LocatorId": "Addressables",
        "m_InternalIdPrefix": "file://",
        "m_BucketDataSerializeIndex": [
            {"b": 0, "a": 0},
            {"b": 0, "a": 1}
        ],
        "m_BucketData": [
            {
                "m_Hash": "sw-rep-clone-trooper",
                "m_Bundles": ["sw-rep-clone-trooper"]
            },
            {
                "m_Hash": "example-unit-1",
                "m_Bundles": ["example-unit-1"]
            }
        ]
    }"""
    catalog_file.write_text(catalog_content)
    return catalog_file
