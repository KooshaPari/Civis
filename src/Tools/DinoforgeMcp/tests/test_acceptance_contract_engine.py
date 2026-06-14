"""Focused Tier-1 MVP acceptance-contract tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from dinoforge_mcp.acceptance_contract_engine import (
    SPEC_RELATIVE_PATH,
    evaluate_tier1_mvp_contract,
)


REPO_ROOT = Path(__file__).resolve().parents[4]


def test_tier1_mvp_acceptance_contract_scores_pass():
    """The canonical Tier-1 MVP spec should wire to an MVP pass."""
    spec_path = REPO_ROOT / SPEC_RELATIVE_PATH
    result = evaluate_tier1_mvp_contract(spec_path).to_autograder_dict()

    assert result["tier"] == "mvp"
    assert result["score"] == pytest.approx(1.0)
    assert result["pass"] is True
    assert result["gaps"] == []

