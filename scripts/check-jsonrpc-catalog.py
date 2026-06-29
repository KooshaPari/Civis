#!/usr/bin/env python3
"""Compare JsonRpcMethod wire names with docs/api/jsonrpc-surface.md."""

from __future__ import annotations

import re
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def methods_from_rust(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    match = re.search(r"impl JsonRpcMethod \{([\s\S]*?)\n\}", text)
    if not match:
        raise RuntimeError(f"JsonRpcMethod impl block not found in {path}")
    body = match.group(1)
    names = set(re.findall(r'Self::\w+\s*=>\s*"([^"]+)"', body))
    names.update(re.findall(r'"([^"]+)"\s*=>\s*Some\(Self::', body))
    if not names:
        raise RuntimeError(f"No JsonRpcMethod wire names found in {path}")
    return names


def methods_from_doc(path: Path) -> set[str]:
    names: set[str] = set()
    in_catalog = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if re.match(r"^## Method catalog", line):
            in_catalog = True
            continue
        if in_catalog and re.match(r"^---\s*$", line):
            break
        if not in_catalog:
            continue
        match = re.match(r"^\|\s*`([a-z][a-z0-9_.]*)`\s*\|", line)
        if match:
            names.add(match.group(1))
    if not names:
        raise RuntimeError(f"No method rows found in Method catalog section of {path}")
    return names


def main() -> int:
    root = repo_root()
    rust = methods_from_rust(root / "crates/server/src/jsonrpc.rs")
    doc = methods_from_doc(root / "docs/api/jsonrpc-surface.md")

    only_rust = sorted(rust - doc)
    only_doc = sorted(doc - rust)
    if not only_rust and not only_doc:
        print(f"jsonrpc catalog OK ({len(rust)} methods)")
        return 0

    print("jsonrpc catalog DRIFT")
    print(f"  rust ({len(rust)}): {', '.join(sorted(rust))}")
    print(f"  doc  ({len(doc)}): {', '.join(sorted(doc))}")
    if only_rust:
        print("  in rust only:")
        for name in only_rust:
            print(f"    + {name}")
    if only_doc:
        print("  in doc only:")
        for name in only_doc:
            print(f"    - {name}")
    return 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)
