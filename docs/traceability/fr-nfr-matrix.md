# Non-Functional Requirements Traceability Matrix

**Status:** Active — promotes the 34 unique `NFR-*` IDs from [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) (0% traced before this file).
**Source of truth:** [`docs/reference/non-functional-requirements.md`](../reference/non-functional-requirements.md) + 3D extension [`NFR-CIV-SCALE-PERF.md`](../specs/requirements/NFR-CIV-SCALE-PERF.md).
**Format:** NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | **Acceptance Contract** | Status

Status values: `traced` | `code-only` (NFR in code/docs, oracle gate TODO) | `stub` (spec only).

The **Acceptance Contract** column is the machine-checkable oracle hook — threshold predicates agents assert in batch iteration (per AAA audit: traceability keystone).

---

## Performance (NFR-CIV-PERF-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-PERF-001 | Bevy client sustains 60 FPS at 1440p with 1k visible entities | `clients/bevy-ref/` | `benches/client_render_1k_entities` | P95 frame time ≤ 16.67 ms AND P99 ≤ 20 ms over 60 s at 2560×1440 with 1,000 entities | code-only |
| NFR-CIV-PERF-002 | Bevy client sustains 60 FPS on Apple M1 Metal at 1440p with 500 entities | `clients/bevy-ref/` | `benches/client_render_500_entities_metal` | P95 frame time ≤ 16.67 ms AND P99 ≤ 22 ms over 60 s at 2560×1440 with 500 entities on M1 | code-only |
| NFR-CIV-PERF-003 | Engine tick completes within budget at 200 civilians (nominal load) | `crates/engine/` | `benches/tick_budget_200_civilians` | P99 tick duration ≤ 10 ms over 1,000 consecutive headless ticks at 200 agents | code-only |
| NFR-CIV-PERF-004 | Tick budget scales sub-linearly to 10k and 100k agents | `crates/engine/` | `benches/tick_scaling_series` | P99 tick ≤ 80 ms at 10k agents AND P99 tick ≤ 500 ms at 100k agents (headless) | code-only |
| NFR-CIV-PERF-005 | Terrain mesh generation for 256×256 chunk within budget | `crates/voxel/` | `benches/terrain_mesh_gen_256` | P99 single-chunk mesh generation ≤ 50 ms | code-only |
| NFR-CIV-PERF-006 | Combined engine+client RSS ceiling at 10k entities | `crates/engine/` | `tests/memory_ceiling_10k_entities` | Peak RSS ≤ 4,096 MB after 500 ticks with 10,000 entities | code-only |
| NFR-CIV-PERF-007 | Server outbound bandwidth ceiling with 10 clients | `crates/server/` | `tests/proto/bandwidth_10_clients` | Aggregate outbound bandwidth ≤ 10 Mbps sustained over 60 s with 10 WS clients at 60 tps | code-only |
| NFR-CIV-PERF-900 | ≥60 fps reference desktop; ≥30 fps floor under heavy load | `clients/bevy-ref/` | TODO: `profiler_budget_scenario` | P95 frame time ≤ 16.67 ms in representative scenario; floor ≥ 30 fps under heavy load | stub |
| NFR-CIV-PERF-901 | GPU instancing for massive agent+voxel counts | `clients/bevy-ref/` | TODO: `instancing_draw_call_bound` | 100k+ instanced agents render within frame budget; draw-call count bounded | stub |
| NFR-CIV-PERF-902 | Async dirty-queue drain off render thread | `crates/voxel/` | `voxel::dirty_queue_deterministic` | No frame hitch > 16 ms attributable to chunk mesh on worker-thread drain path | code-only |

---

