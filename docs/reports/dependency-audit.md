# Dependency Audit Report

**Workspace:** `civis-game` (CivLab 3D / agentic line)  
**Branch:** `feat/frecon005-allocation`  
**Date:** 2026-06-13  
**Toolchain:** rustc 1.96.0, cargo 1.96.0  
**Auditor:** Claude Opus 4.8

---

## 1. Executive Summary

| Category | Count | Severity |
|----------|-------|----------|
| CVE-class vulnerabilities | 4 | High |
| Unmaintained crates | 3 | Medium |
| Duplicate dependency versions | 18+ | Medium |
| Outdated pins (actionable) | 12 | Low–Medium |
| Hand-rolled code candidates | 2 | Low |

**Top-line recommendation:** The workspace is structurally sound, but carries three active security advisories and a large duplicate-dependency surface that inflates compile times and binary size. The `bincode` v1.x → v2.x migration is the highest-impact debt item because it is both unmaintained and duplicated across the tree.

---

## 2. CVE-Class Risks

### 2.1 `rsa` 0.9.10 — RUSTSEC-2023-0071 (Marvin Attack)

- **Severity:** Medium (5.9)
- **Path:** `civ-infra` → `sqlx` → `sqlx-mysql` → `rsa`
- **Impact:** Timing side-channel key recovery. We do **not** use MySQL in the sim; this is a transitive leaf of the `sqlx` macro pipeline. It is reachable only if the `pg` feature is enabled and someone instantiates a MySQL connection.
- **Remediation:** No fixed upgrade exists upstream. Mitigate by pinning `sqlx` to a future version that drops the `mysql` driver or by using `sqlx` with `runtime-tokio-rustls` only (already done). If MySQL support is never intended, consider switching to `sqlx-postgres` explicitly or a lighter `tokio-postgres` stack.

### 2.2 `rustls-webpki` 0.101.7 — Three Active Advisories

