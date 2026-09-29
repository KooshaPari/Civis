#!/usr/bin/env python3
"""Normalize the accidental `NFR-` double prefix in FR/NFR references.

A generator that names traceability files `fr_<id>.rs` (lowercased) and then
prepends `FR-` to the filename slug produced references that double up the
prefix, so an NFR requirement got written as FR- + NFR- + <rest>. The real
requirement IDs drop the leading `FR-`, and those IDs already exist in the
spec tree. The double-prefixed spellings are phantom IDs: they created 30
bogus rows in docs/audits/fr-matrix.json, including 19 fake SPEC-ONLY gaps
and the repo's only CODE-ONLY-no-spec row.

This script rewrites the double prefix down to the real one in the source,
test, and intent files that carry it. Generated/audit artifacts under
docs/audits/ are left alone; they are regenerated from source by
_gather_ids.py + gen-fr-audit.py.

It also fixes a second, narrower class: three LEGENDS word-form ids that
exist under BOTH an FR- and an NFR- spelling, where the NFR- spelling is the
one that resolves to real code and tests. Those three FR- spellings are pure
duplicates. The other thirteen LEGENDS word-form ids appear only under the
FR- spelling with no NFR- counterpart anywhere, so they are left alone:
renaming them would invent requirement IDs that no spec defines.

NOTE: this file is itself scanned for FR/NFR tokens by the audit, so it must
not contain any fully-formed phantom ID. That is why the examples above are
described rather than written literally.

Idempotent: running it twice is a no-op.

Usage:
    python scripts/traceability/fix-nfr-prefix.py           # rewrite in place
    python scripts/traceability/fix-nfr-prefix.py --check   # report only
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# The bad prefix, assembled at runtime so this file does not literally contain
# a complete phantom ID that the audit would pick up as a code reference.
BAD = "FR-" + "NFR-"
GOOD = BAD[3:]

# Second class: FR-/NFR- duplicate spellings where the NFR- form is canonical
# because it is the one carrying real code and test references.
DUP_PREFIX_BAD = "FR-CIV-LEGENDS-"
DUP_PREFIX_GOOD = "NFR-CIV-LEGENDS-"
# Only the three that exist under both spellings with the NFR- one resolved.
DUP_SUFFIXES = ("CONFIG-04", "SCALE-02", "PERF-01")

# Generated or historical artifacts: not rewritten, they get regenerated.
SKIP_PREFIXES = ("docs/audits/", "docs/audits", "target/")


def tracked_files(*needles: str) -> list[Path]:
    files: dict[str, Path] = {}
    for needle in needles:
        out = subprocess.run(
            ["git", "grep", "-l", "-F", needle],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        for rel in out.stdout.split():
            if any(rel == s or rel.startswith(s) for s in SKIP_PREFIXES):
                continue
            files[rel] = ROOT / rel
    return list(files.values())


def rewrite(text: str) -> tuple[str, int, int]:
    """Apply both rewrite classes. Returns (new_text, double_hits, dup_hits)."""
    double_hits = text.count(BAD)
    text = text.replace(BAD, GOOD)

    dup_hits = 0
    for suffix in DUP_SUFFIXES:
        # A plain str.replace would also match the already-correct NFR- form,
        # because "NFR-..." contains "FR-..." as a substring, producing
        # "NNFR-...". Anchor on a boundary that is not a letter, digit, or
        # hyphen so only a bare FR- spelling is rewritten.
        pattern = re.compile(
            rf"(?<![A-Za-z0-9-]){re.escape(DUP_PREFIX_BAD + suffix)}(?![A-Za-z0-9-])"
        )
        text, n = pattern.subn(DUP_PREFIX_GOOD + suffix, text)
        dup_hits += n
    return text, double_hits, dup_hits


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="report without rewriting")
    args = ap.parse_args()

    files = tracked_files(BAD, *(DUP_PREFIX_BAD + s for s in DUP_SUFFIXES))
    if not files:
        print("No phantom FR/NFR spellings outside docs/audits/. Nothing to do.")
        return 0

    changed = 0
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as exc:
            print(f"  SKIP {rel}: {exc}")
            continue
        new_text, double_hits, dup_hits = rewrite(text)
        if not double_hits and not dup_hits:
            continue
        parts = []
        if double_hits:
            parts.append(f"{double_hits} double-prefix")
        if dup_hits:
            parts.append(f"{dup_hits} legends-duplicate")
        detail = ", ".join(parts)
        if args.check:
            print(f"  WOULD FIX {rel}: {detail}")
        else:
            path.write_text(new_text, encoding="utf-8")
            print(f"  FIXED {rel}: {detail}")
        changed += 1

    verb = "would fix" if args.check else "fixed"
    print(f"\n{verb} {changed} file(s).")
    if args.check and changed:
        print("Re-run without --check to apply.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
