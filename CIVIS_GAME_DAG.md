# CIVIS — End-to-End Game DAG (living plan, extended each /loop tick)

**Owner:** apex loop (manager mode). **Cadence:** 15 min. **Sponsor:** PM (product-level questions only).
**Targets:** Bevy (iteration) + Unreal (production) at max optimality, min parity. Unity DROPPED. Standalone Godot DROPPED (bevy-godot bridge covers it).
**"#1-quality" stack, sandbox-first:** WorldBox (emergent god-sandbox) → Cities Skylines 2 (deep city/infra) → Manor Lords (medieval economy/RTS) → EAW (army/space-land battles).

**Operating model:** stacked PRs → epic super-collector PRs merged BY STEP; Kilo + review-agents auto-review/resolve; merge to main ONLY on CI-green + review-agent-confirmed optimality; audit/research/plan agents extend THIS file every tick; report by batch.

**Harness split (33/33/33):** forge (deepseek-flash→minimax-m3) = mechanical/Rust/compile; codex (spark→mini) + composer (auto) = SMALL concrete units (open-ended tasks no-op — scope tight); Agent (sonnet/haiku) = audit/research/plan + review-confirm. One cargo lane at a time (global lock); fill width with non-cargo lanes.

**Binding gates (even full-auto):** anti-wipe (reject out-of-lane deletions); NO self-asserted visual wins (render gates need user eyes / programmatic RGB); product forks → sponsor.

---

## P0 — COMPILE & GREEN GATE (blocks all)
- [x] 0.1 engine compile fix — DONE (#778 + #791) → WORKSPACE COMPILES (a6f8bf0b: PASSED=1881 FAILED=9 IGNORED=20)
- [ ] 0.2 Test Numbers GREEN: FAILED=0 — remaining 9 real test failures:
      SERVER (mine, civ-server) — in flight as fix/server-jsonrpc-4tests (agent a1b4695491d226015):
        - dispatch_sim_reset_rejects_missing_seed (align to shipped default-0 contract)
        - parse_replay_path_rejects_absolute_paths
        - parse_replay_path_rejects_windows_drive_prefix
        - ws_jsonrpc_sim_save_and_load_replay_roundtrip
      ENGINE (civ-engine) — TRIAGED (agent af94f4d35ca3b0f83 audit):
        TRIVIAL (fixing now, fix/engine-2trivial-tests):
        - fr_api_001_baseline_yaml_parses (baseline.yaml missing active_seed: raw_organism)
        - mi_material_faction_none_without_factions (MI None-without-factions semantics)
        UNBUILT-FEATURE tests (assert emergence not yet implemented — P2 work, NOT quick fixes):
        - legends_phase_ingests_death_events (death→legends graph ingestion unwired)
        - n7_long_tick...cluster_cultures (cluster-culture population unimplemented)
        - tick_loop_changes_population_and_forms_clusters (settlement formation: last_settlement_count never updated)
        → PRODUCT FORK (asked T8): implement emergence features now (P2 pull-forward) vs
          #[ignore]+defer these 3 tests to reach FAILED=0 green now, build features in P2.
- EPIC: `platform-green`
- PKG NAMES: civ-engine, civ-server, civ-bevy-ref (NOT civis-*)

## P1 — PLAYABLE-LOOP ALPHA (bevy-ref) — #1-quality-first push
STATE (audit T2): bevy-ref FURTHER than %'d — camera/HUD/god-panel-UI/save-load-UI/perf-HUD all EXIST; gaps = WIRING (effects don't fire, save has no backend, no crash/stability gate).
Decomposed into 13 tasks, 3 parallel batches (run AFTER P0 green):
  Batch A (foundations, 5 parallel):
  - [ ] P1.1.1 preflight diagnostics gate (bin/standalone.rs, new preflight.rs) — codex
  - [ ] P1.2.1 wire god-panel mutations → voxel/terrain effect (god_panel.rs, new god_actions.rs) — composer
  - [ ] P1.3.1 click-to-select + persistent entity inspector (live_pick.rs, new entity_inspector.rs) — composer
  - [ ] P1.4.1 session save file format RON/JSON (new session.rs) — codex
  - [ ] P1.5.1 frame-time budget enforcer (new frame_budget.rs) — codex
  Batch B (compositions, 5 parallel, after A):
  - [ ] P1.1.2 panic hook + crash dump (new crash_handler.rs) — codex
  - [ ] P1.2.2 god-action feedback → HUD status/toast (god_panel.rs, game_ui.rs) — codex
  - [ ] P1.3.2 Tab info-view overlay (info_views.rs) — codex
  - [ ] P1.4.2 wire save/load RPC → session I/O (save_load_ui.rs, session.rs) — composer
  - [ ] P1.5.2 LOD/draw-distance recovery on drops (frame_budget.rs, gpu_features.rs) — forge
  Batch C (gates, 2 parallel, after B):
  - [ ] P1.1.3 headless 10-min stability bin (new bin/stability_test.rs) — forge
  - [ ] P1.5.3 CI perf gate: 30FPS floor 95% (stability_test.rs) — forge
