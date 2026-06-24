# Non-Functional Requirements — CivLab / Civis

**Version:** 1.0
**Status:** Draft
**Date:** 2026-05-29
**Traces to:** PRD.md v1.0; FUNCTIONAL_REQUIREMENTS.md v1.0
**Scope:** Headless simulation engine (`crates/engine`), Bevy desktop client (`clients/`), mod host (`civ-mod-host`)

---

## Overview

This document specifies non-functional requirements (NFRs) for the CivLab/Civis codebase. Each NFR carries:

- **Statement** — the requirement in SHALL/SHOULD language
- **Rationale** — why the constraint exists
- **Measurable Target** — the concrete, testable threshold
- **Verification Method** — how compliance is confirmed
- **Constrained FRs** — FR-CIV-* identifiers whose correctness depends on this NFR being met

NFRs are stable identifiers; once assigned, IDs are never reused.

---

## Category: PERFORMANCE

### NFR-CIV-PERF-001 — Bevy Client Frame Rate (RTX 3090 Ti / DX12)

**Statement:** The Bevy desktop client SHALL sustain a minimum of 60 frames per second at 1440p resolution with 1,000 simultaneously visible entities rendered in the scene graph.

**Rationale:** FR-CLIENT-001 accepts criteria require 60 FPS with 1,000 entities; the RTX 3090 Ti (DX12 Ultimate) is the primary development and showcase target per the project hardware profile.

**Measurable Target:** P95 frame time ≤ 16.67 ms; P99 ≤ 20 ms; zero frames below 30 FPS sustained over a 60-second benchmark window at 2560×1440 with 1,000 active entities.

**Verification Method:** Automated Bevy criterion benchmark `benches/client_render_1k_entities` running on CI against the RTX 3090 Ti build target; results stored as benchmark artifact and regressed against baseline.

**Constrained FRs:** FR-CLIENT-001, FR-PROTO-004

---

### NFR-CIV-PERF-002 — Bevy Client Frame Rate (Apple M1 / Metal)

**Statement:** The Bevy desktop client SHALL sustain a minimum of 60 frames per second at 1440p resolution with 500 simultaneously visible entities on Apple M1 using the Metal backend.

**Rationale:** The project targets both the Ryzen/RTX desktop and the M1 MacBook as primary development hardware; Metal is the only GPU API available on macOS and has different bottlenecks than DX12.

**Measurable Target:** P95 frame time ≤ 16.67 ms; P99 ≤ 22 ms; no sustained dip below 45 FPS over a 60-second window at 2560×1440 with 500 active entities on M1.

**Verification Method:** CI macOS runner executing `benches/client_render_500_entities_metal`; manual validation checkpoint during M1 release sign-off.

**Constrained FRs:** FR-CLIENT-001, NFR-CIV-PORT-001

---

### NFR-CIV-PERF-003 — Simulation Tick Budget (200 Civilians, Nominal)

**Statement:** The engine simulation tick SHALL complete all phases (policy, deterministic transition, stochastic event, market clearing, allocation) within a 10 ms wall-clock budget at a load of 200 civilian agents on the reference CI hardware (4-core / 16 GB RAM).

**Rationale:** FR-CORE-007 requires a 14 ms P99 target on commodity hardware; a 10 ms target at 200 civilians provides headroom for I/O, command queue flushing, and network serialization before the 16 ms hard warning threshold.

**Measurable Target:** P99 tick duration ≤ 10 ms at 200 civilian agents, measured over 1,000 consecutive ticks in headless mode with no connected clients.

**Verification Method:** `cargo bench --bench tick_budget_200_civilians`; CI gate blocks merge if P99 regresses beyond 10 ms.

**Constrained FRs:** FR-CORE-001, FR-CORE-007, FR-ECON-003, FR-ECON-005

---

### NFR-CIV-PERF-004 — Simulation Tick Budget Scaling Target

**Statement:** The engine tick budget SHALL scale sub-linearly with agent count such that the P99 tick duration at 10,000 agents does not exceed 80 ms and the P99 tick duration at 100,000 agents does not exceed 500 ms (both in headless mode on the RTX 3090 Ti host CPU).

**Rationale:** The v1/v2 scalability roadmap (PRD.md) targets 1,000,000 agents; establishing measurable scaling checkpoints at 10 k and 100 k ensures architectural decisions (SVO chunking, parallel ECS queries) remain on the correct trajectory.

**Measurable Target:**
- 10,000 agents → P99 tick ≤ 80 ms
- 100,000 agents → P99 tick ≤ 500 ms
- Slope from 200 → 10,000 agents must be O(n log n) or better as measured by linear regression on benchmark series.

**Verification Method:** `cargo bench --bench tick_scaling_series` emitting a three-point (200, 10k, 100k) regression; CI plot artifact reviewed at each roadmap milestone.

**Constrained FRs:** FR-CORE-001, FR-CORE-007, FR-ECON-005, NFR-CIV-SCALE-001

---

### NFR-CIV-PERF-005 — Mesh Generation / Terrain Budget

**Statement:** Procedural terrain mesh generation for a 256×256 chunk SHALL complete within 50 ms on the host CPU thread, permitting background streaming without hitching the main render thread.

