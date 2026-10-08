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
import json
import os
import re
import sys
import tempfile
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


# --- rule 7: a self-minted alias hid five implemented requirements ---------
#
# `crates/species/src/speciation.rs` implements the whole Hamming-distance
# speciation behaviour that `docs/design/species-sentience.md:120-124` defines as
# FR-CIV-SPECIES-300..304, but the file tagged itself with an invented
# digitless ID `FR-CIV-SPECIATION`. Consequences, both real:
#
#   * SPECIES-300..304 stayed SPEC-ONLY even though code and tests exist, so the
#     matrix reported a spec gap that does not exist.
#   * `FR-CIV-SPECIATION` would have been admitted by any digitless allowlist as
#     a phantom row.
#
# The fix is to re-tag the code with the authoritative IDs, NOT to admit the
# alias. The alias must stay out of the inventory entirely.


def test_speciation_alias_is_not_admitted_as_an_id() -> None:
    """`FR-CIV-SPECIATION` must never become a traceability row."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    ids = {r["id"] for r in data.get("ids", [])}
    assert "FR-CIV-SPECIATION" not in ids, "self-minted alias leaked into the inventory"


def test_species_3xx_are_not_spec_only() -> None:
    """Each SPECIES-3xx requirement has code plus tests, so none may be SPEC-ONLY."""
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    data = json.loads(matrix.read_text(encoding="utf-8"))
    rows = {r["id"]: r for r in data.get("rows", [])}
    for n in range(300, 305):
        eid = f"FR-CIV-SPECIES-{n}"
        assert eid in rows, f"{eid} missing from the matrix"
        assert rows[eid]["status"] != "SPEC-ONLY", (
            f"{eid} is implemented in crates/species/src/speciation.rs but reported SPEC-ONLY"
        )


@pytest.mark.parametrize(
    "ref",
    [
        "crates/species/src/speciation.rs",
        "crates/genetics/src/lib.rs",
    ],
)
def test_speciation_implementation_cites_species_3xx(ref: str) -> None:
    """The implementing files must cite the authoritative IDs, not the alias."""
    text = (ROOT / ref).read_text(encoding="utf-8")
    assert "FR-CIV-SPECIATION" not in text, (
        f"{ref} still cites the self-minted alias FR-CIV-SPECIATION"
    )
    assert "FR-CIV-SPECIES-30" in text, f"{ref} cites no authoritative SPECIES-3xx ID"


# --- rule 9: two self-minted CONTENT IDs double-tag FR-API-001 -------------
#
# `crates/engine/src/scenario.rs` declares `FR-API-001` at line 1 and defines the
# scenario YAML schema. Inside that same schema, `SeedWeight` (line 31) and
# `ScenarioStartingConditions` (line 43) tag themselves with invented digitless
# IDs `FR-CONTENT-SEEDMIX` and `FR-CONTENT-STARTCOND`.
#
# `agileplus-specs/civ-013-research-api/spec.md:25` defines FR-API-001 as
# "Scenario YAML format -- versioned schema specifying map dimensions, entity
# placement, starting conditions, policy parameters". "starting conditions" is
# named verbatim. Both types are fields of the FR-API-001 schema struct, so the
# digitless labels split one specified requirement into two unspecified ones.
#
# `FR-CONTENT` has zero rows in the matrix; the namespace is entirely self-minted.
# The fix is to re-tag with FR-API-001, not to admit FR-CONTENT-* as requirements.

SCENARIO_ALIASES = ["FR-CONTENT-SEEDMIX", "FR-CONTENT-STARTCOND"]


@pytest.mark.parametrize("alias", SCENARIO_ALIASES)
def test_scenario_alias_is_not_admitted_as_an_id(alias: str) -> None:
    """A self-minted CONTENT alias must never become a traceability row."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    ids = {r["id"] for r in data.get("ids", [])}
    assert alias not in ids, f"self-minted alias {alias} leaked into the inventory"


@pytest.mark.parametrize(
    "ref",
    [
        "crates/engine/src/scenario.rs",
        "crates/engine/src/engine/engine_tests.rs",
    ],
)
def test_scenario_implementation_cites_fr_api_001(ref: str) -> None:
    """The scenario schema files must cite FR-API-001, not a CONTENT alias.

    These files already declared FR-API-001 before the aliases existed. The
    aliases were added later, splitting one requirement into three names.
    """
    text = (ROOT / ref).read_text(encoding="utf-8")
    for alias in SCENARIO_ALIASES:
        assert alias not in text, f"{ref} still cites the self-minted alias {alias}"
    assert "FR-API-001" in text, f"{ref} cites no authoritative FR-API-001"


def test_fr_api_001_is_covered() -> None:
    """FR-API-001 owns the scenario schema, so it must not regress to SPEC-ONLY."""
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    data = json.loads(matrix.read_text(encoding="utf-8"))
    rows = {r["id"]: r for r in data.get("rows", [])}
    assert "FR-API-001" in rows, "FR-API-001 missing from the matrix"
    assert rows["FR-API-001"]["status"] == "COVERED", (
        "FR-API-001 owns crates/engine/src/scenario.rs and has integration tests in "
        f"crates/engine/tests/fr_matrix_batch1.rs, but is {rows['FR-API-001']['status']}"
    )


# --- rule 10: the audit cannot see digitless IDs by construction ------------
#
# `docs/audits/_gather_ids.py` requires a numeric segment, so all 9 digitless
# code-side tokens are invisible to the matrix. That is why they survived three
# audit rounds. The classification is recorded in
# docs/audits/digitless-code-ids-2026-10-02.md: 2 are misfiled duplicates of
# FR-API-001, 7 are undocumented behaviour.
#
# The 7 undocumented ones must NOT be admitted. Admitting a token found only in
# code comments would mint a requirement from an implementation, which is the
# defect class this audit exists to detect. They stay recorded, not counted.
#
# This test pins the classification so it cannot silently drift into the matrix,
# and it guards the gatherer's blind spot: if someone widens the ID regex to
# admit digitless tokens, these 9 must still be excluded by name.

UNDOCUMENTED_DIGITLESS = [
    "FR-CIV-LEGAL-PRECEDENT",
    "FR-CIV-NICHE-ADAPT",
    "FR-CIV-TECH-OBSOLETE",
    "FR-CIV-phasewire",
    "FR-CLIENT-godbuttons",
    "FR-ENGINE-phaseorder",
    "FR-RELIG-readapi",
]


@pytest.mark.parametrize("token", UNDOCUMENTED_DIGITLESS)
def test_undocumented_digitless_token_is_not_admitted(token: str) -> None:
    """An ID that only exists in code comments must not become a requirement row."""
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    if not inventory.exists():
        pytest.skip("id inventory not generated")
    import json

    data = json.loads(inventory.read_text(encoding="utf-8"))
    ids = {r["id"] for r in data.get("ids", [])}
    assert token not in ids, (
        f"{token} was found only in code comments with no spec backing, "
        "but it leaked into the inventory as a requirement"
    )


def test_digitless_gather_blind_spot_is_documented() -> None:
    """The gatherer's numeric-segment requirement is what hid all 9 tokens.

    If this ever stops being true, the 9 tokens start reaching the matrix by
    accident rather than by decision, and this audit's premise changes.
    """
    source = (ROOT / "docs" / "audits" / "_gather_ids.py").read_text(encoding="utf-8")
    assert re.search(r"\\d", source), (
        "_gather_ids.py no longer requires a numeric segment; the 9 digitless tokens "
        "recorded in docs/audits/digitless-code-ids-2026-10-02.md must be re-classified "
        "before any of them can reach the matrix"
    )


# --- rule 11: traceability dirs are template-filled, not authored ----------
#
# `docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md` contains, verbatim,
# "TBD -- The architectural decision for FR-CIV-TECH-003 needs to be finalized
# based on implementation exploration." Every row in the matrix credits ~6 such
# template files in its spec_refs. They restate the ID rather than specify
# behaviour, which is why `docs/traceability/index.md`'s Completeness column was
# relabelled `Artifacts` in 95b506cf.
#
# This test does not delete the template credit -- that is a status-affecting
# change and is recorded as an open question in the findings doc rather than
# made silently. It asserts the honest thing: the templates are identifiable, so
# any future attempt to credit them as requirements can be caught.


def test_traceability_templates_are_identifiable() -> None:
    """Template-filled traceability docs are detectable by their placeholder text."""
    adr = (ROOT / "docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md").read_text(
        encoding="utf-8"
    )
    assert "needs to be finalized based on implementation exploration" in adr, (
        "fr-civ-tech-003-adr.md no longer contains the template placeholder; if it was "
        "genuinely authored, update docs/audits/digitless-code-ids-2026-10-02.md, which "
        "records this file as template-filled"
    )


