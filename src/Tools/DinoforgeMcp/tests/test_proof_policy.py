"""Tests for proof_policy module."""
from __future__ import annotations
import json
from pathlib import Path

import pytest

from dinoforge_mcp import proof_policy


REPO_ROOT = Path(__file__).resolve().parents[4]
POLICY_YAML = REPO_ROOT / "policies" / "proof-policy.yaml"


def test_default_policy_loads():
    p = proof_policy.load_policy(POLICY_YAML)
    assert p.version == "1.0"
    assert p.policy_id == "dinoforge-default-2026-04"
    assert "f9_overlay" in p.features
    assert "f10_modmenu" in p.features
    assert "pack_load" in p.features


def test_default_policy_forbids_anthropic_family():
    p = proof_policy.load_policy(POLICY_YAML)
    assert proof_policy.is_judge_forbidden("claude-haiku-4-5", p) is True
    assert proof_policy.is_judge_forbidden("claude-opus-4-7", p) is True
    assert proof_policy.is_judge_forbidden("codex-spark-5.3", p) is True
    assert proof_policy.is_judge_forbidden("anthropic-something", p) is True
    # Allowed
    assert proof_policy.is_judge_forbidden("moonshot-v1-8k", p) is False
    assert proof_policy.is_judge_forbidden("kimi-k2", p) is False


def test_empty_judge_treated_as_forbidden():
    p = proof_policy.load_policy(POLICY_YAML)
    assert proof_policy.is_judge_forbidden("", p) is True
    assert proof_policy.is_judge_forbidden(None, p) is True


def test_per_feature_artifacts_required():
    p = proof_policy.load_policy(POLICY_YAML)
    f9 = p.features["f9_overlay"]
    assert "validate_f9.png" in f9.required_artifacts
    assert "raw_f9.mp4" in f9.required_artifacts
    assert f9.require_external_judge is True


def test_load_policy_raises_on_missing_keys(tmp_path):
    bad = tmp_path / "bad.yaml"
    bad.write_text("nothing: useful\n")
    with pytest.raises(ValueError):
        proof_policy.load_policy(bad)


def test_glob_case_insensitive():
    p = proof_policy.load_policy(POLICY_YAML)
    assert proof_policy.is_judge_forbidden("CLAUDE-haiku", p) is True
    assert proof_policy.is_judge_forbidden("Claude-Sonnet-4-6", p) is True


# ---------------------------------------------------------------------------
# Evaluator tests (Phase 2 — spec section 8)
# ---------------------------------------------------------------------------

from datetime import datetime, timedelta, timezone


def _make_judge(feature_id: str,
                model: str = "moonshot-v1-128k",
                endpoint: str = "https://api.moonshot.cn/v1",
                ts: str | None = None) -> dict:
    if ts is None:
        ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    return {
        "kind": "JudgeReceipt",
        "timestamp_utc": ts,
        "subject": {
            "feature_id": feature_id,
            "judge_model": model,
            "judge_endpoint": endpoint,
            "verdict": "PASS",
        },
    }


def _make_bridge(feature_id: str) -> dict:
    return {
        "kind": "BridgeReceipt",
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "subject": {"feature_id": feature_id, "tool": "game_query_entities"},
    }


def _all_features_bundle(features: list[str], judges: list[dict] | None = None,
                         bridges: list[dict] | None = None) -> dict:
    leaves = []
    if "f9_overlay" in features:
        leaves += [{"path": "validate_f9.png", "sha256": "00"}, {"path": "raw_f9.mp4", "sha256": "00"}]
    if "f10_modmenu" in features:
        leaves += [{"path": "validate_f10.png", "sha256": "00"}, {"path": "raw_f10.mp4", "sha256": "00"}]
    if "pack_load" in features:
        leaves += [{"path": "validate_mods.png", "sha256": "00"}]
    return {
        "version": "1.0",
        "kind": "BundleManifest",
        "bundle_id": "test-bundle-1",
        "policy_id": "dinoforge-default-2026-04",
        "leaves": leaves,
        "judges": judges if judges is not None else [_make_judge(f) for f in features],
        "bridges": bridges if bridges is not None else [_make_bridge(f) for f in features],
    }


