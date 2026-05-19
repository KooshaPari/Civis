#!/usr/bin/env python3
"""Trait-fraud detector — rejects tests that claim E2E/Journey/UserStory traits but use a Fake bridge.

Usage: python scripts/analysis/check_trait_fraud.py

Exit 0 = clean
Exit 1 = fraud found, reports each violation

Background: see task #190 in CLAUDE memory. Audit identified two HIGH-severity sites
(InGameAutomationTests, WorkflowE2ETests) that carry [Trait("Category","E2E")]
or Journey/UserStory traits while their body uses FakeGameBridge. This script
enforces the rule: if your class wears an E2E badge AND uses a Fake bridge body,
you must ALSO wear the Bridge:Fake label so reviewers/grep can see the truth.

The deeper question (is this test really E2E?) is a separate decision. This
script only enforces honest labeling.
"""
import re
import sys
from pathlib import Path

E2E_TRAIT_PATTERNS = [
    re.compile(r'\[\s*Trait\s*\(\s*"Category"\s*,\s*"E2E"\s*\)\s*\]'),
    re.compile(r'\[\s*Trait\s*\(\s*"Category"\s*,\s*"Journey"\s*\)\s*\]'),
    re.compile(r'\[\s*Trait\s*\(\s*"Category"\s*,\s*"UserStory"\s*\)\s*\]'),
    re.compile(r'\[\s*Trait\s*\(\s*"Journey"\s*,'),
    re.compile(r'\[\s*Trait\s*\(\s*"UserStory"\s*,'),
]

FAKE_BRIDGE_TRAIT = re.compile(r'\[\s*Trait\s*\(\s*"Bridge"\s*,\s*"Fake"\s*\)\s*\]')

CLASS_DECL = re.compile(
    r'((?:\[[^\]]*\]\s*\n\s*)*)'  # zero or more attribute blocks before class
    r'(public\s+(?:abstract\s+|sealed\s+|static\s+)?(?:partial\s+)?class\s+(\w+))'
)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    tests_dir = repo_root / "src" / "Tests"
    if not tests_dir.exists():
        print(f"ERROR: tests dir not found: {tests_dir}", file=sys.stderr)
        return 2

    violations = []
    scanned = 0

    for cs_file in tests_dir.rglob("*.cs"):
        if "/obj/" in cs_file.as_posix() or "/bin/" in cs_file.as_posix():
            continue
        scanned += 1
        try:
            text = cs_file.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = cs_file.read_text(encoding="utf-8", errors="replace")

        for m in CLASS_DECL.finditer(text):
            attrs, _, classname = m.groups()
            attrs = attrs or ""

            has_e2e_trait = any(p.search(attrs) for p in E2E_TRAIT_PATTERNS)
            has_fake_trait = bool(FAKE_BRIDGE_TRAIT.search(attrs))

            class_start = m.end()
            class_body = text[class_start:class_start + 8000]
            uses_fake = (
                "FakeGameBridge" in class_body
                or "new MockGameBridgeServer" in class_body
            )

            if has_e2e_trait and uses_fake and not has_fake_trait:
                rel = cs_file.relative_to(repo_root).as_posix()
                violations.append(
                    f"{rel}::{classname} — E2E/Journey/UserStory trait + Fake-bridge body without [Trait(\"Bridge\",\"Fake\")]"
                )

    if violations:
        print("TRAIT FRAUD DETECTED:")
        for v in violations:
            print(f"  - {v}")
        print()
        print("Fix by ONE of:")
        print("  1. Add [Trait(\"Bridge\",\"Fake\")] to the class (honest label).")
        print("  2. Remove the E2E/Journey/UserStory trait (test is not really E2E).")
        print("  3. Replace FakeGameBridge with a real bridge gated behind WINDOWS_GAME_AVAILABLE.")
        print()
        print(f"Scanned {scanned} .cs files in {tests_dir.relative_to(repo_root).as_posix()}.")
        return 1

    print(f"Trait-fraud check: CLEAN ({scanned} files scanned)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
