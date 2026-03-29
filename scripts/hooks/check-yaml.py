#!/usr/bin/env python3
"""Validate YAML files in the repo (excluding fuzz corpus, archives, generated dirs)."""
import sys
import glob
import os

SKIP_DIRS = {
    "FuzzCorpus", ".git", "node_modules", "__pycache__",
    "docs/sessions", "docs/archive", "packs/_archived", "docs/research",
    ".claude",
}

try:
    import yaml
except ImportError:
    print("pyyaml not installed — skipping YAML check (pip install pyyaml)")
    sys.exit(0)

files = []
for pattern in ("**/*.yaml", "**/*.yml"):
    for path in glob.glob(pattern, recursive=True):
        normalized = path.replace("\\", "/")
        parts = normalized.split("/")
        if any(d in parts or normalized.startswith(d) for d in SKIP_DIRS):
            continue
        files.append(path)

errors = []
for path in files:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            list(yaml.safe_load_all(fh))
    except yaml.YAMLError as e:
        errors.append(f"{path}: {e}")

if errors:
    for err in errors:
        print(err, file=sys.stderr)
    sys.exit(1)

print(f"check-yaml: {len(files)} files OK")
