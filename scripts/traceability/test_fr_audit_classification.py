"""Regression tests for the FR traceability classifier.

These cover three rules that were silently wrong and inflated the reported
coverage:

1. Documents under `docs/` were classified as `code`, so an ID whose only
   reference was a roadmap or design note looked implemented.
2. The ID regex accepted a bare trailing hyphen, so prose like
   `FR-CIV-TACTICS-025-int` produced a phantom `FR-CIV-TACTICS-025-` row.
3. `gen-fr-audit` folded "spec + test, no code reference" into `SPEC-ONLY`,
   which hid the fact that a test already existed.

Run with:  python -m pytest scripts/traceability/test_fr_audit_classification.py
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


gather = _load("fr_gather_ids", ROOT / "docs" / "audits" / "_gather_ids.py")
audit = _load("fr_audit", ROOT / "scripts" / "traceability" / "gen-fr-audit.py")


# --- rule 1: documents are spec sources, not code -------------------------


@pytest.mark.parametrize(
    "path",
    [
        "docs/IMPLEMENTATION_STATUS.md",
        "docs/research/bevy-ecosystem-reference.md",
        "docs/development-guide/p-w1-kickoff.md",
        "docs/guides/COPILOT_L3_AGENTS.md",
        "docs/reference/FR_TRACKER.md",
        "docs/reports/STATUS_REPORT.md",
        "docs/agileplus/epics/civ-w5-scale.md",
        "docs/models/civ-sim/USER_SPEC.md",
    ],
)
def test_documents_classify_as_spec_not_code(path: str) -> None:
    assert gather.classify(path) == "spec"


def test_traceability_docs_stay_trace() -> None:
    assert gather.classify("docs/traceability/index.md") == "trace"


@pytest.mark.parametrize(
    "path",
    [
        "crates/engine/src/lib.rs",
        "web/dashboard/src/lib/civisServer.ts",
        "clients/bevy-ref/src/lib.rs",
    ],
)
def test_real_source_classifies_as_code(path: str) -> None:
    assert gather.classify(path) == "code"


@pytest.mark.parametrize(
    "path",
    [
        "crates/engine/tests/fr_matrix_batch1.rs",
        "web/tests/frame3d.test.mjs",
        "crates/engine/src/engine/engine_tests.rs",
        "crates/watch/src/api_tests.rs",
    ],
)
def test_tests_classify_as_test(path: str) -> None:
    assert gather.classify(path) == "test"


# --- rule 2: no phantom trailing-dash IDs ---------------------------------


def _ids_in(text: str) -> list[str]:
    return [m.group(0) for m in gather.ID_RE.finditer(text)]


def _harvested(text: str) -> list[str]:
    """IDs the scanner actually keeps, applying the truncation filter."""
    out = []
    for m in gather.ID_RE.finditer(text):
        if gather.is_truncated_glob(text, m.start(), m.end(), m.group(0)):
            continue
        out.append(m.group(0))
    return out


@pytest.mark.parametrize(
    "text,expected",
    [
        ("/// FR-CIV-TACTICS-025-int — replay restores queued damage", "FR-CIV-TACTICS-025"),
        ("FR-CIV-BUILD-010-live | Demand allocation graph", "FR-CIV-BUILD-010"),
        ("FR-CIV-PROTO3D-009-live", "FR-CIV-PROTO3D-009"),
        ("Section C, FR-CIV-0100-int1..int4) promote", "FR-CIV-0100"),
        ("FR-CIV-EMERGENCE-N10", "FR-CIV-EMERGENCE-N10"),
        ("FR-CIV-ARCH-A-001", "FR-CIV-ARCH-A-001"),
        ("FR-CIV-PSYCHE-920", "FR-CIV-PSYCHE-920"),
    ],
)
def test_id_regex_stops_at_real_id(text: str, expected: str) -> None:
    assert expected in _ids_in(text)


@pytest.mark.parametrize(
    "text",
    [
        "/// FR-CIV-TACTICS-025-int and more",
        "FR-CIV-BUILD-010-live",
        "Section C, FR-CIV-0100-int1..int4",
    ],
)
def test_id_regex_never_ends_in_hyphen(text: str) -> None:
    for found in _ids_in(text):
        assert not found.endswith("-"), f"phantom ID with trailing hyphen: {found}"


# --- rule 3: tested-but-untagged IDs stay visible -------------------------


def test_classify_spec_plus_test_without_code_is_visible() -> None:
    row = {
        "in_specs": ["docs/traceability/x.md:1"],
        "in_code": [],
        "in_tests": ["crates/engine/tests/x.rs:1"],
    }
    assert audit.classify(row) == "TEST-NO-CODE-REF"


def test_classify_full_coverage() -> None:
    row = {
        "in_specs": ["docs/traceability/x.md:1"],
        "in_code": ["crates/engine/src/lib.rs:1"],
        "in_tests": ["crates/engine/tests/x.rs:1"],
    }
    assert audit.classify(row) == "COVERED"


def test_classify_impl_without_test() -> None:
    row = {
        "in_specs": ["docs/traceability/x.md:1"],
        "in_code": ["crates/engine/src/lib.rs:1"],
        "in_tests": [],
    }
    assert audit.classify(row) == "IMPL-NO-TEST"


def test_classify_spec_only() -> None:
    row = {"in_specs": ["docs/traceability/x.md:1"], "in_code": [], "in_tests": []}
    assert audit.classify(row) == "SPEC-ONLY"


def test_status_order_covers_every_legend_entry() -> None:
    assert set(audit.STATUS_ORDER) == set(audit.STATUS_LEGEND)


# --- rule 4: a self-assertion is not an implementation ----------------------
#
# An ID whose only code reference is a `#[cfg(test)]` block inside its own
# src/*.rs file was counted as code, so a self-assertion produced COVERED.
# 220 IDs are in that state.
#
# Scope limit, stated so a future reader does not over-trust this rule: it
# catches IDs whose code evidence is *entirely* self-test. It does NOT catch
# the broader dead-substrate case, where a symbol is defined, re-exported and
# self-tested but consumed by nothing, because those IDs also carry a
# reference on their definition line. Separating "implemented but unconsumed"
# from "implemented and wired up" needs call-graph analysis, which this
# scanner does not attempt. Those IDs are still reported COVERED today; see
# docs/audits/spec-only-triage-2026-09-29.md finding 2.


def test_classify_self_test_only_is_not_covered() -> None:
    """An ID whose only 'code' ref is its own unit test is not COVERED."""
    row = {
        "in_specs": ["docs/traceability/x.md:1"],
        "in_code": ["crates/hud/src/accessibility.rs:428"],
        "in_tests": ["crates/hud/src/accessibility.rs:428"],
        "self_test_only": True,
    }
    assert audit.classify(row) == "SELF-TEST-ONLY"


@pytest.mark.parametrize(
    "text",
    [
        # Real shape, from crates/mod-host/src/lib.rs:82: the same line carries
        # FR-CIV-MOD-000 and FR-CIV-MOD-001, so the quoted short form is a grep
        # prefix for them and must not become a row of its own.
        '// [unbound] FR-CIV-MOD-000: x. `git grep -n "FR-CIV-MOD-00" -- '
        'docs/specs/CIV-0700-modding-api-spec.md` returns FR-CIV-MOD-001 through -020',
        '```git grep -n "FR-CIV-MOD-00"``` FR-CIV-MOD-000 FR-CIV-MOD-001',
    ],
)
def test_quoted_truncated_prefix_is_not_an_id(text: str) -> None:
    """`FR-CIV-MOD-00` is a quoted grep prefix, not a requirement.

    It existed as an inventory row purely because a comment quoted the shell
    glob `git grep -n "FR-CIV-MOD-00"`. The real ID on that same line is
    FR-CIV-MOD-000. A truncating prefix followed by a quote must not be
    harvested when a longer sibling on the same line proves it is a prefix.
    """
    got = _harvested(text)
    assert "FR-CIV-MOD-00" not in got, f"{text!r} admitted the phantom {got!r}"
    assert "FR-CIV-MOD-000" in got, f"{text!r} lost the real ID, got {got!r}"


def test_truncation_filter_needs_a_longer_sibling() -> None:
    """A quoted complete ID with no longer sibling must survive.

    This is the case that killed the two regex-only attempts: `FR-CIV-MOD-00`
    alone cannot be distinguished from a real truncated-looking ID by quoting
    alone, so the filter additionally requires a strictly longer match of the
    same family somewhere on the same line.
    """
    assert _harvested('See "FR-CIV-MOD-00" for details') == ["FR-CIV-MOD-00"]
    assert _harvested(
        'FR-CIV-MOD-000: x, `git grep -n "FR-CIV-MOD-00"` returns FR-CIV-MOD-001'
    ) == ["FR-CIV-MOD-000", "FR-CIV-MOD-001"]


@pytest.mark.parametrize(
    "text",
    [
        # A *complete* ID is routinely quoted in prose and spec tables. It must
        # still be harvested; only a *truncated* prefix followed by more digits
        # is a glob.
        '`NFR-CIV-001` needs a budget',
        '"FR-CIV-ARCH-003" is a stub',
        "See FR-CIV-INFRA-070 for details.",
        "| FR-ASSET-001 | ... |",
    ],
)
def test_quoted_complete_ids_are_still_harvested(text: str) -> None:
    """Regression: `\d*` in the lookahead deleted 12 real NFR-CIV rows.

    Allowing zero digits made the lookahead fire on any quoted ID, so
    NFR-CIV-001..012 vanished from the matrix entirely rather than changing
    status. The tail must require at least one digit.
    """
    got = _harvested(text)
    assert got, f"{text!r} should still harvest a complete quoted ID, got {got!r}"


def test_real_mod_000_id_is_still_recognised() -> None:
    """The fix must not remove the genuine FR-CIV-MOD-000."""
    assert _ids_in("// [unbound] FR-CIV-MOD-000: DATA-SHAPE-ONLY.") == ["FR-CIV-MOD-000"]


def test_classify_self_test_only_with_no_spec_is_not_covered() -> None:
    """A self-test-only ID with no spec is a self-assertion with no requirement.

    `has_spec` used to gate this branch, so an ID with no spec/traceability
    reference fell through to CODE-ONLY-no-spec. That reads as "there is code
    but nobody wrote the requirement", which is the opposite of what a lone
    `/// FR-...-000` doc comment on a unit test actually is: the ID was minted
    by the test itself and never existed anywhere else.
    """
    row = {
        "in_specs": [],
        "in_traceability": [],
        "in_func_req": None,
        "in_code": ["crates/physics-substrate/src/lib.rs:780"],
        "in_tests": ["crates/physics-substrate/src/lib.rs:780"],
        "self_test_only": True,
    }
    assert audit.classify(row) == "SELF-TEST-ONLY"


def test_classify_self_test_only_with_no_spec_is_not_code_only() -> None:
    """The 8 FR-PHYS-substrate rows must not read as real requirements."""
    row = {
        "in_specs": [],
        "in_code": ["crates/physics-substrate/src/lib.rs:792"],
        "in_tests": ["crates/physics-substrate/src/lib.rs:792"],
        "self_test_only": True,
    }
    assert audit.classify(row) != "CODE-ONLY-no-spec"


def test_classify_code_only_without_self_test_stays_code_only() -> None:
    """Guard the fix: a genuine code-only ID with real code must still bucket."""
    row = {
        "in_specs": [],
        "in_traceability": [],
        "in_func_req": None,
        "in_code": ["crates/mod-host/src/lib.rs:120"],
        "in_tests": [],
        "self_test_only": False,
    }
    assert audit.classify(row) == "CODE-ONLY-no-spec"


def test_classify_real_impl_ref_stays_covered() -> None:
    """A genuine implementation line keeps COVERED even with a self-test."""
    row = {
        "in_specs": ["docs/traceability/x.md:1"],
        "in_code": ["crates/render/src/palette.rs:12"],
        "in_tests": ["crates/hud/src/accessibility.rs:428"],
        "self_test_only": False,
    }
    assert audit.classify(row) == "COVERED"


def test_gather_emits_a_bool_self_test_flag_on_every_row() -> None:
    """The flag must never be a silently missing key."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    rows = data.get("ids", [])
    assert rows, "inventory has no ids"
    for row in rows:
        flagged = row.get("self_test_only")
        assert isinstance(flagged, bool), f"{row['id']} missing self_test_only"
        if flagged:
            assert row.get("in_code"), (
                f"{row['id']} flagged self_test_only but has no code refs"
            )