# --- rule 12: the spec_refs a row rests on, not the count of them ----------
#
# `has_spec` is a boolean (gen-fr-audit.py:95), so padding spec_refs with template
# files cannot buy coverage. That is the reassuring direction, and it is only
# half the story: the other half is that deleting the whole channel is NOT free.
#
# Measured on the committed inventory with the committed generator:
#   stripping in_traceability from 1408 entries (10179 refs) moves 180 statuses,
#   COVERED 840 -> 728, and would mint 180 unsupported CODE-ONLY-no-spec rows.
# So traceability docs are load-bearing spec evidence, not padding.
#
# Two claims are asserted here, and both were false before this audit:
#   * no row is COVERED on boilerplate alone (the earlier estimate was 112)
#   * no classifier of "authored vs template" may be derived from similarity
# See docs/audits/traceability-template-credit-2026-10-02.md.

PLACEHOLDER_TEXT = (
    "TBD -- The architectural decision for",
    "> _To be implemented._",
    "needs to be finalized based on implementation exploration",
    "> _No test coverage yet._",
)

# Six rows that are implemented and tested against a spec that only restates the
# epic name. Their `Covers` lines and test names are real; their intent docs are
# generator output. Recorded as documentation gaps, not implementation gaps.
DOC_GAP_ROWS = (
    "FR-CIV-ENGINE-INT-001",
    "FR-CIV-ENGINE-INT-005",
    "FR-CIV-ENGINE-INT-011",
    "FR-CIV-ENGINE-INT-014",
    "FR-CIV-ENGINE-REPLAY-003",
    "FR-CIV-LIFE-035",
)


def _inventory() -> dict:
    inv = ROOT / "docs/audits/_id_inventory_v3.json"
    if not inv.exists():
        pytest.skip("id inventory not present")
    return {e["id"]: e for e in json.loads(inv.read_text(encoding="utf-8"))["ids"]}


def _matrix_status() -> dict:
    m = ROOT / "docs/audits/fr-matrix.json"
    if not m.exists():
        pytest.skip("fr-matrix not present")
    return {r["id"]: r["status"] for r in json.loads(m.read_text(encoding="utf-8"))["rows"]}


def _is_placeholder(path: str) -> bool:
    p = ROOT / path
    if not p.exists():
        return False
    text = p.read_text(encoding="utf-8", errors="replace")
    return any(h in text for h in PLACEHOLDER_TEXT)


def test_no_row_is_covered_by_boilerplate_alone() -> None:
    """No COVERED row has every one of its spec refs in a placeholder file.

    This is the guard on the audit's headline claim. `has_spec` being a boolean
    is what makes it true; if that ever becomes a count, this test fails.
    """
    inv, status = _inventory(), _matrix_status()
    offenders = []
    for eid, e in inv.items():
        refs = [r.split(":")[0] for r in (e.get("in_traceability") or [])]
        if not refs:
            continue
        if status.get(eid) != "COVERED":
            continue
        if e.get("in_specs") or e.get("in_func_req"):
            continue
        if all(_is_placeholder(p) for p in refs):
            offenders.append(eid)
    assert not offenders, (
        "these COVERED rows rest entirely on placeholder traceability docs, which "
        "means their coverage is credited to boilerplate: " + ", ".join(sorted(offenders))
    )


def test_has_spec_is_a_boolean_not_a_count() -> None:
    """`has_spec` must stay a boolean or a single template ref could outweigh spec.

    A regression guard on the exact expression that made rule 12's claim hold.
    """
    src = (ROOT / "scripts/traceability/gen-fr-audit.py").read_text(encoding="utf-8")
    m = re.search(r"^\s*has_spec\s*=.*$", src, re.M)
    assert m, "gen-fr-audit.py no longer defines has_spec"
    line = m.group(0)
    assert "bool(" in line, f"has_spec is no longer wrapped in bool(): {line.strip()!r}"
    assert "len(" not in line, (
        f"has_spec now counts references instead of testing truthiness: {line.strip()!r}. "
        "That would let template padding buy coverage."
    )


@pytest.mark.parametrize("eid", DOC_GAP_ROWS)
def test_doc_gap_rows_carry_real_code_and_tests(eid: str) -> None:
    """The six doc-gap rows must keep real implementation evidence.

    Their specification is generator boilerplate, so the only thing separating
    them from `CODE-ONLY-no-spec` is genuine code and a genuine test. If someone
    deletes the code and leaves the tags, this fails.
    """
    inv = _inventory()
    e = inv.get(eid)
    assert e is not None, f"{eid} vanished from the inventory"

    code = [r for r in (e.get("in_code") or [])]
    tests = [r for r in (e.get("in_tests") or [])]
    assert code, f"{eid} has no code refs but is recorded as a documentation gap"
    assert tests, f"{eid} has no test refs but is recorded as a documentation gap"

    # Every cited file:line must resolve to a non-empty line. A tag pointing at
    # a moved or deleted line is exactly how these rows become phantom.
    for ref in code + tests:
        path, _, line = ref.partition(":")
        p = ROOT / path
        assert p.exists(), f"{eid} cites missing file {path}"
        assert line.isdigit(), f"{eid} cites a non-numeric line: {ref!r}"
        lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        n = int(line)
        assert 1 <= n <= len(lines), f"{eid} cites out-of-range {ref} (file has {len(lines)})"
        assert lines[n - 1].strip(), f"{eid} cites a blank line: {ref}"


@pytest.mark.parametrize("eid", DOC_GAP_ROWS)
def test_doc_gap_rows_still_credit_boilerplate(eid: str) -> None:
    """Pin the defect class: these specs really are placeholders.

    If someone writes the requirements for these six, this test fails and that
    is the correct signal -- remove the ID from DOC_GAP_ROWS at that point, and
    the finding doc can be updated. Silently rewriting the doc instead would
    hide that the generator had been credited for real requirements.
    """
    d = ROOT / "docs/traceability" / eid.lower()
    if not d.is_dir():
        pytest.skip(f"{eid} traceability dir not present")
    refs = sorted({p.name for p in d.glob("*.md")})
    assert refs, f"{eid} has no traceability docs at all"
    assert any(_is_placeholder(f"docs/traceability/{eid.lower()}/{n}") for n in refs), (
        f"{eid}'s traceability docs no longer contain placeholder text; they may now be "
        "genuine requirements. Remove the ID from DOC_GAP_ROWS and update "
        "docs/audits/traceability-template-credit-2026-10-02.md."
    )


def test_shared_traceability_credits_are_not_phantom() -> None:
    """47 rows cite a shared matrix instead of a per-ID dir. All must name their ID.

    A shared matrix that mentions an ID in passing is not a spec reference. This
    checks the reference resolves to the ID actually being credited.
    """
    shared = {
        "docs/traceability/emergent-systems-tracelinks.md",
        "docs/traceability/fr-emergence-matrix.md",
        "docs/traceability/nfr-matrix.md",
    }
    inv, status = _inventory(), _matrix_status()
    cache: dict[str, str] = {}
    bad = []
    checked = 0
    for eid, e in inv.items():
        for ref in (e.get("in_traceability") or []):
            path = ref.split(":")[0]
            if path not in shared:
                continue
            checked += 1
            if path not in cache:
                p = ROOT / path
                if not p.exists():
                    bad.append(f"{eid} -> missing {path}")
                    break
                cache[path] = p.read_text(encoding="utf-8", errors="replace")
            if eid not in cache[path]:
                bad.append(f"{eid} credited by {path} but absent from it")
    assert checked, "no shared-matrix references found; the count changed"
    assert not bad, "phantom shared-matrix credits: " + "; ".join(sorted(set(bad))[:10])


def test_reserved_nfr_rows_are_reported_spec_only() -> None:
    """`nfr-matrix.md` rows read 'Reserved ... tbd' and have no code or tests.

    That is a placeholder, and SPEC-ONLY is the honest verdict for one. If they
    ever gain code refs, SPEC-ONLY becomes wrong and this fails.
    """
    path = ROOT / "docs/traceability/nfr-matrix.md"
    if not path.exists():
        pytest.skip("nfr-matrix.md not present")
    text = path.read_text(encoding="utf-8", errors="replace")
    reserved = set(re.findall(r"`(NFR-CIV-\d{3})`[^|\n]*\|[^|\n]*\|[^|\n]*\|\s*planned", text))
    reserved |= set(re.findall(r"`(NFR-CIV-\d{3})`[^|\n]*\|\s*tbd", text))
    if not reserved:
        pytest.skip("no reserved rows found; the table format changed")

    inv, status = _inventory(), _matrix_status()
    for eid in sorted(reserved):
        if eid not in inv:
            continue
        e = inv[eid]
        assert status.get(eid) == "SPEC-ONLY", (
            f"{eid} is a reserved 'tbd' row but is reported {status.get(eid)}"
        )
        assert not (e.get("in_code") or []), f"{eid} is reserved but has code refs"
        assert not (e.get("in_tests") or []), f"{eid} is reserved but has test refs"


def _spec_refs(entry: dict) -> list[str]:
    """Every spec-side ref of an entry.

    The inventory stores some `in_*` values as a bare string rather than a list.
    Iterating such a value directly yields single characters, which is a bug that
    once inflated this audit's "unresolved citations" count to 3354. Normalise
    first, so this helper can never reintroduce it.
    """
    out: list[str] = []
    for key in ("in_specs", "in_traceability", "in_func_req"):
        v = entry.get(key)
        if isinstance(v, str):
            out.append(v)
        else:
            out.extend(v or [])
    return out


