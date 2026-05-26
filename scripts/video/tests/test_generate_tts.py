"""Tests for scripts/video/generate_tts.py (SPEC-006 TTS step)."""
import builtins
import importlib.util
from pathlib import Path

import pytest

VIDEO_DIR = Path(__file__).parent.parent
GENERATE_TTS = VIDEO_DIR / "generate_tts.py"


def test_generate_tts_exists_with_cli_entrypoint():
    """generate_tts.py exists and exposes argparse + main."""
    assert GENERATE_TTS.is_file(), f"missing script: {GENERATE_TTS}"

    source = GENERATE_TTS.read_text(encoding="utf-8")
    assert "argparse" in source
    assert "async def main" in source
    assert 'if __name__ == "__main__"' in source


def test_import_fails_when_edge_tts_unavailable(monkeypatch):
    """Top-level edge_tts import fails without calling the network."""
    real_import = builtins.__import__

    def fake_import(name, globals=None, locals=None, fromlist=(), level=0):
        if name == "edge_tts":
            raise ImportError("No module named 'edge_tts'")
        return real_import(name, globals, locals, fromlist, level)

    monkeypatch.setattr(builtins, "__import__", fake_import)

    spec = importlib.util.spec_from_file_location(
        "generate_tts_import_probe", GENERATE_TTS
    )
    module = importlib.util.module_from_spec(spec)

    with pytest.raises(ImportError, match="edge_tts"):
        spec.loader.exec_module(module)
