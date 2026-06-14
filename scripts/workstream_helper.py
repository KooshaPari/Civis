#!/usr/bin/env python3
"""Shared helper utilities for scripts/ workstream collectors and checks."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Iterator, Sequence


def repo_root(start: Path | None = None) -> Path:
    """Resolve the repository root from ``start`` or the current file location."""

    current = (start or Path.cwd()).resolve()
    for candidate in (current, *current.parents):
        if (candidate / ".git").exists() or (candidate / "src").exists():
            return candidate
    return current


def iter_files(
    root: Path,
    *,
    suffixes: Sequence[str] | None = None,
    include_names: Sequence[str] | None = None,
    exclude_dirs: Sequence[str] = ("bin", "obj", ".git", "node_modules", "__pycache__"),
) -> Iterator[Path]:
    """Yield files under ``root`` with optional suffix/name filters."""

    suffix_set = {suffix.lower() for suffix in suffixes or ()}
    name_set = {name.lower() for name in include_names or ()}
    excluded = {item.lower() for item in exclude_dirs}

    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if any(part.lower() in excluded for part in path.parts):
            continue
        if suffix_set and path.suffix.lower() not in suffix_set:
            continue
        if name_set and path.name.lower() not in name_set:
            continue
        yield path


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def read_json(path: Path):
    return json.loads(read_text(path))


def write_json(data, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def relpath(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def line_count(path: Path) -> int:
    text = read_text(path)
    return 0 if not text else text.count("\n") + (0 if text.endswith("\n") else 1)


@dataclass(frozen=True)
class ScriptFinding:
    path: str
    detail: str


def emit_summary(title: str, rows: Iterable[str]) -> None:
    print(title)
    for row in rows:
        print(row)