# --- rule 5: the audit must not cite itself as evidence ---------------------
#
# `scripts/traceability/**` holds the audit tooling itself. Before it was added
# to SELF_REF_DIRS, a literal FR-ID inside a test fixture or a docstring there
# was recorded as a code or test reference, so the tooling counted as evidence
# that the requirement was implemented. Measured: 24 IDs carried such a
# reference. Two of them (FR-CIV-3D and NFR-CIV-PERF-001) had tooling
# references as their ONLY evidence and moved COVERED -> TEST-NO-CODE-REF.


def test_gather_excludes_the_audit_tooling_directory() -> None:
    """`scripts/traceability` must be self-referential, like `docs/audits`."""
    assert "scripts/traceability" in gather.SELF_REF_DIRS
    assert "docs/audits" in gather.SELF_REF_DIRS


def test_is_self_ref_rejects_tooling_paths() -> None:
    assert gather.is_self_ref("scripts/traceability/gen-fr-audit.py")
    assert gather.is_self_ref("scripts/traceability/test_fr_audit_classification.py")
    assert gather.is_self_ref("docs/audits/fr-matrix.json")


def test_is_self_ref_keeps_real_product_code() -> None:
    """The exclusion must not leak into crates/, clients/ or scripts/ generally."""
    assert not gather.is_self_ref("crates/hud/src/accessibility.rs")
    assert not gather.is_self_ref("crates/engine/src/lib.rs")
    assert not gather.is_self_ref("scripts/traceability-tests/helper.py")