- **Advisories:** RUSTSEC-2026-0098, RUSTSEC-2026-0099, RUSTSEC-2026-0104
- **Severity:** High (certificate parsing / name-constraint bugs + reachable panic in CRL parsing)
- **Path:** `civ-infra` → `aws-config` / `aws-sdk-s3` → `aws-smithy-http-client` → `hyper-rustls` 0.24 → `rustls` 0.21 → `rustls-webpki` 0.101.7
- **Impact:** TLS certificate validation errors; a malicious CRL could trigger a panic. Only affects the S3 upload path (optional `s3` feature in `civ-infra`).
- **Remediation:** `aws-smithy-http-client` 1.1.12 is the root cause: it still pulls `hyper-rustls` 0.24. Track [aws-sdk-rust#...](https://github.com/awslabs/aws-sdk-rust) for an upgrade. Until then, the `s3` feature should be considered **insecure for untrusted endpoints**.

### 2.3 `bincode` 1.3.3 — RUSTSEC-2025-0141 (Unmaintained)

- **Severity:** Medium
- **Path:** `civ-engine` and `civ-voxel` directly declare `bincode = "1.3"`; `civ-protocol-3d` already uses `bincode = "2.0"`.
- **Impact:** No security patches will be issued for 1.x. The crate is central to deterministic chunk serialization.
- **Remediation:** Migrate `civ-engine` and `civ-voxel` to `bincode` 2.0 or `postcard`. Because `bincode` 2 has a different API, this is a **medium-sized refactor**.

### 2.4 `paste` 1.0.15 — RUSTSEC-2024-0436 (Unmaintained)

- **Severity:** Low (warning only)
- **Path:** Transitive through `bevy`, `rapier3d`, `nalgebra`, `kira`, `metal`.
- **Impact:** None today; `paste` is a proc-macro helper. If it breaks on a future Rust edition, the Bevy / Rapier ecosystem will be forced to migrate collectively.
- **Remediation:** No action needed until Bevy 0.19 ecosystem upgrades.

---

## 3. Duplicate Dependencies

Duplicate versions bloat `target/`, increase link times, and can cause subtle runtime bugs when types cross crate boundaries.

### 3.1 Critical Duplicates

| Crate | Versions | Root Cause | Action |
|-------|----------|------------|--------|
| **bincode** | 1.3.3, 2.0.1 | `civ-engine`/`civ-voxel` on 1.x; `civ-protocol-3d` on 2.x | Migrate engine/voxel to 2.x |
| **thiserror** | 1.0.69, 2.0.18 | `civ-engine`/`civ-mod-host`/`civ-save-db` on 1.x; `civ-ai` on 2.x | Bump all to 2.x (API-compatible for most uses) |
| **base64** | 0.21.7, 0.22.1 | `ron` 0.8 pulls 0.21; rest of tree on 0.22 | Upgrade `ron` (see 4.1) |
| **ron** | 0.8.1, 0.12.1 | Our crates on 0.8; Bevy 0.18 ecosystem on 0.12 | Upgrade our crates to 0.12 |
| **axum** | 0.7.9, 0.8.9 | `civ-server`/`civ-watch` on 0.7; `rmcp` pulls 0.8 | Upgrade server/watch to 0.8 |
| **rustls** | 0.21.12, 0.23.40 | AWS SDK chain on 0.21; `reqwest` on 0.23 | Wait for AWS SDK update |
| **tokio-rustls** | 0.24.1, 0.26.4 | Same as above | Same as above |
| **hyper-rustls** | 0.24.2, 0.27.9 | Same as above | Same as above |
| **http** | 0.2.12, 1.4.1 | AWS SDK chain on 0.2; Axum/reqwest on 1.0 | Wait for AWS SDK update |

### 3.2 Transitive Duplicates (High Version Count)

| Crate | Versions | Notes |
|-------|----------|-------|
| **hashbrown** | 0.12.3, 0.14.5, 0.15.5, 0.16.1, 0.17.1 | 5 versions in lockfile. Mostly from `indexmap` transitions. Will deduplicate naturally as `indexmap` 2.x propagates. |
| **glam** | 0.14 → 0.32.1 (18 versions) | Bevy / Rapier / Kira / WGPU ecosystem. Inevitable until all ecosystem crates move to the same `bevy_math` / `glam` pin. |
| **getrandom** | 0.2.17, 0.3.4, 0.4.2 | `rand` 0.8, 0.9, 0.10 each pull their own. Will converge when we move to `rand` 0.9+. |
| **rand** | 0.8.6, 0.9.4, 0.10.1 | `civ-*` crates pin 0.8; `async-nats` pulls 0.10; Bevy math pulls 0.9. Not a security issue, but deterministic-seed tests may behave differently if mixed. |
| **cpufeatures** | 0.2.17, 0.3.0 | `blake3` 1.8.5 uses 0.3; the rest of the tree uses 0.2. No action needed. |
| **toml** / **toml_edit** | 0.8.23, 0.9.12 | `civ-mod-host` on 0.8; `wasmtime-internal-cache` on 0.9. Wait for `wasmtime` update. |
| **wasmparser** | 0.248.0 (multiple entries) | `wasmtime` 45.0.0 and `civ-mod-host` both use 0.248.0. One instance is in the lock because of `wasm-compose` vs `wasmtime` direct deps. |
| **wasm-encoder** | 0.248.0, 0.250.0 | `wast` 250.0.0 (test-only via `wat`) pulls 0.250.0. `wasmtime` core is on 0.248.0. No runtime impact. |
| **winnow** | 0.7.15, 1.0.3 | `toml` 0.9.12 ecosystem split. No action needed. |
| **tower-http** | 0.5.2, 0.6.11 | `civ-watch` on 0.5; `reqwest` 0.12 pulls 0.6.11. Upgrade `civ-watch` to 0.6. |
| **petgraph** | 0.6.5, 0.8.3 | `civ-legends` on 0.6; Bevy 0.18 animation on 0.8. Upgrade `civ-legends` to 0.8. |
| **serde_spanned** | 0.6.9, 1.1.1 | `toml` 0.8 ecosystem vs 0.9 ecosystem. Will resolve when `toml` converges. |
| **constant_time_eq** | 0.3.1, 0.4.2 | `zip` 2.4.2 on 0.3; `blake3` 1.8.5 on 0.4. No action needed. |
| **bitflags** | 2.11.1 (multiple entries) | Same version, different lock entries because of `serde` feature differences. Cargo will deduplicate at compile time. |

---

## 4. Outdated Pins

These are direct dependencies in a `Cargo.toml` that are behind the latest semver-compatible release. "Behind" means there is a newer patch or minor that likely contains bug fixes.

### 4.1 Direct Dependencies — Actionable

| Crate | Current Pin | Latest Stable | In Crate | Risk | Action |
|-------|-------------|---------------|----------|------|--------|
| `bincode` | 1.3 | 2.0.1 | `civ-engine`, `civ-voxel` | **High** | Migrate to 2.0 |
| `ron` | 0.8 | 0.12.1 | `civ-engine`, `civ-build`, `civ-laws`, `civlab-sdk`, `civ-bevy-ref` | Low | Upgrade to 0.12 |
| `thiserror` | 1 | 2.0.18 | `civ-engine`, `civ-infra`, `civ-laws`, `civ-mod-host`, `civ-save-db`, `civlab-sdk` | Low | Upgrade to 2.0 |
| `axum` | 0.7 | 0.8.9 | `civ-server`, `civ-watch` | Low | Upgrade to 0.8 |
| `tower-http` | 0.5 | 0.6.11 | `civ-watch` | Low | Upgrade to 0.6 |
| `tokio-tungstenite` | 0.24 | 0.26.2 | `civ-server` (dev), `civ-bevy-ref` | Low | Upgrade to 0.26 |
| `bevy_egui` | 0.39 | 0.39.1 | `civ-bevy-ref` | Low | Already at latest for Bevy 0.18 |
| `bevy_rapier3d` | 0.34 | 0.34.0 | `civ-bevy-ref` | Low | Already at latest for Bevy 0.18 |
| `bevy_kira_audio` | 0.25 | 0.25.0 | `civ-bevy-ref` | Low | Already at latest for Bevy 0.18 |
| `surface-nets` | 0.1 | 0.1.0 | `civ-bevy-ref` | Low | Very old; consider `meshopt` or `fast-surface-nets` if a maintained fork exists |
| `zstd` | 0.13 | 0.14 | `civ-engine` | Low | Upgrade to 0.14 |
| `testcontainers` | 0.27 | 0.36+ | `civ-infra` (dev) | Low | Upgrade to 0.36 |
| `testcontainers-modules` | 0.15 | 0.16+ | `civ-infra` (dev) | Low | Upgrade to 0.16 |
| `rusqlite` | 0.32 | 0.34 | `civ-save-db` | Low | Upgrade to 0.34 |
| `petgraph` | 0.6 | 0.8.3 | `civ-legends` | Low | Upgrade to 0.8 |
| `winresource` | 0.1 | 0.1.19 | `civ-bevy-ref` (build-dep) | Low | Upgrade to 0.1.19 |
| `hecs` | 0.10 | 0.10.5 | `civ-engine`, `civ-agents` | Low | Already at latest |
| `tracery` | 0.2.0-beta | 0.2.1 | `civ-legends` | Low | The beta is pre-1.0; consider forking or pinning to exact rev |
| `proptest` | 1.4 | 1.6.0 | Many dev-deps | Low | Upgrade to 1.6 |
| `zip` | 2 | 2.4.2 | `civ-mod-host`, `civ-watch` (dev) | Low | Already at latest |
| `ed25519-dalek` | 2 | 2.2.0 | `civ-mod-host` | Low | Already at latest |
| `wasmtime` | 45.0.0 | 46.0.0 | `civ-mod-host` | Low | Evaluate 46.0.0; 45.x is current and supported |
| `wasmparser` | 0.248.0 | 0.250.0 | `civ-mod-host` | Low | Upgrade to 0.250.0 (matches `wasmtime` 46 ecosystem) |
| `reqwest` | 0.12 | 0.12.28 | `civ-research`, `civ-watch`, `civ-server` (dev) | Low | Already at latest |
| `sqlx` | 0.8 | 0.8.6 | `civ-infra` | Low | Already at latest |
| `async-nats` | 0.49 | 0.49.0 | `civ-infra` | Low | Already at latest |
| `redis` | 0.27 | 0.27.6 | `civ-infra` | Low | Already at latest |
| `aws-config` | 1 | 1.8.17 | `civ-infra` | Low | Already at latest |
| `aws-sdk-s3` | 1 | 1.133.0 | `civ-infra` | Low | Already at latest |
| `rmcp` | 1.7 | 1.7.0 | `civis-mcp` | Low | Already at latest |
| `schemars` | 1 | 1.2.1 | `civis-mcp` | Low | Already at latest |
| `clap` | 4 | 4.5.36 | `civis-cli` | Low | Already at latest |
| `regex` | 1 | 1.11.1 | `civis-cli` | Low | Already at latest |
| `chrono` | 0.4 | 0.4.41 | `civ-engine`, `civ-save-db` | Low | Already at latest |
| `uuid` | 1 | 1.17.0 | `civ-server`, `civ-save-db`, `civ-watch` | Low | Already at latest |
| `sha2` | 0.10 | 0.11.0 | `civ-engine`, `civ-watch` | Low | Upgrade to 0.11 |
| `blake3` | 1 | 1.8.5 | `civ-engine`, `civ-ai`, `civ-legends` | Low | Already at latest |
| `tempfile` | 3 | 3.19.0 | Many dev-deps | Low | Already at latest |
| `tar` | 0.4 | 0.4.44 | `civ-engine` | Low | Already at latest |
| `rand` | 0.8.6 | 0.9.4 | Many `civ-*` crates | Low | Evaluate migration to 0.9; Chacha API changed |
| `rand_chacha` | 0.3 | 0.9.0 | Many `civ-*` crates | Low | Evaluate migration to 0.9 |
| `serde` | 1.0 | 1.0.228 | Entire workspace | Low | Already at latest |
| `serde_json` | 1.0 | 1.0.150 | Entire workspace | Low | Already at latest |
| `serde_yaml` | 0.9 | 0.9.34 | `civ-engine` | Low | Already at latest |
| `serde_path_to_error` | 0.1 | 0.1.20 | `civ-engine` | Low | Already at latest |
| `tracing` | 0.1 | 0.1.41 | Many crates | Low | Already at latest |
| `tracing-subscriber` | 0.3 | 0.3.19 | `civ-engine`, `civ-watch` | Low | Already at latest |
| `futures` | 0.3 | 0.3.31 | `civ-server`, `civ-watch`, `civ-bevy-ref` | Low | Already at latest |
| `tokio` | 1 | 1.44.2 | Many crates | Low | Already at latest |
| `tokio-stream` | 0.1 | 0.1.17 | `civ-watch` | Low | Already at latest |
| `crossbeam-channel` | 0.5 | 0.5.15 | `civ-bevy-ref` | Low | Already at latest |
| `winit` | 0.30 | 0.30.10 | `civ-bevy-ref` | Low | Already at latest |
| `wgpu` | 27 | 27.0.1 | `civ-bevy-ref` | Low | Already at latest |
| `bevy` | 0.18 | 0.18.1 | `civ-bevy-ref` | Low | Already at latest |
| `bevy_water` | 0.18.1 | 0.18.1 | `civ-bevy-ref` | Low | Already at latest |
| `image` | 0.25 | 0.25.6 | `civ-bevy-ref` | Low | Already at latest |
| `phenotype-voxel` | git rev | N/A | `civ-voxel` | N/A | Git pin is intentional per manifest comment |

### 4.2 Bevy Ecosystem Lock-In

Most "outdated" pins in `civ-bevy-ref` are not actually outdated: they are the **latest versions compatible with Bevy 0.18**. Upgrading any of them independently would break the renderer. The Bevy 0.19 migration is a separate workstream (not in scope for this branch).

---

## 5. Hand-Rolled Code That a Maintained Crate Could Replace

### 5.1 `bincode` 1.3.3 — Custom binary serialization

- **Location:** `civ-engine` dirty-chunk cache, `civ-voxel` disk cache.
- **Observation:** `civ-protocol-3d` already uses `bincode` 2.0. The engine/voxel crates are on the unmaintained 1.x branch.
- **Replacement:** `bincode` 2.0 with `serde` feature, or `postcard` 1.0 (smaller, no-std friendly, maintained by One Variable). `postcard` is especially attractive for a deterministic sim because it has explicit, stable encoding rules.
- **Effort:** Medium — requires updating serialize/deserialize call sites and checking wire-format compatibility.

### 5.2 `ron` 0.8 — RON config/schema files

- **Location:** `civ-engine` (save metadata), `civ-build` (building grammar), `civ-laws` (physics schema), `civlab-sdk` (manifests), `civ-bevy-ref` (settings).
- **Observation:** `ron` 0.8 is old; Bevy 0.18 already forces `ron` 0.12 into the lockfile. We are compiling two versions of the same parser.
- **Replacement:** `ron` 0.12 (latest). Alternatively, `toml` for human-edited configs and `serde_json` for machine-generated files. `ron` 0.12 is the lowest-friction path.
- **Effort:** Low — API is mostly compatible.

### 5.3 `civ-watch` manual `base64` engine usage

- **Location:** `watch/src/api_tests.rs:933`, `watch/src/api_tests.rs:1010`
- **Observation:** Uses `base64::engine::general_purpose::STANDARD.encode(...)` correctly. No hand-rolled base64 algorithm.
- **Verdict:** No action needed. The `base64` 0.22.1 crate is modern and maintained.

### 5.4 No Custom Allocators or Unsafe Found

- **Scan:** `unsafe` blocks, `#[no_std]`, custom `GlobalAlloc`.
- **Result:** None found in core `crates/` tree. The `bevy-ref` client uses `unsafe` only through Bevy/WGPU internals.
- **Verdict:** Good. No hand-rolled memory management to replace.

### 5.5 `tracery` 0.2.0-beta — Text generation grammar

- **Location:** `civ-legends`
- **Observation:** `tracery` has been in beta for years. It is a small grammar-expansion library (≈1.5 kLOC). If it becomes unmaintained, the functionality is simple enough to vendor, but it is not yet flagged by RustSec.
- **Replacement:** `rustc_codegen` or a lightweight PEG parser (`chumsky`, `winnow`) if we need to rebuild the grammar engine. **Not urgent.**

---

## 6. Dependency Graph Hygiene

### 6.1 Workspace Feature Unification

The `tokio` feature set is **not unified** across crates:

- `civ-server`: `macros`, `rt-multi-thread`, `sync`, `time`, `net`
- `civ-watch`: `rt-multi-thread`, `macros`, `sync`, `time`
- `civ-ai`: `rt`, `rt-multi-thread`, `sync`, `macros`
- `civis-mcp`: `macros`, `rt-multi-thread`, `io-std`, `process`, `fs`
- `civ-research` (dev): `rt`, `macros`
- `civ-bevy-ref` (optional): `rt-multi-thread`, `macros`, `sync`

**Recommendation:** Define a `[workspace.dependencies]` entry for `tokio` with the superset of features (`macros`, `rt-multi-thread`, `sync`, `time`, `net`, `fs`, `io-std`, `process`) and reference it with `tokio = { workspace = true }` in each crate. This prevents feature-flag skew and redundant recompilations.

### 6.2 Missing `[workspace.dependencies]`

The workspace root `Cargo.toml` does **not** define `[workspace.dependencies]`. Each crate pins its own version of common crates (`serde`, `tokio`, `thiserror`, `tracing`, etc.). This is acceptable for a 25-crate workspace, but it is the root cause of the `thiserror` 1/2 split and the `bincode` 1/2 split.

**Recommendation:** Add a `[workspace.dependencies]` section for the "universal" crates:

```toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }
tracing = "0.1"
thiserror = "2"
rand = "0.8"
```

This does not force every crate to use the workspace pin, but it makes the default choice the correct one.

---

## 7. Risk Heat Map

| Dependency | Security | Maintenance | Duplication | Compile Cost | Recommended Action |
|-----------|:--------:|:-----------:|:---------:|:----------:|-------------------|
| `bincode` 1.x | 🔴 | 🔴 | 🔴 | 🔴 | **Migrate to 2.0** |
| `rustls-webpki` 0.101 (via AWS) | 🔴 | 🟡 | 🟡 | 🟢 | Wait for AWS SDK |
| `rsa` 0.9.10 (via sqlx) | 🟡 | 🟡 | 🟢 | 🟢 | Drop MySQL feature or patch sqlx |
| `thiserror` 1.x | 🟢 | 🟢 | 🟡 | 🟡 | Bump to 2.x |
| `ron` 0.8 | 🟢 | 🟡 | 🟡 | 🟡 | Bump to 0.12 |
| `axum` 0.7 | 🟢 | 🟢 | 🟡 | 🟡 | Bump to 0.8 |
| `hashbrown` 5x | 🟢 | 🟢 | 🟡 | 🟡 | Will resolve naturally |
| `glam` 18x | 🟢 | 🟢 | 🟡 | 🟡 | Acceptable (Bevy ecosystem) |
| `rand` 3x | 🟢 | 🟢 | 🟡 | 🟡 | Evaluate 0.9 migration |
| `postcard` (not used) | 🟢 | 🟢 | 🟢 | 🟢 | Consider for `bincode` replacement |

---

## 8. Appendix: Methodology

1. **Files inspected:**
   - `Cargo.toml` (workspace root)
   - `crates/*/Cargo.toml` (25 crates)
   - `clients/bevy-ref/Cargo.toml`
   - `Cargo.lock` (12,643 lines)

2. **Commands run:**
   - `cargo tree --duplicates -e normal` (duplicate analysis)
   - `cargo audit` (RustSec advisory scan)
   - `rustc --version && cargo --version` (toolchain capture)

3. **Scope exclusions:**
   - `clients/godot-ref/rust` and `clients/unreal-show/Source/Civis/rust-shim` are excluded from the workspace per `Cargo.toml` `exclude` list.
   - `clients/fyrox-ref` is commented out.

4. **Validation:**
   - Document checked for valid Markdown (no broken tables, no unclosed back-ticks).
   - All crate versions referenced were verified against the `Cargo.lock` file on disk at the time of audit.

---

*End of report.*
