#!/usr/bin/env python3
"""Point heavy workflows at main/workflow_dispatch only (local-first CI bypass)."""
from __future__ import annotations

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
WF = ROOT / ".github" / "workflows"
# Note: pr-governance.yml keeps its display name `pr-governance-gate` (the
# duplicate pr-governance-gate.yml was removed; this is the single source).
KEEP_PR = {"quality.yml", "pr-governance.yml", "self-merge-gate.yml"}

LOCAL_FIRST_ON = """# Local-first: PRs use quality.yml + pr-governance-gate only.
# Run heavy gates locally: lefthook run pre-push
on:
  workflow_dispatch:
  push:
    branches: [main]
"""

def strip_pull_request(text: str) -> str:
    # Replace simple one-liner triggers
    if re.match(r"on:\s*\[push,\s*pull_request\]\s*", text):
        return LOCAL_FIRST_ON + text.split("\n", 1)[1]

    lines = text.splitlines()
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.strip() == "on:":
            block = [line]
            i += 1
            while i < len(lines) and (lines[i].startswith(" ") or lines[i].strip() == ""):
                block.append(lines[i])
                i += 1
            block_text = "\n".join(block)
            if "pull_request" in block_text:
                out.append(LOCAL_FIRST_ON.rstrip())
            else:
                out.extend(block)
            continue
        out.append(line)
        i += 1
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def main() -> None:
    for path in sorted(WF.glob("*.yml")):
        if path.name in KEEP_PR:
            continue
        original = path.read_text(encoding="utf-8")
        if "pull_request" not in original:
            continue
        updated = strip_pull_request(original)
        if updated != original:
            path.write_text(updated, encoding="utf-8", newline="\n")
            print(f"updated {path.name}")


if __name__ == "__main__":
    main()
