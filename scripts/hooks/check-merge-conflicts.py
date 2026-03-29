#!/usr/bin/env python3
"""Detect merge conflict markers in staged or all tracked text files.

A real merge conflict requires ALL THREE markers present in the same file:
  <<<<<<< (conflict start)
  ======= (separator)
  >>>>>>> (conflict end)
Single '=======' lines are common in docs/logs and are NOT conflicts.
"""
import sys
import subprocess
import os

SKIP_EXT = {
    ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tga", ".psd",
    ".fbx", ".obj", ".dae", ".blend",
    ".mp3", ".ogg", ".wav", ".mp4", ".avi",
    ".dll", ".exe", ".bin", ".zip", ".7z", ".gz", ".tar",
    ".meta", ".prefab", ".mat", ".asset", ".unity",
    ".pyc", ".pdb",
    ".log",   # build logs contain === dividers
}

SKIP_DIRS = {
    ".git", "unity-assetbundle-builder", "node_modules",
    "docs/archive", "docs/sessions", "docs/research",
    ".claude",
}

# Get staged files if in hook context
result = subprocess.run(
    ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"],
    capture_output=True, text=True
)
files = [f.strip() for f in result.stdout.splitlines() if f.strip()]

# Fall back to all tracked files (--all-files run)
if not files:
    result = subprocess.run(["git", "ls-files"], capture_output=True, text=True)
    files = [f.strip() for f in result.stdout.splitlines() if f.strip()]

conflicts = []
for path in files:
    if not os.path.isfile(path):
        continue
    ext = os.path.splitext(path)[1].lower()
    if ext in SKIP_EXT:
        continue
    # Skip binary asset bundles (no extension, in bundles/ dir)
    if "bundles" + os.sep in path or "/bundles/" in path:
        continue
    parts = path.replace("\\", "/").split("/")
    if any(d in parts or path.replace("\\", "/").startswith(d) for d in SKIP_DIRS):
        continue
    try:
        with open(path, encoding="utf-8", errors="ignore") as fh:
            content = fh.read()
        # All three markers must be present for a real conflict
        if "<<<<<<< " in content and "=======" in content and ">>>>>>> " in content:
            conflicts.append(path)
    except OSError:
        pass

if conflicts:
    print("Merge conflict markers found:", file=sys.stderr)
    for c in conflicts:
        print(f"  {c}", file=sys.stderr)
    sys.exit(1)

print(f"check-merge-conflicts: {len(files)} files clean")
