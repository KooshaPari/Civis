"""Smart-contract proof signing — ed25519 local key (Phase 1).
cosign-sigstore integration deferred to Phase 2.

Per spec section 3: docs/design/2026-04-25-smart-contract-proof-system.md
"""
from __future__ import annotations
import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


LOCAL_KEY_PATH = Path.home() / ".dinoforge" / "proof_signing.key"
LOCAL_PUBKEY_PATH = Path.home() / ".dinoforge" / "proof_signing.pub"


@dataclass
class SigningResult:
    method: str
    signature_path: Path
    signer_identity: str


@dataclass
class VerificationResult:
    valid: bool
    method: str
    signer_identity: str
    error: Optional[str] = None


def _import_ed25519():
    from cryptography.hazmat.primitives.asymmetric import ed25519
    from cryptography.hazmat.primitives import serialization
    from cryptography.exceptions import InvalidSignature
    return ed25519, serialization, InvalidSignature


def _ensure_local_keypair() -> tuple[Path, Path]:
    LOCAL_KEY_PATH.parent.mkdir(parents=True, exist_ok=True)
    if LOCAL_KEY_PATH.exists() and LOCAL_PUBKEY_PATH.exists():
        return LOCAL_KEY_PATH, LOCAL_PUBKEY_PATH

    ed25519, serialization, _ = _import_ed25519()
    priv = ed25519.Ed25519PrivateKey.generate()
    LOCAL_KEY_PATH.write_bytes(priv.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    ))
    LOCAL_PUBKEY_PATH.write_bytes(priv.public_key().public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    ))
    LOCAL_KEY_PATH.chmod(0o600)
    return LOCAL_KEY_PATH, LOCAL_PUBKEY_PATH


def _fingerprint(pub_path: Path) -> str:
    return "ed25519:" + hashlib.sha256(pub_path.read_bytes()).hexdigest()[:16]


def sign_blob(path: Path) -> SigningResult:
    """Sign a file with ed25519 local key. Returns path to .sig file."""
    path = Path(path)
    if not path.is_file():
        raise FileNotFoundError(f"sign_blob: {path}")

    priv_path, pub_path = _ensure_local_keypair()
    _, serialization, _ = _import_ed25519()
    priv = serialization.load_pem_private_key(priv_path.read_bytes(), password=None)
    sig = priv.sign(path.read_bytes())
    sig_path = path.with_suffix(path.suffix + ".sig")
    sig_path.write_bytes(sig)
    return SigningResult("ed25519-localkey", sig_path, _fingerprint(pub_path))


def verify_blob(path: Path, sig_path: Optional[Path] = None) -> VerificationResult:
    """Verify a file's signature against the local public key."""
    path = Path(path)
    if sig_path is None:
        sig_path = path.with_suffix(path.suffix + ".sig")
    sig_path = Path(sig_path)
    if not sig_path.exists():
        return VerificationResult(False, "ed25519-localkey", "", error=f"sig missing: {sig_path}")
    if not LOCAL_PUBKEY_PATH.exists():
        return VerificationResult(False, "ed25519-localkey", "", error="local pubkey missing")

    _, serialization, InvalidSignature = _import_ed25519()
    pub = serialization.load_pem_public_key(LOCAL_PUBKEY_PATH.read_bytes())
    try:
        pub.verify(sig_path.read_bytes(), path.read_bytes())
        return VerificationResult(True, "ed25519-localkey", _fingerprint(LOCAL_PUBKEY_PATH))
    except InvalidSignature as e:
        return VerificationResult(False, "ed25519-localkey", _fingerprint(LOCAL_PUBKEY_PATH), error=str(e))


def get_signer_identity() -> str:
    if LOCAL_PUBKEY_PATH.exists():
        return _fingerprint(LOCAL_PUBKEY_PATH)
    _, pub_path = _ensure_local_keypair()
    return _fingerprint(pub_path)


# ---------------------------------------------------------------------------
# CLI shim — invoked by prove-features-gate.ps1 (Phase 2)
# Usage:
#   python -m dinoforge_mcp.proof_signing sign <file>
#   python -m dinoforge_mcp.proof_signing verify <file> [<sig_path>]
#   python -m dinoforge_mcp.proof_signing identity
# Exit codes:
#   0 = ok / valid
#   1 = invalid signature / verification failed
#   2 = usage / IO error
# Output:
#   JSON status to stdout. Errors to stderr.
# ---------------------------------------------------------------------------
def _cli(argv: list[str]) -> int:
    import argparse
    import json as _json
    import sys

    parser = argparse.ArgumentParser(prog="dinoforge_mcp.proof_signing")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_sign = sub.add_parser("sign", help="sign a file with the local ed25519 key")
    p_sign.add_argument("path", type=Path)

    p_verify = sub.add_parser("verify", help="verify a file's signature")
    p_verify.add_argument("path", type=Path)
    p_verify.add_argument("sig_path", type=Path, nargs="?", default=None)

    sub.add_parser("identity", help="print the local signer identity")

    args = parser.parse_args(argv)

    try:
        if args.cmd == "sign":
            r = sign_blob(args.path)
            print(_json.dumps({
                "ok": True,
                "method": r.method,
                "signature_path": str(r.signature_path),
                "signer_identity": r.signer_identity,
            }))
            return 0
        if args.cmd == "verify":
            r = verify_blob(args.path, args.sig_path)
            print(_json.dumps({
                "ok": r.valid,
                "method": r.method,
                "signer_identity": r.signer_identity,
                "error": r.error,
            }))
            return 0 if r.valid else 1
        if args.cmd == "identity":
            print(_json.dumps({"ok": True, "signer_identity": get_signer_identity()}))
            return 0
    except FileNotFoundError as e:
        print(_json.dumps({"ok": False, "error": f"file not found: {e}"}), file=sys.stderr)
        return 2
    except Exception as e:  # pragma: no cover - defensive
        print(_json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"}), file=sys.stderr)
        return 2

    return 2  # pragma: no cover


if __name__ == "__main__":
    import sys
    sys.exit(_cli(sys.argv[1:]))