def test_every_spec_citation_resolves_to_a_real_line() -> None:
    """No spec citation may point at a missing file or an out-of-range line.

    This is the audit's largest claim: all 12648 spec-side references in the
    committed inventory resolve. A phantom `file:line` credit would be a defect
    of the same family as a phantom ID, and it is cheap to detect.

    A reference may legitimately omit the line number. 129 entries cite the root
    `FUNCTIONAL_REQUIREMENTS.md` with no line at all, so absence of `:` is a
    valid shape and only the file must exist. `docs/audits/_id_inventory_v3.json`
    stores those as bare strings rather than one-element lists, which is why
    `_spec_refs` normalises first.
    """
    inv = _inventory()
    bad: list[str] = []
    total = 0
    with_line = 0
    for eid, e in inv.items():
        for ref in _spec_refs(e):
            total += 1
            path, sep, tail = ref.rpartition(":")
            if not sep or not tail.isdigit():
                # file-level reference: the path itself must exist
                if (ROOT / ref).exists():
                    continue
                bad.append(f"{eid}: missing file {ref!r}")
                continue
            p = ROOT / path
            if not p.exists():
                bad.append(f"{eid}: missing file {path}")
                continue
            n = int(tail)
            nlines = len(p.read_text(encoding="utf-8", errors="replace").splitlines())
            if not (1 <= n <= nlines):
                bad.append(f"{eid}: {ref} out of range (file has {nlines} lines)")
            else:
                with_line += 1
    assert total > 10000, f"only {total} spec refs examined; inventory shape changed"
    assert with_line > 10000, (
        f"only {with_line} line-level refs examined; inventory shape changed"
    )
    assert not bad, (
        f"{len(bad)} spec citations do not resolve, first few: "
        + "; ".join(bad[:8])
    )


def test_no_row_rests_only_on_id_restatement() -> None:
    """No row's entire spec evidence is a line that merely repeats the ID.

    A cited line is treated as carrying no evidence of its own when, after
    deleting the ID and every other FR-/NFR- token plus markdown decoration and
    the generator's fixed labels, nothing is left. Title lines and `> Epic:`
    frontmatter qualify. A row whose every citation is such a line is crediting
    the ID with the ID.
    """
    labels = re.compile(
        r"(?i)\b(intent|adr|spec|plan|research|date|status|deciders|epic|"
        r"relates to|traceability id|implementing crate)\b\s*:?")
    cache: dict[str, list[str]] = {}

    def residue(line: str, eid: str) -> str:
        s = line.replace(eid, " ")
        s = re.sub(r"(?i)\b(fr|nfr)-[a-z0-9-]*\b", " ", s)
        s = re.sub(r"(?i)^#+\s*", " ", s)
        s = re.sub(r"(?i)^>\s*", " ", s)
        s = labels.sub(" ", s)
        return re.sub(r"[\s#>*|`\-:=\[\](){}.,]+", " ", s).strip()

    inv = _inventory()
    offenders = []
    for eid, e in inv.items():
        refs = _spec_refs(e)
        if not refs:
            continue
        states = []
        for ref in refs:
            path, sep, tail = ref.rpartition(":")
            if not sep or not tail.isdigit():
                states.append("unknown")
                continue
            if path not in cache:
                p = ROOT / path
                cache[path] = (p.read_text(encoding="utf-8", errors="replace").splitlines()
                               if p.exists() else [])
            lines = cache[path]
            n = int(tail)
            if not (1 <= n <= len(lines)):
                states.append("unknown")
                continue
            states.append("empty" if not residue(lines[n - 1], eid) else "content")
        if states and all(s == "empty" for s in states):
            offenders.append(eid)

    assert not offenders, (
        "these rows cite only lines that restate their own ID, so their spec "
        "evidence is circular: " + ", ".join(sorted(offenders)[:12])
    )


# --- rule 8: the coverage gate can pass on a stale inventory ---------------
#
# `gen-fr-audit.py` and `_build_matrix.py` both read
# `docs/audits/_id_inventory_v3.json`. Neither regenerates it. `_gather_ids.py`
# is a separate manual step, so `check-fr-coverage.py` happily rebuilt a matrix
# from an inventory that predated the source under audit.
#
# Measured: after re-tagging `speciation.rs` with FR-CIV-SPECIES-300..304, the
# gate ran green and reported all five still SPEC-ONLY, because the inventory on
# disk was 14 minutes older than the edit. The gate passed on stale evidence.
#
# Detecting this by comparing artifacts is impossible in both directions. The
# matrix is a pure function of the inventory, so an inventory-vs-matrix check
# agrees even when both are stale; and both orderings write the inventory first,
# so timestamps agree too. Only comparing the inventory against live source
# discriminates, which is what the tests below do.


def test_inventory_is_fresh_with_respect_to_source() -> None:
    """The inventory must be rebuilt whenever source changes.

    Comparing artifacts cannot work, and neither can comparing `path:line`
    offsets. Three approaches were tried and each was verified vacuous:

    * Inventory rows vs matrix rows — the matrix is a pure function of the
      inventory, so rebuilding the matrix from stale data makes them agree again.
    * mtime / `generated_at` — both the healthy and the stale orderings write the
      inventory before the matrix, and `generated_at` is only day-granular.
    * `path:line` still contains the ID — a stale inventory only fails this if the
      source edit happened to move or drop the cited line. Re-tagging
      `speciation.rs` kept every cited line intact, so the check passed anyway.

    What does discriminate is recomputing the inventory from source and comparing
    the result. That is the only check that cannot be satisfied by a stale file,
    because it re-derives the evidence instead of trusting it.
    """
    inventory = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
    gather = ROOT / "docs" / "audits" / "_gather_ids.py"
    if not inventory.exists() or not gather.exists():
        pytest.skip("audit tooling not present")
    import importlib.util
    import io
    import json
    from contextlib import redirect_stdout, redirect_stderr

    # Capture the bytes on disk BEFORE re-running the generator, because
    # `main()` overwrites the file it scans.
    before = json.loads(inventory.read_text(encoding="utf-8"))

    spec = importlib.util.spec_from_file_location("fr_gather", gather)
    module = importlib.util.module_from_spec(spec)
    sys.modules["fr_gather"] = module
    spec.loader.exec_module(module)

    buf = io.StringIO()
    with redirect_stdout(buf), redirect_stderr(buf):
        module.main()

    after = json.loads(module.OUT_JSON.read_text(encoding="utf-8"))

    def fingerprint(doc: dict) -> dict:
        # Ignore generated_at: it is day-granular and would mask a same-day
        # rebuild.
        return {r["id"]: {
            "in_code": r.get("in_code"),
            "in_tests": r.get("in_tests"),
            "in_stub_tests": r.get("in_stub_tests"),
            "in_specs": r.get("in_specs"),
        } for r in (doc.get("ids") or [])}

    old, new = fingerprint(before), fingerprint(after)
    assert new, "recomputed inventory is empty"

    only_old = sorted(set(old) - set(new))
    only_new = sorted(set(new) - set(old))
    drifted = sorted(k for k in set(old) & set(new) if old[k] != new[k])

    assert not (only_old or only_new or drifted), (
        f"{inventory.name} is stale with respect to source; re-run "
        f"docs/audits/_gather_ids.py (then gen-fr-audit.py).\n"
        f"  dropped IDs   : {len(only_old)} {only_old[:8]}\n"
        f"  added IDs     : {len(only_new)} {only_new[:8]}\n"
        f"  changed refs  : {len(drifted)} {drifted[:8]}"
    )


def test_coverage_gate_regenerates_the_inventory_before_the_matrix() -> None:
    """The gate must rebuild the inventory, not grade a possibly stale one.

    `gen-fr-audit.py` and `_build_matrix.py` both read the inventory and neither
    regenerates it, so the gate has to do it or it silently grades source the
    inventory predates.

    Load the module and check execution order directly. Grepping the source text
    is not enough: the first mention of each script is in the docstring, which
    says the right thing while the code could still do the opposite.
    """
    gate = ROOT / "scripts" / "traceability" / "check-fr-coverage.py"
    if not gate.exists():
        pytest.skip("gate script not present")
    import importlib.util

    spec = importlib.util.spec_from_file_location("fr_gate", gate)
    module = importlib.util.module_from_spec(spec)
    sys.modules["fr_gate"] = module
    spec.loader.exec_module(module)

    root = Path(module.ROOT)
    gather = root / "docs" / "audits" / "_gather_ids.py"
    assert gather.exists(), "inventory generator missing"

    # Instrument subprocess.run so we record real execution order without
    # actually running a 70-second rescan.
    calls: list[str] = []
    real_run = module.subprocess.run

    def fake_run(cmd, *a, **kw):
        # Record the bare filename; the caller passes a Path, so compare on
        # Path(...).name rather than on the raw argument.
        calls.append(Path(str(cmd[-1])).name)
        return real_run(cmd, *a, **kw)

    module.subprocess.run = fake_run
    try:
        module.regenerate_matrix()
    finally:
        module.subprocess.run = real_run

    joined = " | ".join(calls)
    assert gather.name in joined, f"gate never ran the inventory generator: {joined}"
    assert "gen-fr-audit.py" in joined, f"gate never ran the matrix builder: {joined}"
    assert calls.index(gather.name) < calls.index("gen-fr-audit.py"), (
        f"gate builds the matrix before the inventory: {joined}"
    )


