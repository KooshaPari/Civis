"""Mutation-check the waste-heat overflow fix.

Reverting the u128 widening must make `large_consumption_no_overflow` fail again,
and narrowing the clamp must not silently restore safety. A fix to a real
overflow that no test can detect is not a fix.
"""
import pathlib
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
PATH = REPO / "crates" / "economy" / "src" / "waste.rs"
TEST = ["--test", "fr_fr_econ_005"]
ORIGINAL = PATH.read_text(encoding="utf-8")

WIDENED = "let waste = (consumption as u128 * fraction as u128 / BP_DENOM as u128) as i64;"
NARROW = "let waste = consumption * fraction / BP_DENOM;"

MUTANTS = [
    ("revert u128 widening (the original bug)", WIDENED, NARROW),
    (
        "widen the numerator only",
        WIDENED,
        "let waste = (consumption as u128 * fraction as i128 / BP_DENOM as i128) as i64;",
    ),
    (
        "drop the clamp so fraction can exceed the denominator",
        "let fraction = config.fraction_bp.clamp(0, BP_DENOM);",
        "let fraction = config.fraction_bp.max(0);",
    ),
]


def run(name, old, new):
    if old not in ORIGINAL:
        print(f"  {name:44s} SKIPPED (anchor not found)")
        return None
    PATH.write_text(ORIGINAL.replace(old, new, 1), encoding="utf-8")
    p = subprocess.run(
        ["cargo", "test", "-p", "civ-economy", *TEST],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    killed = p.returncode != 0
    print(f"  {name:44s} {'KILLED' if killed else 'SURVIVED'}")
    return killed


try:
    base = subprocess.run(
        ["cargo", "test", "-p", "civ-economy", *TEST],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    ok = base.returncode == 0
    print(f"baseline (unmutated)                     {'PASSES' if ok else 'ALREADY FAILING'}")
    if not ok:
        # The whole point of the baseline is to prove the fix holds, so show why
        # it did not rather than reporting a bare verdict.
        print("---- baseline stdout ----")
        print(base.stdout[-3000:])
        print("---- baseline stderr ----")
        print(base.stderr[-3000:])
        raise SystemExit(1)
    print("mutants:")
    results = [run(*m) for m in MUTANTS]
finally:
    PATH.write_text(ORIGINAL, encoding="utf-8")
    print("source restored")

survivors = [m[0] for m, r in zip(MUTANTS, results) if r is False]
skipped = [m[0] for m, r in zip(MUTANTS, results) if r is None]
print()
print(f"killed: {sum(1 for r in results if r)}  survived: {len(survivors)}  skipped: {len(skipped)}")
for s in survivors:
    print(f"  SURVIVED: {s}")
if survivors or skipped:
    raise SystemExit(1)
print("ALL MUTANTS KILLED")