def test_no_inventory_row_cites_audit_tooling() -> None:
    """No ID may carry a reference into the audit tooling as evidence."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    offenders = []
    for row in data.get("ids", []):
        for bucket in ("in_code", "in_tests", "in_stub_tests"):
            for ref in row.get(bucket, []):
                path = ref.split(":", 1)[0]
                if path.startswith(("scripts/traceability/", "docs/audits/")):
                    offenders.append(f"{row['id']} {bucket} -> {ref}")
    assert not offenders, "audit tooling cited as evidence:\n" + "\n".join(offenders)


# --- rule 6: an uppercase-only pattern hid a whole namespace ---------------
#
# `crates/physics-substrate` ships a separately-numbered namespace written
# `FR-PHYS-substrate-000..007`, documented item-by-item in src/lib.rs:780-1023.
# The `[A-Z]+` leading-segment pattern rejected all eight, so real implemented
# requirements were invisible to the audit entirely.


@pytest.mark.parametrize(
    "eid",
    ["FR-PHYS-substrate-000", "FR-PHYS-substrate-007"],
)
def test_lowercase_namespace_ids_are_recognised(eid: str) -> None:
    assert gather.ID_RE.fullmatch(eid), f"{eid} should be a valid ID"


@pytest.mark.parametrize(
    "eid",
    ["FR-CIV-3D", "FR-CIV-PROTO3D", "NFR-CIV-ACC-001", "FR-CIV-TACTICS-025"],
)
def test_uppercase_ids_still_recognised(eid: str) -> None:
    assert gather.ID_RE.fullmatch(eid), f"{eid} should still be a valid ID"


@pytest.mark.parametrize(
    "text,expected",
    [
        # Prose fragments of an already-numbered ID must NOT become IDs.
        ("FR-CIV-TACTICS-025-int", "FR-CIV-TACTICS-025"),
        ("FR-CIV-0100-int1..int4", "FR-CIV-0100"),
        ("FR-CIV-BUILD-010-live", "FR-CIV-BUILD-010"),
        # Placeholder-style wildcards are not IDs.
        ("FR-CIV-INSPECT-9xx", None),
        ("FR-CIV-NOTIFY-9xx", None),
        ("FR-CIV-LIFE-014a", None),
    ],
)
def test_lowercase_change_does_not_admit_prose_fragments(text: str, expected) -> None:
    """Lowercase leading segments must not resurrect the phantom-row bug."""
    m = gather.ID_RE.search(text)
    got = m.group(0) if m else None
    assert got == expected, f"{text!r} extracted {got!r}, expected {expected!r}"


def test_physics_substrate_ids_are_in_the_inventory() -> None:
    """The eight substrate IDs must be visible after the regex fix."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    ids = {r["id"] for r in data.get("ids", [])}
    for n in range(8):
        eid = f"FR-PHYS-substrate-{n:03d}"
        assert eid in ids, f"{eid} missing from the inventory"


def test_covers_marker_recognises_lowercase_ids() -> None:
    assert gather.COVERS_RE.match("    /// Covers FR-PHYS-substrate-004 — ok")