def test_matrix_declares_its_source_inventory() -> None:
    """Provenance must be recorded, so staleness is detectable at all."""
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    data = json.loads(matrix.read_text(encoding="utf-8"))
    assert data.get("source_inventory") == "docs/audits/_id_inventory_v3.json"


# --- rule 9: the gate fails OPEN, and guards nothing on 1068 rows -----------
#
# Two more holes in the same gate, both reachable without any change to the
# classifier. Found by probing the real gate rather than reading it.
#
# 9a. Fail-open. `regenerate_matrix()` printed a warning when
# `_gather_ids.py` was absent and then carried on to build the matrix from the
# stale inventory anyway. Measured: exit code 0, warning emitted, gate green.
# That is the original SPECIES-300..304 defect with a rename away. A gate whose
# own diagnostic says the evidence is stale must not report success.
#
# 9b. Unguarded statuses. `REGRESSION_BUDGET` and `ABSOLUTE_CEILINGS` cover
# only the four gap statuses. `COVERED` (840 rows) and `SELF-TEST-ONLY` (228)
# appear in no budget and no ceiling, so 1068 of 1430 rows could fall out of the
# matrix and every count would still sit inside its budget. Losing coverage is
# the failure this gate exists to catch.


def _load_gate():
    gate = ROOT / "scripts" / "traceability" / "check-fr-coverage.py"
    if not gate.exists():
        pytest.skip("gate script not present")
    spec = importlib.util.spec_from_file_location("fr_gate_r9", gate)
    module = importlib.util.module_from_spec(spec)
    sys.modules["fr_gate_r9"] = module
    spec.loader.exec_module(module)
    return module


def test_gate_refuses_to_run_on_a_missing_inventory_generator() -> None:
    """A missing `_gather_ids.py` must be fatal, not a warning.

    Proved against the real gate: with the generator renamed aside it printed
    "matrix may be built on a stale inventory" and exited 0. The diagnostic was
    correct, so the response to it has to be an error.
    """
    module = _load_gate()
    root = Path(module.ROOT)
    gather = root / "docs" / "audits" / "_gather_ids.py"
    if not gather.exists():
        pytest.skip("inventory generator not present")

    import subprocess

    hidden = gather.with_suffix(".hidden-by-test")
    gather.rename(hidden)
    try:
        proc = subprocess.run(
            [sys.executable, str(root / "scripts" / "traceability" / "check-fr-coverage.py"),
             "--no-write"],
            cwd=root, capture_output=True, text=True,
        )
    finally:
        hidden.rename(gather)

    output = proc.stdout + proc.stderr
    assert proc.returncode != 0, (
        "gate exited 0 with _gather_ids.py missing; it graded a stale inventory "
        f"and reported success. Output:\n{output}"
    )


def test_only_one_ci_workflow_guards_the_fr_audit() -> None:
    """Pins the fact that `fr-coverage-gate.yml` is the ONLY CI guard on this data.

    Two other workflows look relevant and are not:

    - `audit-fr-coverage.yml` runs `Tools/audit-fr-coverage/audit.sh`, which
      reads only three hand-maintained markdown tables under
      `docs/traceability/` and greps them for `| dormant |` status cells. It
      never opens `fr-matrix.json` or `_id_inventory_v3.json`, so a change to
      the gatherer cannot affect its counts or its 200-row threshold. Running
      it is not evidence that the audit is sound.
    - `fr-coverage.yml` is `workflow_dispatch` only, and its coverage job runs
      `scripts/fr-coverage/run-fr-coverage.sh`, which likewise never reads the
      matrix. `scripts/traceability/check-traceability.sh` (pre-push) does not
      either; it checks that FR-CIV ids appear in TRACEABILITY_MATRIX.md.

    If a second workflow starts grading the audit artifacts, this test should
    be updated to name it. If one ever stops grading them, this test failing is
    the signal that the audit has lost its safety net.
    """
    wf = ROOT / ".github" / "workflows"
    if not wf.exists():
        pytest.skip("no workflows directory")

    guards = {
        p.name
        for p in wf.glob("*.yml")
        if "check-fr-coverage.py" in p.read_text(encoding="utf-8")
    }
    assert guards == {"fr-coverage-gate.yml"}, (
        f"expected fr-coverage-gate.yml to be the only workflow running the "
        f"gate; found {sorted(guards)}. A new guard is good news and should be "
        f"recorded here; a removed one means the audit lost its CI safety net."
    )


def test_provenance_header_withdraws_its_ids_file_scoped() -> None:
    """A header declaring an id undefined withdraws it for that whole file.

    The id-provenance corrections left three artifacts the earlier `[unbound]`
    and "Removed, with the reason" rules all missed: the header itself, the
    surviving mention inside it, and `/// Frame budget struct (FR-PERF-003).`
    left on a real declaration.

    Shape three is indistinguishable from a genuine tag by pattern alone --
    `/// Text (FR-X-NNN).` is the project's normal convention, and 1,929 such
    mentions exist in files with no header. So the rule keys on the file's own
    header, never on the doc line's shape.
    """
    # Mirrors crates/render/src/atlas.rs: the header-bearing line names the ids
    # it withdrew, and the same ids survive on real declarations further down.
    src = (
        "//! NOTE: this module previously carried `FR-PROV-001`, `FR-PROV-002`.\n"
        "//! No authoritative spec defines those ids.\n"
        "\n"
        "/// Pack sprites into one atlas per LOD level (FR-PROV-002).\n"
        "pub fn pack_atlas_per_lod() {}\n"
        "\n"
        "/// Deterministic replay satisfies FR-PROV-003 on every run.\n"
        "pub fn replay() {}\n"
    )
    lines = src.split("\n")
    withdrawn = gather.withdrawn_ids(lines)

    # The header's own ids are withdrawn. Ids named only on CONTINUATION lines of
    # the header are deliberately NOT: see the next test.
    assert withdrawn == {"FR-PROV-001", "FR-PROV-002"}, f"got {withdrawn}"
    assert "FR-PROV-003" not in withdrawn

    # Both leftover citations are withdrawn; the genuine one is not. This is
    # the assertion a line-shaped rule could not make.
    assert gather.cites_withdrawn(lines[3], withdrawn)
    assert gather.cites_withdrawn(lines[0], withdrawn)
    assert not gather.cites_withdrawn(lines[6], withdrawn)

    # A file with no header withdraws nothing, so the check is inert there.
    clean = ["/// Deterministic replay satisfies FR-PROV-003 on every run."]
    assert gather.withdrawn_ids(clean) == set()
    assert not gather.cites_withdrawn(clean[0], set())


def test_provenance_scan_is_line_scoped_not_block_scoped() -> None:
    """Regression: a block-scoped header scan withdraws ids the file vouches for.

    The obvious generalization -- scan the whole comment block containing the
    header -- is wrong, and measurably so. A provenance header routinely names
    BOTH what it withdrew and the one real requirement that survived:

        //! NOTE: this module previously carried an `FR-ASSET-004` tag. No
        //! authoritative spec defines that id ... The real
        //! CIV-0601 spec numbers its requirements `FR-CIV-3D-001..015`.

    Block-scanning that header would withdraw FR-CIV-3D-001. Measured over the
    render crate, it would additionally withdraw FR-CIV-AUDIO-004 and
    FR-CIV-ASSET-001, which `lib.rs:31-33` explicitly names as "the surviving
    tag" and "the one audio requirement this crate genuinely implements".

    So only ids on a header-bearing line itself are withdrawn. An id named on a
    continuation line is either part of the withdrawal prose or a genuine
    survivor, and the file gives no way to tell the two apart. That residual is
    recorded as an accepted limit rather than guessed at.
    """
    src = (
        "//! NOTE: this module previously carried an `FR-PROV-001` tag. No\n"
        "//! authoritative spec defines that id. The real spec numbers its\n"
        "//! requirements `FR-CIV-SURVIVOR-002..015`.\n"
    )
    assert gather.withdrawn_ids(src.split("\n")) == {"FR-PROV-001"}


def test_provenance_header_matches_only_its_own_literal_openings() -> None:
    """Guard the over-reach case: prose that merely discusses a withdrawal.

    Matching on 'does not exist' or 'no authoritative spec' would catch
    legitimate discussion in ordinary module docs, so the rule is anchored to
    the literal openings the correction tools emit.
    """
    discussing = [
        "//! The spec file CIV-0600 does not exist; use the asset pipeline spec.",
        "//! No authoritative spec defines the entity budget, so we derive one.",
    ]
    assert gather.withdrawn_ids(discussing) == set()

    assert gather.withdrawn_ids(
        ["//! Provenance: this module previously carried a `FR-X-001` tag."]
    ) == {"FR-X-001"}
    assert gather.withdrawn_ids(
        ["//! This crate previously advertised `FR-Y-002` implementations."]
    ) == {"FR-Y-002"}


