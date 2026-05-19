"""Tests for merkle module."""
from __future__ import annotations
import json
from pathlib import Path

import pytest

from dinoforge_mcp import merkle


def _write(p: Path, content: bytes) -> Path:
    p.write_bytes(content)
    return p


def test_merkle_root_deterministic(tmp_path):
    a = _write(tmp_path / "a.txt", b"hello")
    b = _write(tmp_path / "b.txt", b"world")
    root1, leaves1 = merkle.compute_merkle_root([a, b], bundle_root=tmp_path)
    root2, leaves2 = merkle.compute_merkle_root([b, a], bundle_root=tmp_path)
    assert root1 == root2  # Sorting makes order independent
    assert [l.path for l in leaves1] == [l.path for l in leaves2]
    assert leaves1[0].path == "a.txt"  # Sorted alphabetically


def test_merkle_with_one_file(tmp_path):
    a = _write(tmp_path / "only.txt", b"single")
    root, leaves = merkle.compute_merkle_root([a], bundle_root=tmp_path)
    assert len(leaves) == 1
    assert root == leaves[0].sha256  # Tree of one node = that node


def test_merkle_with_three_files_odd_pad(tmp_path):
    a = _write(tmp_path / "a.txt", b"a")
    b = _write(tmp_path / "b.txt", b"b")
    c = _write(tmp_path / "c.txt", b"c")
    root, leaves = merkle.compute_merkle_root([a, b, c], bundle_root=tmp_path)
    assert len(leaves) == 3
    assert len(root) == 64  # sha256 hex length


def test_merkle_detects_tampering(tmp_path):
    a = _write(tmp_path / "a.txt", b"original")
    b = _write(tmp_path / "b.txt", b"original2")
    root, leaves = merkle.compute_merkle_root([a, b], bundle_root=tmp_path)

    # Verify clean
    assert merkle.verify_merkle(root, leaves, tmp_path) is True

    # Tamper with one file
    _write(tmp_path / "a.txt", b"TAMPERED")
    assert merkle.verify_merkle(root, leaves, tmp_path) is False


def test_merkle_detects_missing_file(tmp_path):
    a = _write(tmp_path / "a.txt", b"a")
    b = _write(tmp_path / "b.txt", b"b")
    root, leaves = merkle.compute_merkle_root([a, b], bundle_root=tmp_path)

    (tmp_path / "a.txt").unlink()
    assert merkle.verify_merkle(root, leaves, tmp_path) is False


def test_compute_self_hash_excludes_self_hash_field(tmp_path):
    manifest = {"version": "1.0", "merkle_root": "abc", "self_hash": "old", "signature": None}
    h1 = merkle.compute_self_hash(manifest)
    manifest["self_hash"] = "different"
    h2 = merkle.compute_self_hash(manifest)
    assert h1 == h2  # self_hash field doesn't affect computation

    manifest["merkle_root"] = "DIFFERENT"
    h3 = merkle.compute_self_hash(manifest)
    assert h1 != h3  # other fields do


def test_bundle_manifest_serializes_to_json(tmp_path):
    m = merkle.BundleManifest(
        bundle_id="2026-04-25T12:00:00Z-abc123",
        merkle_root="0" * 64,
        leaves=[merkle.MerkleLeaf(path="a.txt", sha256="0" * 64)],
        policy_id="dinoforge-default-2026-04",
    )
    s = m.to_json()
    parsed = json.loads(s)
    assert parsed["bundle_id"] == "2026-04-25T12:00:00Z-abc123"
    assert parsed["leaves"][0]["path"] == "a.txt"
