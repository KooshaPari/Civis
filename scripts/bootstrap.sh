#!/usr/bin/env bash
set -euo pipefail
cargo check
cargo test
cargo run -p civ-server