def test_test_no_code_ref_rows_carry_no_code_refs() -> None:
    """The status name is the contract: this status means zero code references.

    `TEST-NO-CODE-REF` exists precisely to say a requirement is exercised by
    tests but no implementation can be pointed at. If a row in that status
    carries a `code_refs` entry, the gatherer found implementation the matrix
    builder did not, and the status is a lie in the optimistic direction: the
    row looks audited while actually being credited to code.

    This is the artifact-level half of the removal-block fix. The gatherer
    rule keeps provenance prose out of `in_code`, and this asserts the rebuilt
    matrix actually reflects that, rather than trusting the comment describing
    it.
    """
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    rows = json.loads(matrix.read_text(encoding="utf-8")).get("rows") or []
    violations = [
        {"id": r.get("id"), "code_refs": r.get("code_refs")}
        for r in rows
        if r.get("status") == "TEST-NO-CODE-REF" and r.get("code_refs")
    ]
    assert not violations, (
        f"{len(violations)} TEST-NO-CODE-REF row(s) carry code_refs, so they "
        f"are credited to implementation while labelled as having none: "
        f"{violations[:10]}"
    )


def test_every_regression_budget_is_strictly_below_its_live_count() -> None:
    """A budget at or above its count guards nothing.

    `REGRESSION_BUDGET` entries are checked as a delta against the snapshot,
    `if delta > budget`. So a budget wider than the status' entire population
    means the status could fall all the way to zero and the gate would still
    pass. The gate file states this invariant in prose; nothing enforced it,
    and `IMPL-NO-TEST` sat at budget 10 against a live count of 9.
    """
    module = _load_gate()
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    rows = json.loads(matrix.read_text(encoding="utf-8")).get("rows") or []
    counts: dict[str, int] = {}
    for r in rows:
        s = r.get("status", "UNKNOWN")
        counts[s] = counts.get(s, 0) + 1

    # Only statuses that actually hold rows are in scope. A budget on a status
    # with a live count of 0 is an anti-fabrication budget: its job is to fail
    # when rows APPEAR, so it is legitimately wider than zero.
    offenders = {
        s: {"budget": b, "live": counts.get(s, 0)}
        for s, b in module.REGRESSION_BUDGET.items()
        if counts.get(s, 0) > 0 and counts.get(s, 0) <= b
    }
    assert not offenders, (
        f"regression budget(s) not strictly below the live count: {offenders}"
    )


def test_every_status_in_the_matrix_is_guarded_by_the_gate() -> None:
    """No row may sit in a status the gate cannot detect its loss in.

    A status absent from both REGRESSION_BUDGET and ABSOLUTE_CEILINGS is
    invisible to the gate: its rows can be deleted, reclassified, or fabricated
    and every checked count stays inside budget.
    """
    module = _load_gate()
    matrix = ROOT / "docs" / "audits" / "fr-matrix.json"
    if not matrix.exists():
        pytest.skip("fr matrix not generated")
    import json

    data = json.loads(matrix.read_text(encoding="utf-8"))
    present = {r.get("status", "UNKNOWN") for r in data.get("rows") or []}
    assert present, "matrix has no rows"

    guarded = (
        set(module.REGRESSION_BUDGET)
        | set(module.ABSOLUTE_CEILINGS)
        | set(module.STATUS_FLOORS)
    )
    unguarded = sorted(present - guarded)
    assert not unguarded, (
        "these statuses appear in the matrix but in none of "
        f"REGRESSION_BUDGET, ABSOLUTE_CEILINGS, or STATUS_FLOORS: {unguarded}"
    )


def test_status_floors_fire_when_covered_rows_are_lost() -> None:
    """The floors must actually catch the loss they exist to catch.

    Checked behaviourally rather than by asserting on the constants. A static
    property such as "every budget is narrower than the count it guards" is
    wrong for the gap statuses: `IMPL-NO-TEST` falling is an improvement, so a
    budget wider than its count is harmless there. `COVERED` falling is the
    regression that matters, and that is what this exercises.

    Injects a synthetic matrix with rows removed and asserts the gate rejects it.
    """
    module = _load_gate()

    import io
    from contextlib import redirect_stdout

    # Baseline derived from the gate's own floor plus a margin, so rebasing the
    # floor (which happened on 2026-10-02 when 97 `[unbound]` false credits were
    # removed) does not require editing this test by hand, and the test cannot
    # silently drift away from the value it is supposed to be policing.
    floor = module.STATUS_FLOORS["COVERED"]
    baseline = {
        "COVERED": floor + 140,
        "SELF-TEST-ONLY": 228,
        "SPEC-ONLY": 197,
        "TEST-NO-CODE-REF": 156,
        "IMPL-NO-TEST": 9,
    }

    def rows_for(counts: dict[str, int]) -> list[dict]:
        return [
            {"id": f"FR-TEST-{i:04d}", "status": s}
            for s, n in counts.items()
            for i in range(n)
        ]

    def run_with(counts: dict[str, int]) -> tuple[int, str]:
        matrix = {"rows": rows_for(counts)}
        real = module.regenerate_matrix
        real_snapshot = module.load_snapshot
        module.regenerate_matrix = lambda: matrix
        module.load_snapshot = lambda: dict(baseline)
        buf = io.StringIO()
        try:
            with redirect_stdout(buf):
                import sys as _sys

                _old = _sys.argv
                _sys.argv = ["check-fr-coverage.py", "--no-write"]
                try:
                    rc = module.main()
                finally:
                    _sys.argv = _old
        finally:
            module.regenerate_matrix = real
            module.load_snapshot = real_snapshot
        return rc, buf.getvalue()

    rc_ok, out_ok = run_with(baseline)
    assert rc_ok == 0, f"baseline should pass, got rc={rc_ok}:\n{out_ok}"

    # floor + 140 -> floor - 1 is a 141-row loss, and it has to land strictly
    # BELOW the floor: the check is `actual < floor`, so a count sitting exactly
    # on the floor passes by design. Far beyond any plausible audit churn, and
    # entirely invisible before STATUS_FLOORS existed.
    loss = dict(baseline, COVERED=floor - 1)
    rc_loss, out_loss = run_with(loss)
    assert rc_loss != 0, (
        "gate accepted a 141-row drop in COVERED; rows can be lost undetected"
    )
    assert "COVERED" in out_loss, f"failure did not name COVERED:\n{out_loss}"

    # The same loss reported as a rise in a gap status must also fail, because a
    # row that vanishes from COVERED has to land somewhere.
    inflated = dict(baseline, COVERED=floor - 1)
    inflated["SPEC-ONLY"] += 141
    rc_inf, _ = run_with(inflated)
    assert rc_inf != 0, "gate accepted a 141-row shift into SPEC-ONLY"


def test_a_new_unguarded_status_fails_the_gate() -> None:
    """Introducing a status the gate does not know about must be fatal.

    Otherwise a new bucket can be invented and filled, and its rows become
    invisible to every check at once.
    """
    module = _load_gate()

    import io
    from contextlib import redirect_stdout

    counts = {
        "COVERED": 840,
        "SELF-TEST-ONLY": 228,
        "SPEC-ONLY": 197,
        "TEST-NO-CODE-REF": 156,
        "IMPL-NO-TEST": 9,
    }
    matrix = {"rows": [
        {"id": f"FR-TEST-{i:04d}", "status": s}
        for s, n in counts.items()
        for i in range(n)
    ]}
    matrix["rows"].extend(
        {"id": f"FR-TEST-NEW-{i:04d}", "status": "PLACEHOLDER-DISGUISE"}
        for i in range(50)
    )

    real = module.regenerate_matrix
    real_snapshot = module.load_snapshot
    module.regenerate_matrix = lambda: matrix
    module.load_snapshot = lambda: dict(counts)
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            import sys as _sys

            _old = _sys.argv
            _sys.argv = ["check-fr-coverage.py", "--no-write"]
            try:
                rc = module.main()
            finally:
                _sys.argv = _old
    finally:
        module.regenerate_matrix = real
        module.load_snapshot = real_snapshot

    out = buf.getvalue()
    assert rc != 0, f"gate accepted an unguarded status:\n{out}"
    assert "PLACEHOLDER-DISGUISE" in out, f"failure did not name the status:\n{out}"


# --- rule 10: index.md is a second, stale source of truth ------------------
#
# `docs/traceability/index.md` claims "Auto-generated 2026-09-16 for 1231 FRs"
# but no generator for it exists in the repo, and it has drifted from the matrix
# on 435 of the 1229 rows they share. It also uses a status vocabulary the matrix
# never emits: 631 rows claim `CODE-ONLY-no-spec`.
#
# Two sources of truth for coverage is the root problem, not the drift itself.
# Rather than hand-patch 435 rows (which would be cosmetic and would hide the
# next divergence), the file is required to stop asserting coverage statuses at
# all and to point at the matrix instead. This test enforces that.


