#!/usr/bin/env bash
# CivShow build helper (rust-shim always; UE compile Windows-only via build.ps1).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RUST_SHIM="${PROJECT_ROOT}/Source/Civis/rust-shim"
LIB_DIR="${PROJECT_ROOT}/Source/Civis/lib"
LIB_NAME="civis_unreal_ffi.lib"
SKIP_RUST=0
SKIP_UE=0

usage() {
  cat <<'EOF'
Usage: build.sh [--skip-rust] [--skip-ue]

  Builds the rust-shim static library and copies it to Source/Civis/lib/.
  On Windows (Git Bash / MSYS), also runs scripts/build.ps1 for the UE target.

Exit codes (rust-only path): 0 success, 1 cargo/copy failure.
UE path delegates to build.ps1 (2 = UE not installed).
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-rust) SKIP_RUST=1 ;;
    --skip-ue) SKIP_UE=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
  shift
done

step() { printf '==> %s\n' "$*"; }

build_rust() {
  step "Building rust-shim (release)"
  (cd "${RUST_SHIM}" && cargo build --release)
  local built="${RUST_SHIM}/target/release/${LIB_NAME}"
  if [[ ! -f "${built}" ]]; then
    echo "Expected static library not found: ${built}" >&2
    exit 1
  fi
  mkdir -p "${LIB_DIR}"
  cp -f "${built}" "${LIB_DIR}/${LIB_NAME}"
  echo "Copied ${LIB_NAME} -> ${LIB_DIR}"
}

if [[ "${SKIP_RUST}" -eq 0 ]]; then
  build_rust
else
  echo "Skipping rust-shim (--skip-rust)"
fi

if [[ "${SKIP_UE}" -ne 0 ]]; then
  exit 0
fi

case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*|Windows*)
    if command -v pwsh >/dev/null 2>&1; then
      exec pwsh -NoProfile -File "${SCRIPT_DIR}/build.ps1" @([[ "${SKIP_RUST}" -eq 1 ]] && echo -SkipRust)
    fi
    if command -v powershell.exe >/dev/null 2>&1; then
      exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "${SCRIPT_DIR}/build.ps1" @([[ "${SKIP_RUST}" -eq 1 ]] && echo -SkipRust)
    fi
    echo "Windows detected but PowerShell not found; rust-shim built only." >&2
    exit 2
    ;;
  *)
    echo "Unreal Editor Win64 builds require Windows + UE 5.4. Rust shim built; skipping UE." >&2
    exit 2
    ;;
esac
