"""Generate a fresh ed25519 keypair for the DINOForge smart-contract proof system.

The public key is committed to docs/proof/keys/<name>.pub; the private key MUST stay
out of git (use docs/proof/keys/.gitignore + a sealed location like ~/.dinoforge/).

Usage:
    python scripts/proof/generate-keys.py --output docs/proof/keys/ed25519-fallback
    # writes <output>.pub (PEM, public)
    # writes <output>.key (PEM, private — DO NOT COMMIT)

The fingerprint printed to stdout matches proof_signing._fingerprint:
    ed25519:<first-16-hex-of-sha256(pub_pem_bytes)>

Exit codes:
    0 on success
    2 on argument error / IO error
"""
from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path


def generate_keypair(output_stem: Path, force: bool = False) -> dict:
    """Write a fresh ed25519 keypair under <output_stem>.{pub,key}.

    Returns a dict {pub_path, key_path, fingerprint} on success.
    Raises FileExistsError if either output exists and `force` is False.
    """
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    output_stem = Path(output_stem)
    pub_path = output_stem.with_suffix(".pub")
    key_path = output_stem.with_suffix(".key")

    if not force:
        for p in (pub_path, key_path):
            if p.exists():
                raise FileExistsError(f"refusing to overwrite existing key: {p} (use --force)")

    output_stem.parent.mkdir(parents=True, exist_ok=True)

    priv = Ed25519PrivateKey.generate()
    pub_pem = priv.public_key().public_bytes(
        serialization.Encoding.PEM,
        serialization.PublicFormat.SubjectPublicKeyInfo,
    )
    key_pem = priv.private_bytes(
        serialization.Encoding.PEM,
        serialization.PrivateFormat.PKCS8,
        serialization.NoEncryption(),
    )

    pub_path.write_bytes(pub_pem)
    key_path.write_bytes(key_pem)
    try:
        # POSIX-only; harmless on Windows
        key_path.chmod(0o600)
    except (OSError, NotImplementedError):
        pass

    fingerprint = "ed25519:" + hashlib.sha256(pub_pem).hexdigest()[:16]
    return {
        "pub_path": str(pub_path),
        "key_path": str(key_path),
        "fingerprint": fingerprint,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Generate a fresh ed25519 keypair for proof signing")
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="output stem (no extension); writes <stem>.pub + <stem>.key",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="overwrite existing key files (DANGEROUS)",
    )
    args = parser.parse_args(argv)

    try:
        result = generate_keypair(args.output, force=args.force)
    except FileExistsError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
    except Exception as e:  # pragma: no cover - defensive
        print(f"ERROR: {type(e).__name__}: {e}", file=sys.stderr)
        return 2

    print(f"public key  : {result['pub_path']}")
    print(f"private key : {result['key_path']}  (DO NOT COMMIT)")
    print(f"fingerprint : {result['fingerprint']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
