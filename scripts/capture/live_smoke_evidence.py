#!/usr/bin/env python3
"""Emit redacted, provenance-bound evidence for the Civis live smoke gate.

The emitter intentionally does not infer a CA or windowed-client result. Those
claims remain ``not_claimed`` until an explicitly linked raw artifact exists.
Commands are executed without a shell so query strings and secrets can be
redacted before they are persisted in the evidence bundle.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "civis.live-smoke-evidence.v1"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SENSITIVE_KEY_RE = re.compile(
    r"(?i)(?P<key>(?:api[_-]?key|authorization|bearer|password|secret|token))"
    r"(?P<sep>\s*[:=]\s*)(?P<value>[^\s,;]+)"
)
URL_QUERY_RE = re.compile(r"(?P<url>https?://[^\s'\"]+?)\?(?P<query>[^\s'\"]*)", re.I)
BEARER_RE = re.compile(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+")
TOKEN_RE = re.compile(r"\b(?:sk|ghp|github_pat|xoxb|xoxp)-[A-Za-z0-9._-]+\b")
MAX_OUTPUT = 4096


class EvidenceError(RuntimeError):
    """Raised when evidence cannot be tied to an exact repository state."""


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def redact_text(value: str) -> str:
    """Redact URLs' queries/fragments and common secret-bearing values."""
    value = URL_QUERY_RE.sub(lambda m: f"{m.group('url')}?<redacted>", value)
    value = SENSITIVE_KEY_RE.sub(lambda m: f"{m.group('key')}=<redacted>", value)
    value = BEARER_RE.sub("Bearer <redacted>", value)
    value = TOKEN_RE.sub("<redacted-token>", value)
    return value


def bounded_output(value: str) -> str:
    value = redact_text(value)
    if len(value) <= MAX_OUTPUT:
        return value
    return f"{value[:MAX_OUTPUT]}\n<redacted-output-truncated>"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _git(root: Path, *args: str) -> str:
    try:
        return subprocess.check_output(
            ["git", *args], cwd=root, text=True, stderr=subprocess.STDOUT
        ).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        detail = getattr(exc, "output", "") or str(exc)
        raise EvidenceError(
            f"git provenance unavailable: {redact_text(detail)}"
        ) from exc


def collect_provenance(root: Path, manifest: Path) -> dict[str, str]:
    """Require a valid quality manifest attesting the exact checked-out HEAD."""
    root = root.resolve()
    repo_sha = _git(root, "rev-parse", "HEAD")
    if not SHA_RE.fullmatch(repo_sha):
        raise EvidenceError(f"invalid repository SHA: {redact_text(repo_sha)}")
    manifest = manifest.resolve()
    if not manifest.is_file():
        raise EvidenceError(f"quality manifest missing: {manifest.name}")
    try:
        body = json.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise EvidenceError(f"quality manifest unreadable: {manifest.name}") from exc
    manifest_sha = body.get("git_sha")
    if not isinstance(manifest_sha, str) or not SHA_RE.fullmatch(manifest_sha):
        raise EvidenceError("quality manifest git_sha is missing or invalid")
    if manifest_sha != repo_sha:
        raise EvidenceError(
            f"quality manifest is stale: manifest={manifest_sha} repo={repo_sha}"
        )
    manifest_hash = body.get("manifest_hash")
    if not isinstance(manifest_hash, str) or not manifest_hash:
        raise EvidenceError("quality manifest manifest_hash is missing")
    gates = body.get("gates")
    if not isinstance(gates, dict):
        raise EvidenceError("quality manifest gates must be an object")
    attestation = {
        "git_sha": manifest_sha,
        "gates": sorted(
            [
                {"key": key, "status": value.get("status", "")}
                for key, value in gates.items()
            ],
            key=lambda item: item["key"],
        ),
    }
    expected_hash = hashlib.blake2b(
        json.dumps(attestation, separators=(",", ":")).encode(), digest_size=32
    ).hexdigest()
    if manifest_hash != expected_hash:
        raise EvidenceError("quality manifest manifest_hash mismatch")
    try:
        manifest_path = manifest.relative_to(root).as_posix()
    except ValueError as exc:
        raise EvidenceError("quality manifest must be inside repository root") from exc
    return {
        "repo_sha": repo_sha,
        "manifest_sha": manifest_sha,
        "manifest_hash": manifest_hash,
        "manifest_file_sha256": sha256_file(manifest),
        "manifest_path": manifest_path,
    }