**Rationale:** Smooth camera navigation over large maps requires chunk generation to finish within two to three frame intervals so the render thread never stalls waiting for geometry.

**Measurable Target:** Worst-case mesh generation time for a single 256×256 chunk ≤ 50 ms at P99 in `benches/terrain_mesh_gen`.

**Verification Method:** Criterion benchmark `benches/terrain_mesh_gen_256`; manually validated via Bevy frame-time overlay with a flying camera that crosses chunk boundaries.

**Constrained FRs:** FR-CLIENT-001, NFR-CIV-PERF-001

---

### NFR-CIV-PERF-006 — Process Memory Ceiling

**Statement:** The combined engine + Bevy client process SHALL consume no more than 4 GB of resident RAM at steady state with 10,000 entities loaded on a 2,048×2,048 cell map.

**Rationale:** PRD.md targets ≤ 2 GB for 1,000,000 agents; a 4 GB ceiling at 10,000 entities on the present codebase establishes an early guard against allocator regressions and unbounded ECS archetype fragmentation.

**Measurable Target:** Peak RSS ≤ 4,096 MB after 500 ticks with 10,000 entities, measured by a harness that samples `/proc/self/status` (Linux) or task_info (macOS).

**Verification Method:** `cargo test --test memory_ceiling_10k_entities`; the test fails if peak RSS exceeds the threshold.

**Constrained FRs:** FR-CORE-002, FR-CLIENT-001, NFR-CIV-SCALE-002

---

### NFR-CIV-PERF-007 — Network Bandwidth Ceiling

**Statement:** The simulation server SHALL NOT consume more than 10 Mbps aggregate outbound bandwidth when serving 10 simultaneous game clients receiving binary delta frames at 60 ticks per second.

**Rationale:** FR-PROTO-004 acceptance criterion sets the same bound; this NFR makes it formally testable and tied to CI.

**Measurable Target:** Aggregate outbound bandwidth ≤ 10 Mbps sustained over a 60-second load test with 10 WebSocket clients.

**Verification Method:** Integration test `tests/proto/bandwidth_10_clients` using a loopback network and byte counter interceptor; CI gate.

**Constrained FRs:** FR-PROTO-004, FR-PROTO-005

---

## Category: DETERMINISM

### NFR-CIV-DET-001 — Cross-Run Bit-Identical Replay (Same Platform)

**Statement:** Two headless runs started from the same seed, scenario YAML, and tick count SHALL produce bit-identical world-state hashes at every tick, verified by the replay harness.

**Rationale:** Deterministic replay is the project's core invariant (PRD.md "Determinism & Auditability"; FR-CORE-003; FR-REPLAY-002). Without this guarantee, the .civreplay audit trail and research reproducibility are meaningless.

**Measurable Target:** 100% of ticks in a replay diverge from the original run by zero bits (hash equality). Test suite: 50-tick scenario run twice; all 50 hashes must match.

**Verification Method:** CI mandatory gate `cargo test --test determinism_replay_same_platform`; blocks merge on any failure. Runs on every commit per FR-CORE-003.

**Constrained FRs:** FR-CORE-003, FR-CORE-004, FR-REPLAY-001, FR-REPLAY-002, FR-METRICS-003

---

### NFR-CIV-DET-002 — Cross-Platform Bit-Identical Replay

**Statement:** A .civreplay file produced on Windows (DX12 host CPU) SHALL replay to identical state hashes on macOS (M1 Metal host CPU) and Linux (x86-64 Vulkan host CPU).

**Rationale:** Multi-platform determinism is required for research reproducibility (researchers on M1 laptops must reproduce results from Linux CI servers) and for the multi-client architecture where different clients may run on different OS/hardware.

**Measurable Target:** Cross-platform hash equality at 100% of ticks for the canonical `starting_settlement.yaml` 50-tick scenario, executed as a matrix CI job across Windows, macOS, and Linux runners.

**Verification Method:** CI matrix job `determinism_cross_platform` comparing SHA-256 of per-tick state snapshots exported via `--replay-hash-file`; all three platform outputs must be identical.

**Constrained FRs:** FR-CORE-003, FR-REPLAY-002, FR-METRICS-003, NFR-CIV-PORT-001

---

### NFR-CIV-DET-003 — Fixed-Point Arithmetic Enforcement

**Statement:** All state-mutating simulation paths (production, allocation, market clearing, taxation, metrics computation) SHALL use the project's `Fixed` type (i64 scaled by 10^6) exclusively; floating-point types SHALL NOT appear in any state-mutation code path.

**Rationale:** FR-CORE-003 bans f32/f64 in state-mutating paths and requires type-system enforcement; FR-METRICS-003 extends this to metrics. Floating-point results are non-deterministic across platforms due to FMA contraction, denormal flushing, and compiler reordering.

**Measurable Target:** Zero occurrences of `f32` or `f64` in state-mutation modules (`crates/engine/src/{production,allocation,market,taxation,metrics}.rs`), verified statically. Fixed-point and float paths agree to within 10^-6 for identical inputs.

