# docs/sessions/ — Index & Retrospective Roadmap

**Last Updated**: 2026-05-18  
**Total Files**: 84 markdown documents  
**Scope**: DINOForge development retrospectives, audit-rotation methodology, infrastructure investigations, and release milestones

---

## Quick Navigation

- **Active Development** — Files from 2026-04-26 onward  
- **Audit-Rotation Archive** — Complete methodology docs (2026-04-24 to 2026-04-25)  
- **Investigation & Prototypes** — Research on isolation layers, rendering, MCP, asset pipelines  
- **Historical Release Checkpoints** — v0.17.0, v0.24.0 delivery reports

---

## Active Development (since 2026-04-26)

### Latest Retrospectives & Release Status
- [iter-1-to-119-retrospective.md](iter-1-to-119-retrospective.md) **[2026-05-18]** — Final audit-rotation convergence report. 3,411 tests passing, 0 failures. 16 Tier 1 Roslyn analyzers, 28 pattern detectors, v0.24.0 released (commit f222cd3). **CANONICAL** for post-release iteration summary.
- [steamworks-goldberg-investigation.md](steamworks-goldberg-investigation.md) **[2026-05-17]** — Research on Goldberg Emulator as Steam.dll drop-in replacement. Headless multi-instance testing via Steamless path. Status: viable, not yet integrated.
- [v0.24.0-execution-status.md](v0.24.0-execution-status.md) **[2026-04-26]** — v0.24.0 release readiness dashboard. Closure-gate green. Tests 2,785p/0f, Tier 1 analyzers 16/16, CI 41 workflows. **[RELEASED 2026-05-06]**

### Proof System & Validation
- [PROOF_OF_COMPLETION_20260420.md](PROOF_OF_COMPLETION_20260420.md) **[2026-04-27]** — Delivery proof via 4 Remotion video reels (3.2 MB), CLI run-through, Kimi judge tier wiring. Supersedes earlier prove-features claims.
- [TRACEABILITY_VERIFICATION_20260420.md](TRACEABILITY_VERIFICATION_20260420.md) **[2026-04-27]** — 100% user story → test → docs linkage verification. v0.23.0 quality gate checklist. Headless automation runbook.

### Phase 4 Closure Reports
- [2026-04-26-phase-4a-retrospective.md](2026-04-26-phase-4a-retrospective.md) **[2026-04-26]** — Phase 4a (iter-85-91) closure. Infra pivot from DINOBox to playCUA. 5 HIGH-severity pattern categories cleared. GameBridgeServer async refactor (22 sites).
- [2026-04-25-infra-pivot-plan.md](2026-04-25-infra-pivot-plan.md) **[2026-04-26]** — Rationalization of infrastructure strategy. Steamless & HiddenDesktop BROKEN → playCUA adoption path. Multi-instance testing via TEST_INSTANCE path. Decouples DINOBox dependency.

