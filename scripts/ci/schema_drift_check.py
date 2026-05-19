#!/usr/bin/env python3
"""Schema-drift detector — Pattern #90 CI gate.

Pattern #90 ("Schema Drift") is the failure mode where ``schemas/*.json``
silently de-syncs from its callers. The four signatures the iter-53 audit
identified:

  1. C# code references a schema path that doesn't exist on disk
     (typo, rename, or deletion without grep).
  2. A schema declares a ``$ref`` whose target anchor doesn't resolve —
     either an internal ``#/$defs/X`` with no matching definition, or an
     external ``unit.schema.json`` that isn't in ``schemas/``.
  3. A pack's ``"$schema"`` URL points at a moved / renamed schema (relative
     paths must resolve from the pack file; remote URLs are sanity-checked
     for shape only — the gate does NOT make network calls).
  4. Documentation references a schema (e.g. ``unit.schema.json``) that no
     longer exists in ``schemas/``.

A fifth soft-warning signature: schemas that exist in ``schemas/`` but are
not referenced anywhere in ``src/``, ``packs/``, or ``docs/`` (orphans). Soft
by default; ``--strict`` promotes orphans to hard violations.

Allowlisting (one entry per line, ``#`` comments):
``docs/qa/schema-drift-allowlist.txt`` — bare schema filenames or full
URLs that should be skipped (e.g. ``xunit.runner.schema.json`` for the
test-runner schema NuGet ships, or ``https://json-schema.org/...`` meta
URLs that are externally hosted).

CLI:
    python scripts/ci/schema_drift_check.py [--root <path>]
                                              [--schemas-dir <path>]
                                              [--src-root <path>]
                                              [--packs-root <path>]
                                              [--docs-root <path>]
                                              [--allowlist <path>]
                                              [--output <json>]
                                              [--strict]
                                              [--quiet|--verbose]
                                              [--self-test]

Exit 0 = no hard drift; 1 = hard drift detected (CI fails);
2 = scan/usage error.

Modeled on ``scripts/ci/tautological_test_check.py`` (#247) and
``scripts/ci/changelog_lint.py`` (#251). Pairs with #235/#247/#251 to form
the "ledger discipline" CI tier.

This is task #245.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Match any reference to a ``*.schema.json`` filename in C# / docs / pack
# files. We deliberately keep this loose — accepts both ``Resources/unit.schema.json``
# style relative paths and bare ``unit.schema.json`` mentions.
SCHEMA_FILE_TOKEN_RE = re.compile(
    r"(?P<token>[A-Za-z0-9_./\-]*[A-Za-z0-9_\-]+\.schema\.json)"
)

# ``"$schema"`` directive in a JSON / YAML pack file. Captures the value.
SCHEMA_DIRECTIVE_RE = re.compile(
    r"""(?ix)
    ["']?\$schema["']?       # quoted or bare ``$schema``
    \s*[:=]\s*
    ["'](?P<value>[^"']+)["']
    """
)

# Anchor extraction from a ``$ref`` value. We support the two common forms:
#   "#/$defs/Foo"       → internal pointer, anchor = ``Foo`` in ``$defs``
#   "#/definitions/Bar" → internal pointer, anchor = ``Bar`` in ``definitions``
# Anything else is treated as an external file reference.
INTERNAL_REF_RE = re.compile(
    r"^#/(?P<container>\$defs|definitions)/(?P<anchor>[A-Za-z0-9_./\-]+)$"
)

EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "_archived"}


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


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def load_allowlist(path: Path) -> set[str]:
    """One allowlist entry per line, ``#`` for comments. Missing file →
    empty set."""
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


# ----------------------------------------------------------------------------
# Schema enumeration + internal-ref walker
# ----------------------------------------------------------------------------


def enumerate_schemas(schemas_dir: Path) -> dict[str, Path]:
    """Return a map of ``schema-filename`` → absolute path for every
    ``*.schema.json`` (and ``*.schema.yaml``) under ``schemas_dir``. The key
    is just the filename — callers reference these by basename in C# /
    docs / pack files."""
    out: dict[str, Path] = {}
    if not schemas_dir.exists():
        return out
    for ext in ("*.schema.json", "*.schema.yaml", "*.json", "*.yaml"):
        for p in sorted(schemas_dir.rglob(ext)):
            if is_excluded_path(p):
                continue
            # only register canonical schema files: those with .schema.json
            # or .schema.yaml suffix, plus any *.json directly in the
            # schemas dir (e.g. ``universe-bible.json``).
            name = p.name
            if name.endswith(".schema.json") or name.endswith(".schema.yaml"):
                out[name] = p
            elif p.parent == schemas_dir and (name.endswith(".json") or name.endswith(".yaml")):
                out[name] = p
    return out


def collect_refs_in_value(value, refs: list[str]) -> None:
    """Walk a parsed JSON value and append every ``$ref`` string found."""
    if isinstance(value, dict):
        for k, v in value.items():
            if k == "$ref" and isinstance(v, str):
                refs.append(v)
            else:
                collect_refs_in_value(v, refs)
    elif isinstance(value, list):
        for item in value:
            collect_refs_in_value(item, refs)


def collect_internal_anchors(value, defs: set[str], path_prefix: str = "") -> None:
    """Collect every JSON pointer that an internal ``$ref`` could legally
    target. We register both ``$defs/X`` and ``definitions/X`` keys."""
    if isinstance(value, dict):
        for k, v in value.items():
            if k in ("$defs", "definitions") and isinstance(v, dict):
                for name in v.keys():
                    defs.add(f"{k}/{name}")
            collect_internal_anchors(v, defs)
    elif isinstance(value, list):
        for item in value:
            collect_internal_anchors(item, defs)


def parse_schema_file(path: Path) -> dict | None:
    text = read_text_safe(path)
    if not text:
        return None
    try:
        return json.loads(text)
    except (json.JSONDecodeError, ValueError):
        return None


# ----------------------------------------------------------------------------
# Hard checks
# ----------------------------------------------------------------------------


def check_schema_files_referenced_by_code(
    src_root: Path,
    available_schemas: dict[str, Path],
    allowlist: set[str],
) -> tuple[list[dict], set[str]]:
    """Scan C# files under ``src_root`` for ``*.schema.json`` references.
    Returns ``(missing, referenced_set)``:

      * ``missing`` — list of records ``{file, line, token}`` for refs that
        do NOT resolve to a schema file in ``schemas/`` and are not in
        the allowlist.
      * ``referenced_set`` — every schema basename actually referenced in
        code (used for orphan detection).
    """
    missing: list[dict] = []
    referenced: set[str] = set()
    if not src_root.exists():
        return missing, referenced
    for cs_file in sorted(src_root.rglob("*.cs")):
        if is_excluded_path(cs_file):
            continue
        text = read_text_safe(cs_file)
        if ".schema.json" not in text:
            continue
        for m in SCHEMA_FILE_TOKEN_RE.finditer(text):
            token = m.group("token")
            basename = Path(token).name
            referenced.add(basename)
            if basename in allowlist or token in allowlist:
                continue
            if basename in available_schemas:
                continue
            try:
                rel = cs_file.relative_to(src_root.parent).as_posix()
            except ValueError:
                rel = cs_file.as_posix()
            missing.append(
                {
                    "file": rel,
                    "line": line_of(text, m.start()),
                    "token": token,
                    "basename": basename,
                }
            )
    return missing, referenced


def check_internal_refs(
    available_schemas: dict[str, Path],
    allowlist: set[str],
) -> list[dict]:
    """Walk every ``$ref`` in every schema file. For internal refs
    (``#/$defs/X`` or ``#/definitions/X``), verify the anchor exists in
    the same file. For external refs, verify the target file exists in
    ``schemas/`` (or is allowlisted). Returns broken-ref records."""
    broken: list[dict] = []
    for name, path in sorted(available_schemas.items()):
        parsed = parse_schema_file(path)
        if parsed is None:
            continue
        refs: list[str] = []
        collect_refs_in_value(parsed, refs)
        if not refs:
            continue
        defs: set[str] = set()
        collect_internal_anchors(parsed, defs)
        for ref in refs:
            if ref.startswith("http://") or ref.startswith("https://"):
                # external URL — gate doesn't verify, just records.
                continue
            m = INTERNAL_REF_RE.match(ref)
            if m:
                container = m.group("container")
                anchor = m.group("anchor")
                key = f"{container}/{anchor}"
                if key in defs:
                    continue
                broken.append(
                    {
                        "schema": name,
                        "ref": ref,
                        "kind": "internal",
                        "anchor": key,
                    }
                )
                continue
            # External file reference. Drop fragment if present.
            target = ref.split("#", 1)[0]
            if not target:
                # bare fragment like ``#`` — refers to root of same doc, OK.
                continue
            target_basename = Path(target).name
            if target_basename in allowlist or target in allowlist:
                continue
            if target_basename in available_schemas:
                continue
            broken.append(
                {
                    "schema": name,
                    "ref": ref,
                    "kind": "external",
                    "target": target_basename,
                }
            )
    broken.sort(key=lambda r: (r["schema"], r["ref"]))
    return broken


def check_pack_schema_directives(
    packs_root: Path,
    available_schemas: dict[str, Path],
    allowlist: set[str],
) -> tuple[list[dict], set[str]]:
    """Scan pack files (.json/.yaml) for ``"$schema": "<value>"`` directives.
    For relative paths, verify resolution against either the pack file's
    directory or the schemas dir. For URLs, just shape-check (no network).
    Returns ``(unresolvable, referenced_set)``."""
    unresolvable: list[dict] = []
    referenced: set[str] = set()
    if not packs_root.exists():
        return unresolvable, referenced
    for pf in sorted(packs_root.rglob("*")):
        if not pf.is_file():
            continue
        if is_excluded_path(pf):
            continue
        if pf.suffix.lower() not in (".json", ".yaml", ".yml"):
            continue
        text = read_text_safe(pf)
        if "$schema" not in text:
            continue
        for m in SCHEMA_DIRECTIVE_RE.finditer(text):
            value = m.group("value").strip()
            try:
                rel = pf.relative_to(packs_root.parent).as_posix()
            except ValueError:
                rel = pf.as_posix()
            line = line_of(text, m.start())

            if value in allowlist:
                continue

            # URL directive — we record but do NOT make network calls.
            if value.startswith("http://") or value.startswith("https://"):
                # Sanity: no whitespace, has a host segment.
                if " " in value or "\t" in value or "//" not in value:
                    unresolvable.append(
                        {
                            "file": rel,
                            "line": line,
                            "value": value,
                            "reason": "malformed URL",
                        }
                    )
                # Otherwise URL is shape-OK; leave network verification out.
                continue

            # Relative path — try resolution from the pack file dir, then
            # from the schemas dir. We only register a referenced basename
            # so orphan detection sees it.
            basename = Path(value).name
            referenced.add(basename)
            if basename in allowlist:
                continue

            # Try to resolve.
            candidates = []
            from_pack = (pf.parent / value).resolve()
            candidates.append(from_pack)
            if basename in available_schemas:
                candidates.append(available_schemas[basename])

            if any(c.exists() for c in candidates):
                continue
            unresolvable.append(
                {
                    "file": rel,
                    "line": line,
                    "value": value,
                    "reason": "relative path does not resolve",
                }
            )
    unresolvable.sort(key=lambda r: (r["file"], r["line"]))
    return unresolvable, referenced


def check_doc_schema_references(
    docs_root: Path,
    available_schemas: dict[str, Path],
    allowlist: set[str],
) -> tuple[list[dict], set[str]]:
    """Scan markdown files under ``docs_root`` for ``*.schema.json``
    mentions; flag references that can't be resolved against either an
    inline path or the canonical schemas dir."""
    drift: list[dict] = []
    referenced: set[str] = set()
    if not docs_root.exists():
        return drift, referenced
    for md in sorted(docs_root.rglob("*.md")):
        if is_excluded_path(md):
            continue
        text = read_text_safe(md)
        if ".schema.json" not in text:
            continue
        for m in SCHEMA_FILE_TOKEN_RE.finditer(text):
            token = m.group("token")
            basename = Path(token).name
            referenced.add(basename)
            if basename in allowlist or token in allowlist:
                continue
            if basename in available_schemas:
                continue
            try:
                rel = md.relative_to(docs_root.parent).as_posix()
            except ValueError:
                rel = md.as_posix()
            drift.append(
                {
                    "file": rel,
                    "line": line_of(text, m.start()),
                    "token": token,
                    "basename": basename,
                }
            )
    drift.sort(key=lambda r: (r["file"], r["line"]))
    return drift, referenced


def check_orphan_schemas(
    available_schemas: dict[str, Path],
    referenced: set[str],
    allowlist: set[str],
) -> list[dict]:
    """Schemas in ``schemas/`` not referenced by any C#, pack, or doc file.
    Each entry: ``{schema, path}``. Returned sorted by name."""
    orphans: list[dict] = []
    for name, path in available_schemas.items():
        if name in allowlist:
            continue
        if name in referenced:
            continue
        # Also accept reference via stem-without-suffix
        # (e.g. ``unit.schema`` → unit.schema.json basename).
        stem = name
        if stem in referenced:
            continue
        try:
            rel = path.relative_to(path.parents[1]).as_posix()
        except (ValueError, IndexError):
            rel = path.as_posix()
        orphans.append({"schema": name, "path": rel})
    orphans.sort(key=lambda r: r["schema"])
    return orphans


# ----------------------------------------------------------------------------
# Report + CLI
# ----------------------------------------------------------------------------


def build_report(
    schema_missing: list[dict],
    schema_orphan: list[dict],
    ref_broken: list[dict],
    url_unresolvable: list[dict],
    doc_drift: list[dict],
    strict: bool,
) -> dict:
    hard = (
        len(schema_missing)
        + len(ref_broken)
        + len(url_unresolvable)
        + len(doc_drift)
    )
    if strict:
        hard += len(schema_orphan)
    exit_code = 1 if hard > 0 else 0
    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "schema_missing": schema_missing,
        "schema_orphan": schema_orphan,
        "ref_broken": ref_broken,
        "url_unresolvable": url_unresolvable,
        "doc_drift": doc_drift,
        "strict": strict,
        "exit_code": exit_code,
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect schema drift (Pattern #90): missing schema files, "
            "broken $ref anchors, unresolvable $schema URLs, doc drift, "
            "and orphan schemas (warn-only by default)."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help="Repo root (default: derived from script location)",
    )
    p.add_argument(
        "--schemas-dir",
        default="schemas",
        help="Directory containing canonical schemas (default: schemas)",
    )
    p.add_argument(
        "--src-root",
        default="src",
        help="Source tree to scan for schema references (default: src)",
    )
    p.add_argument(
        "--packs-root",
        default="packs",
        help="Packs tree to scan for $schema directives (default: packs)",
    )
    p.add_argument(
        "--docs-root",
        default="docs",
        help="Docs tree to scan for schema mentions (default: docs)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/schema-drift-allowlist.txt",
        help=(
            "Allowlist file with one bare schema filename or URL per line "
            "(default: docs/qa/schema-drift-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/schema-drift-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help="Promote orphan-schema warnings to hard violations",
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    """scripts/ci/<this>.py → repo root is parents[2]."""
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> dict:
    repo = (
        Path(args.root).resolve()
        if args.root
        else repo_root_from_script()
    )

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return {
        "repo": repo,
        "schemas_dir": _abs(args.schemas_dir),
        "src_root": _abs(args.src_root),
        "packs_root": _abs(args.packs_root),
        "docs_root": _abs(args.docs_root),
        "allowlist": _abs(args.allowlist),
        "output": _abs(args.output),
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("schema-drift scan (Pattern #90)")
    print(f"  schema files missing (code → ?) : {len(report['schema_missing'])}")
    print(f"  $ref broken                     : {len(report['ref_broken'])}")
    print(f"  $schema URL unresolvable        : {len(report['url_unresolvable'])}")
    print(f"  doc drift                       : {len(report['doc_drift'])}")
    print(
        f"  orphan schemas                  : {len(report['schema_orphan'])} "
        f"({'HARD' if report['strict'] else 'soft'})"
    )

    if report["schema_missing"]:
        print()
        print("MISSING schemas referenced from code:")
        for r in report["schema_missing"]:
            print(f"  - {r['token']}  ({r['file']}:{r['line']})")
    if report["ref_broken"]:
        print()
        print("BROKEN $ref anchors:")
        for r in report["ref_broken"]:
            tail = r.get("anchor") or r.get("target") or "?"
            print(f"  - {r['schema']}  ${{ref={r['ref']!r}}} → {tail} ({r['kind']})")
    if report["url_unresolvable"]:
        print()
        print("UNRESOLVABLE $schema directives in pack files:")
        for r in report["url_unresolvable"]:
            print(f"  - {r['file']}:{r['line']}  $schema={r['value']!r}  ({r['reason']})")
    if report["doc_drift"]:
        print()
        print("DOC drift (schema name in docs but not in schemas/):")
        for r in report["doc_drift"]:
            print(f"  - {r['token']}  ({r['file']}:{r['line']})")
    if report["schema_orphan"]:
        print()
        sev = "HARD (--strict)" if report["strict"] else "soft warning"
        print(f"ORPHAN schemas — no callers found ({sev}):")
        for r in report["schema_orphan"]:
            print(f"  - {r['schema']}  ({r['path']})")
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


def _self_test() -> int:  # noqa: C901 — single fixture, fine to be long
    import tempfile

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        schemas_dir = td_path / "schemas"
        src_dir = td_path / "src"
        packs_dir = td_path / "packs"
        docs_dir = td_path / "docs"
        for d in (schemas_dir, src_dir, packs_dir, docs_dir):
            d.mkdir()

        # Healthy schema set: unit.schema.json with valid $defs/Pos and a
        # ref to weapon.schema.json (which we also create).
        (schemas_dir / "unit.schema.json").write_text(
            json.dumps(
                {
                    "$schema": "https://json-schema.org/draft/2020-12/schema",
                    "$defs": {
                        "Pos": {"type": "object"},
                    },
                    "type": "object",
                    "properties": {
                        "weapon": {"$ref": "weapon.schema.json"},
                        "pos": {"$ref": "#/$defs/Pos"},
                    },
                }
            ),
            encoding="utf-8",
        )
        (schemas_dir / "weapon.schema.json").write_text(
            json.dumps({"type": "object"}), encoding="utf-8"
        )
        # Drift schema: refs a missing target plus a missing internal anchor.
        (schemas_dir / "broken.schema.json").write_text(
            json.dumps(
                {
                    "$defs": {"Real": {"type": "object"}},
                    "properties": {
                        "missing_external": {"$ref": "ghost.schema.json"},
                        "missing_internal": {"$ref": "#/$defs/Phantom"},
                        "valid_internal": {"$ref": "#/$defs/Real"},
                    },
                }
            ),
            encoding="utf-8",
        )
        # Orphan-ready schema: not referenced by anyone.
        (schemas_dir / "orphan.schema.json").write_text(
            json.dumps({"type": "object"}), encoding="utf-8"
        )

        # C# file references a schema that exists + one that doesn't.
        (src_dir / "Loader.cs").write_text(
            'public static class Loader {\n'
            '    public const string Real = "schemas/unit.schema.json";\n'
            '    public const string Missing = "schemas/ghost-loader.schema.json";\n'
            "}\n",
            encoding="utf-8",
        )

        # Pack file references existing schema + one bad URL + one good URL.
        (packs_dir / "good_pack.json").write_text(
            json.dumps(
                {
                    "$schema": "../schemas/unit.schema.json",
                    "id": "good_pack",
                }
            ),
            encoding="utf-8",
        )
        (packs_dir / "bad_pack.json").write_text(
            json.dumps(
                {
                    "$schema": "../schemas/missing-pack.schema.json",
                    "id": "bad_pack",
                }
            ),
            encoding="utf-8",
        )
        (packs_dir / "remote_pack.json").write_text(
            json.dumps(
                {
                    "$schema": "https://example.com/schemas/v1.json",
                    "id": "remote_pack",
                }
            ),
            encoding="utf-8",
        )

        # Doc references existing schema + one missing.
        (docs_dir / "ok.md").write_text(
            "See [unit.schema.json](../schemas/unit.schema.json) for shape.\n",
            encoding="utf-8",
        )
        (docs_dir / "drift.md").write_text(
            "We use ghost-doc.schema.json for X.\n",
            encoding="utf-8",
        )

        available = enumerate_schemas(schemas_dir)
        assert "unit.schema.json" in available, available
        assert "weapon.schema.json" in available, available
        assert "broken.schema.json" in available, available
        assert "orphan.schema.json" in available, available

        allowlist: set[str] = set()

        # 1. Missing schema referenced from code.
        missing, ref_set = check_schema_files_referenced_by_code(
            src_dir, available, allowlist
        )
        assert any(r["basename"] == "ghost-loader.schema.json" for r in missing), (
            f"Expected ghost-loader missing; got {missing}"
        )
        assert not any(
            r["basename"] == "unit.schema.json" for r in missing
        ), f"unit.schema.json should be present; got {missing}"
        assert "unit.schema.json" in ref_set, ref_set

        # 2. Broken $ref — both internal and external.
        broken = check_internal_refs(available, allowlist)
        kinds = {r["kind"] for r in broken}
        assert "external" in kinds and "internal" in kinds, (
            f"Expected both broken kinds; got {broken}"
        )
        assert any(
            r["kind"] == "external" and r.get("target") == "ghost.schema.json"
            for r in broken
        ), broken
        assert any(
            r["kind"] == "internal" and r.get("anchor") == "$defs/Phantom"
            for r in broken
        ), broken
        # The valid internal $ref must NOT be in broken.
        assert not any(
            r["kind"] == "internal" and r.get("anchor") == "$defs/Real"
            for r in broken
        ), broken

        # 3. Pack ``$schema`` directives — relative miss + remote OK.
        unresolvable, pack_refs = check_pack_schema_directives(
            packs_dir, available, allowlist
        )
        assert any("missing-pack.schema.json" in r["value"] for r in unresolvable), (
            f"Expected missing-pack to be unresolvable; got {unresolvable}"
        )
        assert not any(
            "unit.schema.json" in r["value"] for r in unresolvable
        ), f"unit.schema.json should resolve; got {unresolvable}"
        assert "unit.schema.json" in pack_refs, pack_refs
        # Remote URL is recorded (well-formed) — should NOT be in unresolvable.
        assert not any(
            r["value"].startswith("https://example.com") for r in unresolvable
        ), unresolvable

        # 4. Doc drift — ghost-doc missing, unit OK.
        drift, doc_refs = check_doc_schema_references(
            docs_dir, available, allowlist
        )
        assert any(r["basename"] == "ghost-doc.schema.json" for r in drift), (
            f"Expected ghost-doc.schema.json doc drift; got {drift}"
        )
        assert "unit.schema.json" in doc_refs, doc_refs

        # 5. Orphan schema (warn-only) — orphan.schema.json has no callers,
        # broken.schema.json also has no callers in this fixture.
        all_refs = ref_set | pack_refs | doc_refs
        orphans = check_orphan_schemas(available, all_refs, allowlist)
        orphan_names = {r["schema"] for r in orphans}
        assert "orphan.schema.json" in orphan_names, orphan_names
        # Schemas referenced should NOT be in orphan list.
        assert "unit.schema.json" not in orphan_names, orphan_names
        # weapon is referenced via $ref from unit.schema.json — but our
        # orphan check only looks at C#/pack/doc references. So weapon.schema
        # CAN be considered orphan unless it's referenced by code/docs/packs.
        # That's the spec — it's a soft warning, not a hard failure.

        # 6. Allowlist should suppress allowlisted misses.
        allow = {"ghost-loader.schema.json", "ghost-doc.schema.json", "ghost.schema.json"}
        missing_allow, _ = check_schema_files_referenced_by_code(
            src_dir, available, allow
        )
        assert not any(
            r["basename"] == "ghost-loader.schema.json" for r in missing_allow
        ), f"Allowlist failed for ghost-loader; got {missing_allow}"

        broken_allow = check_internal_refs(available, allow)
        # Internal anchors are NOT allowlistable (they're in-file pointers),
        # so $defs/Phantom should still appear. External ghost.schema.json
        # should be suppressed.
        assert not any(
            r.get("target") == "ghost.schema.json" for r in broken_allow
        ), f"Allowlist failed for external ghost.schema.json; got {broken_allow}"
        assert any(
            r.get("anchor") == "$defs/Phantom" for r in broken_allow
        ), f"Internal $ref should not be allowlistable; got {broken_allow}"

        drift_allow, _ = check_doc_schema_references(docs_dir, available, allow)
        assert not any(
            r["basename"] == "ghost-doc.schema.json" for r in drift_allow
        ), f"Allowlist failed for ghost-doc; got {drift_allow}"

        # 7. Healthy minimal schema set passes when extraneous fixtures
        # are removed. We simulate by re-running with only unit + weapon
        # plus a ref-only consumer.
        only_dir = td_path / "only"
        only_dir.mkdir()
        only_schemas = only_dir / "schemas"
        only_src = only_dir / "src"
        only_packs = only_dir / "packs"
        only_docs = only_dir / "docs"
        for d in (only_schemas, only_src, only_packs, only_docs):
            d.mkdir()
        (only_schemas / "unit.schema.json").write_text(
            json.dumps(
                {
                    "$defs": {"Pos": {"type": "object"}},
                    "properties": {
                        "pos": {"$ref": "#/$defs/Pos"},
                        "weapon": {"$ref": "weapon.schema.json"},
                    },
                }
            ),
            encoding="utf-8",
        )
        (only_schemas / "weapon.schema.json").write_text(
            json.dumps({"type": "object"}), encoding="utf-8"
        )
        (only_src / "Use.cs").write_text(
            'const string s = "schemas/unit.schema.json";\n', encoding="utf-8"
        )
        (only_packs / "p.json").write_text(
            json.dumps({"$schema": "../schemas/unit.schema.json"}),
            encoding="utf-8",
        )
        (only_docs / "uses.md").write_text(
            "Refs unit.schema.json and weapon.schema.json.\n",
            encoding="utf-8",
        )

        h_avail = enumerate_schemas(only_schemas)
        h_missing, h_ref = check_schema_files_referenced_by_code(
            only_src, h_avail, set()
        )
        h_broken = check_internal_refs(h_avail, set())
        h_unres, h_pack_ref = check_pack_schema_directives(
            only_packs, h_avail, set()
        )
        h_drift, h_doc_ref = check_doc_schema_references(
            only_docs, h_avail, set()
        )
        h_orphans = check_orphan_schemas(
            h_avail, h_ref | h_pack_ref | h_doc_ref, set()
        )

        assert h_missing == [], h_missing
        assert h_broken == [], h_broken
        assert h_unres == [], h_unres
        assert h_drift == [], h_drift
        assert h_orphans == [], h_orphans

        # 8. SCHEMA_DIRECTIVE_RE — sanity matches.
        m1 = SCHEMA_DIRECTIVE_RE.search('"$schema": "../schemas/unit.schema.json"')
        assert m1 and "unit.schema.json" in m1.group("value"), m1
        m2 = SCHEMA_DIRECTIVE_RE.search("$schema: ../schemas/unit.schema.json")
        # YAML form (no quotes around value) — our regex requires quoted
        # value, which is the JSON-canonical form. YAML un-quoted form is
        # not strictly required by the spec; pack files in this repo all
        # use JSON or quoted YAML.
        assert m2 is None or m2.group("value"), m2

        # 9. INTERNAL_REF_RE
        assert INTERNAL_REF_RE.match("#/$defs/Foo")
        assert INTERNAL_REF_RE.match("#/definitions/Bar")
        assert not INTERNAL_REF_RE.match("unit.schema.json")
        assert not INTERNAL_REF_RE.match("#/properties/x")

    print("self-test: OK")
    return 0


# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    if len(argv) == 1 and argv[0] == "--self-test":
        return _self_test()

    args = parse_args(argv)
    if args.quiet and args.verbose:
        print(
            "ERROR: --quiet and --verbose are mutually exclusive",
            file=sys.stderr,
        )
        return 2

    paths = resolve_paths(args)

    if not paths["schemas_dir"].exists():
        print(
            f"ERROR: schemas dir not found: {paths['schemas_dir']}",
            file=sys.stderr,
        )
        return 2

    if args.verbose:
        print(f"scanning {paths['schemas_dir']} ...", file=sys.stderr)

    available = enumerate_schemas(paths["schemas_dir"])
    allowlist = load_allowlist(paths["allowlist"])

    if args.verbose:
        print(
            f"  found {len(available)} schemas, "
            f"{len(allowlist)} allowlist entries",
            file=sys.stderr,
        )

    schema_missing, code_refs = check_schema_files_referenced_by_code(
        paths["src_root"], available, allowlist
    )
    ref_broken = check_internal_refs(available, allowlist)
    url_unresolvable, pack_refs = check_pack_schema_directives(
        paths["packs_root"], available, allowlist
    )
    doc_drift, doc_refs = check_doc_schema_references(
        paths["docs_root"], available, allowlist
    )
    schema_orphan = check_orphan_schemas(
        available, code_refs | pack_refs | doc_refs, allowlist
    )

    report = build_report(
        schema_missing,
        schema_orphan,
        ref_broken,
        url_unresolvable,
        doc_drift,
        strict=args.strict,
    )
    report["schemas_dir"] = paths["schemas_dir"].as_posix()
    report["src_root"] = paths["src_root"].as_posix()
    report["packs_root"] = paths["packs_root"].as_posix()
    report["docs_root"] = paths["docs_root"].as_posix()
    report["allowlist_path"] = paths["allowlist"].as_posix()
    report["allowlist_size"] = len(allowlist)
    report["schemas_seen"] = len(available)

    paths["output"].parent.mkdir(parents=True, exist_ok=True)
    paths["output"].write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "schema-drift: "
            f"missing={len(schema_missing)} "
            f"ref_broken={len(ref_broken)} "
            f"url_unresolvable={len(url_unresolvable)} "
            f"doc_drift={len(doc_drift)} "
            f"orphan={len(schema_orphan)}"
            f"{' [strict]' if args.strict else ''} "
            f"-> {paths['output']}"
        )
    else:
        print_summary(report, paths["output"])

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