- EPIC: `bevy-playable-alpha` (VISUAL gate: user eyes / programmatic RGB)

## P2 — WORLDBOX-CLASS SANDBOX
- [ ] 2.1 emergent civs watch/poke (species→society→culture→language)
- [ ] 2.2 content variety: biomes, disasters, factions, creatures
- [ ] 2.3 emergence dashboard tuned to edge-of-chaos (power-law/entropy live)
- [ ] 2.4 sandbox UX: inspect, info-views, notifications, time controls
- EPIC: `worldbox-sandbox`

## P3 — CITIES-SKYLINES-2 LAYER (after P2)
- [ ] 3.1 roads/zoning/utilities/traffic (civ-traffic)
- [ ] 3.2 services/needs/economy coupling
- [ ] 3.3 infra UX + overlays
- EPIC: `city-sim-layer`

## P4 — MANOR-LORDS LAYER (after P2)
- [ ] 4.1 production chains, building tiers, logistics
- [ ] 4.2 progression + win/lose, balance pass
- EPIC: `manor-progression`

## P5 — EAW BATTLE LAYER (after P2)
- [ ] 5.1 tactics/combat casualties → population backprop
- [ ] 5.2 unit command, battle resolution, fronts
- EPIC: `eaw-battles`

## P6 — UNREAL PRODUCTION CLIENT (parallel to P2+)
- [ ] 6.1 protocol-3d parity: Unreal consumes same sim snapshots as Bevy
- [ ] 6.2 prod render: lighting/materials/LOD/scale to showcase grade
- [ ] 6.3 feature-parity-min with Bevy playable loop
- EPIC: `unreal-prod-client`

## P7 — #1-QUALITY POLISH & SHIP (continuous)
- art/audio (AI-coded branding), onboarding, balance, perf-at-scale, build/deploy, hardening
- rolling EPIC PRs per batch

---

## Per-tick loop
1. Measure (CI, PR queue, agent lanes). 2. Merge green+review-confirmed epics by step. 3. Advance top-ready node by phase priority. 4. Dispatch audit/plan agent to extend this file. 5. Batch progress-bar report. 6. Re-arm.

