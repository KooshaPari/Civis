#!/usr/bin/env bash
# pre-validate-bash.sh — PreToolUse hook for civ
# Validates Bash syntax before execution
set -euo pipefail

# Only check Bash tool
if [[ "${TOOL_NAME:-}" != "Bash" ]]; then
  exit 0
fi

# Get the command from INPUT
COMMAND="${TOOL_CONTENT:-}"

# Check for dangerous patterns
if echo "$COMMAND" | grep -qE 'rm\s+-rf\s+/|rm\s+-rf\s+~|#\!\s*/bin/bash.*curl.*\|\s*bash'; then
  echo "⚠️  DANGEROUS COMMAND DETECTED"
  echo "This command appears dangerous. Please review carefully."
fi

exit 0
