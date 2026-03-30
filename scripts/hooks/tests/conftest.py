"""Pytest configuration and shared fixtures for hook tests."""
import os
import sys
import tempfile
from pathlib import Path
import pytest

# Add parent directory to path so we can import hook modules
hooks_dir = Path(__file__).parent.parent
sys.path.insert(0, str(hooks_dir))

# Export hook scripts location for use in tests
HOOKS_DIR = hooks_dir


@pytest.fixture
def temp_dir():
    """Create a temporary directory for test files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def change_to_temp_dir(temp_dir):
    """Change to temp directory for the test and restore afterward."""
    original_cwd = os.getcwd()
    try:
        os.chdir(str(temp_dir))
        yield temp_dir
    finally:
        os.chdir(original_cwd)
