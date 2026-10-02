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