### Audit-Rotation Focused Topics
- [2026-04-25-ui-sdk-audit.md](2026-04-25-ui-sdk-audit.md) — UI domain plugin wiring audit. Pattern #86 false-positive cleanup.
- [2026-04-25-bridge-bypass-audit.md](2026-04-25-bridge-bypass-audit.md) — GameBridgeServer silent-bypass analysis. 22 blocking .Result sites, swallow-catch clustering.
- [2026-04-25-ci-manifest-gate-audit.md](2026-04-25-ci-manifest-gate-audit.md) — Smart-contract proof system (#191) design. cosign + merkle root + policy evaluation. **Status**: Not yet fully implemented.
- [2026-04-25-sandbox-isolation-audit.md](2026-04-25-sandbox-isolation-audit.md) — HiddenDesktop vs playCUA vs VDD tier fallback. Per-tier startup cost, capability matrix.
- [2026-04-25-steamless-multi-instance-audit.md](2026-04-25-steamless-multi-instance-audit.md) — Steamless.EXE + Goldberg Emulator research. Multi-instance via separate Steam accounts or CCP path.

### Earlier Phase Snapshots
- [2026-04-24-test-suite-honest-audit.md](2026-04-24-test-suite-honest-audit.md) — Delete 298 mock-theater tests, gate 73 real-game tests. Test count baseline stabilization.
- [2026-04-24-asset-swap-truth-audit.md](2026-04-24-asset-swap-truth-audit.md) — AssetSwapSystem Phase 1 + Phase 2 wiring. 12/30 Star Wars bundles render 0/36 in-game (#101 open).
- [2026-04-24-hidden-desktop-ground-truth.md](2026-04-24-hidden-desktop-ground-truth.md) — HiddenDesktopBackend BROKEN analysis. Win32 CreateDesktop(-1, -1) fails on Parsec. Tier 2 candidate. Documented fallback strategy.
- [2026-04-24-kimi-judge-completion.md](2026-04-24-kimi-judge-completion.md) — External VLM judge tier (Kimi) wired into prove-features. **Status**: MOONSHOT_API_KEY not provided; inference not tested.
- [2026-04-24-playcua-smoke-test.md](2026-04-24-playcua-smoke-test.md) — playCUA standalone launch validation. Bare-CUA binary exists. HiddenDesktop → playCUA fallback chain designed.
- [2026-04-24-session-summary.md](2026-04-24-session-summary.md) **[2026-04-24]** — Phase 3 close. Audit-rotation methodology locked. HiddenDesktop replacement path → playCUA. Multi-instance TEST_INSTANCE path active.

---

## Superseded Historical Archive

> These files document completed investigations, prior releases, or methodology milestones. Kept for traceability; **authoritative docs are in CLAUDE.md, TRUTH_TABLE.md, README.md**.

### Release & Validation Snapshots (Replaced by TRUTH_TABLE.md)
- [FINAL_RELEASE_VALIDATION_20260420_120000.md](FINAL_RELEASE_VALIDATION_20260420_120000.md) — v0.23.0 pre-tag quality gate. [REPLACED-BY: docs/TRUTH_TABLE.md updates 1-67]
- [RELEASE_READINESS_SUMMARY.md](RELEASE_READINESS_SUMMARY.md) — v0.23.0 gate checklist. [REPLACED-BY: v0.24.0-execution-status.md]
- [FINAL_VALIDATION_REPORT_20260422.md](FINAL_VALIDATION_REPORT_20260422.md) — Unit + integration test closure. [REPLACED-BY: iter-1-to-119-retrospective.md]
- [SESSION_SUMMARY_20260420.md](SESSION_SUMMARY_20260420.md) — Post-M11 recap. [REPLACED-BY: 2026-04-24-session-summary.md]
- [FINAL_SESSION_SUMMARY_20260420.md](FINAL_SESSION_SUMMARY_20260420.md) — Duplicate of above. [SUPERCEDED]

### PhenoCompose & Isolation Layer Research (v0.24.0-dev roadmap, not merged)
- [phenocompose_nvms_investigation.md](phenocompose_nvms_investigation.md) **[2026-04-23]** — KooshaPari/phenocompose analysis. Tier 3 multi-VM orchestration. Status: External dependency, roadmap only (v0.25.0+).
- [phenocompose_integration_technical.md](phenocompose_integration_technical.md) **[2026-04-23]** — Technical integration plan (3-phase). nanovms Tier 2 (90ms startup), Firecracker Tier 3 (125ms, GPU passthrough). [STATUS: Roadmap-only]
- [isolation_layer_implementation_report.md](isolation_layer_implementation_report.md) **[2026-04-21]** — playCUA isolation backend (HiddenDesktop + playCUA auto-detect). IsolationContext.get('auto'). Implemented in main.
- [playCUA_integration_audit.md](playCUA_integration_audit.md) **[2026-04-21]** — Complete playCUA tool surface. Image diff (BLAKE3), input injection, window enumeration, process management, screenshot (WGC/X11/CoreGraphics). Backend selection matrix.
- [playcua_phase3_5_spec.md](playcua_phase3_5_spec.md) **[2026-04-21]** — Phase 3.5 isolations-layer spec finalization. playCUA on port 9000, auto-fallback to HiddenDesktop.

### MCP & Bridge Protocol Deep Dives (Iter-42-48 analysis)
- [fastmcp_jsonrpc_protocol_analysis.md](fastmcp_jsonrpc_protocol_analysis.md) **[2026-04-11]** — FastMCP σ-frame protocol. Comparison to bare JSON-RPC 2.0. Frame encoding: `size:uint32 | type:uint8 | content:bytes`. FD multiplexing semantics.
- [mcp_pytest_suite_report_20260331.md](mcp_pytest_suite_report_20260331.md) **[2026-03-31]** — MCP server unit tests in Python. MockGameBridgeServer wiring. ~40 tests. [STATUS: Historical baseline]

### Asset Pipeline & Zig Module (Iter-44-47)
- [zig_module_creation_report.md](zig_module_creation_report.md) **[2026-04-11]** — Zig module for Rust FFI (AssetctlPipeline). BVH construction, mesh decimation, LOD validation. [STATUS: Prototype, not integrated]
- [sandbox-validation-implementation.md](sandbox-validation-implementation.md) **[2026-04-12]** — Sandbox + manifest validation pipeline. File-integrity proofs via HMAC-SHA256. [STATUS: Milestone, pre-impl]

### Build & Environment Stabilization (Iter-35-42, Mar 30-31)
- [environment_compatibility_matrix_20260330_091430.md](environment_compatibility_matrix_20260330_091430.md) **[2026-03-30]** — .NET 11 preview, Rust 1.81, Unity 2021.3.45f2 compatibility matrix. WSL2 vs Windows native differences. [STATUS: Baseline validation, historical]
- [environment_compatibility_fix_applied_20260330_093200.md](environment_compatibility_fix_applied_20260330_093200.md) — BepInEx.ConfigurationManager nuget downgrade (workaround for .NET 11 pack issue). [REPLACED-BY: Global Directory.Build.props pins]
- [environment_compatibility_final_20260330.md](environment_compatibility_final_20260330.md) — Final resolution (use WSL2 or GitHub Actions for Windows CI). Windows-native .NET 11 CLI hangs (known limitation, no code fix). [STATUS: Accepted limitation]

### Validation System & Test Infrastructure (Iter-38-42, Mar 27-30)
- [validation_system_completion_summary.md](validation_system_completion_summary.md) **[2026-03-30]** — Schema validation (NJsonSchema), pack compat checking, framework_version constraints. [STATUS: v0.14.0 feature complete]
- [validation_matrix_completion_report.md](validation_matrix_completion_report.md) **[2026-03-31]** — Comprehensive test matrix for validator subsystem (21 scenarios). [STATUS: Historical test log]
- [test-suite-2026-03-27.md](test-suite-2026-03-27.md) **[2026-03-27]** — Snapshot of CI test counts (2,100+ passing). [REPLACED-BY: Current baselines in iter-1-to-119-retrospective.md]

### Multi-Instance & Hidden Desktop Prototypes (Iter-30-35, Mar 25-28)

**Infrastructure Delivery Series** (delivered working prototypes, infrastructure later deprecated):
- [HIDDEN_DESKTOP_DELIVERY_SUMMARY.md](HIDDEN_DESKTOP_DELIVERY_SUMMARY.md) **[2026-03-25]** — Win32 CreateDesktop test script. P/Invoke reference. Script deleted post-Parsec failure (#202 noted).
- [HIDDEN_DESKTOP_FILES_MANIFEST.md](HIDDEN_DESKTOP_FILES_MANIFEST.md) — Source file inventory for hidden desktop delivery.
- [HIDDEN_DESKTOP_TEST_PLAN.md](HIDDEN_DESKTOP_TEST_PLAN.md) **[2026-03-25]** — Comprehensive test plan (5 scenarios, success criteria). Prototype verified locally; Parsec render failure later (#86 open).
- [HIDDEN_DESKTOP_TEST_QUICKSTART.md](HIDDEN_DESKTOP_TEST_QUICKSTART.md) — Quick execution guide (pre-Parsec failure).
- [HIDDEN_DESKTOP_PROTOTYPE.md](HIDDEN_DESKTOP_PROTOTYPE.md) — Full P/Invoke implementation (33 KB, 500+ lines). [STATUS: BROKEN on Parsec; replaced by playCUA]
- [HIDDEN_DESKTOP_PINVOKE_REFERENCE.md](HIDDEN_DESKTOP_PINVOKE_REFERENCE.md) — P/Invoke definitions (Win32 modules: Desktop, Process, Window, GDI).
- [HIDDEN_DESKTOP_CONCURRENT_INSTANCES_FINAL_REPORT.md](HIDDEN_DESKTOP_CONCURRENT_INSTANCES_FINAL_REPORT.md) **[2026-03-31]** — Concurrent-instance test results (2 game instances side-by-side). Logic proven; infrastructure deprecated.
- [CONCURRENT_INSTANCES_IMPLEMENTATION_STATUS.md](CONCURRENT_INSTANCES_IMPLEMENTATION_STATUS.md) **[2026-03-31]** — Implementation checklist for TEST_INSTANCE path (now in CLAUDE.md Deploying Fixes section).

### Game Rendering & Capture Research (Iter-29-35, Mar 25-27)
- [SCREENSHOT_CAPTURE_SOLVED.md](SCREENSHOT_CAPTURE_SOLVED.md) — Screenshot via named pipe + ScreenCapture.CaptureScreenshot. Minimal proof. [STATUS: Replaced by MCP game_screenshot tool]
- [VIDEO_CAPTURE_RESEARCH.md](VIDEO_CAPTURE_RESEARCH.md) — Remotion (React + FFmpeg) for VHS-tape-style demos. Used in proof bundles (iter-50+).
- [parsec_dxgi_diagnostic_2026-03-25.md](parsec_dxgi_diagnostic_2026-03-25.md) — BitBlt + DXGI failures on Parsec (render corruption). Diagnosis: Parsec VDD doesn't support CreateDesktop rendering.
- [bundle_build_report_2026-03-25.md](bundle_build_report_2026-03-25.md) — Star Wars + Modern warfare asset bundle compile log. 12 of 30 bundles are stubs (90 bytes); real bundles render 0/36 in-game.

### Game Automation & Testing Foundations (Iter-29-34, Mar 25-27)
- [TITAN_GAME_TEST_IMPLEMENTATION.md](TITAN_GAME_TEST_IMPLEMENTATION.md) — TITAN-inspired coverage-driven test agent (stub, 208 bytes). [REPLACED-BY: /game-test-task skill]
- [virtual_display_research_20260327.md](virtual_display_research_20260327.md) **[2026-03-28]** — VDD (Virtual Display Driver) research. IDD/WDDM-based isolation. Future Tier 1 (not v0.24.0).

### Bridge, Deployment & Architecture (Iter-25-31)
- [DEPLOYMENT_AND_DEBUG_GUIDE.md](DEPLOYMENT_AND_DEBUG_GUIDE.md) **[2026-03-28]** — Asset swap + override + log-tail workflow. Bridge naming conventions. Now in CLAUDE.md Deploying Fixes section.
- [CODE_COMPARISON.md](CODE_COMPARISON.md) **[2026-03-25]** — C# vs Python mock-server implementations. Reference for McpServer.cs vs server.py debate.
- [SANDBOX_LAYER_DESIGN.md](SANDBOX_LAYER_DESIGN.md) **[2026-03-25]** — Full isolation-layer architecture (25 KB). HiddenDesktop + playCUA + VDD tiers, fallback chain. [INTEGRATED into CLAUDE.md]

### Foundation & Status Documents (Iter-20-28)
- [git_audit_20260327.md](git_audit_20260327.md) **[2026-03-28]** — Branch cleanup, stale-ref removal. Local-branch state snapshot. [STATUS: Historical baseline]
- [stash_extraction_report_20260328.md](stash_extraction_report_20260328.md) **[2026-03-28]** — Recovered 860-line ModPlatform.cs (pack refresh + HMR) from stash@{5}. Now in commit abba75f.
- [session_audit_mar13_14.md](session_audit_mar13_14.md) **[2026-03-27]** — Initial audit-rotation lens shape exploration (logging, async discipline, exception flow). [STATUS: Methodology validation]
- [AGENT_ISOLATION_RESEARCH.md](AGENT_ISOLATION_RESEARCH.md) **[2026-03-25]** — Comprehensive isolation strategy (41 KB). Win32 process group, security descriptors, desktop isolation. [INTEGRATED into CLAUDE.md + isolation_layer.py]
- [MULTI_INSTANCE_RESEARCH.md](MULTI_INSTANCE_RESEARCH.md) **[2026-03-25]** — Unity mutex + TEST_INSTANCE path planning. Now implemented in CLAUDE.md + csproj.
- [00_START_HERE.md](00_START_HERE.md) **[2026-03-25]** — Project onboarding. [SUPERSEDED by docs/README.md + CLAUDE.md]

### Quick Reference & Documentation (Iter-19-25)
- [WORKLOG.md](WORKLOG.md) — Session work diary. [STATUS: Historical log]
- [MINIMAL_FIX_PLAN.md](MINIMAL_FIX_PLAN.md) **[2026-03-25]** — Focused scope plan (Iter-19). [REPLACED-BY: CLAUDE.md Agent Operational Rules]
- [RUN_HIDDEN_DESKTOP_TEST_NOW.md](RUN_HIDDEN_DESKTOP_TEST_NOW.md) — Execution quickstart. [SUPERSEDED by iter-30+ discoveries (Parsec failure)]
- [HIDDEN_DESKTOP_TEST_PLAN.md](HIDDEN_DESKTOP_TEST_PLAN.md) — Plan only; execution failed on Parsec.
- [POWERSHELL_SETUP.md](POWERSHELL_SETUP.md) — PowerShell environment notes. [SUPERSEDED by docs/setup/SETUP.md + scripts/]
- [DOORSTOP_SOURCE_REFERENCE.md](DOORSTOP_SOURCE_REFERENCE.md) — SketchfabDl + YamlSchema reference (pre-refactor). [HISTORICAL]
- [FUZZING.md](FUZZING.md) — Fuzzing strategy note. [REPLACED-BY: ParameterizedTests/ + fuzz.yml]

### Planning & Analysis (Iter-16-23, late Mar-early Apr)
- [coverage-95-plan.md](coverage-95-plan.md) **[2026-04-01]** — Target 95% line coverage plan. [STATUS: v0.24.0 achieved 95%+]
- [polyglot-build-workflow.md](polyglot-build-workflow.md) **[2026-04-01]** — C# + Rust + Zig + Python orchestration. AssetsTools.NET + RustAssetPipeline + AssetctlPipeline + server.py.
- [phase-3d-integration-checklist.md](phase-3d-integration-checklist.md) **[2026-04-01]** — Phase 3d scope (UI + asset optimization). Now M11 complete in v0.24.0.

### Test & Release Logs (Iter-21-26, early Apr)
- [test_run_20260408.md](test_run_20260408.md) **[2026-04-08]** — CI test snapshot (2 workflows, build + unit tests green).
- [v0.17.0-RELEASE-REPORT.md](v0.17.0-RELEASE-REPORT.md) **[2026-04-08]** — v0.17.0 release notes (ancient, pre-audit-rotation). [HISTORICAL]
- [parallel_automation_test_20260411.md](parallel_automation_test_20260411.md) **[2026-04-11]** — Parallel subagent automation test. [STATUS: Methodology validation]

---

## Investigations & Deep Dives (By Topic)

### Isolation Infrastructure (Tier 1-3 cascade)
1. [AGENT_ISOLATION_RESEARCH.md](AGENT_ISOLATION_RESEARCH.md) — Architecture (Win32 + ECS considerations)
2. [HIDDEN_DESKTOP_PROTOTYPE.md](HIDDEN_DESKTOP_PROTOTYPE.md) — Win32 CreateDesktop prototype (BROKEN on Parsec)
3. [virtual_display_research_20260327.md](virtual_display_research_20260327.md) — VDD research (future Tier 1)
4. [isolation_layer_implementation_report.md](isolation_layer_implementation_report.md) — playCUA backend (CURRENT Tier 2)
5. [playCUA_integration_audit.md](playCUA_integration_audit.md) — Full tool surface (capture, input, process, analysis)
6. [playcua_phase3_5_spec.md](playcua_phase3_5_spec.md) — Phase 3.5 finalization spec

### Multi-Instance Testing Path
1. [MULTI_INSTANCE_RESEARCH.md](MULTI_INSTANCE_RESEARCH.md) — Early exploration
2. [HIDDEN_DESKTOP_CONCURRENT_INSTANCES_FINAL_REPORT.md](HIDDEN_DESKTOP_CONCURRENT_INSTANCES_FINAL_REPORT.md) — Proof of concept (2 instances)
3. [CONCURRENT_INSTANCES_IMPLEMENTATION_STATUS.md](CONCURRENT_INSTANCES_IMPLEMENTATION_STATUS.md) — Implementation checklist
4. [2026-04-25-steamless-multi-instance-audit.md](2026-04-25-steamless-multi-instance-audit.md) — Current path via TEST_INSTANCE

### Asset Pipeline & Visual Validation
1. [bundle_build_report_2026-03-25.md](bundle_build_report_2026-03-25.md) — Asset bundle compile log
2. [zig_module_creation_report.md](zig_module_creation_report.md) — Zig decimation module prototype
3. [2026-04-24-asset-swap-truth-audit.md](2026-04-24-asset-swap-truth-audit.md) — In-game rendering validation

### MCP Bridge & Protocol
1. [CODE_COMPARISON.md](CODE_COMPARISON.md) — C# vs Python implementations
2. [fastmcp_jsonrpc_protocol_analysis.md](fastmcp_jsonrpc_protocol_analysis.md) — σ-frame protocol deep dive
3. [mcp_pytest_suite_report_20260331.md](mcp_pytest_suite_report_20260331.md) — Python unit tests

### External Tools & Ecosystem
1. [phenocompose_nvms_investigation.md](phenocompose_nvms_investigation.md) — Multi-tier VM orchestration (roadmap)
2. [phenocompose_integration_technical.md](phenocompose_integration_technical.md) — Integration phases (roadmap)
3. [steamworks-goldberg-investigation.md](steamworks-goldberg-investigation.md) — Goldberg Emulator as Steamless alternative

### Proof System & Validation
1. [PROOF_OF_COMPLETION_20260420.md](PROOF_OF_COMPLETION_20260420.md) — Remotion reels + CLI proof
2. [TRACEABILITY_VERIFICATION_20260420.md](TRACEABILITY_VERIFICATION_20260420.md) — Story→test→docs linkage
3. [2026-04-24-kimi-judge-completion.md](2026-04-24-kimi-judge-completion.md) — External VLM integration

### Build & Environment
1. [environment_compatibility_matrix_20260330_091430.md](environment_compatibility_matrix_20260330_091430.md) — Baseline compatibility (Mar 30)
2. [environment_compatibility_fix_applied_20260330_093200.md](environment_compatibility_fix_applied_20260330_093200.md) — BepInEx workaround
3. [environment_compatibility_final_20260330.md](environment_compatibility_final_20260330.md) — Final resolution (WSL2 for Windows CI)

---

## Special Files & Governance

### Methodology & Roadmap
- [INFRASTRUCTURE_GAPS_FINAL_STATUS_20260331.md](INFRASTRUCTURE_GAPS_FINAL_STATUS_20260331.md) **[2026-03-31]** — Iter-31 infrastructure summary. HiddenDesktop proto delivery complete, Parsec render failure documented.
- [2026-04-25-audit-rotation-session-summary.md](2026-04-25-audit-rotation-session-summary.md) **[2026-04-26]** — Complete methodology description. Lens-rotation + Pattern Catalog + CI gates formalized. [CANONICAL for audit methodology]

---

## File Size & Organization Summary

| Category | Files | Total Size | Notes |
|----------|-------|-----------|-------|
| **Active (2026-04-26+)** | 13 | ~110 KB | Retrospectives, release snapshots, recent audits |
| **Audit-Rotation (2026-04-24-25)** | 8 | ~65 KB | Phase 3 close, infrastructure pivot, methodology lock |
| **PhenoCompose & Isolation** | 5 | ~48 KB | Roadmap docs (v0.25.0+) + playCUA integration |
| **Hidden Desktop Research** | 8 | ~110 KB | Prototypes, test plans, P/Invoke reference (DEPRECATED) |
| **Game Automation & Bridge** | 4 | ~26 KB | MCP protocol, test automation, asset validation |
| **Build & Environment** | 3 | ~36 KB | Compatibility matrix, environment fixes |
| **Validation & Testing** | 3 | ~38 KB | Validator system, test infrastructure |
| **Foundation & Setup** | 10 | ~80 KB | Project onboarding, architecture, research |
| **Quick Reference** | 7 | ~30 KB | Worklog, plans, setup notes |
| **Historical Release** | 5 | ~45 KB | v0.17.0 + prior validation snapshots |
| **TOTAL** | 84 | ~588 KB | — |

---

## How to Use This Index

1. **First time here?** Start with [00_START_HERE.md](00_START_HERE.md) (onboarding), then jump to **Active Development** section.

2. **Looking for methodology?** See [2026-04-25-audit-rotation-session-summary.md](2026-04-25-audit-rotation-session-summary.md) (formalized lens-rotation) or [iter-1-to-119-retrospective.md](iter-1-to-119-retrospective.md) (complete results).

3. **Investigating infrastructure?** Browse **Isolation Infrastructure** or **Multi-Instance Testing Path** subsections under **Investigations & Deep Dives**.

4. **Checking release status?** See [v0.24.0-execution-status.md](v0.24.0-execution-status.md) (latest) or [PROOF_OF_COMPLETION_20260420.md](PROOF_OF_COMPLETION_20260420.md) (proof bundles).

5. **Needing old API/proto details?** Check **Superseded Historical Archive** (marked with `[REPLACED-BY: ...]` or `[STATUS: ...]`).

---

## Cleanup Notes (Not Performed)

- **No files deleted** — per DINOForge governance (all artifacts retained for traceability).
- **No archival subdirs created** — kept flat structure for simplicity; this INDEX provides topical grouping.
- **Cross-linked to TRUTH_TABLE.md** — authoritative task closure log; these docs provide narrative + supporting research.

---

**Last audit**: 2026-05-18 — Catalogued 84 files, identified 13 active, 8 audit-rotation, 63 superseded/historical.