def _relative_artifact(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return "<external>/" + path.name


def artifact_record(root: Path, path: Path, kind: str) -> dict[str, Any]:
    if not path.is_file():
        raise EvidenceError(f"raw artifact missing: {path.name}")
    return {
        "kind": kind,
        "path": _relative_artifact(root, path),
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def run_command(root: Path, command: str, timeout: float | None) -> dict[str, Any]:
    try:
        argv = shlex.split(command)
    except ValueError as exc:
        raise EvidenceError(f"invalid command syntax: {redact_text(command)}") from exc
    if not argv:
        raise EvidenceError("empty command is not evidence")
    started = utc_now()
    monotonic = time.monotonic()
    timed_out = False
    try:
        result = subprocess.run(
            argv,
            cwd=root,
            env=os.environ.copy(),
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout,
        )
        exit_code = result.returncode
        stdout = result.stdout
        stderr = result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        exit_code = 124
        stdout = exc.stdout or ""
        stderr = (exc.stderr or "") + "\n<command-timeout>"
    ended = utc_now()
    return {
        "argv": [redact_text(item) for item in argv],
        "command": redact_text(shlex.join(argv)),
        "started_at": started,
        "ended_at": ended,
        "duration_ms": round((time.monotonic() - monotonic) * 1000, 3),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "stdout": bounded_output(stdout),
        "stderr": bounded_output(stderr),
    }


def emit(
    root: Path,
    manifest: Path,
    commands: list[str],
    artifacts: list[tuple[str, Path]],
    timeout: float | None = None,
) -> dict[str, Any]:
    provenance = collect_provenance(root, manifest)
    started = utc_now()
    timeline: list[dict[str, str]] = [{"at": started, "event": "run_started"}]
    command_records: list[dict[str, Any]] = []
    for command in commands:
        record = run_command(root, command, timeout)
        command_records.append(record)
        timeline.append({"at": record["started_at"], "event": "command_started"})
        timeline.append({"at": record["ended_at"], "event": "command_finished"})
    artifact_records = [artifact_record(root, path, kind) for kind, path in artifacts]
    command_passed = bool(command_records) and all(
        item["exit_code"] == 0 and not item["timed_out"] for item in command_records
    )
    linked = {item["kind"] for item in artifact_records}
    finished = utc_now()
    timeline.append({"at": finished, "event": "run_finished"})
    return {
        "schema_version": SCHEMA_VERSION,
        "repo": {"name": "Civis", **provenance},
        "run": {
            "started_at": started,
            "ended_at": finished,
            "status": "pass" if command_passed else "fail",
        },
        "commands": command_records,
        "timeline": timeline,
        "artifacts": artifact_records,
        "claims": {
            "live_smoke": {
                "status": "claimed" if command_passed else "not_claimed",
                "evidence": "commands" if command_passed else None,
            },
            "ca": {
                "status": "claimed" if "ca" in linked else "not_claimed",
                "evidence": next(
                    (item["path"] for item in artifact_records if item["kind"] == "ca"),
                    None,
                ),
            },
            "windowed": {
                "status": "claimed" if "windowed" in linked else "not_claimed",
                "evidence": next(
                    (
                        item["path"]
                        for item in artifact_records
                        if item["kind"] == "windowed"
                    ),
                    None,
                ),
            },
        },
        "redaction": {
            "applied": True,
            "scope": ["commands", "stdout", "stderr", "artifact_paths"],
            "secret_and_query_values_omitted": True,
        },
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--manifest", type=Path, default=None)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--command", action="append", default=[])
    parser.add_argument("--artifact", action="append", type=Path, default=[])
    parser.add_argument("--ca-artifact", type=Path)
    parser.add_argument("--windowed-artifact", type=Path)
    parser.add_argument("--timeout", type=float, default=None)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    root = args.root.resolve()
    manifest = (args.manifest or root / ".ci/quality-manifest.json").resolve()
    artifacts = [("raw", path.resolve()) for path in args.artifact]
    if args.ca_artifact:
        artifacts.append(("ca", args.ca_artifact.resolve()))
    if args.windowed_artifact:
        artifacts.append(("windowed", args.windowed_artifact.resolve()))
    try:
        evidence = emit(root, manifest, args.command, artifacts, args.timeout)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    except EvidenceError as exc:
        print(f"live-smoke-evidence: {exc}", file=sys.stderr)
        return 2
    print(
        f"wrote {args.output} (repo_sha={evidence['repo']['repo_sha']}, "
        f"manifest_sha={evidence['repo']['manifest_sha']})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