def test_index_md_does_not_assert_its_own_coverage_statuses() -> None:
    """index.md must not carry a coverage status column that can drift.

    Measured drift before this rule: 163 agreements, 435 disagreements, 631 rows
    in a status the matrix cannot produce, out of 1229 comparable rows. The file
    is dated 2026-09-16 and has no generator, so nothing keeps it current.

    Spot-checks confirmed the matrix is the correct side of every disagreement
    sampled: `FR-AI-001` is defined at docs/FR.md:43, implemented at
    crates/ai/src/decision.rs:3, and tested at crates/ai/tests/fr_fr_ai_001.rs:1,
    yet index.md still called it SPEC-ONLY.
    """
    index = ROOT / "docs" / "traceability" / "index.md"
    if not index.exists():
        pytest.skip("traceability index not present")

    import re

    text = index.read_text(encoding="utf-8")

    # The stale vocabulary specifically. The matrix derives these from evidence;
    # index.md asserts them by hand.
    for stale in ("CODE-ONLY-no-spec",):
        offenders = [
            i for i, line in enumerate(text.splitlines(), 1)
            if stale in line and line.startswith("|")
        ]
        assert not offenders, (
            f"{index.name} still asserts {stale!r} on {len(offenders)} rows "
            f"(first at line {offenders[0] if offenders else '-'}). That status is "
            "hand-maintained, has no generator, and disagrees with the matrix."
        )

    # A header that claims auto-generation with no generator behind it.
    m = re.search(r"^>\s*Auto-generated\s+([0-9-]+)", text, re.M)
    assert m is None, (
        f"{index.name} claims 'Auto-generated {m.group(1) if m else ''}' but no "
        "generator for it exists in the repo. Either restore the generator or "
        "remove the claim."
    )

    # It must point readers at the authoritative artifact.
    assert "fr-matrix.json" in text or "fr-coverage-audit" in text, (
        f"{index.name} does not point at the authoritative coverage artifact "
        "(docs/audits/fr-matrix.json)"
    )


def test_index_md_generator_is_idempotent() -> None:
    """Running the generator twice must produce identical bytes.

    A generator that is not idempotent cannot be trusted to be the thing that
    maintains the file, which was the original problem: nothing regenerated it,
    so it drifted for two weeks. This checks the file on disk already matches what
    the generator produces.
    """
    gen = ROOT / "scripts" / "traceability" / "gen-traceability-index.py"
    index = ROOT / "docs" / "traceability" / "index.md"
    if not gen.exists() or not index.exists():
        pytest.skip("index generator not present")

    import subprocess

    before = index.read_bytes()
    proc = subprocess.run(
        [sys.executable, str(gen)], cwd=ROOT, capture_output=True, text=True,
    )
    assert proc.returncode == 0, f"generator failed:\n{proc.stdout}{proc.stderr}"

    after = index.read_bytes()
    assert before == after, (
        f"{index.name} is not what its generator produces; re-run "
        f"scripts/traceability/gen-traceability-index.py"
    )


def test_index_generator_mints_no_ids_that_look_doubled_prefix() -> None:
    """`fr-nfr-*` directories must yield `NFR-*`, never `FR-NFR-*`.

    The directory convention is `fr-` + lowercased ID, so an NFR requirement
    lands in `fr-nfr-civ-port-001/`. Reading the `fr-` as the ID kind produced
    `FR-NFR-CIV-PORT-001`, and regenerating the index added exactly 10 such
    phantom SPEC-ONLY rows that no spec anywhere backs.

    This asserts on the generator's own mapping rather than on the output file,
    because the phantoms only appear when the matrix is rebuilt from the index,
    which the output file alone cannot show.
    """
    gen = ROOT / "scripts" / "traceability" / "gen-traceability-index.py"
    if not gen.exists():
        pytest.skip("index generator not present")

    spec = importlib.util.spec_from_file_location("fr_gen_index", gen)
    module = importlib.util.module_from_spec(spec)
    sys.modules["fr_gen_index"] = module
    spec.loader.exec_module(module)

    cases = {
        "fr-civ-species-204": "FR-CIV-SPECIES-204",
        "fr-ai-001": "FR-AI-001",
        "fr-civ-0001-tick": "FR-CIV-0001-TICK",
        # The doubled-prefix directories, which must not become FR-NFR-*.
        "fr-nfr-civ-port-001": "NFR-CIV-PORT-001",
        "fr-nfr-s-01": "NFR-S-01",
        "fr-nfr-civ-dev-hygiene-001": "NFR-CIV-DEV-HYGIENE-001",
        # Already-correct NFR directories.
        "nfr-scale-02": "NFR-SCALE-02",
        "nfr-s-05": "NFR-S-05",
    }
    for slug, expected in cases.items():
        got = module.slug_to_id(slug)
        assert got == expected, (
            f"slug_to_id({slug!r}) returned {got!r}, expected {expected!r}"
        )
        assert "FR-NFR-" not in got and "NFR-NFR-" not in got, (
            f"slug_to_id({slug!r}) produced a doubled-prefix phantom: {got!r}"
        )


# --- rule: an [unbound] rationale is not an implementation ------------------
#
# `docs/audits/_apply_verdicts.py` replaces a wrong requirement tag with a
# `// [unbound] <id>: <reason>` comment recording why the tag was removed. The
# gatherer credited that comment as `in_code`, so requirements whose comments
# read "NOT IMPLEMENTED" reported COVERED. These tests fail on the pre-fix
# gatherer.


