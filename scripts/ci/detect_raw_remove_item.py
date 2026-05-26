#!/usr/bin/env python3
"""
detect_raw_remove_item.py — P1 #862 governance detector.

Scans scripts/ for raw `Remove-Item` usage outside the shared helper at
scripts/shared/Remove-ToRecycleBin.ps1.

Severity:
  HIGH  : Remove-Item with both -Recurse AND -Force (irrecoverable)
  MED   : Remove-Item with -Recurse only
  LOW   : Bare Remove-Item (single-file delete)

Outputs:
  docs/qa/raw-remove-item-report.json

Exit codes:
  0 : HIGH <= threshold (default 0)
  1 : HIGH > threshold (CI fail)

Allowlist: docs/qa/raw-remove-item-allowlist.txt (one repo-relative path per line,
'#' comments and blank lines ignored).

Inline marker (per line): `# remove-item-ok: <reason>` suppresses that line.

Governance: CLAUDE.md > "File Deletion Protocol (MANDATORY)"
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
HELPER_PATH = (SCRIPTS_DIR / "shared" / "Remove-ToRecycleBin.ps1").resolve()
ALLOWLIST_PATH = REPO_ROOT / "docs" / "qa" / "raw-remove-item-allowlist.txt"
REPORT_PATH = REPO_ROOT / "docs" / "qa" / "raw-remove-item-report.json"

HIGH_THRESHOLD = 0  # any HIGH violation outside allowlist fails CI

REMOVE_ITEM_RE = re.compile(r"\bRemove-Item\b", re.IGNORECASE)
RECURSE_RE = re.compile(r"-Recurse\b", re.IGNORECASE)
FORCE_RE = re.compile(r"-Force\b", re.IGNORECASE)
INLINE_OK_RE = re.compile(r"#\s*remove-item-ok\b", re.IGNORECASE)


def load_allowlist() -> set[str]:
    allow: set[str] = set()
    if not ALLOWLIST_PATH.exists():
        return allow
    for line in ALLOWLIST_PATH.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        # Strip trailing inline comment
        s = s.split("#", 1)[0].strip()
        if s:
            allow.add(s.replace("\\", "/"))
    return allow


def classify(line: str) -> str:
    has_recurse = bool(RECURSE_RE.search(line))
    has_force = bool(FORCE_RE.search(line))
    if has_recurse and has_force:
        return "HIGH"
    if has_recurse:
        return "MED"
    return "LOW"


def scan() -> dict:
    allow = load_allowlist()
    violations: list[dict] = []
    files_scanned = 0

    # Walk scripts/ for .ps1, .psm1 files; also include .sh/.cmd which may shell out
    patterns = ("*.ps1", "*.psm1", "*.cmd", "*.sh")
    candidates: list[Path] = []
    for pat in patterns:
        candidates.extend(SCRIPTS_DIR.rglob(pat))

    for path in candidates:
        try:
            rp = path.resolve()
        except OSError:
            continue
        if rp == HELPER_PATH:
            continue  # the helper itself is exempt
        rel = rp.relative_to(REPO_ROOT).as_posix()
        if rel in allow:
            continue
        files_scanned += 1
        try:
            text = rp.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for lineno, line in enumerate(text.splitlines(), start=1):
            if not REMOVE_ITEM_RE.search(line):
                continue
            if INLINE_OK_RE.search(line):
                continue
            sev = classify(line)
            violations.append({
                "file": rel,
                "line": lineno,
                "severity": sev,
                "text": line.strip()[:200],
            })

    counts = {"HIGH": 0, "MED": 0, "LOW": 0}
    for v in violations:
        counts[v["severity"]] += 1

    return {
        "files_scanned": files_scanned,
        "violation_count": len(violations),
        "counts": counts,
        "threshold_high": HIGH_THRESHOLD,
        "violations": violations,
        "allowlist_path": ALLOWLIST_PATH.relative_to(REPO_ROOT).as_posix(),
        "helper_path": HELPER_PATH.relative_to(REPO_ROOT).as_posix(),
    }


def main() -> int:
    report = scan()
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(json.dumps(report, indent=2), encoding="utf-8")

    c = report["counts"]
    print(f"[detect_raw_remove_item] scanned={report['files_scanned']} "
          f"HIGH={c['HIGH']} MED={c['MED']} LOW={c['LOW']} "
          f"threshold_high={HIGH_THRESHOLD}")
    print(f"[detect_raw_remove_item] report: {REPORT_PATH.relative_to(REPO_ROOT).as_posix()}")

    # Print top 10 worst (HIGH first, then MED)
    ordered = sorted(report["violations"],
                     key=lambda v: ({"HIGH": 0, "MED": 1, "LOW": 2}[v["severity"]],
                                    v["file"], v["line"]))
    if ordered:
        print("[detect_raw_remove_item] top violations:")
        for v in ordered[:10]:
            print(f"  {v['severity']:4s} {v['file']}:{v['line']}  {v['text'][:120]}")

    if c["HIGH"] > HIGH_THRESHOLD:
        print(f"[detect_raw_remove_item] FAIL: HIGH={c['HIGH']} > {HIGH_THRESHOLD}",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