## Determinism (NFR-CIV-DET-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-DET-001 | Cross-run bit-identical replay on same platform | `crates/engine/` | `tests/determinism_replay_same_platform` | 100% of per-tick state hashes match between two runs (same seed, scenario, tick count) | traced |
| NFR-CIV-DET-002 | Cross-platform bit-identical replay | `crates/engine/` | `determinism_cross_platform` (CI matrix) | Per-tick state hashes identical across Windows, macOS, Linux for canonical 50-tick scenario | code-only |
| NFR-CIV-DET-003 | Fixed-point arithmetic in state-mutation paths | `crates/engine/` | clippy float deny + `tests/fixed_point_float_agreement` | Zero f32/f64 in sim state-mutation modules; fixed vs float agree within 10⁻⁶ | code-only |
| NFR-CIV-DET-004 | Every RNG draw logged before consumption | `crates/engine/` | `tests/rng_draw_log_completeness` | `rng_draw` event count == instrumented draw count after 1,000 ticks (delta = 0) | code-only |

---

## Scalability (NFR-CIV-SCALE-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-SCALE-001 | Agent-count roadmap tier gates (200 / 10k / 100k) | `crates/engine/` | `benches/tick_scaling_series` | Tier 1: P99 ≤ 10 ms @ 200; Tier 2: P99 ≤ 80 ms @ 10k; Tier 3: P99 ≤ 500 ms @ 100k | code-only |
| NFR-CIV-SCALE-002 | Chunk streaming keeps render thread hitch-free | `clients/bevy-ref/` | manual fly-through + frame-time recording | Zero frames with duration > 2× median attributable to chunk I/O during 30 s fly-through at 100 cells/s | stub |
| NFR-CIV-SCALE-003 | 10 simultaneous WS clients without tick regression | `crates/server/` | `tests/proto/10_client_load` | P99 tick processing regression ≤ 2 ms vs 0-client baseline; P99 delta delivery ≤ 50 ms loopback | code-only |
| NFR-CIV-SCALE-900 | 20mi×20mi world extent via SVO + 16³ leaf chunks | `crates/voxel/` | TODO: `full_map_extent_instantiates` | `WorldCoord` addressing covers 32km×32km extent without precision loss | stub |
| NFR-CIV-SCALE-901 | Active working set resident; rest streams from disk | `crates/voxel/` | TODO: `streaming_memory_budget` | Resident chunk count ≤ configured budget during full-map pan; evicted chunks reload on approach | stub |
| NFR-CIV-SCALE-902 | Compact on-disk chunk format (LOD pyramids + compression) | `crates/voxel/` | TODO: `disk_footprint_estimate` | Documented bytes/voxel estimate for 20mi target; chunks stored compressed with LOD mips | stub |
| NFR-CIV-SCALE-910 | LOD-tiered agent simulation (Hot/Warm/Cold) | `crates/agents/` | `agents::lod_gestalt_no_divergence` | Cold→Hot promotion yields same determinism hash as always-Hot run (same seed/path) | code-only |
| NFR-CIV-SCALE-920 | LOD/streaming transitions preserve determinism | `crates/engine/` | TODO: `determinism_across_lod` | Same seed + camera path → bit-identical sim state regardless of chunk residency history | stub |

---

## Reliability (NFR-CIV-REL-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-REL-001 | No panics in steady-state simulation | `crates/engine/` | clippy deny unwrap + `tests/stress_10k_ticks` | Zero panics in 10,000-tick stress run; clippy unwrap/expect deny passes | code-only |
| NFR-CIV-REL-002 | Loud preflight failure for missing required deps | `crates/server/` | `tests/preflight_missing_deps` | Non-zero exit within 5 s; stderr lists each missing dependency by name | code-only |
| NFR-CIV-REL-003 | Autosave checkpoint at least every 60 s wall-clock | `crates/engine/` | `tests/autosave_cadence` | ≥ 1 checkpoint event per 60 s sim time in 10-minute headless run | code-only |
| NFR-CIV-REL-004 | Corrupted replay checksum is hard fatal | `crates/replay/` | `tests/replay_checksum_corruption` | Bit-flipped `.civreplay` load returns `Err` containing `checksum`; never loads silently | code-only |

---