**Verification Method:** Clippy lint rule `no_float_in_sim_core` (custom lint or `#[deny(clippy::float_arithmetic)]` scope); property test `tests/fixed_point_float_agreement` asserting convergence to 6 decimal places.

**Constrained FRs:** FR-CORE-003, FR-METRICS-002, FR-METRICS-003

---

### NFR-CIV-DET-004 — Seeded RNG Logging Completeness

**Statement:** Every draw from the simulation's ChaCha20Rng instance SHALL emit a `rng_draw` event to the event log before the draw result is consumed by any state-mutation code.

**Rationale:** FR-CORE-004 requires all RNG draws to be logged; if any draw is silent, the event log is incomplete and replay from that log will diverge.

**Measurable Target:** In a 1,000-tick stress run, the count of RNG draws (measured by instrumenting the wrapper type) must equal the count of `rng_draw` events in the event log. Discrepancy = 0.

**Verification Method:** Integration test `tests/rng_draw_log_completeness` comparing draw counter to event log `rng_draw` count after 1,000 ticks.

**Constrained FRs:** FR-CORE-004, FR-REPLAY-001, NFR-CIV-DET-001

---

## Category: SCALABILITY

### NFR-CIV-SCALE-001 — Agent Count Roadmap Gates

**Statement:** The engine architecture SHALL support agent populations at three roadmap tiers without requiring a rewrite of the ECS query or tick-loop structure:

- **Tier 1 (MVP):** 200 civilians — tick budget ≤ 10 ms (see NFR-CIV-PERF-003)
- **Tier 2 (v1):** 10,000 civilians — tick budget ≤ 80 ms (see NFR-CIV-PERF-004)
- **Tier 3 (v2):** 100,000 civilians — tick budget ≤ 500 ms (see NFR-CIV-PERF-004)

**Rationale:** PRD.md targets 1,000,000 agents long-term; the three-tier gate structure prevents architectural shortcuts at MVP that would block v1 scale.

**Measurable Target:** Each tier passes its tick-budget criterion benchmark before the corresponding milestone tag is cut.

**Verification Method:** `cargo bench --bench tick_scaling_series` reporting all three tier results; milestone release checklist requires all three passing.

**Constrained FRs:** FR-CORE-001, FR-CORE-002, FR-CORE-007, NFR-CIV-PERF-003, NFR-CIV-PERF-004

---

### NFR-CIV-SCALE-002 — Chunk Streaming Throughput

**Statement:** The terrain streaming subsystem SHALL load and unload 64×64-cell chunks at a rate sufficient to keep all chunks within a 3-chunk radius of the camera resident without hitching the render thread, at a camera traversal speed of 100 cells per second.

**Rationale:** Smooth strategic-view navigation across large maps (10,000×10,000 cells per PRD.md) requires background chunk streaming with a guaranteed minimum throughput.

**Measurable Target:** Zero render-thread stalls (frame time spikes > 2× median) attributable to chunk I/O during a 30-second fly-through at 100 cells/second over a 512×512 pre-generated map.

**Verification Method:** Manual benchmark using Bevy's built-in frame time overlay plus automated headless pass that records per-frame durations and flags spikes.

**Constrained FRs:** FR-CLIENT-001, NFR-CIV-PERF-005

---

### NFR-CIV-SCALE-003 — Simultaneous Client Connections

**Statement:** The WebSocket server SHALL handle at least 10 simultaneous client connections without degradation in tick processing latency or client delta delivery latency.

**Rationale:** PRD.md v1 success criterion requires ≥ 10 simultaneous clients; FR-PROTO-001 sets the same bound.

**Measurable Target:** With 10 connected clients each receiving binary delta frames: P99 tick processing time does not regress beyond the baseline established with 0 clients by more than 2 ms; P99 client delta delivery latency ≤ 50 ms (loopback).

**Verification Method:** Integration test `tests/proto/10_client_load` spinning up 10 simulated clients, running 500 ticks, and asserting both latency constraints.

**Constrained FRs:** FR-PROTO-001, FR-PROTO-004, FR-PROTO-005, NFR-CIV-PERF-007

---

## Category: RELIABILITY

### NFR-CIV-REL-001 — No Panics in Steady State

**Statement:** The engine and server processes SHALL NOT panic during normal simulation operation. Any error condition that occurs after successful startup SHALL be handled by returning a `Result` or `Option`, logging the error, and either recovering or terminating with a structured error message — never an unhandled `unwrap()` or `expect()` on runtime data.

**Rationale:** The project's CLAUDE.md stance requires loud, explicit failures over silent degradation; a panic produces an unstructured crash with no context. Panics in a headless server are especially disruptive because they terminate all connected clients without a graceful close handshake.

**Measurable Target:** Zero panics observed in the 10,000-tick stress run (`cargo test --test stress_10k_ticks`). Clippy lint `clippy::unwrap_used` and `clippy::expect_used` set to `deny` for all engine and server crates (exceptions require inline `#[allow]` with justification comment).

**Verification Method:** CI runs `cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used`; stress test `tests/stress_10k_ticks` asserts exit code 0 with no panic output.

