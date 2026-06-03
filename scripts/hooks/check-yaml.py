#!/usr/bin/env python3
"""Validate YAML files in the repo (excluding fuzz corpus, archives, generated dirs)."""
import sys
import os

SKIP_DIRS = {
    "FuzzCorpus", ".git", "node_modules", "__pycache__",
    "docs", "packs", ".claude",
}
# Keep these sub-paths explicitly (not everything under docs/ is bad, but
# we allow them through the per-path filter below instead of os.walk prune).

# Dirs to prune from os.walk entirely (never recurse into)
PRUNE_DIRS = {
    "FuzzCorpus", ".git", "node_modules", "__pycache__", ".claude",
}

# Top-level dir name prefixes to skip entirely (stray worktree artifacts)
SKIP_TOP_PREFIXES = ("UserskooshDino.",)

# Path segments that mean skip this file
SKIP_PATH_SEGMENTS = {
    "docs/sessions", "docs/archive", "packs/_archived", "docs/research",
}

try:
    import yaml
except ImportError:
    print("pyyaml not installed — skipping YAML check (pip install pyyaml)")
    sys.exit(0)


def should_skip(path: str) -> bool:
    normalized = path.replace("\\", "/")
    parts = normalized.split("/")
    # Skip stray top-level worktree artifact directories
    if parts and any(parts[0].startswith(p) for p in SKIP_TOP_PREFIXES):
        return True
    # Skip if any known skip segment is a prefix of the path
    for seg in SKIP_PATH_SEGMENTS:
        if normalized.startswith(seg):
            return True
    return False


files = []
for dirpath, dirnames, filenames in os.walk("."):
    # Prune directories in-place so os.walk won't recurse into them
    rel = dirpath.replace("\\", "/").lstrip("./")
    top = rel.split("/")[0] if rel else ""
    # Skip stray top-level artifact dirs
    if top and any(top.startswith(p) for p in SKIP_TOP_PREFIXES):
        dirnames.clear()
        continue
    dirnames[:] = [d for d in dirnames if d not in PRUNE_DIRS]

    for fname in filenames:
        if not (fname.endswith(".yaml") or fname.endswith(".yml")):
            continue
        raw = os.path.join(dirpath, fname).replace("\\", "/")
        # Strip leading "./" path prefix only (not individual leading dots in filenames)
        path = raw[2:] if raw.startswith("./") else raw
        if not should_skip(path):
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
