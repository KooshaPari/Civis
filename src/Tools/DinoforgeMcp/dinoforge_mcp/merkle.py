"""Bundle merkle root + manifest builder.

Per spec section 4: docs/design/2026-04-25-smart-contract-proof-system.md
"""
from __future__ import annotations
import hashlib
import json
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional


@dataclass
class MerkleLeaf:
    path: str  # relative to bundle root, forward-slash separators
    sha256: str  # hex


@dataclass
class BundleManifest:
    version: str = "1.0"
    bundle_id: str = ""
    previous_bundle_hash: Optional[str] = None
    merkle_root: str = ""
    leaves: list[MerkleLeaf] = field(default_factory=list)
    policy_id: Optional[str] = None
    policy_version: Optional[str] = None
    judge_receipts: list[str] = field(default_factory=list)
    bridge_receipts: list[str] = field(default_factory=list)
    # Phase 4d: aggregate root over all per-call BridgeReceipts in this bundle.
    # None for legacy bundles that pre-date Phase 4d. When non-null, this root
    # is also folded into ``merkle_root`` as a synthesised leaf so the cosign
    # signature on the manifest covers it.
    bridge_receipts_root: Optional[str] = None
    bridge_receipts_count: int = 0
    bridge_receipts_warn_count: int = 0
    self_hash: Optional[str] = None
    signature: Optional[dict] = None

    def to_json(self, indent: int = 2) -> str:
        d = asdict(self)
        return json.dumps(d, indent=indent, sort_keys=True)


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            chunk = f.read(65536)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def _sha256_pair(left: str, right: str) -> str:
    """Concatenate hex digests as bytes, hash, return hex."""
    return hashlib.sha256(bytes.fromhex(left) + bytes.fromhex(right)).hexdigest()


def compute_merkle_root(file_paths: list[Path], bundle_root: Optional[Path] = None) -> tuple[str, list[MerkleLeaf]]:
    """Compute merkle root over a list of files.

    Files are sorted deterministically by relative path (forward-slash).
    Odd-leaf padding: duplicate the last leaf at each level (Bitcoin convention).

    Returns (root_hex, sorted_leaves).
    """
    if not file_paths:
        raise ValueError("compute_merkle_root requires at least one file")

    # Build leaves list
    leaves: list[MerkleLeaf] = []
    for fp in file_paths:
        fp = Path(fp)
        if not fp.is_file():
            raise FileNotFoundError(f"merkle: not a file: {fp}")
        relpath = str(fp.relative_to(bundle_root)) if bundle_root else fp.name
        relpath = relpath.replace("\\", "/")
        leaves.append(MerkleLeaf(path=relpath, sha256=_sha256_file(fp)))

    # Deterministic sort
    leaves.sort(key=lambda x: x.path)

    # Build tree, level by level
    level = [leaf.sha256 for leaf in leaves]
    while len(level) > 1:
        nxt: list[str] = []
        for i in range(0, len(level), 2):
            left = level[i]
            right = level[i + 1] if i + 1 < len(level) else level[i]  # pad odd by duplicating
            nxt.append(_sha256_pair(left, right))
        level = nxt

    return level[0], leaves


def verify_merkle(expected_root: str, leaves: list[MerkleLeaf], file_dir: Path) -> bool:
    """Recompute merkle root from a directory + expected leaves; compare to expected_root.

    Returns True iff: (a) every claimed leaf file exists, (b) every leaf's sha256 matches the file on disk,
    (c) the recomputed root equals expected_root.
    """
    file_dir = Path(file_dir)
    # Re-hash the leaf files; preserve the leaves' declared paths as the canonical order
    declared_leaves = sorted(leaves, key=lambda x: x.path)
    recomputed: list[MerkleLeaf] = []
    for leaf in declared_leaves:
        leaf_path = file_dir / leaf.path
        if not leaf_path.is_file():
            return False
        actual_sha = _sha256_file(leaf_path)
        if actual_sha != leaf.sha256:
            return False
        recomputed.append(MerkleLeaf(path=leaf.path, sha256=actual_sha))

    # Recompute root from declared leaf shas (verifying tree shape)
    level = [leaf.sha256 for leaf in declared_leaves]
    while len(level) > 1:
        nxt: list[str] = []
        for i in range(0, len(level), 2):
            left = level[i]
            right = level[i + 1] if i + 1 < len(level) else level[i]
            nxt.append(_sha256_pair(left, right))
        level = nxt

    return level[0] == expected_root


