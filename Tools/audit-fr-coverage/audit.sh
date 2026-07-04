#!/usr/bin/env bash
#
# audit-fr-coverage.sh — regenerate the FR/NFR coverage summary
# at the bottom of docs/traceability/COVERAGE_AUDIT.md.
#
# Usage:
#   bash tools/audit-fr-coverage/audit.sh
#
# This script does NOT modify COVERAGE_AUDIT.md.  It prints the
# computed counts to stdout.  The author (or the audit regen
# workflow) is expected to paste the output back into the audit
# doc.
#
# Schema:
#   - reads fr-3d-matrix.md, fr-emergence-matrix.md, nfr-matrix.md
#   - counts rows by status
#   - reports totals
#
# Per COVERAGE_AUDIT.md follow-up #5.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TRACE_DIR="$REPO_ROOT/docs/traceability"

FR_3D="$TRACE_DIR/fr-3d-matrix.md"
FR_EMERGENCE="$TRACE_DIR/fr-emergence-matrix.md"
NFR_MATRIX="$TRACE_DIR/nfr-matrix.md"

if [[ ! -d "$TRACE_DIR" ]]; then
    echo "error: $TRACE_DIR does not exist" >&2
    exit 1
fi

count_status() {
    local file="$1"
    if [[ -f "$file" ]]; then
        # grep -c counts matching lines; we use -E for portability
        grep -cE "^\| \`?(NFR|FR)-" "$file" 2>/dev/null || echo 0
    else
        echo 0
    fi
}

echo "=== FR / NFR coverage audit (regenerated $(date -u +"%Y-%m-%dT%H:%M:%SZ")) ==="
echo

echo "Source documents:"
echo "  - $FR_3D              ($(count_status "$FR_3D") rows)"
echo "  - $FR_EMERGENCE       ($(count_status "$FR_EMERGENCE") rows)"
echo "  - $NFR_MATRIX         ($(count_status "$NFR_MATRIX") rows)"
echo

echo "Per-file status breakdown:"
for f in "$FR_3D" "$FR_EMERGENCE" "$NFR_MATRIX"; do
    if [[ -f "$f" ]]; then
        echo "  $f:"
        for status in planned dormant recovered implemented in-progress; do
            n=$(grep -cE "\\| $status \\|" "$f" 2>/dev/null || echo 0)
            echo "    $status: $n"
        done
    fi
done
echo

echo "Aggregate:"
total_rows=$(($(count_status "$FR_3D") + $(count_status "$FR_EMERGENCE") + $(count_status "$NFR_MATRIX")))
echo "  total traced rows across all 3 matrices: $total_rows"
echo

echo "Notes:"
echo "  - This script does NOT modify any file; paste the output above"
echo "    into docs/traceability/COVERAGE_AUDIT.md §3.5 to keep it in sync."
echo "  - Run from the repo root or any subdirectory; script auto-locates"
echo "    the matrices under docs/traceability/."
echo