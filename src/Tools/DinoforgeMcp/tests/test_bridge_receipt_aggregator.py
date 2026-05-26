"""Tests for the BridgeReceiptAggregator (Wave 2 Phase 4d, task #232).

Spec: ``docs/design/2026-04-26-bridge-receipt-aggregator.md``.

These tests cover the standalone aggregator + merkle-integration surface.
Production wire-up (server.py middleware) is intentionally out of scope —
it lands in a follow-up task per the spec's Phase 4d-a/b/c sub-phases.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

from dinoforge_mcp import merkle
from dinoforge_mcp.bridge_receipt_aggregator import (
    BridgeReceiptAggregator,
    compute_bridge_receipts_root,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_receipt(
    *,
    feature_id: str = "f9_overlay",
    tool: str = "applyOverride",
    session_id: str = "550e8400-e29b-41d4-a716-446655440000",
    world_frame: int = 12345,
    state_sha: str = "9f2c3a1d" * 8,
    hmac: str = "BASE64_HMAC_PLACEHOLDER==",
    status: str = "ok",
    timestamp_utc: str = "2026-04-26T14:01:22Z",
) -> dict:
    return {
        "kind": "BridgeReceipt",
        "version": "1.0",
        "timestamp_utc": timestamp_utc,
        "subject": {
            "feature_id": feature_id,
            "tool": tool,
            "session_id": session_id,
            "world_frame": world_frame,
            "state_snapshot_sha256": state_sha,
        },
        "hmac": hmac,
        "verification_status": status,
    }


# ---------------------------------------------------------------------------
# Aggregator behaviour
# ---------------------------------------------------------------------------


def test_aggregator_records_and_flushes_buffer(tmp_path: Path) -> None:
    """record() buffers in-memory; flush() writes one JSON file per receipt and clears."""
    agg = BridgeReceiptAggregator(tmp_path, max_buffer=10)

    agg.record(_make_receipt(world_frame=100))
    agg.record(_make_receipt(world_frame=101, tool="getStat"))
    agg.record(_make_receipt(world_frame=102, tool="status"))
    assert agg.buffer_size() == 3
    assert agg.total_seen() == 3

    written = agg.flush(sequence_id=1)

    # Three receipts -> three files, plus index.json (which is NOT in `written`).
    assert len(written) == 3
    assert agg.buffer_size() == 0  # buffer cleared
    assert agg.total_seen() == 3   # all_seen preserved for get_root()

    receipts_dir = tmp_path / "bridge-receipts"
    files_on_disk = sorted(p.name for p in receipts_dir.glob("*.json"))
    assert len(files_on_disk) == 4  # 3 receipts + index.json
    assert (receipts_dir / "index.json").is_file()

    # Filenames follow the spec: <seq:06>-<method>-<sha8>.json
    receipt_files = sorted(p for p in written)
    first = json.loads(receipt_files[0].read_text(encoding="utf-8"))
    assert first["seq"] == 0
    assert first["method"] == "applyOverride"
    assert first["world_frame"] == 100
    assert first["session_id"] == "550e8400-e29b-41d4-a716-446655440000"
    assert first["hmac"] == "BASE64_HMAC_PLACEHOLDER=="

    # index.json summary matches manifest_summary
    index = json.loads((receipts_dir / "index.json").read_text(encoding="utf-8"))
    assert index["bridge_receipts_count"] == 3
    assert index["bridge_receipts_warn_count"] == 0
    assert index["last_flush_sequence_id"] == 1
    assert len(index["bridge_receipts_root"]) == 64  # sha256 hex

    # Structural validation rejects malformed input
    with pytest.raises(ValueError):
        agg.record({"hmac": "x"})  # missing subject
    with pytest.raises(ValueError):
        agg.record({"subject": {"session_id": "s", "world_frame": 0}})  # missing hmac

    # Buffer-full guard
    small = BridgeReceiptAggregator(tmp_path / "small", max_buffer=2)
    small.record(_make_receipt())
    small.record(_make_receipt())
    with pytest.raises(BufferError):
        small.record(_make_receipt())


def test_aggregator_get_root_is_deterministic(tmp_path: Path) -> None:
    """Determinism contract — within a single aggregator AND for the pure helper.

    The aggregator stamps each receipt with ``received_utc = now()``, which is
    intentional (the cosign signature on the manifest binds those timestamps to
    the run). So roots from two *different* aggregator instances do NOT match
    even with identical input — that is correct behaviour, not a bug.

    What MUST be deterministic:
      1. ``get_root()`` is stable across calls on the same instance.
      2. ``get_root()`` is unchanged by ``flush()`` (flush is just disk I/O).
      3. ``compute_bridge_receipts_root(receipts)`` (the pure helper, no time
         stamping) is byte-identical for identical input dicts.
    """
    receipts = [
        _make_receipt(world_frame=10, tool="applyOverride", state_sha="aa" * 32),
        _make_receipt(world_frame=11, tool="getStat", state_sha="bb" * 32),
        _make_receipt(world_frame=12, tool="status", state_sha="cc" * 32),
    ]

    agg = BridgeReceiptAggregator(tmp_path / "a", max_buffer=10)
    for r in receipts:
        agg.record(r)

    # 1. Same instance, same call -> same root
    root_call_1 = agg.get_root()
    root_call_2 = agg.get_root()
    assert root_call_1 == root_call_2
    assert len(root_call_1) == 64

    # 2. Flush does not change the root (disk I/O, not state mutation for hashing)
    agg.flush()
    root_post_flush = agg.get_root()
    assert root_call_1 == root_post_flush

    # 3. Empty aggregator returns empty root (per spec convention)
    empty = BridgeReceiptAggregator(tmp_path / "empty", max_buffer=10)
    assert empty.get_root() == ""

    # 4. Pure helper is deterministic across calls + across input copies
    inline_root_a = compute_bridge_receipts_root(receipts)
    inline_root_b = compute_bridge_receipts_root(list(receipts))
    inline_root_c = compute_bridge_receipts_root([dict(r) for r in receipts])
    assert inline_root_a == inline_root_b == inline_root_c
    assert len(inline_root_a) == 64

    # 5. Aggregator root differs from pure-helper root (received_utc stamping is the difference)
    assert inline_root_a != root_call_1, (
        "aggregator MUST stamp received_utc — if these match, time stamping was skipped"
    )


def test_aggregator_get_root_changes_with_new_receipt(tmp_path: Path) -> None:
    """Adding a receipt MUST mutate the merkle root (else the receipt isn't covered)."""
    agg = BridgeReceiptAggregator(tmp_path, max_buffer=10)

    agg.record(_make_receipt(world_frame=1))
    root_1 = agg.get_root()

    agg.record(_make_receipt(world_frame=2))
    root_2 = agg.get_root()

    agg.record(_make_receipt(world_frame=3))
    root_3 = agg.get_root()

    assert root_1 != root_2
    assert root_2 != root_3
    assert root_1 != root_3
    assert all(len(r) == 64 for r in (root_1, root_2, root_3))

    # since_sequence slicing: get_root(since=1) excludes seq 0
    root_since_1 = agg.get_root(since_sequence=1)
    assert root_since_1 != root_3  # subset != full set

    # since_sequence past the end -> empty root
    root_past_end = agg.get_root(since_sequence=999)
    assert root_past_end == ""

    # manifest_summary reports correct count + warn count
    summary = agg.manifest_summary()
    assert summary["bridge_receipts_count"] == 3
    assert summary["bridge_receipts_warn_count"] == 0
    assert summary["bridge_receipts_root"] == root_3
    assert "550e8400-e29b-41d4-a716-446655440000" in summary["bridge_receipts_sessions"]

    # warn_count tracks verification_status != ok
    agg.record(_make_receipt(world_frame=4, status="warn_mismatch"))
    summary2 = agg.manifest_summary()
    assert summary2["bridge_receipts_warn_count"] == 1

    # clear() resets everything
    agg.clear()
    assert agg.total_seen() == 0
    assert agg.get_root() == ""


# ---------------------------------------------------------------------------
# Merkle integration — bundle manifest with bridge_receipts_root
# ---------------------------------------------------------------------------


def test_bundle_with_bridge_receipts_root_validates(tmp_path: Path) -> None:
    """A bundle whose manifest claims a bridge_receipts_root validates end-to-end:

    1. ``BundleManifest`` carries the new ``bridge_receipts_root`` field.
    2. ``fold_bridge_receipts_root`` produces a stable folded root.
    3. ``verify_bundle_dir`` still passes (file-leaf merkle is independent).
    4. Tampering with a receipt mutates ``bridge_receipts_root`` deterministically.
    """
    # 1. Compute receipts root via the aggregator's pure helper (genesis-style inline bundle)
    inline_receipts = [
        _make_receipt(world_frame=0, tool="status", feature_id="f9_overlay"),
        _make_receipt(world_frame=1, tool="getStat", feature_id="f10_modmenu"),
        _make_receipt(world_frame=2, tool="applyOverride", feature_id="pack_load"),
    ]
    receipts_root = compute_bridge_receipts_root(inline_receipts)
    assert len(receipts_root) == 64

    # 2. Build a real file-leaf bundle on disk + compute its merkle root
    leaf_a = tmp_path / "validate_f9.png"
    leaf_a.write_bytes(b"PNG\xfff9 frame")
    leaf_b = tmp_path / "raw_f9.mp4"
    leaf_b.write_bytes(b"MP4 raw bytes")

    file_root, leaves = merkle.compute_merkle_root([leaf_a, leaf_b], bundle_root=tmp_path)

    # 3. Fold receipts root into the overall manifest root
    folded_root = merkle.fold_bridge_receipts_root(file_root, receipts_root)
    assert folded_root != file_root, "fold MUST mutate the root when receipts_root is present"
    assert len(folded_root) == 64

    # The fold helper passes through unchanged when there are no receipts
    assert merkle.fold_bridge_receipts_root(file_root, None) == file_root
    assert merkle.fold_bridge_receipts_root(file_root, "") == file_root

    # 4. Build a manifest carrying the new fields
    manifest = merkle.BundleManifest(
        bundle_id="bundle-test-with-receipts",
        merkle_root=folded_root,
        leaves=leaves,
        policy_id="dinoforge-default-2026-04",
        bridge_receipts_root=receipts_root,
        bridge_receipts_count=len(inline_receipts),
        bridge_receipts_warn_count=0,
    )
    payload = json.loads(manifest.to_json())
    assert payload["bridge_receipts_root"] == receipts_root
    assert payload["bridge_receipts_count"] == 3
    assert payload["merkle_root"] == folded_root

    # 5. Round-trip on disk: write manifest + leaves, recompute, verify the file-leaf root
    #    (the receipts root is checked separately — verify_bundle_dir doesn't know about
    #    the fold yet; that's deferred to the bundle verifier per spec).
    manifest_path = tmp_path / "manifest.json"
    # Write manifest with the file-only root (verify_bundle_dir's contract today).
    manifest_for_disk = merkle.BundleManifest(
        bundle_id=manifest.bundle_id,
        merkle_root=file_root,
        leaves=leaves,
        policy_id=manifest.policy_id,
        bridge_receipts_root=receipts_root,
        bridge_receipts_count=manifest.bridge_receipts_count,
        bridge_receipts_warn_count=manifest.bridge_receipts_warn_count,
    )
    manifest_path.write_text(manifest_for_disk.to_json(), encoding="utf-8")
    ok, msg = merkle.verify_bundle_dir(tmp_path)
    assert ok, f"verify_bundle_dir failed: {msg}"

    # 6. Tampering with a receipt mutates the folded root (catches replay/forgery)
    tampered_receipts = list(inline_receipts)
    tampered_receipts[1] = {**tampered_receipts[1], "hmac": "TAMPERED=="}
    tampered_root = compute_bridge_receipts_root(tampered_receipts)
    assert tampered_root != receipts_root
    assert merkle.fold_bridge_receipts_root(file_root, tampered_root) != folded_root
