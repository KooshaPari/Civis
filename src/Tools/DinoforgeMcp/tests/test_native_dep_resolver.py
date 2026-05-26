"""Tests for native_dep_resolver — env-var / paths.json / fallback precedence.

Mirrors src/Tests/NativeDepResolverTests.cs (Task #197).
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from unittest import mock

import pytest

from dinoforge_mcp import native_dep_resolver
from dinoforge_mcp.native_dep_resolver import NativeDepNotFound


TEST_ENV = "DINOFORGE_TEST_NATIVE_DEP_PY"


@pytest.fixture(autouse=True)
def _clear_env(monkeypatch):
    """Ensure the test env var doesn't leak between cases."""
    monkeypatch.delenv(TEST_ENV, raising=False)
    yield


def _touch(p: Path) -> Path:
    p.write_text("test")
    return p


def test_env_var_pointing_to_existing_file_takes_precedence(tmp_path, monkeypatch):
    env_file = _touch(tmp_path / "env.bin")
    fallback = _touch(tmp_path / "fallback.bin")
    monkeypatch.setenv(TEST_ENV, str(env_file))

    # Force installer paths to a non-existent location so it's skipped
    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file",
                        lambda: tmp_path / "no-such-paths.json")

    result = native_dep_resolver.resolve(
        key="test_env_precedence",
        env_var=TEST_ENV,
        hardcoded_fallbacks=[str(fallback)],
        description="test dep",
    )
    assert result == str(env_file), "env var should win when its file exists"


def test_env_var_pointing_to_missing_file_falls_through(tmp_path, monkeypatch):
    fallback = _touch(tmp_path / "fallback.bin")
    monkeypatch.setenv(TEST_ENV, str(tmp_path / "definitely-missing.exe"))
    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file",
                        lambda: tmp_path / "no-such-paths.json")

    result = native_dep_resolver.resolve(
        key="test_env_missing",
        env_var=TEST_ENV,
        hardcoded_fallbacks=[str(fallback)],
        description="test dep",
    )
    assert result == str(fallback), "missing env-var file should fall through"


def test_installer_paths_json_is_consulted(tmp_path, monkeypatch):
    real_target = _touch(tmp_path / "from-installer.bin")
    paths_json = tmp_path / "paths.json"
    paths_json.write_text(json.dumps({"my_key": str(real_target)}))

    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file", lambda: paths_json)
    # No env var, no fallback that exists
    result = native_dep_resolver.resolve(
        key="my_key",
        env_var=TEST_ENV,
        hardcoded_fallbacks=[str(tmp_path / "nope.exe")],
        description="test dep",
    )
    assert result == str(real_target), "installer paths.json should resolve when env unset"


def test_all_probes_miss_raises_loud_error(tmp_path, monkeypatch):
    monkeypatch.setenv(TEST_ENV, str(tmp_path / "ghost-env.exe"))
    bogus_fallback = str(tmp_path / "ghost-fallback.exe")
    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file",
                        lambda: tmp_path / "no-such-paths.json")

    with pytest.raises(NativeDepNotFound) as exc_info:
        native_dep_resolver.resolve(
            key="test_all_miss",
            env_var=TEST_ENV,
            hardcoded_fallbacks=[bogus_fallback],
            description="imaginary binary",
        )

    msg = str(exc_info.value)
    assert "imaginary binary" in msg
    assert TEST_ENV in msg
    # Fallback path appears via repr(list) inside the message — match the basename instead
    # of the raw string to avoid Windows backslash-escaping noise.
    assert os.path.basename(bogus_fallback) in msg
    assert "test_all_miss" in msg


def test_hardcoded_fallback_first_existing_wins(tmp_path, monkeypatch):
    bogus = str(tmp_path / "missing1.exe")
    real = _touch(tmp_path / "real-fallback.bin")
    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file",
                        lambda: tmp_path / "no-such-paths.json")

    result = native_dep_resolver.resolve(
        key="test_fallback",
        env_var=TEST_ENV,
        hardcoded_fallbacks=[bogus, str(real)],
        description="test dep",
    )
    assert result == str(real)


def test_try_resolve_returns_none_on_miss(tmp_path, monkeypatch):
    monkeypatch.setattr(native_dep_resolver, "_installer_paths_file",
                        lambda: tmp_path / "no-such-paths.json")

    result = native_dep_resolver.try_resolve(
        key="test_try_miss",
        env_var=TEST_ENV,
        hardcoded_fallbacks=[str(tmp_path / "nope.exe")],
    )
    assert result is None
