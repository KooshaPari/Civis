"""Bridge receipt aggregator — Wave 2 Phase 4d.

Per spec: ``docs/design/2026-04-26-bridge-receipt-aggregator.md``.

Lifecycle (production wire-up is a separate task — this module is standalone):

    agg = BridgeReceiptAggregator(bundle_dir, max_buffer=200)
    agg.record(receipt_dict)        # called per IGameBridge response
    ...
    paths = agg.flush(seq=42)       # writes files under bundle_dir/bridge-receipts/
    root  = agg.get_root(since=0)   # merkle root over receipts since seq=0 (for manifest)

The aggregator deliberately does NOT verify per-call HMACs — that is the in-run
client's job (``BridgeReceiptVerifier`` C# side, Phase 4b). The aggregator's
contract is purely structural: it accepts a receipt dict, stamps an aggregator
sequence id, and produces deterministic on-disk + merkle artifacts so the cosign
signature on the bundle manifest covers all of them.

Receipt dict shape (from C# ``BridgeReceipt`` DTO + per-call wrapper)::

    {
      "kind": "BridgeReceipt",
      "version": "1.0",
      "timestamp_utc": "2026-04-26T14:01:22Z",
      "subject": {
        "feature_id":      "f9_overlay",          # optional, set by tool wrapper
        "tool":            "applyOverride",       # JSON-RPC method name
        "session_id":      "550e8400-...",
        "world_frame":     12345,
        "state_snapshot_sha256": "9f2c3a1d...",   # optional; only on stateful calls
      },
      "hmac": "BASE64(HMAC-SHA256(session_key, canonical_payload))",
      "verification_status": "ok" | "warn_no_key" | "warn_mismatch"
    }

Only ``hmac`` and ``subject.session_id`` + ``subject.world_frame`` are mandatory
for structural acceptance. Empty / missing fields raise ``ValueError`` from
:meth:`BridgeReceiptAggregator.record` so the production caller fails fast.
"""
from __future__ import annotations

import hashlib
import json
import threading
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


_SUBDIR = "bridge-receipts"
_INDEX_NAME = "index.json"


def _canonicalize(obj: Any) -> bytes:
    """Canonical JSON for hashing — UTF-8, sort_keys, no whitespace.

    Matches the C# ``CanonicalJsonSerializer`` byte-identical contract used by
    ``BridgeReceiptVerifier`` so an on-disk receipt re-hashes identically on
    both sides.
    """
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def _sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _short_sha(hex_digest: str, n: int = 8) -> str:
    return hex_digest[:n]


def _validate_receipt(receipt: dict) -> None:
    """Structural-only validation. Raises ValueError on shape violation."""
    if not isinstance(receipt, dict):
        raise ValueError("receipt must be a dict")
    if not receipt.get("hmac"):
        raise ValueError("receipt missing hmac")
    subject = receipt.get("subject")
    if not isinstance(subject, dict):
        raise ValueError("receipt missing or invalid 'subject' mapping")
    if not subject.get("session_id"):
        raise ValueError("receipt.subject missing session_id")
    if "world_frame" not in subject:
        raise ValueError("receipt.subject missing world_frame")


@dataclass
class _BufferedReceipt:
    """In-memory record: original receipt + aggregator-stamped fields."""
    seq: int
    received_utc: str
    receipt: dict

    def to_disk_dict(self) -> dict:
        """Serialize to the on-disk shape (per Phase 4d spec section 'Receipt-file shape').

        The disk shape is a flat dict combining the aggregator-stamped fields
        with the original receipt subject — convenient for the verifier and
        consistent with the existing ``bridges:`` inline shape in policy bundles.
        """
        subject = self.receipt.get("subject", {}) or {}
        return {
            "seq": self.seq,
            "received_utc": self.received_utc,
            "method": subject.get("tool", ""),
            "session_id": subject.get("session_id", ""),
            "world_frame": int(subject.get("world_frame", 0)),
            "timestamp_utc": self.receipt.get("timestamp_utc", ""),
            "feature_id": subject.get("feature_id", ""),
            "state_snapshot_sha256": subject.get("state_snapshot_sha256", ""),
            "hmac": self.receipt.get("hmac", ""),
            "verification_status": self.receipt.get("verification_status", "ok"),
        }