def test_unbound_marker_is_not_counted_as_code_evidence() -> None:
    """A `[unbound]` rationale must not make an ID look implemented."""
    assert gather.UNBOUND_TOKEN == "[unbound]"

    src = (
        "// [unbound] FR-CIV-ZZQA-001: NOT IMPLEMENTED. The requirement needs a\n"
        "// rendering pass that does not exist in this repository.\n"
        "pub struct Whatever;\n"
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs" / "audits").mkdir(parents=True)
        crate = root / "crates" / "zzqa"
        crate.mkdir(parents=True)
        (crate / "src").mkdir()
        (crate / "src" / "lib.rs").write_text(src, encoding="utf-8")
        old = os.environ.get("CIVIS_AUDIT_WORK")
        os.environ["CIVIS_AUDIT_WORK"] = str(root)
        try:
            gather.WORK = root.resolve()
            gather.OUT_JSON = root / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()
            gather.main()
            data = json.loads(gather.OUT_JSON.read_text(encoding="utf-8"))
        finally:
            if old is None:
                os.environ.pop("CIVIS_AUDIT_WORK", None)
            else:
                os.environ["CIVIS_AUDIT_WORK"] = old
            gather.WORK = ROOT
            gather.OUT_JSON = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()

    entry = next(e for e in data["ids"] if e["id"] == "FR-CIV-ZZQA-001")
    assert entry["in_code"] == [], (
        "the [unbound] rationale was credited as a code reference: "
        f"{entry['in_code']}"
    )
    assert entry["unbound_refs"], "the rationale should be recorded, not dropped"
    # The scratch crate carries no spec source, so the row is CODE-ONLY-no-spec
    # rather than SPEC-ONLY. Either is correct; the point of the assertion is
    # that it is no longer COVERED on a comment saying NOT IMPLEMENTED.
    assert audit.classify(entry) != "COVERED", (
        "a requirement documented as NOT IMPLEMENTED reported COVERED"
    )


def test_real_tag_next_to_an_unbound_note_still_counts() -> None:
    """The fix must not disable legitimate tags in the same file.

    `crates/engine/src/build/src/lib.rs` carries a stack of `[unbound]`
    rationales directly above `SCHEMA_VERSION`, and `crates/build` has other
    IDs tagged on real declarations. Excluding `[unbound]` lines must not
    disturb them.
    """
    src = (
        "// [unbound] FR-CIV-ZZQB-001: NOT IMPLEMENTED, no such symbol.\n"
        "pub const SCHEMA_VERSION: &str = \"0.1.0-stub\";\n"
        "\n"
        "/// Implements FR-CIV-ZZQB-002.\n"
        "pub fn real_thing() -> u32 { 7 }\n"
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs" / "audits").mkdir(parents=True)
        crate = root / "crates" / "zzqb"
        crate.mkdir(parents=True)
        (crate / "src").mkdir()
        (crate / "src" / "lib.rs").write_text(src, encoding="utf-8")
        old = os.environ.get("CIVIS_AUDIT_WORK")
        os.environ["CIVIS_AUDIT_WORK"] = str(root)
        try:
            gather.WORK = root.resolve()
            gather.OUT_JSON = root / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()
            gather.main()
            data = json.loads(gather.OUT_JSON.read_text(encoding="utf-8"))
        finally:
            if old is None:
                os.environ.pop("CIVIS_AUDIT_WORK", None)
            else:
                os.environ["CIVIS_AUDIT_WORK"] = old
            gather.WORK = ROOT
            gather.OUT_JSON = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()

    by_id = {e["id"]: e for e in data["ids"]}
    assert by_id["FR-CIV-ZZQB-001"]["in_code"] == []
    assert by_id["FR-CIV-ZZQB-002"]["in_code"] == ["crates/zzqb/src/lib.rs:4"], (
        "a real tag on a real declaration was dropped: "
        f"{by_id['FR-CIV-ZZQB-002']['in_code']}"
    )


def test_no_committed_row_is_covered_only_by_unbound_refs() -> None:
    """No row in the committed inventory may be covered by `[unbound]` alone.

    This is the repo-wide assertion for the defect: before the fix, 97 of the
    1430 matrix rows reported COVERED with every code reference sitting on a
    comment that says the requirement is NOT IMPLEMENTED.

    The population assertion below is load-bearing, not decoration. Asserting
    only "no offenders" passes vacuously when `unbound_refs` is absent
    entirely -- which is exactly what a reverted gatherer produces, because it
    never writes the key. The offender loop then skips every ID via
    `if not refs or not unbound: continue` and reports an empty list, so the
    test goes green against the very defect it exists to catch. Requiring a
    non-empty population first makes the emptiness conclusion meaningful.
    """
    inv = _inventory()
    populated = [eid for eid, e in inv.items() if e.get("unbound_refs")]
    assert populated, (
        "no ID carries an [unbound] reference, so the offender check below "
        "would pass vacuously: the invariant is untested, not satisfied"
    )
    assert len(populated) >= 100, (
        f"only {len(populated)} IDs carry [unbound] refs; the 2026-10-02 "
        f"measurement was 139, so the gatherer is probably not recording them"
    )
    offenders = []
    for eid, e in inv.items():
        refs = e.get("in_code") or []
        unbound = e.get("unbound_refs") or []
        if not refs or not unbound:
            continue
        if set(unbound).issuperset(set(refs)):
            offenders.append(eid)
    assert not offenders, (
        f"{len(offenders)} rows have only [unbound] code evidence, e.g. "
        f"{offenders[:8]}"
    )


def test_unbound_refs_never_appear_in_in_code() -> None:
    """`unbound_refs` and `in_code` must be disjoint sets for every ID.

    As in the sibling test, the disjointness result is meaningless unless
    `unbound_refs` is actually populated: a gatherer that never records the
    key trivially satisfies "no overlap" while crediting every `[unbound]`
    comment as code. The population check keeps the assertion honest.
    """
    inv = _inventory()
    populated = [eid for eid, e in inv.items() if e.get("unbound_refs")]
    assert len(populated) >= 100, (
        f"only {len(populated)} IDs carry [unbound] refs; the 2026-10-02 "
        f"measurement was 139, so disjointness would be vacuous"
    )
    both = []
    for eid, e in inv.items():
        overlap = set(e.get("in_code") or []) & set(e.get("unbound_refs") or [])
        if overlap:
            both.append((eid, sorted(overlap)))
    assert not both, f"{len(both)} IDs list a ref as both code and unbound: {both[:5]}"


# --- rule 13: removal-rationale blocks are not implementation evidence ------


def test_removal_block_lines_are_not_counted_as_code_evidence() -> None:
    """A per-id line inside a removal block must not credit `in_code`.

    `_apply_verdicts.py` and its two siblings write

        // Removed, with the reason each cannot be discharged here:
        // FR-SESSION-004: needs a NationAction queue; that type does not exist

    The second line names the requirement only to record that its tag was
    removed from the declaration below. Crediting it asserts exactly the
    coverage the comment denies.
    """
    assert gather.REMOVAL_MARKER == (
        "Removed, with the reason each cannot be discharged here"
    )
    src = (
        "// The following 1 requirement tag was removed from Helper.\n"
        "// It is not discharged by this symbol.\n"
        "//\n"
        "// Removed, with the reason each cannot be discharged here:\n"
        "// FR-CIV-ZZZB-001: needs a NationAction queue; no such type exists\n"
        "pub fn helper() -> u32 { 1 }\n"
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs" / "audits").mkdir(parents=True)
        crate = root / "crates" / "zzzb"
        (crate / "src").mkdir(parents=True)
        (crate / "src" / "lib.rs").write_text(src, encoding="utf-8")
        old = os.environ.get("CIVIS_AUDIT_WORK")
        os.environ["CIVIS_AUDIT_WORK"] = str(root)
        try:
            gather.WORK = root.resolve()
            gather.OUT_JSON = root / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()
            gather.main()
            data = json.loads(gather.OUT_JSON.read_text(encoding="utf-8"))
        finally:
            if old is None:
                os.environ.pop("CIVIS_AUDIT_WORK", None)
            else:
                os.environ["CIVIS_AUDIT_WORK"] = old
            gather.WORK = ROOT
            gather.OUT_JSON = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()

    entry = next(e for e in data["ids"] if e["id"] == "FR-CIV-ZZZB-001")
    assert entry["in_code"] == [], (
        "a removal rationale was credited as code evidence: "
        f"{entry['in_code']}"
    )
    assert entry["unbound_refs"], "the rationale should be recorded, not dropped"


def test_doc_comment_after_a_removal_block_still_counts() -> None:
    """A `///` doc comment ends a removal block and is real evidence again.

    This is the regression for the over-broad version of the rule. Treating
    every comment after the marker as in-block swallowed the legitimate tags on
    `Fixed` in `crates/engine/src/fixed_math.rs` and reported 223 affected IDs
    instead of the correct 77.
    """
    src = (
        "// Removed, with the reason each cannot be discharged here:\n"
        "// FR-CIV-ZZZC-001: needs a NationAction queue; no such type exists\n"
        "/// Implements FR-CIV-ZZZC-002.\n"
        "pub struct RealThing(pub u64);\n"
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs" / "audits").mkdir(parents=True)
        crate = root / "crates" / "zzzc"
        (crate / "src").mkdir(parents=True)
        (crate / "src" / "lib.rs").write_text(src, encoding="utf-8")
        old = os.environ.get("CIVIS_AUDIT_WORK")
        os.environ["CIVIS_AUDIT_WORK"] = str(root)
        try:
            gather.WORK = root.resolve()
            gather.OUT_JSON = root / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()
            gather.main()
            data = json.loads(gather.OUT_JSON.read_text(encoding="utf-8"))
        finally:
            if old is None:
                os.environ.pop("CIVIS_AUDIT_WORK", None)
            else:
                os.environ["CIVIS_AUDIT_WORK"] = old
            gather.WORK = ROOT
            gather.OUT_JSON = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
            gather._STUB_HEADER_CACHE.clear()

    by_id = {e["id"]: e for e in data["ids"]}
    assert by_id["FR-CIV-ZZZC-001"]["in_code"] == [], (
        f"removal block not excluded: {by_id['FR-CIV-ZZZC-001']['in_code']}"
    )
    assert by_id["FR-CIV-ZZZC-002"]["in_code"] == ["crates/zzzc/src/lib.rs:3"], (
        "a doc comment right after a removal block was wrongly excluded: "
        f"{by_id['FR-CIV-ZZZC-002']['in_code']}"
    )


def test_removal_block_ranges_stop_at_doc_comments() -> None:
    """The span helper must terminate on `///`, `//!`, blank, and code."""
    lines = [
        "// Removed, with the reason each cannot be discharged here:",   # 1
        "// FR-X-001: reason one",                                       # 2
        "// FR-X-002: reason two",                                       # 3
        "/// doc comment",                                               # 4
        "// FR-X-003: not in the block",                                # 5
        "",                                                              # 6
        "// FR-X-004: not in the block either",                          # 7
        "pub struct S;",                                                 # 8
    ]
    spans = list(gather.removal_block_ranges(lines))
    assert spans == [(1, 3)], spans
    assert gather.in_removal_block(spans, 3)
    for n in (4, 5, 6, 7, 8):
        assert not gather.in_removal_block(spans, n), f"line {n} wrongly in block"


def test_no_committed_code_ref_sits_inside_a_removal_block() -> None:
    """Repo-wide: no `in_code` reference may land inside a removal block."""
    inv = _inventory()
    files = set()
    for e in inv.values():
        for ref in e.get("in_code") or []:
            files.add(ref.rpartition(":")[0])

    offenders = []
    for rel in sorted(files):
        p = ROOT / rel
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except Exception:
            continue
        spans = list(gather.removal_block_ranges(lines))
        if not spans:
            continue
        for eid, e in inv.items():
            for ref in e.get("in_code") or []:
                r2, _, ln2 = ref.rpartition(":")
                if r2 == rel and gather.in_removal_block(spans, int(ln2)):
                    offenders.append((eid, ref))

    assert not offenders, (
        f"{len(offenders)} in_code refs sit inside a removal-rationale block, "
        f"e.g. {offenders[:5]}"
    )


# --- rule 14: a withdrawn id must not be credited as implementation ---------
#
# `withdrawn_ids` is line-scoped on purpose: a provenance header names both the
# ids it withdrew and the one real requirement that survived, so block-scanning
# the comment would withdraw the survivors too. `test_provenance_scan_is_line_
# scoped_not_block_scoped` pins that.
#
# The residual is the leak this rule pins. A withdrawal only takes effect if the
# id lands on the header-BEARING line. When a header wraps its id list across
# two lines, the ids on the continuation line are never harvested, so
# references to them are credited as ordinary `in_code` evidence and the audit
# reports COVERED for requirements that no authoritative spec defines.
#
# `crates/render/src/atlas.rs` had exactly that. Its NOTE said it previously
# carried `FR-ASSET-001`, `FR-ASSET-002`, and `FR-ASSET-003`; the list wrapped
# after the second, so only the first two were withdrawn. FR-ASSET-003 then
# collected two `in_code` references -- the header's own continuation line and
# `atlas_build_event`'s doc comment 250 lines later -- and reported COVERED for
# an id whose only definition was a TRACEABILITY_MATRIX row pointing at
# `docs/specs/CIV-0600-2d-assets.md`, a file that does not exist.
#
# The fix was to the source, not the gatherer: the ids the header already
# withdraws now sit on the header-bearing line, and the stale citation is gone.
# `withdrawn_ids` is unchanged.
#
# What this test asserts, and why not more. The tempting stronger invariant is
# "no file may mention a withdrawn id outside its header". That is false, and
# was measured against this tree: 12 such mentions exist and are all correct.
# Several are header prose naming the ids being withdrawn (`render/src/lib.rs:29`
# lists `FR-UX-001..005` to explain the collision), several are disclaimers
# (`render/src/frame.rs:1` says "no FR-PERF-003 id"), and the rest are doc
# comments on real functions citing an id that `withdrawn_ids` has already
# re-routed to `unbound_refs`. The gatherer handles every one of them correctly.
# Asserting that invariant would demand source changes nobody asked for and
# would have failed against a tree that is not broken.
#
# The invariant that IS true is narrower and is the false claim itself: a
# withdrawn id must never be credited as IMPLEMENTATION evidence. `in_code` is
# the bucket the classifier turns into COVERED.
#
# Known pre-existing failures, deliberately NOT fixed here. The sweep below is
# repo-wide and finds ids whose continuation-line leak or similar defect
# remains in code. Each entry is paired with a removal deadline: when the
# underlying source line is fixed, the entry is deleted from this allowlist
# and the same test fails loudly until the deletion lands, forcing the sweep
# to cover the id again. Listing them achieves visibility; removing the
# entries once the source is fixed prevents this constant from becoming
# permanent concealment of stale credits.
#
# Currently empty: every known continuation-line leak has been fixed in
# source and its credit routed to `unbound_refs`. If a future change
# introduces a new leak, the sweep will populate this set again.

_PREEXISTING_WRONG_IN_CODE: dict[str, str] = {}


def _scanned_code_files() -> list[str]:
    """Repo-relative paths of every file the gatherer scans as code.

    Mirrors the gatherer's own traversal (`SCAN_DIRS` + `SCAN_FILES`, minus
    skips, minus self-reference) so this walk sees exactly what the audit sees
    and cannot silently drift away from it.
    """
    out: list[str] = []
    for d in gather.SCAN_DIRS:
        base = ROOT / d
        if not base.exists():
            continue
        for p in sorted(base.rglob("*")):
            if not p.is_file() or gather.should_skip(p):
                continue
            rel = p.relative_to(ROOT).as_posix()
            if not gather.is_self_ref(rel) and gather.classify(rel) == "code":
                out.append(rel)
    for fname in gather.SCAN_FILES:
        p = ROOT / fname
        if not p.is_file() or gather.should_skip(p):
            continue
        rel = p.relative_to(ROOT).as_posix()
        if not gather.is_self_ref(rel) and gather.classify(rel) == "code":
            out.append(rel)
    return sorted(set(out))


def _withdrawn_in_source() -> set[str]:
    """Every id withdrawn by some file's own header, harvested live from source."""
    out: set[str] = set()
    for rel in _scanned_code_files():
        try:
            lines = (ROOT / rel).read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        out |= gather.withdrawn_ids(lines)
    return out


def test_no_withdrawn_id_is_credited_as_implementation_evidence() -> None:
    """No withdrawn id is credited in `in_code`, so none reports false COVERED.

    Scoped to `in_code` on purpose. Withdrawn ids legitimately keep `in_specs`
    and `in_tests` references -- the TRACEABILITY_MATRIX row records them, and
    test files repeat the provenance note -- and those references are how the
    matrix reports the row at all. `in_code` alone is the bucket the classifier
    turns into COVERED, so it is the only bucket where crediting a withdrawn id
    is a false claim rather than a record.

    Before the fix FR-ASSET-003 carried
    `in_code = ['crates/render/src/atlas.rs:20', 'crates/render/src/atlas.rs:271']`
    and reported COVERED. Reintroduce either reference and this fails by name.
    """
    withdrawn = _withdrawn_in_source()

    # Population guard: if the header detector stopped matching, the offender
    # check below would pass for the wrong reason.
    assert len(withdrawn) >= 5, (
        f"only {len(withdrawn)} withdrawn ids were found in source; the measured "
        f"count is 12, so this check would pass vacuously: {sorted(withdrawn)}"
    )

    inv = _inventory()
    offenders = []
    for eid in sorted(withdrawn & set(inv)):
        if eid in _PREEXISTING_WRONG_IN_CODE:
            continue
        refs = inv[eid].get("in_code") or []
        if isinstance(refs, str):
            refs = [refs]
        if refs:
            offenders.append(f"{eid} in_code={refs}")

    assert not offenders, (
        f"{len(offenders)} withdrawn id(s) are credited as implementation "
        "evidence and so report COVERED for requirements no authoritative spec "
        "defines:\n  " + "\n  ".join(offenders)
    )


def test_pre_existing_wrong_in_code_baseline_is_still_accurate() -> None:
    """Pin the three known failures, so fixing them is a visible change.

    They are pre-existing on `main` and out of scope for this fix. Listing them
    in `_PREEXISTING_WRONG_IN_CODE` is only honest while they are still wrong, so
    this test fails the moment someone fixes one, forcing the entry to be
    removed. The alternative -- leaving them silently -- is what lets a known
    false-COVERED row sit unnoticed.
    """
    inv = _inventory()
    resolved = []
    wrong = []
    for eid, ref in sorted(_PREEXISTING_WRONG_IN_CODE.items()):
        row = inv.get(eid) or {}
        refs = row.get("in_code") or []
        if isinstance(refs, str):
            refs = [refs]
        if refs and ref in refs:
            wrong.append(f"{eid} still credited at {ref}")
        else:
            resolved.append(eid)

    assert not resolved, (
        "these ids are in _PREEXISTING_WRONG_IN_CODE but are no longer credited "
        f"as implementation evidence: {resolved}. Remove them from the allowlist "
        "so the sweep covers them again."
    )
    assert len(wrong) == len(_PREEXISTING_WRONG_IN_CODE), (
        "the known-bad baseline does not match reality, so some id is credited "
        f"at an unexpected location: {wrong}"
    )


def test_fr_asset_003_is_not_credited_and_was_deleted_not_rebound() -> None:
    """The specific defect, pinned so it cannot come back by accident.

    No spec anywhere under `docs/specs/` numbers `FR-ASSET-003`. The real
    CIV-0600 file is `CIV-0600-2d-asset-pipeline-spec.md` and it numbers its
    requirements `FR-CIV-ASSET-001..020`, none of which covers emitting an atlas
    build event. `FR-CIV-ASSET-011` is the 102-sprite render batch under 30 s
    gate, and `FR-CIV-ASSET-001` is SVG template rendering; rebinding to either
    would repeat the defect with better paperwork, which
    `docs/audits/id-provenance-corrections.md` explicitly rules out ("IDs were
    removed rather than rebound wherever no authoritative requirement describes
    the behavior"). This asserts both halves: the id is gone from `in_code`, and
    it is not simply re-pointed at a substitute.
    """
    inv = _inventory()
    row = inv.get("FR-ASSET-003")
    assert row is not None, "FR-ASSET-003 vanished from the inventory entirely"

    code = row.get("in_code") or []
    if isinstance(code, str):
        code = [code]
    assert not code, (
        f"FR-ASSET-003 is credited as implementation evidence again: {code}. "
        "No authoritative spec defines this id."
    )

    assert _matrix_status().get("FR-ASSET-003") != "COVERED", (
        "FR-ASSET-003 reports COVERED, but no spec defines it and "
        "crates/render/src/atlas.rs:19 says so"
    )

    src = (ROOT / "crates/render/src/atlas.rs").read_text(encoding="utf-8")
    doc = [
        ln for ln in src.splitlines()
        if "Build the event payload for an atlas build attempt" in ln
    ]
    assert len(doc) == 1, f"expected one atlas_build_event doc line, got {doc}"
    assert not re.search(r"FR-[A-Z]", doc[0]), (
        f"atlas.rs carries a stale withdrawn citation: {doc[0]!r}"
    )
    assert "pub fn atlas_build_event" in src, (
        "atlas_build_event was removed; it is a real implementation whose doc "
        "comment must survive this fix"
    )
