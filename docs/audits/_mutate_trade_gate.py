"""Mutation-check the trade-flow threshold guard.

Flips `<=` to `<` in compute_trade_flows and confirms the boundary test catches
it, then flips the threshold constant and confirms the ULP test catches that.
A boundary test that cannot fail is worse than no boundary test.
"""
import pathlib
import re
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
PATH = REPO / "crates" / "economy" / "src" / "trade.rs"
TEST = "trade::tests::emergent_trade_flows_follow_price_differentials"
ORIGINAL = PATH.read_text(encoding="utf-8")

MUTANTS = [
    (
        "gate becomes strict <",
        "if diff.abs() <= PRICE_DIFFERENTIAL_THRESHOLD {",
        "if diff.abs() < PRICE_DIFFERENTIAL_THRESHOLD {",
    ),
    (
        "threshold 0.2 -> 0.25",
        "const PRICE_DIFFERENTIAL_THRESHOLD: f32 = 0.2;",
        "const PRICE_DIFFERENTIAL_THRESHOLD: f32 = 0.25;",
    ),
    (
        "threshold 0.2 -> 0.19",
        "const PRICE_DIFFERENTIAL_THRESHOLD: f32 = 0.2;",
        "const PRICE_DIFFERENTIAL_THRESHOLD: f32 = 0.19;",
    ),
]


def run(name, old, new):
    if old not in ORIGINAL:
        print(f"  {name:28s} SKIPPED (anchor not found)")
        return None
    PATH.write_text(ORIGINAL.replace(old, new, 1), encoding="utf-8")
    p = subprocess.run(
        ["cargo", "test", "-p", "civ-economy", "--lib", TEST],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    killed = p.returncode != 0
    print(f"  {name:28s} {'KILLED' if killed else 'SURVIVED'}")
    return killed


try:
    baseline = subprocess.run(
        ["cargo", "test", "-p", "civ-economy", "--lib", TEST],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    print(f"baseline (unmutated)         {'PASSES' if baseline.returncode == 0 else 'ALREADY FAILING'}")
    print("mutants:")
    results = [run(*m) for m in MUTANTS]
finally:
    PATH.write_text(ORIGINAL, encoding="utf-8")
    print("source restored")

survivors = [m[0] for m, r in zip(MUTANTS, results) if r is False]
skipped = [m[0] for m, r in zip(MUTANTS, results) if r is None]
print()
print(f"killed: {sum(1 for r in results if r)}  survived: {len(survivors)}  skipped: {len(skipped)}")
if survivors or skipped:
    raise SystemExit(1)
print("ALL MUTANTS KILLED")
