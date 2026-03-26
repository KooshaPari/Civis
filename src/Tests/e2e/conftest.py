"""
Pytest configuration for DINOForge E2E tests.

Provides fixtures for game status checks, screenshot management, and VLM judge setup.
"""
import os
import sys
import json
import tempfile
import subprocess
from pathlib import Path
from typing import Generator

import pytest
from dotenv import load_dotenv

# Load environment variables from repo root
REPO_ROOT = Path(__file__).parent.parent.parent.parent
if (REPO_ROOT / ".env").exists():
    load_dotenv(REPO_ROOT / ".env")

# Add MCP server to path for direct imports
MCP_SERVER_PATH = REPO_ROOT / "src/Tools/DinoforgeMcp"
if str(MCP_SERVER_PATH) not in sys.path:
    sys.path.insert(0, str(MCP_SERVER_PATH))


def is_game_running() -> bool:
    """Check if DINO game process is running."""
    try:
        # Windows: check for process via tasklist
        result = subprocess.run(
            ["tasklist"],
            capture_output=True,
            text=True,
            timeout=5
        )
        return "Diplomacy is Not an Option.exe" in result.stdout
    except Exception:
        return False


@pytest.fixture(scope="session")
def game_status():
    """Pytest session-scoped fixture to skip tests if game is not running."""
    if not is_game_running():
        pytest.skip("Game is not running. Start the game to run E2E tests.", allow_module_level=True)


@pytest.fixture
def screenshot_dir() -> Generator[Path, None, None]:
    """Fixture that provides a temp directory for screenshots, cleaned up after test."""
    temp_dir = Path(tempfile.gettempdir()) / "dinoforge_e2e_screenshots"
    temp_dir.mkdir(parents=True, exist_ok=True)
    yield temp_dir
    # Cleanup is optional — keep screenshots for debugging


@pytest.fixture
def screenshot_path(screenshot_dir) -> Generator[Path, None, None]:
    """Fixture that provides a path for a single screenshot."""
    path = screenshot_dir / f"screenshot_{os.getpid()}.png"
    yield path
    # Keep file for inspection


@pytest.fixture
def mcp_server():
    """Import and return the MCP server module for direct tool calls."""
    try:
        from dinoforge_mcp import server
        return server
    except ImportError as e:
        pytest.skip(f"Could not import MCP server: {e}")


@pytest.fixture
def anthropic_client():
    """Provide Anthropic client for VLM judge tests."""
    try:
        import anthropic
        return anthropic.Anthropic(api_key=os.getenv("ANTHROPIC_API_KEY"))
    except ImportError:
        pytest.skip("anthropic package not installed")
    except Exception as e:
        pytest.skip(f"Could not initialize Anthropic client: {e}")


def pytest_configure(config):
    """Register custom pytest markers."""
    config.addinivalue_line(
        "markers", "e2e: mark test as an end-to-end integration test"
    )
    config.addinivalue_line(
        "markers", "vlm: mark test as using VLM (vision language model) judgment"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow-running"
    )