class BridgeReceiptAggregator:
    """Buffer per-call BridgeReceipts and flush them as merkle leaves at bundle finalize.

    Thread-safe (a single MCP server instance may dispatch tools concurrently).
    Buffer is bounded by ``max_buffer``: if exceeded, :meth:`record` raises
    ``BufferError`` so the caller cannot silently drop receipts.

    Args:
        bundle_dir: Bundle root directory; receipts go under ``<bundle_dir>/bridge-receipts/``.
        max_buffer: Maximum in-memory receipts before forcing a flush. Default 100.
    """

    def __init__(self, bundle_dir: Path, max_buffer: int = 100) -> None:
        self._bundle_dir = Path(bundle_dir)
        if max_buffer <= 0:
            raise ValueError("max_buffer must be positive")
        self._max_buffer = int(max_buffer)
        self._buffer: list[_BufferedReceipt] = []
        self._all_seen: list[_BufferedReceipt] = []  # full session-history (for get_root)
        self._next_seq: int = 0
        self._lock = threading.RLock()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    @property
    def bundle_dir(self) -> Path:
        return self._bundle_dir

    @property
    def receipts_dir(self) -> Path:
        return self._bundle_dir / _SUBDIR

    def record(self, receipt: dict) -> None:
        """Append a receipt to the in-memory buffer.

        Raises:
            ValueError: receipt is structurally invalid.
            BufferError: buffer is full (caller must flush before recording more).
        """
        _validate_receipt(receipt)
        with self._lock:
            if len(self._buffer) >= self._max_buffer:
                raise BufferError(
                    f"BridgeReceiptAggregator buffer full ({self._max_buffer}); "
                    "flush before recording more receipts"
                )
            seq = self._next_seq
            self._next_seq += 1
            entry = _BufferedReceipt(
                seq=seq,
                received_utc=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
                receipt=dict(receipt),  # shallow copy; receipts are leaf dicts
            )
            self._buffer.append(entry)
            self._all_seen.append(entry)

    def buffer_size(self) -> int:
        """Current in-memory buffer length (cleared on flush)."""
        with self._lock:
            return len(self._buffer)

    def total_seen(self) -> int:
        """Total receipts recorded since aggregator construction (NOT cleared on flush)."""
        with self._lock:
            return len(self._all_seen)

    def flush(self, sequence_id: Optional[int] = None) -> list[Path]:
        """Write the in-memory buffer to ``bundle_dir/bridge-receipts/`` and clear it.

        Each receipt becomes a single JSON file named ``<seq:06>-<method>-<sha8>.json``
        (per spec). Also writes/updates ``index.json`` with the running summary.

        Args:
            sequence_id: Optional flush-batch id (unused for filenames; recorded in index).

        Returns:
            Sorted list of receipt file paths written this flush.
        """
        with self._lock:
            if not self._buffer:
                # Still touch the receipts dir so a downstream verifier sees an empty bundle
                # consistently (vs missing dir => "aggregator never wired").
                self.receipts_dir.mkdir(parents=True, exist_ok=True)
                self._write_index(sequence_id)
                return []

            self.receipts_dir.mkdir(parents=True, exist_ok=True)
            written: list[Path] = []
            for entry in self._buffer:
                disk = entry.to_disk_dict()
                payload = _canonicalize(disk)
                file_sha = _sha256_hex(payload)
                method = (disk.get("method") or "unknown").replace("/", "_").replace("\\", "_")
                fname = f"{entry.seq:06d}-{method}-{_short_sha(file_sha)}.json"
                fpath = self.receipts_dir / fname
                fpath.write_bytes(payload)
                written.append(fpath)

            self._buffer.clear()
            self._write_index(sequence_id)
            return sorted(written)

    def get_root(self, since_sequence: int = 0) -> str:
        """Compute a merkle root over all receipts with ``seq >= since_sequence``.

        The root covers every receipt the aggregator has *seen* (not just the
        currently-buffered ones), so it remains stable across flushes. Hashing
        is done over the canonical JSON of each receipt's disk-form, in seq
        order — same bytes that flush() writes to disk.

        Returns:
            64-char lowercase hex sha256. Empty-input root is ``"" `` (the empty
            string), per the spec's "no receipts => no root" convention.
        """
        with self._lock:
            entries = [e for e in self._all_seen if e.seq >= since_sequence]
            if not entries:
                return ""
            entries.sort(key=lambda e: e.seq)
            leaf_hashes = [_sha256_hex(_canonicalize(e.to_disk_dict())) for e in entries]
            return _merkle_root_from_leaves(leaf_hashes)

    def manifest_summary(self, since_sequence: int = 0) -> dict[str, Any]:
        """Build the manifest sub-document a bundle finalizer should embed.

        Returns a dict with ``bridge_receipts_root``, ``bridge_receipts_count``,
        ``bridge_receipts_warn_count``, and a sorted list of session_ids covered.
        """
        with self._lock:
            entries = [e for e in self._all_seen if e.seq >= since_sequence]
            warn = sum(
                1 for e in entries
                if e.receipt.get("verification_status", "ok") != "ok"
            )
            sessions = sorted({
                str(e.receipt.get("subject", {}).get("session_id", ""))
                for e in entries
                if e.receipt.get("subject", {}).get("session_id")
            })
            return {
                "bridge_receipts_root": self.get_root(since_sequence),
                "bridge_receipts_count": len(entries),
                "bridge_receipts_warn_count": warn,
                "bridge_receipts_sessions": sessions,
            }

    def clear(self) -> None:
        """Reset all state. For long-lived servers running back-to-back bundles."""
        with self._lock:
            self._buffer.clear()
            self._all_seen.clear()
            self._next_seq = 0

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _write_index(self, sequence_id: Optional[int]) -> None:
        """Write ``bridge-receipts/index.json`` summarising the run so far.

        Index is recomputable from the per-receipt files; it exists for human-
        readable diagnostics and for the verifier's count assertions. It is NOT
        a merkle leaf (per spec open-question #1, default proposal).
        """
        summary = self.manifest_summary(since_sequence=0)
        summary["last_flush_sequence_id"] = sequence_id
        summary["last_flush_utc"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")
        index_path = self.receipts_dir / _INDEX_NAME
        # Sort keys for byte-stable diffs; this file is NOT a merkle leaf.
        index_path.write_text(
            json.dumps(summary, sort_keys=True, indent=2),
            encoding="utf-8",
        )


# ---------------------------------------------------------------------------
# Pure helper — also used by merkle.py for inline-bridges manifest computation.
# ---------------------------------------------------------------------------


def _merkle_root_from_leaves(leaf_hashes: list[str]) -> str:
    """Compute a merkle root from a pre-ordered list of hex sha256 leaf hashes.

    Uses the same odd-leaf padding (duplicate last) as ``merkle.compute_merkle_root``
    so receipt roots are interchangeable with file-based merkle roots in the
    bundle manifest.
    """
    if not leaf_hashes:
        return ""
    level = list(leaf_hashes)
    while len(level) > 1:
        nxt: list[str] = []
        for i in range(0, len(level), 2):
            left = level[i]
            right = level[i + 1] if i + 1 < len(level) else level[i]
            nxt.append(hashlib.sha256(bytes.fromhex(left) + bytes.fromhex(right)).hexdigest())
        level = nxt
    return level[0]


def compute_bridge_receipts_root(receipts: list[dict]) -> str:
    """Compute a merkle root over a list of receipt dicts (in given order).

    Used by ``merkle.py`` when a bundle finalizer wants to derive
    ``manifest.bridge_receipts_root`` from inline receipts (typical for
    self-contained bundles like ``genesis-2026-04-26.json``) without going
    through the file-flush path.

    Args:
        receipts: list of receipt dicts; canonicalized in-place for hashing.

    Returns:
        64-char hex sha256 root, or ``""`` for an empty list.
    """
    if not receipts:
        return ""
    leaf_hashes = [_sha256_hex(_canonicalize(r)) for r in receipts]
    return _merkle_root_from_leaves(leaf_hashes)
