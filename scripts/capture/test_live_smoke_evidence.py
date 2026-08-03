import json
import subprocess
import sys
from pathlib import Path

import pytest
from live_smoke_evidence import EvidenceError, collect_provenance, emit, redact_text


def _git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()


def _manifest(root: Path, sha: str) -> Path:
    path = root / ".ci" / "quality-manifest.json"
    path.parent.mkdir()
    path.write_text(
        json.dumps(
            {
                "version": "1",
                "repo": "Civis",
                "git_sha": sha,
                "manifest_hash": "a" * 64,
                "gates": {},
            }
        ),
        encoding="utf-8",
    )
    return path


def test_redact_text_removes_query_and_secrets() -> None:
    raw = "GET https://example.test/smoke?token=secret&query=private Bearer abc sk-test-123"
    redacted = redact_text(raw)
    assert "secret" not in redacted
    assert "private" not in redacted
    assert "abc" not in redacted
    assert "sk-test-123" not in redacted
    assert "<redacted>" in redacted


def test_collect_provenance_requires_exact_manifest_sha(tmp_path: Path) -> None:
    sha = _git(tmp_path, "rev-parse", "HEAD") if (tmp_path / ".git").exists() else ""
    if not sha:
        subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.test"],
            cwd=tmp_path,
            check=True,
        )
        subprocess.run(["git", "config", "user.name", "Test"], cwd=tmp_path, check=True)
        (tmp_path / "tracked").write_text("tracked\n", encoding="utf-8")
        subprocess.run(["git", "add", "tracked"], cwd=tmp_path, check=True)
        subprocess.run(["git", "commit", "-qm", "test"], cwd=tmp_path, check=True)
        sha = _git(tmp_path, "rev-parse", "HEAD")
    manifest = _manifest(tmp_path, sha)
    provenance = collect_provenance(tmp_path, manifest)
    assert provenance["repo_sha"] == sha
    assert provenance["manifest_sha"] == sha
    manifest.write_text(
        manifest.read_text(encoding="utf-8").replace(sha, "0" * 40), encoding="utf-8"
    )
    with pytest.raises(EvidenceError, match="stale"):
        collect_provenance(tmp_path, manifest)


def test_emit_records_exit_timeline_hash_and_not_claimed_defaults(
    tmp_path: Path,
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    subprocess.run(
        ["git", "config", "user.email", "test@example.test"], cwd=tmp_path, check=True
    )
    subprocess.run(["git", "config", "user.name", "Test"], cwd=tmp_path, check=True)
    (tmp_path / "tracked").write_text("tracked\n", encoding="utf-8")
    subprocess.run(["git", "add", "tracked"], cwd=tmp_path, check=True)
    subprocess.run(["git", "commit", "-qm", "test"], cwd=tmp_path, check=True)
    sha = _git(tmp_path, "rev-parse", "HEAD")
    manifest = _manifest(tmp_path, sha)
    artifact = tmp_path / "raw.log"
    artifact.write_text("raw output\n", encoding="utf-8")
    evidence = emit(
        tmp_path,
        manifest,
        [f"{sys.executable} -c 'print(\"ok\")'"],
        [("raw", artifact)],
    )
    assert evidence["repo"]["repo_sha"] == sha
    assert evidence["repo"]["manifest_sha"] == sha
    assert evidence["commands"][0]["exit_code"] == 0
    assert evidence["commands"][0]["duration_ms"] >= 0
    assert [item["event"] for item in evidence["timeline"]] == [
        "run_started",
        "command_started",
        "command_finished",
        "run_finished",
    ]
    assert evidence["artifacts"][0]["sha256"]
    assert evidence["claims"]["ca"]["status"] == "not_claimed"
    assert evidence["claims"]["windowed"]["status"] == "not_claimed"
    assert evidence["claims"]["live_smoke"]["status"] == "claimed"
