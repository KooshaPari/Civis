#!/usr/bin/env python3
"""Normalize the accidental `FR-NFR-` double prefix in FR/NFR references.

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
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# The bad prefix, assembled at runtime so this file does not literally contain
# a complete phantom ID that the audit would pick up as a code reference.
BAD = "FR-" + "NFR-"
GOOD = BAD[3:]

# Generated or historical artifacts: not rewritten, they get regenerated.
SKIP_PREFIXES = ("docs/audits/", "docs/audits", "target/")


def tracked_files_with_bad_prefix() -> list[Path]:
    out = subprocess.run(
        ["git", "grep", "-l", "-F", BAD],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    files: list[Path] = []
    for rel in out.stdout.split():
        if any(rel == s or rel.startswith(s) for s in SKIP_PREFIXES):
            continue
        files.append(ROOT / rel)
    return files


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="report without rewriting")
    args = ap.parse_args()

    files = tracked_files_with_bad_prefix()
    if not files:
        print("No FR-NFR- references outside docs/audits/. Nothing to do.")
        return 0

    changed = 0
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as exc:
            print(f"  SKIP {rel}: {exc}")
            continue
        hits = text.count(BAD)
        if hits == 0:
            continue
        if args.check:
            print(f"  WOULD FIX {rel}: {hits} occurrence(s)")
        else:
            path.write_text(text.replace(BAD, GOOD), encoding="utf-8")
            print(f"  FIXED {rel}: {hits} occurrence(s)")
        changed += 1

    verb = "would fix" if args.check else "fixed"
    print(f"\n{verb} {changed} file(s).")
    if args.check and changed:
        print("Re-run without --check to apply.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
