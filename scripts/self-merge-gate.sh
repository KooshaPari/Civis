#!/usr/bin/env bash
# Phenotype self-merge gate hook for approved PR reviews.
# Called from .github/workflows/self-merge-gate.yml via phenoShared reusable workflow.
set -euo pipefail

echo "self-merge-gate: no additional Civis-specific checks configured"
exit 0