## Security (NFR-CIV-SEC-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-SEC-001 | WASM mod sandbox denies non-allowlist imports | `crates/mod-host/` | `tests/mod_sandbox_filesystem_denied` | Mod calling `fd_write` outside sim channel traps; zero host-side filesystem side effects | code-only |
| NFR-CIV-SEC-002 | No secrets in committed source | repo root | CI `security/secrets-scan` (trufflehog) | trufflehog exits 0; zero new secret findings vs baseline | code-only |
| NFR-CIV-SEC-003 | No network egress without explicit opt-in | `crates/server/` | `tests/security/no_egress_default` | Zero outbound TCP/UDP to non-loopback during 500-tick headless run (default config) | code-only |
| NFR-CIV-SEC-004 | Zero high/critical static-analysis findings | repo root | CI `security/static-analysis` | `bandit -ll` and `semgrep --severity=ERROR` both exit 0 | code-only |

---

## Accessibility (NFR-CIV-ACC-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-ACC-001 | Colorblind-safe faction palette | `assets/palettes/` | `scripts/validate_palette.py` | Every faction color pair WCAG contrast ≥ 3:1 under deuteranopia/protanopia/tritanopia simulation | stub |
| NFR-CIV-ACC-002 | All input actions remappable | `clients/bevy-ref/` | `tests/accessibility/keybind_remap_all_actions` | 100% of `InputMap` actions fire after remap via `settings/keybindings.toml` | stub |
| NFR-CIV-ACC-003 | Minimum UI font size 14px at 1080p reference | `clients/bevy-ref/` | `tests/accessibility/min_font_size` | All `Text` components at startup have `font_size ≥ 14.0` at scale 1.0 | stub |
| NFR-CIV-ACC-004 | Tooltips on all interactive UI nodes | `clients/bevy-ref/` | `tests/accessibility/tooltip_coverage` | Every `Interaction` node has `Tooltip` with `description.len() > 0` | stub |

---

## Portability (NFR-CIV-PORT-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-PORT-001 | Build matrix: Win Vulkan/DX12, macOS Metal, Linux Vulkan | workspace | CI `build/platform-matrix` | `cargo build --release` succeeds on all four platform/backend rows | code-only |
| NFR-CIV-PORT-002 | ADR documents DLSS-vs-Solari backend tradeoff | `docs/adr/` | CI ADR existence lint | `docs/adr/backend-selection-dlss-vs-solari.md` exists (≥ 200 words) and referenced from client Cargo features | stub |
| NFR-CIV-PORT-003 | Headless server builds without GPU on all OS targets | `crates/server/` | CI `build/headless-server-no-gpu` | `cargo build -p civ-server --no-default-features` succeeds; 50-tick determinism test passes on Linux CI | code-only |

---

## Maintainability (NFR-CIV-MAINT-*)

| NFR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|--------|------------------------|-----------------|--------------|---------------------|--------|
| NFR-CIV-MAINT-001 | Workspace line coverage ≥ 90% | workspace | CI `quality/coverage` (`cargo llvm-cov`) | `cargo llvm-cov --fail-under-lines 90` exits 0 | code-only |
| NFR-CIV-MAINT-002 | Cyclomatic ≤ 10 and cognitive ≤ 15 per function | workspace | `hooks/complexity-ratchet.sh` | Zero functions exceed cyclomatic 10 or cognitive 15 | code-only |
| NFR-CIV-MAINT-003 | Code duplication < 5% | workspace | CI `quality/duplication` (jscpd) | `jscpd --threshold 5` exits 0 | code-only |
| NFR-CIV-MAINT-004 | Public API doc-comment coverage ≥ 85% | workspace | CI `quality/docs` | Missing-doc count ≤ 15% of public items | code-only |
| NFR-CIV-MAINT-005 | Import graph matches `tach.toml` boundaries | workspace | CI `quality/architecture` (`tach check`) | `tach check` exits 0; zero boundary violations | code-only |
| NFR-CIV-MAINT-006 | No unjustified lint suppressions | workspace | `hooks/suppression-blocker.sh` | Zero `#[allow]` without inline justification comment | code-only |

---

## Row count

**43 rows** (35 canonical `NFR-CIV-*` from `non-functional-requirements.md` + 8 3D extension `900`-series rows). COVERAGE_AUDIT enumerated 34 unique `NFR-*` tokens before this matrix; this file is the superset oracle spine.

*Generated 2026-06-25 for P3-T2 traceability spine.*