**Constrained FRs:** FR-CORE-001, FR-CORE-007, FR-PROTO-001

---

### NFR-CIV-REL-002 — Loud Failure for Required Dependencies

**Statement:** If a required dependency (database, NATS broker, MinIO, simulation scenario file) is unavailable at startup, the process SHALL emit a structured preflight failure listing each missing dependency by name and exit with a non-zero code. It SHALL NOT start in a degraded mode or silently skip unavailable services.

**Rationale:** CLAUDE.md optionality stance: "Require dependencies where they belong; require clear, loud failures." Silent degradation masks misconfiguration and corrupts reproducibility guarantees.

**Measurable Target:** Preflight check identifies and names every missing required dependency within 5 seconds of process start; exit code is non-zero; log line contains each failed dependency name as a semicolon-delimited list.

**Verification Method:** Integration test `tests/preflight_missing_deps` starts the server with each required dependency blocked in turn, asserts non-zero exit code and presence of dependency name in stderr within 5 seconds.

**Constrained FRs:** FR-CORE-001, FR-REPLAY-001, NFR-CIV-MAINT-001

---

### NFR-CIV-REL-003 — Autosave Cadence

**Statement:** A running simulation SHALL checkpoint its full world state to a durable .civreplay file at least every 60 seconds of wall-clock time, independently of whether any client is connected.

**Rationale:** Loss of more than 60 seconds of simulation history is unacceptable for research integrity and game save reliability. The .civreplay append-only format (FR-REPLAY-001) means each checkpoint flushes the event log tail.

**Measurable Target:** In a 10-minute headless run, at least 9 checkpoint events appear in the event log (one per 60-second interval); the last checkpoint is no older than 60 seconds at any observed moment.

**Verification Method:** Integration test `tests/autosave_cadence` runs a 600-tick headless simulation (100 ms/tick = 60 s sim time) and asserts checkpoint event count ≥ 1 in the event log.

**Constrained FRs:** FR-REPLAY-001, FR-CORE-001

---

### NFR-CIV-REL-004 — Data Integrity (No Silent Corruption)

**Statement:** The .civreplay file's SHA-256 checksum SHALL be verified on every load; a checksum mismatch SHALL be a hard fatal error with an explicit message and SHALL NOT be silently ignored or treated as a warning.

**Rationale:** PRD.md reliability target: "Data Integrity: 100% (no silent corruption)." Silent acceptance of a corrupted replay file would propagate bad data into research results.

**Measurable Target:** 100% of attempted loads of a bit-corrupted .civreplay file produce a process exit with error code and message containing "checksum mismatch"; zero instances of a corrupted file being loaded silently.

**Verification Method:** Unit test `tests/replay_checksum_corruption` injects a single bit flip into a valid replay file and asserts the loader panics or returns `Err` containing "checksum".

**Constrained FRs:** FR-REPLAY-001, FR-REPLAY-002

---

## Category: SECURITY

### NFR-CIV-SEC-001 — WASM Mod Sandbox Capability Limits

**Statement:** Mod code executed through the `wasmtime` host (`civ-mod-host`) SHALL have access only to an explicit, audited allowlist of host imports. Filesystem access, network sockets, process spawning, and clock reads outside of deterministic simulation time SHALL be denied by the WASM linker configuration.

**Rationale:** `docs/guides/mod-sandbox-security.md` specifies that mods cannot assume filesystem, network, or process access; capability creep in host imports is treated as a security finding.

**Measurable Target:** Zero host imports outside the approved allowlist (enforced via WASM linker `--deny-unknown-imports` equivalent); any proposed addition to the allowlist requires a security-tagged PR review. Automated test asserts that a mod attempting `fd_write` to a non-simulation file descriptor receives a WASM trap.

**Verification Method:** Integration test `tests/mod_sandbox_filesystem_denied` loads a test WASM module that calls `fd_write` outside the simulation output channel and asserts the call traps without host-side side effects.

**Constrained FRs:** FR-API-001, FR-API-003

---

### NFR-CIV-SEC-002 — Secrets via .env Only

**Statement:** No secret values (API keys, database passwords, signing keys, Firepass/Kimi credentials) SHALL appear in source files, committed configuration files, or log output. All secrets SHALL be loaded exclusively from environment variables, sourced from a `.env` file that is listed in `.gitignore`.

**Rationale:** Org-wide secrets policy (`feedback_secrets_config.md`); `.env.example` is the committed reference, `.env` is never committed.

**Measurable Target:** Zero secrets detected by `gitleaks` or `trufflehog` in any committed file. The `trufflehog.yml` baseline file in the repo root is the authoritative suppression list.

**Verification Method:** CI step `security/secrets-scan` running `trufflehog filesystem --config trufflehog.yml`; blocks merge on any new finding.

**Constrained FRs:** FR-PROTO-001 (TLS cert paths), FR-API-002

---

### NFR-CIV-SEC-003 — No Network Egress Without Explicit Consent

**Statement:** The engine and client processes SHALL NOT initiate outbound network connections to external hosts (beyond the configured simulation server address) without an explicit opt-in configuration flag set by the operator.

**Rationale:** Research deployments and offline game sessions must be able to run air-gapped; unexpected egress violates user consent and may leak scenario data.

