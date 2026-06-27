"""Append all FR-CIV-EMERGENCE-100..245 batch rows (146) + 4 FR-CIV-0100-int1..4
charter integration rows to fr-emergence-matrix.md.

Replaces the previous append script (which bailed out early because it saw
'Section B' header even when no batch rows followed).
"""

import os
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
MATRIX = ROOT / "docs" / "traceability" / "fr-emergence-matrix.md"
BATCHES = ROOT / "docs" / "traceability" / "fr-emergence-batches.md"

if not MATRIX.exists():
    sys.exit(f"MATRIX_MISSING: {MATRIX}")
if not BATCHES.exists():
    sys.exit(f"BATCHES_MISSING: {BATCHES}")

matrix = MATRIX.read_text(encoding="utf-8")
batches = BATCHES.read_text(encoding="utf-8")

# Count current EMERGENCE-100+ (batch) rows in main matrix; distinguish from
# the 16 §3.4 EMERG-001..005 + EMERGENCE-001..006 + EMERGENCE-010..013 rows.
existing_emergence_batch_rows = len(re.findall(r"FR-CIV-EMERGENCE-1\d{2}", matrix))
existing_int_rows = len(re.findall(r"FR-CIV-0100-int", matrix))

# Extract just the batch table content from batches file (everything after the first table header line).
# Batches file format: starts with "# Emergence batch rows" or similar header.
# The table content is the rows starting with `| \`FR-CIV-EMERGENCE-`.
batch_rows = []
for line in batches.splitlines():
    m = re.match(r"^\|\s*`(FR-CIV-EMERGENCE-\d+)`\s*\|", line)
    if m:
        batch_rows.append(line)

# Charter integration rows: INT-1..INT-4
charter_int_rows = [
    "| `FR-CIV-0100-int1` | charter | cross-system integration (INT-1) | planned | dormant | engine | tbd | FR-CIV-EMERGENCE LANG+POLITY cross-coupling (civ-001 + civ-002) |",
    "| `FR-CIV-0100-int2` | charter | cross-system integration (INT-2) | planned | dormant | engine | tbd | FR-CIV-EMERGENCE MARKET+ECON cross-coupling (civ-004 + civ-007) |",
    "| `FR-CIV-0100-int3` | charter | cross-system integration (INT-3) | planned | dormant | engine | tbd | FR-CIV-EMERGENCE DEMO+LEGENDS cross-coupling (civ-008 + civ-010) |",
    "| `FR-CIV-0100-int4` | charter | cross-system integration (INT-4) | planned | dormant | engine | tbd | FR-CIV-EMERGENCE PSYCHE+AI cross-coupling (civ-009 + civ-011) |",
]

# Build the new section to append
section_header = "## Section B: Emergence batch rows (§3.1 + §3.2, all currently dormant)\n\n"
section_subheaders = []
current_subcluster = None
rows_for_section = []

# Walk batch file again to preserve sub-cluster grouping
batch_with_subheaders = []
for line in batches.splitlines():
    if line.startswith("### Sub-cluster "):
        # Extract subcluster name like "civ-001 LANG family (11 rows)"
        batch_with_subheaders.append(line)
    elif line.startswith("| `FR-CIV-EMERGENCE-"):
        batch_with_subheaders.append(line)

# If 'Section B' header is already in matrix but no batch rows follow it, replace it.
if "## Section B: Emergence batch rows" in matrix and existing_emergence_batch_rows == 0:
    # Strip the existing 'Section B' placeholder section (header + everything up to next ## Section or end)
    pattern = r"## Section B:.*?(?=## |\Z)"
    matrix = re.sub(pattern, "", matrix, flags=re.DOTALL)

if existing_emergence_batch_rows == 0:
    # Append everything to the end of the matrix
    new_content = matrix.rstrip() + "\n\n---\n\n" + section_header
    # Add sub-cluster rows with their headers
    for line in batch_with_subheaders:
        if line.startswith("### Sub-cluster "):
            new_content += line + "\n"
        else:
            new_content += line + "\n"
    # Add charter integration rows
    new_content += "\n### FR-CIV-0100-int1..4 (4 rows — charter integration cross-couplings, §3.3)\n\n"
    new_content += "| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |\n"
    new_content += "|-------|--------|-------------|-------|--------|-------|------------|-------|\n"
    for row in charter_int_rows:
        new_content += row + "\n"
    MATRIX.write_text(new_content, encoding="utf-8")
    print(f"appended {len(batch_rows)} batch rows + {len(charter_int_rows)} charter integration rows")
    print(f"new total rows: {len(re.findall(r'FR-CIV-EMERGENCE-', new_content))} EMERGENCE rows + {len(re.findall(r'FR-CIV-0100-int', new_content))} INT rows")
else:
    print(f"SKIPPED: matrix already has {existing_emergence_batch_rows} EMERGENCE batch rows + {existing_int_rows} INT rows")