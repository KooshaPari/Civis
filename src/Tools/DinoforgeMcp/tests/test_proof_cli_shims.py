"""Tests for the Phase 2 CLI shims (proof_signing, merkle, proof_policy_cli).

These exercise the argparse entry points that prove-features-gate.ps1 invokes.
"""
from __future__ import annotations
import json
import sys
from pathlib import Path

import pytest

from dinoforge_mcp import merkle, proof_policy_cli, proof_signing


REPO_ROOT = Path(__file__).resolve().parents[4]
POLICY_YAML = REPO_ROOT / "policies" / "proof-policy.yaml"


# ---------------------------------------------------------------------------
# proof_policy_cli
# ---------------------------------------------------------------------------

def test_policy_cli_load_emits_json(capsys):
    rc = proof_policy_cli._cli(["load", str(POLICY_YAML)])
    out = capsys.readouterr().out
    assert rc == 0
    data = json.loads(out)
    assert data["ok"] is True
    assert data["policy"]["policy_id"] == "dinoforge-default-2026-04"
    assert "f9_overlay" in data["policy"]["features"]


def test_policy_cli_check_judge_rejects_claude(capsys):
    rc = proof_policy_cli._cli(["check_judge", "claude-opus-4-7", str(POLICY_YAML)])
    out = capsys.readouterr().out
    data = json.loads(out)
    assert rc == 1
    assert data["ok"] is False
    assert data["forbidden"] is True
    assert "matches forbidden" in data["reason"]


def test_policy_cli_check_judge_allows_moonshot(capsys):
    rc = proof_policy_cli._cli(["check_judge", "moonshot-v1-128k", str(POLICY_YAML)])
    out = capsys.readouterr().out
    data = json.loads(out)
    assert rc == 0
    assert data["ok"] is True
    assert data["forbidden"] is False


def test_policy_cli_list_features(capsys):
    rc = proof_policy_cli._cli(["list_features", str(POLICY_YAML)])
    out = capsys.readouterr().out
    data = json.loads(out)
    assert rc == 0
    assert "f9_overlay" in data["features"]


def test_policy_cli_load_missing_file(tmp_path, capsys):
    rc = proof_policy_cli._cli(["load", str(tmp_path / "nope.yaml")])
    err = capsys.readouterr().err
    assert rc == 2
    assert "policy load failed" in err


# ---------------------------------------------------------------------------
# proof_signing CLI
# ---------------------------------------------------------------------------

@pytest.fixture
def isolated_keys(tmp_path, monkeypatch):
    keydir = tmp_path / ".dinoforge"
    monkeypatch.setattr(proof_signing, "LOCAL_KEY_PATH", keydir / "proof_signing.key")
    monkeypatch.setattr(proof_signing, "LOCAL_PUBKEY_PATH", keydir / "proof_signing.pub")
    return tmp_path


def test_signing_cli_sign_then_verify(isolated_keys, capsys):
    sample = isolated_keys / "data.json"
    sample.write_text("{\"k\":1}")

    rc_sign = proof_signing._cli(["sign", str(sample)])
    sign_out = json.loads(capsys.readouterr().out)
    assert rc_sign == 0
    assert sign_out["ok"] is True
    sig_path = Path(sign_out["signature_path"])
    assert sig_path.exists()

    rc_verify = proof_signing._cli(["verify", str(sample), str(sig_path)])
    verify_out = json.loads(capsys.readouterr().out)
    assert rc_verify == 0
    assert verify_out["ok"] is True


def test_signing_cli_verify_tampered(isolated_keys, capsys):
    sample = isolated_keys / "data.json"
    sample.write_text("{\"k\":1}")
    proof_signing._cli(["sign", str(sample)])
    sign_out = json.loads(capsys.readouterr().out)
    sig_path = sign_out["signature_path"]

    sample.write_text("{\"k\":2}")  # tamper
    rc = proof_signing._cli(["verify", str(sample), sig_path])
    verify_out = json.loads(capsys.readouterr().out)
    assert rc == 1
    assert verify_out["ok"] is False


def test_signing_cli_identity(isolated_keys, capsys):
    rc = proof_signing._cli(["identity"])
    out = json.loads(capsys.readouterr().out)
    assert rc == 0
    assert out["signer_identity"].startswith("ed25519:")


# ---------------------------------------------------------------------------
# merkle CLI
# ---------------------------------------------------------------------------

def test_merkle_cli_compute_root(tmp_path, capsys):
    a = tmp_path / "a.txt"; a.write_text("alpha")
    b = tmp_path / "b.txt"; b.write_text("beta")
    rc = merkle._cli(["compute_root", str(a), str(b), "--bundle-root", str(tmp_path)])
    out = json.loads(capsys.readouterr().out)
    assert rc == 0
    assert len(out["merkle_root"]) == 64
    assert {"a.txt", "b.txt"} == {l["path"] for l in out["leaves"]}


def test_merkle_cli_verify_bundle_happy(tmp_path, capsys):
    # Build a bundle, write manifest.json, verify.
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    (bundle / "x.png").write_bytes(b"x" * 32)
    (bundle / "y.json").write_text("{\"y\":1}")

    root, leaves = merkle.compute_merkle_root(
        [bundle / "x.png", bundle / "y.json"], bundle_root=bundle
    )
    manifest = {
        "version": "1.0",
        "bundle_id": "test-bundle",
        "merkle_root": root,
        "leaves": [{"path": l.path, "sha256": l.sha256} for l in leaves],
        "previous_bundle_hash": None,
    }
    (bundle / "manifest.json").write_text(json.dumps(manifest))

    rc = merkle._cli(["verify_bundle", str(bundle)])
    out = json.loads(capsys.readouterr().out)
    assert rc == 0, out
    assert out["ok"] is True


def test_merkle_cli_verify_bundle_tampered(tmp_path, capsys):
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    (bundle / "x.png").write_bytes(b"original")

    root, leaves = merkle.compute_merkle_root([bundle / "x.png"], bundle_root=bundle)
    manifest = {
        "version": "1.0",
        "merkle_root": root,
        "leaves": [{"path": l.path, "sha256": l.sha256} for l in leaves],
    }
    (bundle / "manifest.json").write_text(json.dumps(manifest))

    # Tamper the file
    (bundle / "x.png").write_bytes(b"TAMPERED")

    rc = merkle._cli(["verify_bundle", str(bundle)])
    out = json.loads(capsys.readouterr().out)
    assert rc == 1
    assert out["ok"] is False
    assert "merkle root mismatch" in out["message"]


def test_merkle_cli_verify_bundle_missing_manifest(tmp_path, capsys):
    rc = merkle._cli(["verify_bundle", str(tmp_path)])
    out = json.loads(capsys.readouterr().out)
    assert rc == 1
    assert "manifest missing" in out["message"]
