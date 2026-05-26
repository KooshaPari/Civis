#!/usr/bin/env python3
"""framework_version constraint gate — Pattern #94 CI gate.

Pattern #94 ("Unbounded Range Theatre") is the failure mode where pack
manifests declare ``framework_version: ">=0.1.0"`` (no upper bound) while
the SDK is still pre-1.0. Each minor revision is potentially breaking, so
``CompatibilityChecker`` (#110) silently rubber-stamps every pack against
``<infinity>`` because there is nothing to compare against.

This gate walks ``packs/**/pack.yaml`` (and ``packs/**/manifest.yaml`` if
the file is a manifest), parses each ``framework_version`` constraint, and
rejects any range that:

  1. Lacks a ``<`` upper-bound clause (the headline failure mode).
  2. Has an upper bound ``>= 1.0.0`` while the current SDK major is 0
     (vacuous bound — every pre-1.0 release is still ">=0.x" so capping at
     ``<1.0.0`` is the same as no cap at all in practice).
  3. Has lower bound ``>=0.0.0`` without an explicit ``<`` upper.
  4. Is malformed (cannot be parsed as a ``>=X.Y.Z [<X.Y.Z]`` shape).

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/framework-version-allowlist.txt`` — pack relative paths
(POSIX, e.g. ``packs/test-bad-version/pack.yaml``) that are exempted.
Test fixture packs that intentionally violate the rule belong here.

CLI:
    python scripts/ci/check_framework_version.py [--root <packs/>]
                                                  [--allowlist <path>]
                                                  [--output <json>]
                                                  [--quiet|--verbose]
                                                  [--self-test]

Exit 0 = no violations; 1 = violations detected (CI fails);
2 = scan/usage error.

Modeled on ``scripts/ci/schema_drift_check.py`` (#245) and
``scripts/ci/tautological_test_check.py`` (#247). Pairs with the
schema-side regex in ``schemas/pack-manifest.schema.json`` (defensive
double-check: schema rejects on validation, this gate rejects in CI even
if the manifest somehow bypassed schema validation).

This is task #260.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path

try:
    import yaml  # type: ignore
except ImportError:
    yaml = None  # type: ignore


# ----------------------------------------------------------------------------
# Range parsing
# ----------------------------------------------------------------------------

# Match ``>=X.Y.Z`` with optional whitespace.
LOWER_BOUND_RE = re.compile(r">=\s*(?P<ver>\d+\.\d+\.\d+)")
# Match ``<X.Y.Z`` (NOT ``<=`` — open upper bound is the contract).
UPPER_BOUND_RE = re.compile(r"<\s*(?P<ver>\d+\.\d+\.\d+)")

# Bounded canonical shape: exactly ``>=A.B.C <X.Y.Z`` (one space minimum).
CANONICAL_BOUNDED_RE = re.compile(
    r"^>=\s*\d+\.\d+\.\d+\s+<\s*\d+\.\d+\.\d+$"
)

EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "_archived"}


def parse_version(s: str) -> tuple[int, int, int] | None:
    """Parse ``X.Y.Z`` into a 3-int tuple. Returns None on malformed."""
    parts = s.split(".")
    if len(parts) != 3:
        return None
    try:
        return (int(parts[0]), int(parts[1]), int(parts[2]))
    except ValueError:
        return None


def classify_range(value: str) -> dict:
    """Classify a framework_version range string. Returns a dict with:

      - ``raw``: the original string
      - ``has_lower``: bool
      - ``has_upper``: bool
      - ``lower``: parsed (M,m,p) or None
      - ``upper``: parsed (M,m,p) or None
      - ``canonical``: matches the schema regex
      - ``violations``: list of human-readable violation reasons (may be empty)
    """
    raw = value.strip()
    lo_m = LOWER_BOUND_RE.search(raw)
    up_m = UPPER_BOUND_RE.search(raw)

    has_lower = lo_m is not None
    has_upper = up_m is not None
    lower = parse_version(lo_m.group("ver")) if lo_m else None
    upper = parse_version(up_m.group("ver")) if up_m else None
    canonical = bool(CANONICAL_BOUNDED_RE.match(raw))

    violations: list[str] = []

    if not has_lower:
        violations.append("missing >= lower-bound clause")
    if not has_upper:
        violations.append("missing < upper-bound clause (unbounded range)")

    if has_lower and lower is None:
        violations.append("malformed lower-bound version")
    if has_upper and upper is None:
        violations.append("malformed upper-bound version")

    # Vacuous-bound rule: SDK is still pre-1.0, so capping at ``<1.0.0``
    # or higher is equivalent to no cap. Require a tighter ceiling.
    if has_upper and upper is not None and upper >= (1, 0, 0):
        # Lower bound is >=0.x, upper bound >=1.0.0 → vacuous in practice.
        # Only flag this if the lower bound is also pre-1.0.
        if lower is not None and lower < (1, 0, 0):
            violations.append(
                "vacuous upper bound: SDK is pre-1.0, "
                f"upper={'.'.join(str(x) for x in upper)} effectively unbounded"
            )

    # Vacuous lower-bound: ``>=0.0.0`` with no upper is doubly bad. We
    # already flag the "no upper" case; if there IS an upper we just note
    # the vacuous lower as a soft observation (still passes if upper is OK).
    if has_lower and lower == (0, 0, 0) and not has_upper:
        violations.append("vacuous lower bound (>=0.0.0) with no upper")

    if not canonical and has_lower and has_upper and not violations:
        # Has both bounds, but didn't match the canonical single-space form
        # (e.g. ``>=0.1.0  <0.25.0`` or other whitespace). Soft-warn — the
        # range is functionally OK but the schema regex will reject it.
        violations.append(
            "range has both bounds but does not match canonical "
            "schema regex (extra whitespace, ordering, etc.)"
        )

    return {
        "raw": raw,
        "has_lower": has_lower,
        "has_upper": has_upper,
        "lower": lower,
        "upper": upper,
        "canonical": canonical,
        "violations": violations,
    }


# ----------------------------------------------------------------------------
# IO helpers
# ----------------------------------------------------------------------------


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def is_excluded_path(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def load_allowlist(path: Path) -> set[str]:
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


def parse_yaml_safe(text: str) -> dict | None:
    """Parse YAML, returning the top-level dict or None on failure.

    Falls back to a minimal regex-based ``framework_version`` extractor
    when PyYAML is unavailable (so the gate is usable in stripped CI envs).
    """
    if not text:
        return None
    if yaml is not None:
        try:
            data = yaml.safe_load(text)
            if isinstance(data, dict):
                return data
            return None
        except yaml.YAMLError:
            return None

    # Fallback: pull out ``framework_version: <value>`` line directly.
    m = re.search(
        r'^\s*framework_version\s*:\s*["\']?([^"\'\n]+?)["\']?\s*$',
        text,
        re.MULTILINE,
    )
    if m:
        return {"framework_version": m.group(1)}
    return None


# ----------------------------------------------------------------------------
# Pack walker
# ----------------------------------------------------------------------------


def is_manifest_file(path: Path) -> bool:
    """A YAML file at packs/<pack-id>/pack.yaml or packs/<pack-id>/manifest.yaml
    is a manifest. Subdirectory YAMLs (assets/, units/, etc.) are not."""
    if path.name not in ("pack.yaml", "manifest.yaml"):
        return False
    return True


def enumerate_pack_manifests(packs_root: Path) -> list[Path]:
    """Return all pack-level manifest files under ``packs_root``."""
    out: list[Path] = []
    if not packs_root.exists():
        return out
    for p in sorted(packs_root.rglob("*.yaml")):
        if is_excluded_path(p):
            continue
        if not is_manifest_file(p):
            continue
        # Skip the pack assets/manifest.yaml at packs/<id>/assets/manifest.yaml
        # (it's an asset registry, not a pack manifest). Heuristic: parent
        # directory must be a pack root (contains pack.yaml as a sibling),
        # except for pack.yaml itself.
        if p.name == "manifest.yaml":
            # If sibling pack.yaml exists, this is the alt manifest at pack root
            # (e.g. warfare-starwars/manifest.yaml). If not, it's a sub-asset
            # manifest — skip.
            sibling_pack = p.parent / "pack.yaml"
            if not sibling_pack.exists():
                continue
        out.append(p)
    return out


# ----------------------------------------------------------------------------
# Scan + report
# ----------------------------------------------------------------------------


def scan_pack_manifests(
    packs_root: Path,
    allowlist: set[str],
    repo_root: Path | None = None,
) -> tuple[list[dict], list[dict]]:
    """Walk every pack manifest, classify framework_version, and return
    ``(violations, scanned)``. ``scanned`` is a parallel list of every
    manifest visited (one record per file)."""
    violations: list[dict] = []
    scanned: list[dict] = []
    for pf in enumerate_pack_manifests(packs_root):
        text = read_text_safe(pf)
        data = parse_yaml_safe(text)
        try:
            base = repo_root if repo_root is not None else packs_root.parent
            rel = pf.relative_to(base).as_posix()
        except ValueError:
            rel = pf.as_posix()

        record = {
            "file": rel,
            "framework_version": None,
            "missing": False,
            "violations": [],
        }

        if data is None or "framework_version" not in data:
            # Missing entirely. Not a violation per se for some pack types
            # (e.g. test fixtures with intentionally minimal manifests),
            # but we record it.
            record["missing"] = True
            scanned.append(record)
            continue

        fv = str(data.get("framework_version", "")).strip()
        record["framework_version"] = fv
        cls = classify_range(fv)
        record["violations"] = cls["violations"]
        scanned.append(record)

        if rel in allowlist:
            continue

        if cls["violations"]:
            violations.append(
                {
                    "file": rel,
                    "framework_version": fv,
                    "reasons": cls["violations"],
                }
            )

    violations.sort(key=lambda r: r["file"])
    scanned.sort(key=lambda r: r["file"])
    return violations, scanned


def build_report(
    violations: list[dict],
    scanned: list[dict],
    packs_root: Path,
    allowlist_path: Path,
    allowlist: set[str],
) -> dict:
    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "packs_root": packs_root.as_posix(),
        "allowlist_path": allowlist_path.as_posix(),
        "allowlist_size": len(allowlist),
        "manifests_scanned": len(scanned),
        "violations_count": len(violations),
        "violations": violations,
        "scanned": scanned,
        "exit_code": 1 if violations else 0,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("framework_version gate (Pattern #94)")
    print(f"  manifests scanned : {report['manifests_scanned']}")
    print(f"  violations        : {report['violations_count']}")
    print(f"  allowlist size    : {report['allowlist_size']}")
    if report["violations"]:
        print()
        print("VIOLATIONS:")
        for v in report["violations"]:
            print(f"  - {v['file']}")
            print(f"      framework_version: {v['framework_version']!r}")
            for r in v["reasons"]:
                print(f"      reason: {r}")
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Enforce bounded framework_version constraints on pack manifests "
            "(Pattern #94 — Unbounded Range Theatre)."
        )
    )
    p.add_argument(
        "--root",
        default="packs",
        help="Packs root directory (default: packs)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/framework-version-allowlist.txt",
        help=(
            "Allowlist file with one POSIX-relative manifest path per line "
            "(default: docs/qa/framework-version-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/framework-version-report.json",
        help="JSON report output path",
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> dict:
    repo = repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return {
        "repo": repo,
        "packs_root": _abs(args.root),
        "allowlist": _abs(args.allowlist),
        "output": _abs(args.output),
    }


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


def _self_test() -> int:  # noqa: C901
    import tempfile

    # Fixture 1: classify_range happy paths.
    ok = classify_range(">=0.5.0 <0.25.0")
    assert ok["violations"] == [], f"bounded should be clean; got {ok}"
    assert ok["has_lower"] and ok["has_upper"]
    assert ok["lower"] == (0, 5, 0) and ok["upper"] == (0, 25, 0)
    assert ok["canonical"] is True

    # Fixture 2: unbounded — the headline violation.
    bad1 = classify_range(">=0.1.0")
    assert any("upper-bound" in r for r in bad1["violations"]), bad1
    assert bad1["has_upper"] is False

    # Fixture 3: vacuous upper bound (>=0.x <1.0.0 — pre-1.0 SDK).
    bad2 = classify_range(">=0.1.0 <1.0.0")
    assert any("vacuous" in r for r in bad2["violations"]), bad2

    # Fixture 4: missing both bounds — empty / nonsense.
    bad3 = classify_range("")
    assert any("lower-bound" in r for r in bad3["violations"]), bad3
    assert any("upper-bound" in r for r in bad3["violations"]), bad3

    # Fixture 5: malformed.
    bad4 = classify_range(">=abc <def")
    # Regex won't match abc/def, so has_lower/has_upper are False → flagged
    # as missing both. That's the right answer.
    assert bad4["has_lower"] is False
    assert bad4["has_upper"] is False
    assert bad4["violations"], bad4

    # Fixture 6: high upper but lower also pre-1.0 — vacuous. Lower=0.5.0,
    # upper=2.0.0 → still vacuous because 2.0.0 caps the unreleased >=1.0
    # majors but lower is pre-1.0.
    bad5 = classify_range(">=0.5.0 <2.0.0")
    assert any("vacuous" in r for r in bad5["violations"]), bad5

    # Fixture 7: post-1.0 lower with post-1.0 upper — would be valid (NOT
    # vacuous). E.g. once SDK is at 1.x. We don't flag this.
    ok2 = classify_range(">=1.5.0 <2.0.0")
    # ``<2.0.0`` triggers the upper>=1.0.0 branch but lower is also
    # >=1.0.0, so the vacuous check is skipped. Should be clean.
    assert ok2["violations"] == [], f"post-1.0 range should be clean; got {ok2}"

    # Fixture 8: full pack walk with mixed manifests.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        packs_root = td_path / "packs"
        packs_root.mkdir()

        # Good pack.
        (packs_root / "good").mkdir()
        (packs_root / "good" / "pack.yaml").write_text(
            "id: good\nname: Good\nversion: 1.0.0\nauthor: T\ntype: content\n"
            'framework_version: ">=0.5.0 <0.25.0"\n',
            encoding="utf-8",
        )

        # Bad pack — unbounded.
        (packs_root / "bad-unbounded").mkdir()
        (packs_root / "bad-unbounded" / "pack.yaml").write_text(
            "id: bad-unbounded\nname: Bad\nversion: 1.0.0\nauthor: T\n"
            "type: content\n"
            'framework_version: ">=0.1.0"\n',
            encoding="utf-8",
        )

        # Bad pack — vacuous upper.
        (packs_root / "bad-vacuous").mkdir()
        (packs_root / "bad-vacuous" / "pack.yaml").write_text(
            "id: bad-vacuous\nname: Bad\nversion: 1.0.0\nauthor: T\n"
            "type: content\n"
            'framework_version: ">=0.1.0 <1.0.0"\n',
            encoding="utf-8",
        )

        # Pack with alternate manifest.yaml form.
        (packs_root / "alt-form").mkdir()
        (packs_root / "alt-form" / "pack.yaml").write_text(
            "id: alt-form\nname: Alt\nversion: 1.0.0\nauthor: T\n"
            "type: content\n"
            'framework_version: ">=0.5.0 <0.25.0"\n',
            encoding="utf-8",
        )
        (packs_root / "alt-form" / "manifest.yaml").write_text(
            "id: alt-form\nname: Alt\nversion: 1.0.0\nauthor: T\n"
            "type: content\n"
            'framework_version: ">=0.5.0"\n',  # bad — unbounded
            encoding="utf-8",
        )

        # Subdirectory YAML that should NOT be picked up (asset manifest).
        (packs_root / "good" / "assets").mkdir()
        (packs_root / "good" / "assets" / "manifest.yaml").write_text(
            "# asset manifest, not a pack manifest\n"
            "assets: []\n",
            encoding="utf-8",
        )

        manifests = enumerate_pack_manifests(packs_root)
        # Should pick up: good/pack.yaml, bad-unbounded/pack.yaml,
        # bad-vacuous/pack.yaml, alt-form/pack.yaml, alt-form/manifest.yaml
        # = 5 manifests. The asset manifest at good/assets/manifest.yaml
        # should be skipped (no sibling pack.yaml in that dir).
        assert len(manifests) == 5, f"expected 5 manifests, got {len(manifests)}: {manifests}"

        # Ensure the asset manifest was skipped.
        manifest_paths = {p.relative_to(packs_root).as_posix() for p in manifests}
        assert "good/assets/manifest.yaml" not in manifest_paths, manifest_paths

        violations, scanned = scan_pack_manifests(
            packs_root, set(), repo_root=td_path
        )
        # Should have 3 violations: bad-unbounded, bad-vacuous, alt-form/manifest.yaml.
        assert len(violations) == 3, f"expected 3 violations; got {violations}"
        v_files = {v["file"] for v in violations}
        assert any("bad-unbounded" in f for f in v_files), v_files
        assert any("bad-vacuous" in f for f in v_files), v_files
        assert any(
            f.endswith("alt-form/manifest.yaml") for f in v_files
        ), v_files

        # Allowlist suppresses one.
        allow_path = "packs/bad-unbounded/pack.yaml"
        violations_allow, _ = scan_pack_manifests(
            packs_root, {allow_path}, repo_root=td_path
        )
        assert len(violations_allow) == 2, (
            f"allowlist should drop one; got {violations_allow}"
        )
        assert not any(
            "bad-unbounded" in v["file"] for v in violations_allow
        ), violations_allow

        # Build a full report — exit_code should be 1 with violations.
        rpt = build_report(
            violations,
            scanned,
            packs_root,
            packs_root.parent / "allow.txt",
            set(),
        )
        assert rpt["exit_code"] == 1, rpt
        assert rpt["violations_count"] == 3, rpt
        assert rpt["manifests_scanned"] == 5, rpt

        # Clean run after we delete the bad packs.
        for bad in ("bad-unbounded", "bad-vacuous"):
            (packs_root / bad / "pack.yaml").unlink()
            (packs_root / bad).rmdir()
        (packs_root / "alt-form" / "manifest.yaml").unlink()

        clean_v, clean_s = scan_pack_manifests(packs_root, set(), repo_root=td_path)
        assert clean_v == [], f"after cleanup, no violations; got {clean_v}"
        assert len(clean_s) == 2, f"expected 2 scanned (good + alt-form); got {clean_s}"
        clean_rpt = build_report(
            clean_v,
            clean_s,
            packs_root,
            packs_root.parent / "allow.txt",
            set(),
        )
        assert clean_rpt["exit_code"] == 0, clean_rpt

    # Fixture 9: schema regex parity check. The schema regex is:
    #   ^>=\s*\d+\.\d+\.\d+\s+<\s*\d+\.\d+\.\d+$
    # CANONICAL_BOUNDED_RE in this script must match the same shape. The
    # exact regex source is duplicated above; we re-verify here.
    expected_schema_pattern = (
        r"^>=\s*\d+\.\d+\.\d+\s+<\s*\d+\.\d+\.\d+$"
    )
    schema_re = re.compile(expected_schema_pattern)
    assert schema_re.match(">=0.5.0 <0.25.0")
    assert not schema_re.match(">=0.5.0")
    assert not schema_re.match(">=0.5.0<0.25.0")  # missing space
    assert schema_re.match(">=0.1.0 <1.0.0")  # canonical-shape, but vacuous (caught by classifier)
    # Our local CANONICAL_BOUNDED_RE must agree.
    assert CANONICAL_BOUNDED_RE.match(">=0.5.0 <0.25.0")
    assert not CANONICAL_BOUNDED_RE.match(">=0.5.0")
    assert not CANONICAL_BOUNDED_RE.match("=0.5.0 <0.25.0")

    print("self-test: OK")
    return 0


# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    if len(argv) >= 1 and argv[0] == "--self-test":
        return _self_test()

    args = parse_args(argv)
    if args.quiet and args.verbose:
        print(
            "ERROR: --quiet and --verbose are mutually exclusive",
            file=sys.stderr,
        )
        return 2

    paths = resolve_paths(args)

    if not paths["packs_root"].exists():
        print(
            f"ERROR: packs root not found: {paths['packs_root']}",
            file=sys.stderr,
        )
        return 2

    allowlist = load_allowlist(paths["allowlist"])

    if args.verbose:
        print(
            f"scanning {paths['packs_root']} "
            f"(allowlist={len(allowlist)})",
            file=sys.stderr,
        )

    violations, scanned = scan_pack_manifests(
        paths["packs_root"], allowlist, repo_root=paths["repo"]
    )

    report = build_report(
        violations,
        scanned,
        paths["packs_root"],
        paths["allowlist"],
        allowlist,
    )

    paths["output"].parent.mkdir(parents=True, exist_ok=True)
    paths["output"].write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            f"framework-version: scanned={report['manifests_scanned']} "
            f"violations={report['violations_count']} -> {paths['output']}"
        )
    else:
        print_summary(report, paths["output"])

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