**Measurable Target:** In the default configuration (no opt-in flags set), zero outbound TCP/UDP connections to non-loopback addresses are observed during a 500-tick headless run.

**Verification Method:** Integration test `tests/security/no_egress_default` using a network namespace (Linux) or firewall rule (macOS/Windows) to block external sockets, asserting zero connection attempts.

**Constrained FRs:** FR-PROTO-001, NFR-CIV-REL-002

---

### NFR-CIV-SEC-004 — Zero High/Critical Security Findings

**Statement:** The codebase SHALL maintain zero high-severity or critical-severity security findings as reported by `bandit` (Python), `semgrep` (Rust/Python), and `gitleaks` (secrets) on every CI run.

**Rationale:** CLAUDE.md verifiable constraints table: "Security findings: 0 high/critical."

**Measurable Target:** `bandit -ll` (high+critical only) and `semgrep --severity=ERROR` both exit 0 on every CI commit.

**Verification Method:** CI step `security/static-analysis`; blocks merge on any new finding.

**Constrained FRs:** FR-CLIENT-003, NFR-CIV-SEC-002

---

## Category: ACCESSIBILITY

### NFR-CIV-ACC-001 — Colorblind-Safe Faction Palette

**Statement:** The faction color palette used for unit flags, territory overlays, and UI badges SHALL be distinguishable by users with deuteranopia, protanopia, and tritanopia simulations, as validated by automated palette contrast analysis.

**Rationale:** A game with faction-based strategy mechanics is unusable for colorblind players if factions cannot be distinguished; approximately 8% of male users have red-green color vision deficiency.

**Measurable Target:** Every pair of faction colors has a WCAG contrast ratio ≥ 3:1 under deuteranopia, protanopia, and tritanopia simulation (using the Coblis algorithm). The palette is defined in `assets/palettes/factions.toml` and validated by `scripts/validate_palette.py`.

**Verification Method:** CI step `accessibility/palette-check` running `scripts/validate_palette.py`; fails if any faction pair fails the contrast threshold under any simulated CVD type.

**Constrained FRs:** FR-CLIENT-001, FR-CLIENT-002

---

### NFR-CIV-ACC-002 — Full Keybind Remapping

**Statement:** Every user-facing action in the Bevy desktop client that is bound to a keyboard or mouse button SHALL be remappable by the user via a settings file or in-game UI without recompilation.

**Rationale:** Players with motor disabilities or non-standard keyboards require keybind customization; locked bindings are an accessibility barrier and also a usability issue for non-QWERTY keyboard layouts.

**Measurable Target:** 100% of named input actions registered in the Bevy `InputMap` resource are exposed in `settings/keybindings.toml`; a test remaps every action and asserts the new binding fires correctly.

**Verification Method:** Integration test `tests/accessibility/keybind_remap_all_actions` iterates the registered action list, remaps each to a synthetic key, dispatches the key, and asserts the action fires.

**Constrained FRs:** FR-CLIENT-001

---

### NFR-CIV-ACC-003 — Minimum Readable Font Size

**Statement:** No text rendered in the Bevy desktop client UI (labels, tooltips, HUD counters) SHALL use a font size below 14px at 1080p reference resolution (scaling linearly with DPI).

**Rationale:** Text below 12–14px is illegible at standard viewing distances for users with mild visual impairment and is a baseline UX quality threshold.

**Measurable Target:** All `TextStyle` font size values in UI code are ≥ 14.0 at the 1080p reference scale factor 1.0. A Clippy-style lint or a test that introspects all registered `TextStyle` resources at startup asserts this bound.

**Verification Method:** Integration test `tests/accessibility/min_font_size` queries the Bevy world for all `Text` components at startup and asserts `font_size >= 14.0`.

**Constrained FRs:** FR-CLIENT-001

---

### NFR-CIV-ACC-004 — Tooltips for All Interactive UI Elements

**Statement:** Every interactive UI element (button, icon, map overlay toggle, stat badge) in the Bevy desktop client SHALL display a descriptive tooltip on hover containing the element's name and primary function.

**Rationale:** Tooltips are a baseline accessibility and discoverability requirement; icon-only UIs without tooltips are inaccessible to users who cannot identify icons by sight.

**Measurable Target:** 100% of Bevy UI nodes tagged with the `Interaction` component also carry a `Tooltip` component with a non-empty description string.

**Verification Method:** Integration test `tests/accessibility/tooltip_coverage` queries all `Interaction` nodes and asserts each has a `Tooltip` with `description.len() > 0`.

**Constrained FRs:** FR-CLIENT-001

---

## Category: PORTABILITY

### NFR-CIV-PORT-001 — Target Platform Matrix

**Statement:** The codebase SHALL compile and produce a working binary on the following platform/backend combinations:

| Platform | GPU Backend | Build Target |
|---|---|---|
| Windows 10/11 | Vulkan (primary) | `x86_64-pc-windows-msvc` |
| Windows 10/11 | DX12 (DLSS/Solari path) | `x86_64-pc-windows-msvc` |
| macOS 12+ | Metal | `aarch64-apple-darwin` |
| Linux (Ubuntu 22.04+) | Vulkan | `x86_64-unknown-linux-gnu` |

