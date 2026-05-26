"""Tests for proof_signing module (ed25519 path only, Phase 1)."""
from __future__ import annotations
from pathlib import Path

import pytest

from dinoforge_mcp import proof_signing


@pytest.fixture
def isolated_keys(tmp_path, monkeypatch):
    """Override LOCAL_KEY_PATH to a tmp dir."""
    keydir = tmp_path / ".dinoforge"
    monkeypatch.setattr(proof_signing, "LOCAL_KEY_PATH", keydir / "proof_signing.key")
    monkeypatch.setattr(proof_signing, "LOCAL_PUBKEY_PATH", keydir / "proof_signing.pub")
    return tmp_path


def test_sign_and_verify_roundtrip(isolated_keys):
    sample = isolated_keys / "sample.json"
    sample.write_text('{"hello":"world"}')

    result = proof_signing.sign_blob(sample)
    assert result.method == "ed25519-localkey"
    assert result.signature_path.exists()
    assert result.signer_identity.startswith("ed25519:")

    v = proof_signing.verify_blob(sample, result.signature_path)
    assert v.valid is True
    assert v.error is None


def test_verify_rejects_tampered(isolated_keys):
    sample = isolated_keys / "sample.json"
    sample.write_text('{"hello":"world"}')
    result = proof_signing.sign_blob(sample)

    sample.write_text('{"hello":"TAMPERED"}')

    v = proof_signing.verify_blob(sample, result.signature_path)
    assert v.valid is False
    assert v.error is not None


def test_verify_rejects_missing_signature(isolated_keys):
    sample = isolated_keys / "sample.json"
    sample.write_text("data")
    v = proof_signing.verify_blob(sample, isolated_keys / "nonexistent.sig")
    assert v.valid is False
    assert "sig missing" in v.error


def test_get_signer_identity(isolated_keys):
    identity = proof_signing.get_signer_identity()
    assert identity.startswith("ed25519:")


def test_sign_blob_raises_on_missing_file(isolated_keys):
    with pytest.raises(FileNotFoundError):
        proof_signing.sign_blob(isolated_keys / "nope.json")


def test_keypair_persists(isolated_keys):
    a = isolated_keys / "a.json"; a.write_text("a")
    b = isolated_keys / "b.json"; b.write_text("b")
    r1 = proof_signing.sign_blob(a)
    r2 = proof_signing.sign_blob(b)
    assert r1.signer_identity == r2.signer_identity
