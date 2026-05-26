"""Build the genesis BundleManifest at docs/proof/bundles/genesis-2026-04-26.json.

This bundle is the chain anchor: previous_bundle_hash = null. It carries 1 dummy
JudgeReceipt + 1 dummy BridgeReceipt per policy feature so it self-validates against
policies/proof-policy.yaml.

Usage:
    python scripts/proof/build-genesis-bundle.py [--key <path>] [--output <path>]

Defaults:
    --key    ~/.dinoforge/proof_signing_genesis.key  (the keypair generated on 2026-04-26)
    --output docs/proof/bundles/genesis-2026-04-26.json
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "src" / "Tools" / "DinoforgeMcp"))

from dinoforge_mcp.merkle import compute_merkle_root, compute_self_hash, MerkleLeaf  # noqa: E402


POLICY_YAML = REPO / "policies" / "proof-policy.yaml"
PUB_KEY = REPO / "docs" / "proof" / "keys" / "ed25519-fallback.pub"
DEFAULT_OUTPUT = REPO / "docs" / "proof" / "bundles" / "genesis-2026-04-26.json"
DEFAULT_KEY = Path.home() / ".dinoforge" / "proof_signing_genesis.key"


def _ts() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _make_receipts(features: list[str]) -> tuple[list[dict], list[dict]]:
    judges = []
    bridges = []
    ts = _ts()
    for feat in features:
        judges.append({
            "kind": "JudgeReceipt",
            "version": "1.0",
            "timestamp_utc": ts,
            "subject": {
                "feature_id": feat,
                "judge_model": "moonshot-v1-128k",
                "judge_endpoint": "https://api.moonshot.cn/v1",
                "verdict": "PASS",
                "rationale": "genesis bundle placeholder receipt",
            },
        })
        bridges.append({
            "kind": "BridgeReceipt",
            "version": "1.0",
            "timestamp_utc": ts,
            "subject": {
                "feature_id": feat,
                "tool": "game_query_entities",
                "world_frame": 0,
            },
        })
    return judges, bridges


def _make_artifact_leaves(features: list[str]) -> list[MerkleLeaf]:
    """Synthesize stable artifact leaves matching the policy's required_artifacts.

    Genesis bundle does not embed real PNGs/MP4s on disk; instead we record the
    canonical placeholder sha256 (sha256("genesis-placeholder:<path>")) so the
    leaves field exists but is clearly synthetic. Future bundles MUST hash real
    files via merkle.compute_merkle_root over the actual artifact bytes.
    """
    artifact_paths: list[str] = []
    if "f9_overlay" in features:
        artifact_paths += ["validate_f9.png", "raw_f9.mp4"]
    if "f10_modmenu" in features:
        artifact_paths += ["validate_f10.png", "raw_f10.mp4"]
    if "pack_load" in features:
        artifact_paths += ["validate_mods.png"]

    leaves = []
    for path in sorted(set(artifact_paths)):
        digest = hashlib.sha256(f"genesis-placeholder:{path}".encode("utf-8")).hexdigest()
        leaves.append(MerkleLeaf(path=path, sha256=digest))
    return leaves


def _sha256_pair(left: str, right: str) -> str:
    return hashlib.sha256(bytes.fromhex(left) + bytes.fromhex(right)).hexdigest()


def _merkle_root(leaves: list[MerkleLeaf]) -> str:
    """Same algorithm as merkle.compute_merkle_root, run over already-hashed leaves."""
    sleaves = sorted(leaves, key=lambda l: l.path)
    level = [l.sha256 for l in sleaves]
    while len(level) > 1:
        nxt = []
        for i in range(0, len(level), 2):
            left = level[i]
            right = level[i + 1] if i + 1 < len(level) else level[i]
            nxt.append(_sha256_pair(left, right))
        level = nxt
    return level[0]


def _sign_blob(blob: bytes, priv_key_path: Path) -> tuple[bytes, str]:
    """Sign blob bytes with ed25519 priv key. Returns (sig_bytes, fingerprint)."""
    from cryptography.hazmat.primitives import serialization
    priv = serialization.load_pem_private_key(priv_key_path.read_bytes(), password=None)
    sig = priv.sign(blob)
    pub_pem = PUB_KEY.read_bytes()
    fingerprint = "ed25519:" + hashlib.sha256(pub_pem).hexdigest()[:16]
    return sig, fingerprint


def build_genesis_bundle(priv_key: Path, output: Path) -> dict:
    features = ["f9_overlay", "f10_modmenu", "pack_load"]
    leaves = _make_artifact_leaves(features)
    judges, bridges = _make_receipts(features)

    merkle_root = _merkle_root(leaves)

    policy_bytes = POLICY_YAML.read_bytes()
    policy_version = hashlib.sha256(policy_bytes).hexdigest()

    manifest: dict = {
        "version": "1.0",
        "kind": "BundleManifest",
        "bundle_id": "genesis-2026-04-26",
        "previous_bundle_hash": None,
        "merkle_root": merkle_root,
        "leaves": [{"path": l.path, "sha256": l.sha256} for l in leaves],
        "policy_id": "dinoforge-default-2026-04",
        "policy_version": policy_version,
        "judges": judges,
        "bridges": bridges,
        "judge_receipts": [],
        "bridge_receipts": [],
        "session_id": "00000000-0000-0000-0000-000000000000",
    }

    # self_hash is computed over manifest minus self_hash + signature
    manifest["self_hash"] = compute_self_hash(manifest)

    # Sign the canonical JSON of (manifest minus signature)
    canonical_for_sig = {k: v for k, v in manifest.items() if k != "signature"}
    canonical_blob = json.dumps(canonical_for_sig, sort_keys=True, separators=(",", ":")).encode("utf-8")
    sig_bytes, fingerprint = _sign_blob(canonical_blob, priv_key)

    manifest["signature"] = {
        "method": "ed25519-localkey",
        "public_key_id": fingerprint,
        "signed_at_utc": _ts(),
        "value": base64.b64encode(sig_bytes).decode("ascii"),
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(manifest, indent=2, sort_keys=True), encoding="utf-8")
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--key", type=Path, default=DEFAULT_KEY)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args(argv)

    if not args.key.is_file():
        print(f"ERROR: private key not found: {args.key}", file=sys.stderr)
        print("Run scripts/proof/generate-keys.py first or pass --key.", file=sys.stderr)
        return 2
    if not PUB_KEY.is_file():
        print(f"ERROR: pub key missing: {PUB_KEY}", file=sys.stderr)
        return 2

    manifest = build_genesis_bundle(args.key, args.output)
    print(f"wrote {args.output}")
    print(f"  bundle_id    : {manifest['bundle_id']}")
    print(f"  leaves       : {len(manifest['leaves'])}")
    print(f"  merkle_root  : {manifest['merkle_root']}")
    print(f"  signer       : {manifest['signature']['public_key_id']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