**Rationale:** The project's active development hardware is Windows (RTX 3090 Ti, DX12) and macOS (M1, Metal); Linux is required for CI and headless server deployments; Vulkan on Windows is the portable fallback when DX12 features (DXR, DLSS) are unavailable.

**Measurable Target:** All four configurations produce a `cargo build --release` success with zero errors in the CI matrix.

**Verification Method:** CI matrix job `build/platform-matrix` building each target; no manual steps required. macOS and Linux runners verify Metal and Vulkan compile-time feature flags respectively.

**Constrained FRs:** FR-CLIENT-001, NFR-CIV-DET-002

---

### NFR-CIV-PORT-002 — Backend Selection Tradeoff Documentation

**Statement:** The repository SHALL maintain an ADR documenting the DLSS-requires-Vulkan vs. Solari-requires-DX12 tradeoff, specifying under which conditions each backend is selected and what features are unavailable on the non-selected backend.

**Rationale:** DLSS (Bevy's upscaling via DLSS plugin) requires Vulkan; Bevy's Solari global illumination path requires DX12. These are mutually exclusive feature sets on Windows; the selection policy must be explicit so agent-driven development does not introduce silent regressions.

**Measurable Target:** ADR file `docs/adr/backend-selection-dlss-vs-solari.md` exists, is not a stub (≥ 200 words), and is referenced from the Cargo feature flag documentation for `civ-client`.

**Verification Method:** CI documentation lint asserting the ADR file exists and that `clients/Cargo.toml` contains a comment referencing it in the `dlss` / `solari` feature definitions.

**Constrained FRs:** FR-CLIENT-001, NFR-CIV-PERF-001, NFR-CIV-PERF-002

---

### NFR-CIV-PORT-003 — Headless Server Runs on All Platforms

**Statement:** The headless simulation server (`civlab-server`) SHALL build and run without a GPU backend dependency on all three OS targets (Windows, macOS, Linux), enabling CI and research workloads on machines without a discrete GPU.

**Rationale:** Headless operation for research (FR-API-002) and CI determinism testing (NFR-CIV-DET-001, NFR-CIV-DET-002) must not require GPU hardware.

**Measurable Target:** `cargo build -p civlab-server --no-default-features` succeeds on all three targets; the 50-tick determinism test passes on CI Linux runner (no GPU).

**Verification Method:** CI step `build/headless-server-no-gpu` compiling with no GPU feature flags and running the determinism test on the Linux runner.

**Constrained FRs:** FR-API-002, NFR-CIV-DET-001, NFR-CIV-PORT-001

---

## Category: MAINTAINABILITY / QUALITY

### NFR-CIV-MAINT-001 — Test Coverage Threshold

**Statement:** The combined engine and server crates SHALL maintain a line coverage of at least 90% as reported by `cargo-llvm-cov`.

**Rationale:** CLAUDE.md verifiable constraints table: "Test coverage ≥ 90%; enforced by `pytest --cov-fail-under=90`" (restated here for Rust via `cargo-llvm-cov`). High coverage is the primary guard against regressions in an agent-driven codebase.

**Measurable Target:** `cargo llvm-cov --all-features --workspace --fail-under-lines 90` exits 0 on every CI commit.

**Verification Method:** CI step `quality/coverage`; blocks merge if coverage drops below 90%.

**Constrained FRs:** FR-CORE-001 through FR-REPLAY-002, FR-METRICS-001 through FR-METRICS-003

---

### NFR-CIV-MAINT-002 — Cyclomatic and Cognitive Complexity Caps

**Statement:** No function in the engine, server, or client crates SHALL exceed a cyclomatic complexity of 10 or a cognitive complexity of 15, as measured by `cargo-complexity` or the `complexity-ratchet` hook.

**Rationale:** CLAUDE.md code quality non-negotiables; complex functions resist automated refactoring by agent workflows and are the primary source of subtle determinism bugs.

**Measurable Target:** Zero functions with cyclomatic complexity > 10 or cognitive complexity > 15 in any committed crate.

**Verification Method:** Pre-commit hook `hooks/complexity-ratchet.sh`; CI step `quality/complexity` running the same check.

**Constrained FRs:** FR-CORE-003, FR-CORE-004 (determinism-critical paths are most sensitive to complexity)

---

### NFR-CIV-MAINT-003 — Code Duplication Ceiling

**Statement:** Cross-file code duplication as measured by `jscpd` SHALL remain below 5% of total lines in the repository.

**Rationale:** CLAUDE.md verifiable constraints table; duplication in simulation math is a vector for divergence between the deterministic and float variants (NFR-CIV-DET-003) and complicates cross-project extraction per the Phenotype reuse protocol.

**Measurable Target:** `jscpd --threshold 5` exits 0 on every CI commit.

**Verification Method:** CI step `quality/duplication`.

**Constrained FRs:** NFR-CIV-DET-003, NFR-CIV-MAINT-001

---

### NFR-CIV-MAINT-004 — Docstring / Doc-Comment Coverage

**Statement:** At least 85% of public functions, structs, enums, and trait implementations in all workspace crates SHALL have a non-empty doc-comment.

**Rationale:** CLAUDE.md verifiable constraints table: "Docstring coverage ≥ 85%"; this is especially critical for the simulation API that external research clients (Python, TypeScript) depend on.

**Measurable Target:** `cargo doc --no-deps 2>&1 | grep "missing documentation"` count ≤ 15% of total public items; enforced by `cargo rustdoc -- -D rustdoc::missing_doc_code_examples` or equivalent deny lint.

**Verification Method:** CI step `quality/docs`; blocks merge if threshold is violated.

**Constrained FRs:** FR-API-001, FR-API-002, FR-API-003

---

### NFR-CIV-MAINT-005 — Architecture Boundary Enforcement

**Statement:** Import relationships between workspace crates SHALL conform to the dependency graph defined in `tach.toml`; no crate SHALL import from a crate that is not declared as its dependency.

**Rationale:** CLAUDE.md verifiable constraints: "Architecture boundaries: import-linter enforced via `lint-imports`." Uncontrolled cross-crate dependencies cause coupling that prevents the engine from being consumed as a standalone library.

**Measurable Target:** `tach check` exits 0 on every CI commit; zero violations.

**Verification Method:** CI step `quality/architecture` running `tach check`; blocks merge on any violation.

**Constrained FRs:** FR-API-002, NFR-CIV-PORT-003

---

### NFR-CIV-MAINT-006 — Zero Lint Suppressions Without Justification

**Statement:** No `#[allow(...)]` or `#[cfg_attr(..., allow(...))]` attribute SHALL be committed without an inline comment on the same or preceding line explaining why the suppression is necessary and referencing a tracking issue or ADR where applicable.

**Rationale:** CLAUDE.md: "Zero new lint suppressions without inline justification." Unjustified suppressions accumulate silently and mask real quality issues in agent-driven workflows.

**Measurable Target:** `hooks/suppression-blocker.sh` detects zero unjustified suppressions on every pre-commit check; CI step `quality/suppressions` performs the same scan.

**Verification Method:** Pre-commit hook `hooks/suppression-blocker.sh`; CI `quality/suppressions` step.

**Constrained FRs:** NFR-CIV-MAINT-001, NFR-CIV-MAINT-002

---

## Verification Artifact Summary

The following table maps every NFR to its primary verification artifact. Artifacts marked **CI-GATE** block merge on failure; **CI-REPORT** produce a report but do not block; **MANUAL** require a human or agent validation step at milestone.

| NFR ID | Category | Verification Artifact | Blocks Merge? |
|---|---|---|---|
| NFR-CIV-PERF-001 | Performance | `benches/client_render_1k_entities` (RTX 3090 Ti) | CI-GATE |
| NFR-CIV-PERF-002 | Performance | `benches/client_render_500_entities_metal` + MANUAL M1 sign-off | MANUAL |
| NFR-CIV-PERF-003 | Performance | `benches/tick_budget_200_civilians` | CI-GATE |
| NFR-CIV-PERF-004 | Performance | `benches/tick_scaling_series` (3-point regression) | CI-REPORT (milestone) |
| NFR-CIV-PERF-005 | Performance | `benches/terrain_mesh_gen_256` | CI-GATE |
| NFR-CIV-PERF-006 | Performance | `tests/memory_ceiling_10k_entities` | CI-GATE |
| NFR-CIV-PERF-007 | Performance | `tests/proto/bandwidth_10_clients` | CI-GATE |
| NFR-CIV-DET-001 | Determinism | `tests/determinism_replay_same_platform` | CI-GATE |
| NFR-CIV-DET-002 | Determinism | CI matrix `determinism_cross_platform` | CI-GATE |
| NFR-CIV-DET-003 | Determinism | Clippy deny float + `tests/fixed_point_float_agreement` | CI-GATE |
| NFR-CIV-DET-004 | Determinism | `tests/rng_draw_log_completeness` | CI-GATE |
| NFR-CIV-SCALE-001 | Scalability | `benches/tick_scaling_series` (milestone checklist) | MANUAL (milestone) |
| NFR-CIV-SCALE-002 | Scalability | Manual fly-through + headless frame-time recording | MANUAL |
| NFR-CIV-SCALE-003 | Scalability | `tests/proto/10_client_load` | CI-GATE |
| NFR-CIV-REL-001 | Reliability | Clippy `deny(unwrap_used)` + `tests/stress_10k_ticks` | CI-GATE |
| NFR-CIV-REL-002 | Reliability | `tests/preflight_missing_deps` | CI-GATE |
| NFR-CIV-REL-003 | Reliability | `tests/autosave_cadence` | CI-GATE |
| NFR-CIV-REL-004 | Reliability | `tests/replay_checksum_corruption` | CI-GATE |
| NFR-CIV-SEC-001 | Security | `tests/mod_sandbox_filesystem_denied` | CI-GATE |
| NFR-CIV-SEC-002 | Security | CI `security/secrets-scan` (trufflehog) | CI-GATE |
| NFR-CIV-SEC-003 | Security | `tests/security/no_egress_default` | CI-GATE |
| NFR-CIV-SEC-004 | Security | CI `security/static-analysis` (bandit + semgrep) | CI-GATE |
| NFR-CIV-ACC-001 | Accessibility | CI `accessibility/palette-check` (`validate_palette.py`) | CI-GATE |
| NFR-CIV-ACC-002 | Accessibility | `tests/accessibility/keybind_remap_all_actions` | CI-GATE |
| NFR-CIV-ACC-003 | Accessibility | `tests/accessibility/min_font_size` | CI-GATE |
| NFR-CIV-ACC-004 | Accessibility | `tests/accessibility/tooltip_coverage` | CI-GATE |
| NFR-CIV-PORT-001 | Portability | CI matrix `build/platform-matrix` (4 targets) | CI-GATE |
| NFR-CIV-PORT-002 | Portability | ADR existence lint + Cargo feature comment check | CI-GATE |
| NFR-CIV-PORT-003 | Portability | CI `build/headless-server-no-gpu` + determinism test | CI-GATE |
| NFR-CIV-MAINT-001 | Maintainability | CI `quality/coverage` (`cargo llvm-cov --fail-under 90`) | CI-GATE |
| NFR-CIV-MAINT-002 | Maintainability | Hook `complexity-ratchet.sh` + CI `quality/complexity` | CI-GATE |
| NFR-CIV-MAINT-003 | Maintainability | CI `quality/duplication` (`jscpd --threshold 5`) | CI-GATE |
| NFR-CIV-MAINT-004 | Maintainability | CI `quality/docs` (rustdoc deny lint) | CI-GATE |
| NFR-CIV-MAINT-005 | Maintainability | CI `quality/architecture` (`tach check`) | CI-GATE |
| NFR-CIV-MAINT-006 | Maintainability | Hook `suppression-blocker.sh` + CI `quality/suppressions` | CI-GATE |

---

## FR Constraint Cross-Reference

The table below is the reverse index: for each FR-CIV-* identifier, the NFRs that constrain it.

| FR ID | Constraining NFRs |
|---|---|
| FR-CORE-001 | NFR-CIV-PERF-003, NFR-CIV-PERF-004, NFR-CIV-SCALE-001, NFR-CIV-REL-001, NFR-CIV-REL-002, NFR-CIV-REL-003, NFR-CIV-MAINT-001 |
| FR-CORE-002 | NFR-CIV-PERF-006, NFR-CIV-SCALE-001, NFR-CIV-MAINT-001 |
| FR-CORE-003 | NFR-CIV-DET-001, NFR-CIV-DET-002, NFR-CIV-DET-003, NFR-CIV-MAINT-002 |
| FR-CORE-004 | NFR-CIV-DET-001, NFR-CIV-DET-004, NFR-CIV-MAINT-002 |
| FR-CORE-007 | NFR-CIV-PERF-003, NFR-CIV-PERF-004, NFR-CIV-SCALE-001, NFR-CIV-REL-001 |
| FR-ECON-003 | NFR-CIV-PERF-003 |
| FR-ECON-005 | NFR-CIV-PERF-003, NFR-CIV-PERF-004, NFR-CIV-SCALE-001 |
| FR-PROTO-001 | NFR-CIV-SCALE-003, NFR-CIV-REL-001, NFR-CIV-SEC-002, NFR-CIV-SEC-003 |
| FR-PROTO-004 | NFR-CIV-PERF-001, NFR-CIV-PERF-007, NFR-CIV-SCALE-003 |
| FR-PROTO-005 | NFR-CIV-PERF-007, NFR-CIV-SCALE-003 |
| FR-REPLAY-001 | NFR-CIV-DET-001, NFR-CIV-DET-004, NFR-CIV-REL-003, NFR-CIV-REL-004 |
| FR-REPLAY-002 | NFR-CIV-DET-001, NFR-CIV-DET-002, NFR-CIV-REL-004 |
| FR-API-001 | NFR-CIV-SEC-001, NFR-CIV-MAINT-004 |
| FR-API-002 | NFR-CIV-MAINT-004, NFR-CIV-MAINT-005, NFR-CIV-PORT-003 |
| FR-API-003 | NFR-CIV-SEC-001, NFR-CIV-MAINT-004 |
| FR-CLIENT-001 | NFR-CIV-PERF-001, NFR-CIV-PERF-002, NFR-CIV-PERF-005, NFR-CIV-PERF-006, NFR-CIV-SCALE-002, NFR-CIV-REL-001, NFR-CIV-ACC-001 through NFR-CIV-ACC-004, NFR-CIV-PORT-001, NFR-CIV-PORT-002 |
| FR-CLIENT-003 | NFR-CIV-SEC-004 |
| FR-METRICS-001 | NFR-CIV-MAINT-001 |
| FR-METRICS-002 | NFR-CIV-DET-003 |
| FR-METRICS-003 | NFR-CIV-DET-001, NFR-CIV-DET-002, NFR-CIV-DET-003 |

---

**Document History**

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-05-29 | Initial NFR specification covering all 8 categories; 35 NFRs; full verification artifact table. |
