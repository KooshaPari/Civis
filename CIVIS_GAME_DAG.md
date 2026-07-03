
## P2 — WORLDBOX-CLASS SANDBOX (decomposed T31, fan once cargo frees)
- [x] 2.1 emergent civs (settlements/cultures/legends landed via #813 + emergence phases)
- [ ] 2.2 content variety (concrete tasks):
  - [ ] 2.2a biome variety: expand geology biome set + spawn affinity (crates/planet) — codex
  - [ ] 2.2b disaster variety: extend phase_disasters beyond wildfire (quake/flood/storm thresholds) (crates/engine disasters.rs) — codex
  - [x] 2.2c creature/species variety: more NamedSeeds + divergence presets (crates/genetics, scenarios) — cursor
- [ ] 2.3 emergence dashboard LIVE in HUD: surface power-law alpha + entropy + structure-count + novelty from civ-emergence-metrics into a bevy HUD panel (clients/bevy-ref + read metrics) — cursor [HIGH VALUE: makes sandbox legible]
- [ ] 2.4 sandbox UX: time controls (pause/speed already partial) + notifications/event-feed panel surfacing emergence_feed (clients/bevy-ref) — cursor
- EPIC: `worldbox-sandbox`

## Tick log (newest first)
- 2026-06-25 T31: P1 GREEN-CONFIRMED (1779319c success, all P1 on main). build-prep (bucy8plbx) still compiling bevy standalone (long, holds cargo lock) → no new cargo lane this tick (guard). EXTEND: decomposed P2 into concrete fan-ready tasks (2.2a/b/c content, 2.3 emergence-dashboard-HUD=high-value, 2.4 sandbox-UX). NEXT: build-prep reports launch cmd → relay to USER for async visual verify; then fan P2.3 dashboard + content once cargo frees.

- 2026-06-25 T31b: LAUNCH CMD RESOLVED — bin is `civ-standalone` (NOT standalone); my earlier `--bin standalone` matched nothing → cargo exit-0 no-op. Correct: `CARGO_TARGET_DIR=E:/civis-target cargo run -p civ-bevy-ref --bin civ-standalone --features bevy,egui`. build-prep bucy8plbx producing it (1 cargo proc = only lane, guard honored). VISUAL-VERIFY CHECKLIST for user: god-panel actions produce visible world effects + HUD status/toast; left-click selects entity → inspector panel; Tab → info-view overlay; F5 → save/load; ~2min no crash. Relay once build-prep confirms binary.

- 2026-06-25 T32: build-prep (bucy8plbx) PROGRESSING — E:/civis-target populating (bevy cold-build, slow under 17 cargo procs). No binary yet; will notify. P1 green holds (1779319c). Only #838 open (other-chat wt-cur-law). CARGO SATURATED (17 procs) → held all cargo lanes (guard); P2 fan deferred until build-prep frees lock + contention drops. Legit wait-tick: the user's runnable build is compiling; don't thrash. NEXT: build-prep binary → relay launch cmd to user; then P2.3 dashboard + content once cargo frees.

- 2026-06-25 T33: cargo dropped to 4 (window) → building civ-standalone DIRECTLY (br54ysqcw, correct bin+features bevy,egui) to E:/civis-target; build-prep cursor lane produced no binary (slow/struggling). P2.3 INSIGHT: emergence dashboard is NOT from-scratch — PR #350 landed civ_emergence_metrics crate (Shannon entropy, structure-count, power-law fit, branching σ edge-of-chaos) + design doc + emergence_metrics.rs runtime sampler (sim.emergence JSON-RPC returns EmergenceSample). So P2.3 = SURFACE existing metrics into bevy HUD panel (client-side, reads sim.emergence). Re-scope P2.3 as a thin client task. NEXT: binary → relay launch to user; fan P2.3 HUD-surface + content lanes once build frees cargo.
