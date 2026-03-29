#!/usr/bin/env python3
"""Validate JSON files in the repo (excluding generated, binary-adjacent, and fuzz files)."""
import sys
import glob
import json
import os

SKIP_NAMES = {"asset_manifest.json", "packages.lock.json", "tundra.log.json"}
SKIP_DIRS  = {
    "FuzzCorpus", ".git", "node_modules",
    "unity-assetbundle-builder", "__pycache__",
}

files = []
for path in glob.glob("**/*.json", recursive=True):
    normalized = path.replace("\\", "/")
    name = os.path.basename(path)
    if name in SKIP_NAMES:
        continue
    if any(d in normalized.split("/") or normalized.startswith(d) for d in SKIP_DIRS):
        continue
    files.append(path)

errors = []
for path in files:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            json.load(fh)
    except json.JSONDecodeError as e:
        errors.append(f"{path}: {e}")

if errors:
    for err in errors:
        print(err, file=sys.stderr)
    sys.exit(1)

print(f"check-json: {len(files)} files OK")
