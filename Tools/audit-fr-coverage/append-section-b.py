"""Append Section B batch rows + 4 charter integration rows to fr-emergence-matrix.md.

This is a one-shot tool that:
  1. reads the current matrix
  2. reads the generated batches file (146 rows)
  3. builds 4 charter integration rows (FR-CIV-0100-int1..int4) per COVERAGE_AUDIT §3.3
  4. appends all of them under a Section B header
"""
import sys
from pathlib import Path

if len(sys.argv) < 2:
    print("usage: append-section-b.py <repo-root>")
    sys.exit(1)

root = Path(sys.argv[1])
matrix_p = root / "docs/traceability/fr-emergence-matrix.md"
batch_p  = root / "docs/traceability/fr-emergence-batches.md"

if not matrix_p.exists() or not batch_p.exists():
    print(f"MISSING: matrix={matrix_p.exists()} batch={batch_p.exists()}")
    sys.exit(2)

main = matrix_p.read_text(encoding="utf-8")
batch = batch_p.read_text(encoding="utf-8").lstrip()

# Build 4 charter integration rows per audit §3.3
charter_int_rows = """
| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|-------|--------|-------------|-------|--------|-------|------------|-------|
| FR-CIV-0100-int1 | Integration | charter-umbrella → emergence | dormant | dormant | @kooshapari | docs/traceability/fr-emergence-matrix.md#section-a | Charter row promotes a single emerg spec to COVERED via spec + code + test triple. Promoted by §3.4 first-priority set (FR-CIV-EMERG-001..005). |
| FR-CIV-0100-int2 | Integration | charter-umbrella → recovery | dormant | dormant | @kooshapari | docs/traceability/fr-emergence-matrix.md#section-a | Charter row promoting a recovered-requirement batch (FR-CIV-EMERGENCE-001..006) to COVERED. First spec to satisfy this gets green. |
| FR-CIV-0100-int3 | Integration | charter-umbrella → coupling | dormant | dormant | @kooshapari | docs/traceability/emergent-systems-tracelinks.md | Charter row promoting a 30-coupling point (one of the systems × APIs listed in tracelinks §2) to COVERED via a typed bridge + 5 doc tests. |
| FR-CIV-0100-int4 | Integration | charter-umbrella → audit-script | dormant | dormant | @kooshapari | Tools/audit-fr-coverage/audit.sh | Charter row closed by landing the audit.sh + .github/workflows/audit-fr-coverage.yml CI integration (currently in-flight per pending items). |
"""

section_b_header = "## Section B: Emergence batch rows (§3.1 + §3.2 + §3.3, all currently dormant)\n\n"
section_b_intro = (
    "146 rows generated from COVERAGE_AUDIT.md §3.1 + §3.2 sub-clusters, plus 4 charter "
    "integration rows from §3.3. All marked `dormant` until each acquires spec + code + test "
    "triple. Promote to `planned` once a spec exists; to `implemented` once the code is in a "
    "crate; to `covered` once tests pin the contract.\n\n"
)

# Force-append: remove any prior orphan Section B header (without rows) so the append below
# re-establishes the canonical state. This makes the tool non-idempotent but correct-on-retry.
import re
main = re.sub(
    r"\n+---\n+\n+## Section B[^\n]*\n+\n+[^\n]*\n+\n+(?=#|\Z)",
    "",
    main,
    flags=re.DOTALL,
)
# If the file currently has Section B with rows, this is the no-op case.
if re.search(r"^## Section B[^\n]*\n.*?FR-CIV-EMERGENCE-1\d\d\|", main, flags=re.MULTILINE | re.DOTALL):
    print("ALREADY_HAS_FULL_SECTION_B")
    sys.exit(0)

new_section = (
    "\n\n---\n\n"
    + section_b_header
    + section_b_intro
    + charter_int_rows
    + "\n---\n\n### B.1 Sub-cluster batches\n\n"
    + batch
)

matrix_p.write_text(main.rstrip() + new_section, encoding="utf-8")
print("appended Section B + 4 charter int rows + 146 batch rows")
print("new size kb:", matrix_p.stat().st_size // 1024)