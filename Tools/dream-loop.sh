#!/usr/bin/env bash
# dream-loop.sh — Research-Implementation iteration loop for Civis
#
# Usage:
#   bash TOOLS/dream-loop.sh <task-description>
#   bash TOOLS/dream-loop.sh "Add macOS CI build to build-standalone.yml"
#
# Steps:
#   1. Research: scan codebase, read docs, understand constraints
#   2. Plan: outline approach, identify files to touch
#   3. Implement: make changes
#   4. Test: run relevant tests
#   5. Verify: lint, type-check, build
#   6. Commit: stage and commit with descriptive message
#
# For use with jcode/codex/forge delegation or standalone.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TASK="${1:?Usage: dream-loop.sh <task-description>}"
TASK_HASH=$(echo "$TASK" | md5sum | cut -c1-8)
ITERATION=0

C_OK="\033[32m"; C_FAIL="\033[31m"; C_INFO="\033[36m"; C_DIM="\033[2m"; C_RST="\033[0m"
header() { printf '\n%b=== [%s] %s ===%b\n' "$C_INFO" "$1" "$2" "$C_RST"; }
ok()     { printf '%b  ✓ %b\n' "$C_OK" "$*"; }
fail()   { printf '%b  ✗ %b\n' "$C_FAIL" "$*" >&2; }
dim()    { printf '%b    %b\n' "$C_DIM" "$*"; }

# ── Phase 1: Research ────────────────────────────────────────────────
header "RESEARCH" "$TASK"
dim "Scanning codebase for related files..."

# Find files mentioning the task keywords
KEYWORDS=$(echo "$TASK" | tr '[:upper:]' '[:lower:]' | tr -cs '[:alnum:]' '\n' | sort -u | head -8)
RESEARCH_RESULTS=""
for kw in $KEYWORDS; do
    if [[ ${#kw} -gt 3 ]]; then
        matches=$(rg -l "$kw" --type rust --type yaml --type json --glob '!target/' --glob '!node_modules/' . 2>/dev/null | head -5)
        if [[ -n "$matches" ]]; then
            RESEARCH_RESULTS+="$matches"$'\n'
        fi
    fi
done
RESEARCH_RESULTS=$(echo "$RESEARCH_RESULTS" | sort -u | grep -v '^$' || true)

if [[ -n "$RESEARCH_RESULTS" ]]; then
    dim "Found $(echo "$RESEARCH_RESULTS" | wc -l | tr -d ' ') related files:"
    echo "$RESEARCH_RESULTS" | while read -r f; do
        dim "  $f"
    done
else
    dim "No directly related files found"
fi

# ── Phase 2: Plan ────────────────────────────────────────────────────
header "PLAN" "Files to modify"
dim "Review the task and determine changes."
dim "Run: rg '<keyword>' --type rust to understand current state."
dim ""

# ── Phase 3: Implement ──────────────────────────────────────────────
header "IMPLEMENT" "Make changes"
dim "Use: edit, write, or multiedit tools"
dim "Match existing code style, respect ≤500 line limit"

# ── Phase 4: Test ────────────────────────────────────────────────────
header "TEST" "Run relevant tests"
if [[ -f "Cargo.toml" ]]; then
    dim "cargo test --package <affected-crate> -q"
fi

# ── Phase 5: Verify ─────────────────────────────────────────────────
header "VERIFY" "Lint and quality checks"
if [[ -f "Cargo.toml" ]]; then
    dim "cargo clippy --all-targets -- -D warnings"
    dim "cargo fmt --check"
fi

# ── Phase 6: Commit ──────────────────────────────────────────────────
header "COMMIT" "Stage and commit"
dim "git add -A && git commit -m '<descriptive message>'"

# ── Summary ──────────────────────────────────────────────────────────
header "SUMMARY" ""
dim "Task:    $TASK"
dim "Hash:    $TASK_HASH"
dim "Repo:    $REPO_ROOT"
dim "Iter:    $ITERATION"
echo ""
dim "This is a planning scaffold. Use with jcode delegation:"
dim "  /dream-loop <task>  — then delegate phases to workers"
dim ""
dim "Full loop: research → plan → implement → test → verify → commit"
