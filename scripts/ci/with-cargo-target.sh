#!/usr/bin/env bash
# scripts/ci/with-cargo-target.sh
#
# Shared CARGO_TARGET_DIR wrapper per FR-CIV-VERIFY-010
# (civ-016-devx-verify-harness-and-worktree-hygiene).
#
# Sets `CARGO_TARGET_DIR` to a shared, fast-disk location so per-worktree
# cargo invocations share an incremental build cache. The default path is
# `E:/civis-target` on Windows (matching the convention documented in
# AGENTS.md / docs/guides/agent-smoke.md); override with `CIVIS_CARGO_TARGET_DIR`.
#
# Usage (two forms):
#
#   1. Source then run cargo:
#        source scripts/ci/with-cargo-target.sh
#        cargo build --workspace
#
#   2. Wrap a single command (sourcing in a subshell):
#        bash scripts/ci/with-cargo-target.sh cargo build --workspace
#        bash scripts/ci/with-cargo-target.sh -- cargo test -p foo
#
# Exit codes:
#   0 — env exported, command (if any) exited 0
#   non-zero — propagated from the wrapped command
#
# Notes:
#   * This script is POSIX-shell compatible (no bash-isms that would break on
#     a Linux CI runner; no pwsh-isms that would break on Windows git-bash).
#   * If `CARGO_TARGET_DIR` is already set in the parent environment, the
#     existing value wins (caller always has priority per justfile recipes).
#   * On non-Windows hosts (e.g. ubuntu-24.04 CI), the default falls back to
#     `target-shared` in the repo root if `E:/civis-target` is unreachable.

set -eu

# Determine the default target dir.
default_target_dir() {
  if [ -n "${CIVIS_CARGO_TARGET_DIR:-}" ]; then
    printf '%s' "$CIVIS_CARGO_TARGET_DIR"
    return
  fi

  # Detect Windows-style E: drive first (works under git-bash and pwsh).
  case "$(uname -s 2>/dev/null || echo unknown)" in
    MINGW*|MSYS*|CYGWIN*|Windows*)
      printf '%s' "E:/civis-target"
      ;;
    *)
      # Linux/macOS CI: keep the cache local to the workspace to avoid
      # cross-worktree lock contention on the shared fast disk.
      printf '%s' "${REPO_ROOT:-$(pwd)}/target-shared"
      ;;
  esac
}

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
export REPO_ROOT

# Caller-wins: only set CARGO_TARGET_DIR if it's not already exported.
if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  TARGET_DIR="$(default_target_dir)"
  export CARGO_TARGET_DIR="$TARGET_DIR"
fi

# Make the dir if it doesn't exist (idempotent).
mkdir -p "$CARGO_TARGET_DIR" 2>/dev/null || true

# If invoked with arguments, exec them with the env exported.
if [ "$#" -gt 0 ]; then
  # If the caller passes `--`, strip it so the inner command stays clean.
  if [ "${1:-}" = "--" ]; then
    shift
  fi
  exec "$@"
fi

# Sourced path: the env vars are now exported; print them so the parent shell
# (or CI step log) has a paper trail.
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "REPO_ROOT=$REPO_ROOT"