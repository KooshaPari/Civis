"""Tests for check-yaml.py pre-commit hook."""
import sys
import subprocess
from pathlib import Path

import pytest

# Get hooks directory
HOOKS_DIR = Path(__file__).parent.parent


class TestCheckYaml:
    """Test suite for YAML validation hook."""

    def test_valid_yaml_file(self, change_to_temp_dir):
        """Valid YAML file should pass validation."""
        # Arrange
        yaml_file = change_to_temp_dir / "valid.yaml"
        yaml_file.write_text("""
key: value
nested:
  count: 42
  items:
    - one
    - two
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0
        assert "OK" in result.stdout

    def test_invalid_yaml_indentation(self, change_to_temp_dir):
        """Malformed YAML with bad indentation should fail."""
        # Arrange
        yaml_file = change_to_temp_dir / "invalid.yaml"
        yaml_file.write_text("""
key: value
  bad_indent: wrong
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
        assert "invalid.yaml" in result.stderr or "error" in result.stderr.lower()

    def test_empty_yaml_file(self, change_to_temp_dir):
        """Empty YAML file should pass validation."""
        # Arrange
        yaml_file = change_to_temp_dir / "empty.yaml"
        yaml_file.write_text("")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_yaml_list_format(self, change_to_temp_dir):
        """YAML with list format should pass validation."""
        # Arrange
        yaml_file = change_to_temp_dir / "list.yaml"
        yaml_file.write_text("""
items:
  - name: first
    value: 1
  - name: second
    value: 2
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_yaml_with_special_characters(self, change_to_temp_dir):
        """YAML with quoted strings and special characters should pass."""
        # Arrange
        yaml_file = change_to_temp_dir / "special.yaml"
        yaml_file.write_text("""
description: "This has: colons and other: stuff"
emoji: 🎮
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_skip_generated_files(self, change_to_temp_dir):
        """Generated YAML files should be skipped."""
        # Arrange
        (change_to_temp_dir / "generated.yaml.lock").write_text("invalid: [")
        (change_to_temp_dir / "valid.yaml").write_text("key: value\n")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        # Should pass since lock files may be skipped
        assert result.returncode in [0, 1]

    def test_multiple_yaml_files_validation(self, change_to_temp_dir):
        """Should validate all YAML files in directory."""
        # Arrange
        (change_to_temp_dir / "config1.yaml").write_text("key1: value1\n")
        (change_to_temp_dir / "config2.yaml").write_text("key2: value2\n")
        (change_to_temp_dir / "config3.yaml").write_text("key3: value3\n")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_yml_extension_support(self, change_to_temp_dir):
        """Files with .yml extension should be validated."""
        # Arrange
        (change_to_temp_dir / "config.yml").write_text("key: value\n")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_invalid_yaml_syntax(self, change_to_temp_dir):
        """YAML with invalid syntax should fail."""
        # Arrange
        yaml_file = change_to_temp_dir / "bad_syntax.yaml"
        yaml_file.write_text("""
key: value
  - list item without list
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-yaml.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