def test_policy_passes_with_all_required_judges():
    """Happy path: valid moonshot judge + bridge receipt + all artifacts -> PASS."""
    p = proof_policy.load_policy(POLICY_YAML)
    bundle = _all_features_bundle(["f9_overlay", "f10_modmenu", "pack_load"])
    result = proof_policy.evaluate(bundle, p)
    assert result.passed is True, f"expected pass, got violations: {result.violations} | features: {result.to_dict()['features']}"
    assert all(fr.passed for fr in result.feature_results.values())
    assert result.feature_results["f9_overlay"].matched_judge == "moonshot-v1-128k"


def test_policy_fails_with_forbidden_judge():
    """A claude-* judge violates forbidden_judges and the gate fails closed."""
    p = proof_policy.load_policy(POLICY_YAML)
    bundle = _all_features_bundle(
        ["f9_overlay", "f10_modmenu", "pack_load"],
        judges=[
            _make_judge("f9_overlay", model="claude-opus-4-7", endpoint="https://api.anthropic.com"),
            _make_judge("f10_modmenu"),
            _make_judge("pack_load"),
        ],
    )
    result = proof_policy.evaluate(bundle, p)
    assert result.passed is False
    f9 = result.feature_results["f9_overlay"]
    assert f9.passed is False
    assert any("forbidden_judges" in v for v in f9.violations), f9.violations


def test_policy_fails_when_bridge_receipt_missing():
    p = proof_policy.load_policy(POLICY_YAML)
    bundle = _all_features_bundle(
        ["f9_overlay", "f10_modmenu", "pack_load"],
        bridges=[],
    )
    result = proof_policy.evaluate(bundle, p)
    assert result.passed is False
    for name in ("f9_overlay", "f10_modmenu", "pack_load"):
        fr = result.feature_results[name]
        assert any("require_bridge_receipt" in v for v in fr.violations), (name, fr.violations)


def test_policy_fails_on_stale_judge():
    """A receipt older than max_age_seconds fails the freshness check."""
    p = proof_policy.load_policy(POLICY_YAML)
    stale_ts = (datetime.now(timezone.utc) - timedelta(days=2)).strftime("%Y-%m-%dT%H:%M:%SZ")
    bundle = _all_features_bundle(
        ["f9_overlay", "f10_modmenu", "pack_load"],
        judges=[
            _make_judge("f9_overlay", ts=stale_ts),
            _make_judge("f10_modmenu"),
            _make_judge("pack_load"),
        ],
    )
    result = proof_policy.evaluate(bundle, p)
    assert result.passed is False
    f9 = result.feature_results["f9_overlay"]
    assert any("stale" in v for v in f9.violations), f9.violations


def test_policy_fails_on_missing_artifact():
    p = proof_policy.load_policy(POLICY_YAML)
    bundle = _all_features_bundle(["f9_overlay", "f10_modmenu", "pack_load"])
    bundle["leaves"] = [l for l in bundle["leaves"] if l["path"] != "validate_f9.png"]
    result = proof_policy.evaluate(bundle, p)
    assert result.passed is False
    f9 = result.feature_results["f9_overlay"]
    assert any("validate_f9.png" in v for v in f9.violations), f9.violations


def test_evaluator_cli_smoke(tmp_path):
    """Exercise the CLI; PASS path -> exit 0, FAIL path -> exit 1."""
    bundle = _all_features_bundle(["f9_overlay", "f10_modmenu", "pack_load"])
    bundle_path = tmp_path / "bundle.json"
    bundle_path.write_text(json.dumps(bundle), encoding="utf-8")
    rc = proof_policy._main(["evaluate", str(bundle_path), str(POLICY_YAML)])
    assert rc == 0

    # Tamper -> FAIL
    bundle["judges"][0]["subject"]["judge_model"] = "claude-opus"
    bundle_path.write_text(json.dumps(bundle), encoding="utf-8")
    rc = proof_policy._main(["evaluate", str(bundle_path), str(POLICY_YAML)])
    assert rc == 1