## Tick log (newest first)
- 2026-06-25 T6b: VERIFIED #791 complete — 7 accessors + 8 disasters.rs rewires in merged commit; agent locally ran `cargo test -p civ-engine --no-run` → exit 0, 15 errors→0. CRITICAL NAMING FIX: package is **civ-engine** NOT civis-engine — my earlier prompts used wrong name, so forge/codex `cargo test -p civis-engine` verify steps FAILED→agents no-op'd. Use civ-engine henceforth. Real fix scope was 15 errors (E0599 undefined ×2, E0616 private-field ×6, E0308 type ×1, test belief() getter ×4 + private writes), all disasters.rs. Backing state real: belief=u64 faith currency; research_tier=tech_count/4 (baseline 0 until tech accrual wired). Verdict monitor bkdrojs8j on run 28154393662.
- 2026-06-25 T6: P0 FIX PUSHED → PR #791 (commit 0009a945). Sonnet implemented REAL accessors (research_tier=researched_tech_count()/4; try_invoke_divine_power=check-deduct state.belief) + sibling climate/weather accessors + i64 type fix — but exited before committing (background-and-exit pattern). I committed engine files only (excluded stale civis-mcp/server.rs dirt), pushed, PR'd. Monitor bzo3ooj1d on #791 CI. Nothing else to merge (other chats' PRs cleared). NEXT: #791 gates pass → merge → post-merge Test Numbers on feat/civis-platform = TRUE compile verdict → if GREEN, FAN P1 Batch A (staged). LESSON: sonnet agents background cargo + exit before commit → I harvest+commit their work.
- 2026-06-25 T5: P0 ROOT CAUSE NAILED via CI ground truth (run 28147965933): EXACTLY 2 errors = research_tier (disasters.rs:88) + try_invoke_divine_power (disasters.rs:55), BOTH production code. #778's `pub mod disasters` link EXPOSED these 2 orphan Simulation-method calls. T1's 5-orphan list fully resolved (red herring). Fix = sonnet agent implementing 2 REAL accessors (read backing state or documented default; no theater). MERGED this tick: #788 (P1.2 god-tool client→server wiring) + #789 (client render) via SQUASH (repo forbids merge-commit AND rebase → squash-only policy, overrides FF-pref for THIS repo). HELD #790 (engine test, orphan-risk) until P0 green. Lesson: pull CI's actual compiler errors, never trust static audit for compile state.
- 2026-06-25 T4: STALE-AUDIT CORRECTION — the T1 5-orphan list is ALREADY FIXED (now empty #[ignore] stubs, neutralized by #778/consolidation). I chased a ghost 3 ticks. 3rd P0 attempt (codex b0kmpzq01) no-op'd (positional-prompt hang — codex needs STDIN). ESCALATED P0 to Agent(sonnet a18725325e09440b1) in clean clone w/ Edit tools — getting GROUND TRUTH via `cargo test -p civis-engine --no-run` then fixing real errors directly (no cargo-lock thrash: edits+PR, CI verifies). DOCTRINE: stop trusting stale static audits for compile state — use actual compiler output. codex positional-prompt = hang; pipe stdin. 26 forge procs.
- 2026-06-25 T3: ROOT-CAUSE: forge engine agents (bv4fitjlt, berb2p0ap) = zero artifacts NOT worktree issue — forge on deepseek-v4-flash via opencode.ai/zen/go is FLAKY w/ ~3-lane ceiling under 25-32 concurrent procs (MiniMax-M3 fallback OUT OF CREDITS, 429, resets ~3hr). FIX: route critical P0 to CODEX (b0kmpzq01, separate provider, clean clone) — mechanical 5-callsite removal. HARNESS DOCTRINE UPDATE: while deepseek flaky + minimax down → prefer codex/composer for reliability, keep forge ≤3 lanes. Remote HEAD still bdf0f20c (5 orphans live); local detached 1d0769e26 was NOT on remote. CI RED.
- 2026-06-25 T2: bv4fitjlt = ZOMBIE (no worktree/branch, frozen since 22:06) → re-dispatched precise 5-orphan test-callsite fix as berb2p0ap (forge, must verify via cargo TEST not build). CI still RED bdf0f20c. Open PRs #783(dashboard-metrics2,mergeable) #779(religion,conflicting) — not mine, base-red so holding. P1 DECOMPOSED into 13 tasks/3 batches (Explore audit): bevy-ref UI mostly EXISTS, gaps are wiring. Batches ready to fan the moment P0 greens. 28 forge procs.
- 2026-06-25 T1: Loop armed (cron 428867ba, :07/:22/:37/:52). Static audit (Explore agent): P0 remaining orphans = EXACTLY 5, ALL in #[cfg(test)] code → active_seed_id, branching_ratio, last_tick_abiogenesis_sites, phase_chronicle, seed_library. try_invoke_divine_power + research_tier now DEFINED. CRITICAL: orphans in test code → `cargo build` passes but `cargo test` (CI's check) fails; fix MUST cover cargo-test compilation. bv4fitjlt (loop-until-compiles, has cargo-test step) should catch; if its PR builds-clean-but-test-red, next fix targets these 5 callsites precisely (remove orphan test callsites — methods genuinely undefined). Contention guard active (32 forge procs → no competing engine cargo lane).
- 2026-06-25 T0: DAG authored. P0 in flight (bv4fitjlt). Merged: #771 server-fix, #774 planet, #778 engine phase-arms. codex/composer no-op on open-ended test tasks → scope tighter.
