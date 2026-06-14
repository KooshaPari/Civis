"""Tests for the VLM judge CLI surface."""

from __future__ import annotations

import json
from pathlib import Path

from . import vlm_judge


def test_main_prints_json_and_exits_zero_on_pass(tmp_path, capsys, monkeypatch):
    """The CLI should emit machine-readable JSON and return success on a pass."""
    screenshot = tmp_path / "judge.png"
    screenshot.write_bytes(b"fake png bytes")

    def fake_judge(screenshot_path: str | Path, assertion: str, model: str = ""):
        assert Path(screenshot_path) == screenshot
        assert assertion == "The menu is visible"
        assert model == "claude-haiku-4-5-20251001"
        return {"pass": True, "confidence": 0.94, "reason": "ok"}

    monkeypatch.setattr(vlm_judge, "judge_screenshot_sync", fake_judge)

    exit_code = vlm_judge.main([str(screenshot), "The menu is visible"])

    assert exit_code == 0
    output = json.loads(capsys.readouterr().out)
    assert output == {"confidence": 0.94, "pass": True, "reason": "ok"}


def test_main_prints_json_and_exits_one_on_fail(tmp_path, capsys, monkeypatch):
    """The CLI should return a non-zero code when the judgment fails."""
    screenshot = tmp_path / "judge.png"
    screenshot.write_bytes(b"fake png bytes")

    def fake_judge(screenshot_path: str | Path, assertion: str, model: str = ""):
        assert Path(screenshot_path) == screenshot
        assert assertion == "The menu is visible"
        return {"pass": False, "confidence": 0.21, "reason": "not visible"}

    monkeypatch.setattr(vlm_judge, "judge_screenshot_sync", fake_judge)

    exit_code = vlm_judge.main([str(screenshot), "The menu is visible"])

    assert exit_code == 1
    output = json.loads(capsys.readouterr().out)
    assert output["pass"] is False
    assert output["confidence"] == 0.21
    assert output["reason"] == "not visible"
