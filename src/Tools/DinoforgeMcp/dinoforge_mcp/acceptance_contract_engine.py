"""Tier-1 MVP acceptance-contract engine for the Autograder test surface.

The engine stays deliberately narrow: it validates the Tier-1 MVP spec bundle
against the repo's canonical doc layout and emits a stable autograder payload.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


SPEC_RELATIVE_PATH = Path("docs/specs/SPEC-TIER1-MVP.md")
SESSION_RELATIVE_PATH = Path("docs/sessions/spec-tier1-mvp-20260608.md")
TRACEABILITY_RELATIVE_PATH = Path("docs/specs/traceability-matrix.md")


@dataclass(frozen=True)
class AcceptanceContractResult:
    """Stable autograder result for the Tier-1 MVP contract."""

    tier: str
    score: float
    passed: bool
    gaps: list[str]
    evidence: dict[str, Any]

    def to_autograder_dict(self) -> dict[str, Any]:
        """Return the exact payload the Autograder expects."""
        return {
            "tier": self.tier,
            "score": self.score,
            "pass": self.passed,
            "gaps": list(self.gaps),
            "evidence": dict(self.evidence),
        }


def evaluate_tier1_mvp_contract(spec_path: Path) -> AcceptanceContractResult:
    """Evaluate the Tier-1 MVP acceptance contract from the canonical spec file.

    A pass requires:
    - the canonical spec file itself,
    - the session pointer doc,
    - and a traceability row for the spec/test wire-up.
    """
    spec_path = spec_path.resolve()
    repo_root = _find_repo_root(spec_path)
    expected_spec_path = (repo_root / SPEC_RELATIVE_PATH).resolve()
    session_path = repo_root / SESSION_RELATIVE_PATH
    traceability_path = repo_root / TRACEABILITY_RELATIVE_PATH

    gaps: list[str] = []
    if spec_path != expected_spec_path:
        gaps.append(f"spec_path mismatch: expected {expected_spec_path}, got {spec_path}")

    spec_text = _read_text(spec_path)
    session_text = _read_text(session_path)
    traceability_text = _read_text(traceability_path)

    if "This specification defines the smallest acceptable evidence set" not in spec_text:
        gaps.append("spec file is missing the Tier-1 MVP contract overview")
    if "docs/specs/SPEC-TIER1-MVP.md" not in session_text:
        gaps.append("session pointer does not link the canonical Tier-1 MVP spec")
    if "SPEC-TIER1-MVP" not in traceability_text:
        gaps.append("traceability matrix does not contain the Tier-1 MVP spec row")
    if "test_tier1_mvp_acceptance_contract_scores_pass" not in traceability_text:
        gaps.append("traceability matrix does not point at the focused autograder test")

    passed = len(gaps) == 0
    score = 1.0 if passed else 0.0
    evidence = {
        "spec_path": str(spec_path),
        "session_path": str(session_path.resolve()),
        "traceability_path": str(traceability_path.resolve()),
    }
    return AcceptanceContractResult(
        tier="mvp",
        score=score,
        passed=passed,
        gaps=gaps,
        evidence=evidence,
    )


def _find_repo_root(path: Path) -> Path:
    """Walk upward until the repo root is found."""
    for candidate in path.parents:
        if (candidate / "docs").is_dir() and (candidate / "src").is_dir():
            return candidate
    raise FileNotFoundError(f"could not locate repo root from {path}")


def _read_text(path: Path) -> str:
    """Read UTF-8 text from a required repo artifact."""
    if not path.exists():
        raise FileNotFoundError(f"required artifact is missing: {path}")
    return path.read_text(encoding="utf-8")
