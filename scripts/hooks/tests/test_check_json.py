"""Tests for check-json.py pre-commit hook."""
import json
import sys
import subprocess
from pathlib import Path

import pytest

# Get hooks directory from conftest
HOOKS_DIR = Path(__file__).parent.parent


class TestCheckJson:
    """Test suite for JSON validation hook."""

    def test_valid_json_file(self, change_to_temp_dir):
        """Valid JSON file should pass validation."""
        # Arrange
        json_file = change_to_temp_dir / "valid.json"
        valid_data = {"key": "value", "nested": {"count": 42}}
        json_file.write_text(json.dumps(valid_data))

        # Act
        check_json_script = HOOKS_DIR / "check-json.py"
        result = subprocess.run(
            [sys.executable, str(check_json_script)],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0
        assert "OK" in result.stdout

    def test_invalid_json_syntax(self, change_to_temp_dir):
        """Malformed JSON should fail validation."""
        # Arrange
        json_file = change_to_temp_dir / "invalid.json"
        json_file.write_text('{"key": "value",}')  # Trailing comma is invalid

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
        assert "invalid.json" in result.stderr

    def test_empty_json_object(self, change_to_temp_dir):
        """Empty JSON object should pass validation."""
        # Arrange
        json_file = change_to_temp_dir / "empty.json"
        json_file.write_text("{}")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_json_array(self, change_to_temp_dir):
        """JSON array should pass validation."""
        # Arrange
        json_file = change_to_temp_dir / "array.json"
        json_file.write_text(json.dumps([1, 2, 3, "four"]))

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_json_with_unicode(self, change_to_temp_dir):
        """JSON with unicode characters should pass validation."""
        # Arrange
        json_file = change_to_temp_dir / "unicode.json"
        json_file.write_text(json.dumps({"emoji": "🎮", "text": "中文"}))

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_multiple_files_mixed_validity(self, change_to_temp_dir):
        """Should fail if any file is invalid."""
        # Arrange
        (change_to_temp_dir / "valid1.json").write_text(json.dumps({"ok": True}))
        (change_to_temp_dir / "valid2.json").write_text(json.dumps({"ok": True}))
        (change_to_temp_dir / "invalid.json").write_text('{"bad":}')

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
        assert "invalid.json" in result.stderr

    def test_skip_packages_lock_json(self, change_to_temp_dir):
        """packages.lock.json should be skipped."""
        # Arrange
        (change_to_temp_dir / "packages.lock.json").write_text('{"bad":}')
        (change_to_temp_dir / "valid.json").write_text(json.dumps({"ok": True}))

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0
        assert "packages.lock.json" not in result.stdout

    def test_skip_fuzzcorpus_directory(self, change_to_temp_dir):
        """Files in FuzzCorpus should be skipped."""
        # Arrange
        fuzzcorpus_dir = change_to_temp_dir / "FuzzCorpus"
        fuzzcorpus_dir.mkdir()
        (fuzzcorpus_dir / "seed.json").write_text('{"bad":}')
        (change_to_temp_dir / "valid.json").write_text(json.dumps({"ok": True}))

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-json.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0
        assert "seed.json" not in result.stdout
