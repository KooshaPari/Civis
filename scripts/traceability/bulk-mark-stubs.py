"""Bulk-mark placeholder test files with `//! Stub: TDD-red`.

Detection: any `*.rs` under `crates/*/tests/` whose first 30 lines contain
`//! Epic: auto-generated` is treated as a placeholder. The marker is added
immediately after that line so an audit script can grep for it later.

This script is idempotent: re-running it on an already-marked file is a no-op.
A `--dry-run` flag reports what would change without writing.

Exit codes:
  0  success
  1  unexpected error
  2  nothing to do
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STUB_MARKER = "//! Stub: TDD-red \u2014 replace with real FR assertions"
ANCHOR = "//! Epic: auto-generated"


def is_placeholder(text: str) -> bool:
    head = "\n".join(text.splitlines()[:30])
    return ANCHOR in head


def has_marker(text: str) -> bool:
    head = "\n".join(text.splitlines()[:30])
    return "Stub: TDD-red" in head


def insert_marker(text: str) -> str:
    lines = text.splitlines(keepends=True)
    out = []
    inserted = False
    for line in lines:
        out.append(line)
        if not inserted and ANCHOR in line:
            out.append(STUB_MARKER + "\n")
            inserted = True
    if not inserted:
        return text
    return "".join(out)


def walk_placeholders(root: Path):
    for tests_dir in (root / "crates").rglob("tests"):
        if not tests_dir.is_dir():
            continue
        for rs in tests_dir.glob("*.rs"):
            yield rs


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args(argv)

    scanned = marked = skipped = errors = 0
    for rs in walk_placeholders(ROOT):
        scanned += 1
        try:
            text = rs.read_text(encoding="utf-8")
        except Exception as exc:
            errors += 1
            if args.verbose:
                print(f"ERROR read {rs}: {exc}", file=sys.stderr)
            continue
        if not is_placeholder(text):
            skipped += 1
            continue
        if has_marker(text):
            skipped += 1
            continue
        new_text = insert_marker(text)
        if args.dry_run:
            marked += 1
            if args.verbose:
                print(f"DRY: would mark {rs.relative_to(ROOT)}")
        else:
            try:
                rs.write_text(new_text, encoding="utf-8")
                marked += 1
            except Exception as exc:
                errors += 1
                print(f"ERROR write {rs}: {exc}", file=sys.stderr)

    print(
        f"scanned={scanned} marked={marked} skipped={skipped} errors={errors} "
        f"dry_run={args.dry_run}"
    )
    return 0 if errors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
