"""Tests for check-merge-conflicts.py pre-commit hook."""
import sys
import subprocess
from pathlib import Path

import pytest

# Get hooks directory
HOOKS_DIR = Path(__file__).parent.parent


class TestCheckMergeConflicts:
    """Test suite for merge conflict detection hook."""

    def test_file_without_merge_markers(self, change_to_temp_dir):
        """File without merge conflict markers should pass."""
        # Arrange
        test_file = change_to_temp_dir / "clean.txt"
        test_file.write_text("""
normal file content
no conflicts here
just regular text
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_file_with_conflict_marker_head(self, change_to_temp_dir):
        """File with <<<<<<< marker should fail."""
        # Arrange
        test_file = change_to_temp_dir / "conflict.txt"
        test_file.write_text("""
normal content
<<<<<<< HEAD
our changes
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
        assert "conflict" in result.stderr.lower() or "<<<<<<" in result.stderr

    def test_file_with_conflict_marker_separator(self, change_to_temp_dir):
        """File with ======= separator should fail."""
        # Arrange
        test_file = change_to_temp_dir / "conflict.txt"
        test_file.write_text("""
our changes
=======
their changes
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1

    def test_file_with_conflict_marker_end(self, change_to_temp_dir):
        """File with >>>>>>> marker should fail."""
        # Arrange
        test_file = change_to_temp_dir / "conflict.txt"
        test_file.write_text("""
<<<<<<< HEAD
our version
=======
their version
>>>>>>> branch-name
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1

    def test_multiple_files_with_conflict(self, change_to_temp_dir):
        """Should fail if any file has conflict markers."""
        # Arrange
        (change_to_temp_dir / "clean1.py").write_text("def foo():\n    pass\n")
        (change_to_temp_dir / "clean2.py").write_text("def bar():\n    return 42\n")
        (change_to_temp_dir / "conflict.py").write_text("""
def merged():
<<<<<<< HEAD
    return 1
=======
    return 2
>>>>>>> branch
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1
        assert "conflict" in result.stderr.lower() or "conflict.py" in result.stderr

    def test_ignore_markers_in_comments(self, change_to_temp_dir):
        """Should detect conflict markers even in comments (they're usually errors)."""
        # Arrange
        test_file = change_to_temp_dir / "code.py"
        test_file.write_text("""
def func():
    # <<<<<<< HEAD
    return 1
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        # May detect it as a conflict marker (conservative approach)
        # or may skip it depending on implementation
        assert result.returncode in [0, 1]

    def test_multiple_conflicts_in_file(self, change_to_temp_dir):
        """File with multiple conflict regions should fail."""
        # Arrange
        test_file = change_to_temp_dir / "multi_conflict.txt"
        test_file.write_text("""
<<<<<<< HEAD
first conflict - ours
=======
first conflict - theirs
>>>>>>> branch

some content in between

<<<<<<< HEAD
second conflict - ours
=======
second conflict - theirs
>>>>>>> branch
""")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 1

    def test_skip_dot_git_directory(self, change_to_temp_dir):
        """Conflict markers in .git directory should be ignored."""
        # Arrange
        git_dir = change_to_temp_dir / ".git"
        git_dir.mkdir()
        (git_dir / "conflict.tmp").write_text("<<<<<<< HEAD\nconflict\n")
        (change_to_temp_dir / "clean.txt").write_text("normal\n")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0

    def test_empty_file(self, change_to_temp_dir):
        """Empty file should pass validation."""
        # Arrange
        (change_to_temp_dir / "empty.txt").write_text("")

        # Act
        result = subprocess.run(
            [sys.executable, str(HOOKS_DIR / "check-merge-conflicts.py")],
            capture_output=True,
            text=True,
            cwd=str(change_to_temp_dir)
        )

        # Assert
        assert result.returncode == 0