def fold_bridge_receipts_root(file_root: str, bridge_receipts_root: Optional[str]) -> str:
    """Fold a Phase-4d bridge_receipts_root into a file-tree merkle root.

    The combined root is what the cosign signature on ``manifest.json`` covers.
    Conventions:
      * If ``bridge_receipts_root`` is None or empty, return ``file_root`` unchanged
        (legacy / pre-Phase-4d bundles).
      * Otherwise, concatenate ``file_root`` and ``bridge_receipts_root`` as bytes
        (in that fixed order) and sha256-hash. Same primitive as ``_sha256_pair``.

    This deliberately uses a fixed pairing rather than re-shuffling the leaves —
    bridge receipts and file leaves are independently auditable surfaces, and we
    want a verifier to be able to recompute either side without seeing the other.
    """
    if not bridge_receipts_root:
        return file_root
    if not file_root:
        # Edge case: bundle has no file leaves but does have receipts.
        return bridge_receipts_root
    return _sha256_pair(file_root, bridge_receipts_root)


def compute_self_hash(manifest_data: dict) -> str:
    """Compute sha256 over manifest JSON with self_hash and signature fields zeroed.
    Used for tamper-detection of the manifest itself."""
    sanitized = {k: v for k, v in manifest_data.items() if k not in ("self_hash", "signature")}
    blob = json.dumps(sanitized, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()


def verify_bundle_dir(bundle_dir: Path) -> tuple[bool, str]:
    """Verify a bundle directory's manifest.json.

    Returns (ok, message). ok=True iff:
      - manifest.json exists and parses
      - every leaf file exists and its sha256 matches
      - the recomputed merkle_root matches the manifest's claim
      - the recomputed self_hash matches (if present)
    """
    bundle_dir = Path(bundle_dir)
    manifest_path = bundle_dir / "manifest.json"
    if not manifest_path.is_file():
        # Older bundles wrote the manifest as `bundle_manifest.json`. Accept that name too.
        alt = bundle_dir / "bundle_manifest.json"
        if alt.is_file():
            manifest_path = alt
        else:
            return False, f"manifest missing: {manifest_path}"

    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as e:
        return False, f"manifest parse error: {e}"

    expected_root = data.get("merkle_root")
    raw_leaves = data.get("leaves") or []
    if not expected_root or not raw_leaves:
        return False, "manifest missing merkle_root or leaves"

    leaves = [MerkleLeaf(path=l["path"], sha256=l["sha256"]) for l in raw_leaves]
    if not verify_merkle(expected_root, leaves, bundle_dir):
        return False, "merkle root mismatch (or tampered/missing leaf)"

    declared_self_hash = data.get("self_hash")
    if declared_self_hash:
        if compute_self_hash(data) != declared_self_hash:
            return False, "self_hash mismatch"

    return True, f"bundle verified: {len(leaves)} leaves, root={expected_root[:16]}..."


# ---------------------------------------------------------------------------
# CLI shim — invoked by prove-features-gate.ps1 (Phase 2)
# Usage:
#   python -m dinoforge_mcp.merkle verify_bundle <bundle_dir>
#   python -m dinoforge_mcp.merkle compute_root <file> [<file>...]
# Exit codes:
#   0 = ok / valid; 1 = mismatch / tampered; 2 = usage / IO error.
# ---------------------------------------------------------------------------
def _cli(argv: list[str]) -> int:
    import argparse
    import sys

    parser = argparse.ArgumentParser(prog="dinoforge_mcp.merkle")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_v = sub.add_parser("verify_bundle", help="verify a bundle directory's merkle manifest")
    p_v.add_argument("bundle_dir", type=Path)

    p_c = sub.add_parser("compute_root", help="compute merkle root over files")
    p_c.add_argument("files", type=Path, nargs="+")
    p_c.add_argument("--bundle-root", type=Path, default=None)

    args = parser.parse_args(argv)

    try:
        if args.cmd == "verify_bundle":
            ok, msg = verify_bundle_dir(args.bundle_dir)
            print(json.dumps({"ok": ok, "message": msg}))
            return 0 if ok else 1
        if args.cmd == "compute_root":
            root, leaves = compute_merkle_root(args.files, bundle_root=args.bundle_root)
            print(json.dumps({
                "ok": True,
                "merkle_root": root,
                "leaves": [{"path": l.path, "sha256": l.sha256} for l in leaves],
            }))
            return 0
    except (FileNotFoundError, ValueError) as e:
        print(json.dumps({"ok": False, "error": str(e)}), file=sys.stderr)
        return 2
    except Exception as e:  # pragma: no cover
        print(json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"}), file=sys.stderr)
        return 2

    return 2  # pragma: no cover


if __name__ == "__main__":
    import sys
    sys.exit(_cli(sys.argv[1:]))
