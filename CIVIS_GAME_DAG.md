# CIVIS — Master DAG v2 (canonical, indefinitely extendable)

> **Purpose:** single-source-of-truth task graph for the Civis godgame. Extends without rewriting: append a `P<n>` layer, an `FR-XXXX`, an `ADR-XXXX`, or a `phase_*` emergence phase — DAG stays canonical.
> **Audience:** humans + task agents. Every node carries `id · title · scope · deps · lanes · state · FR/ADR ref`. Every layer has `parallel-safe` and `extendability hooks`.

## 0 · Read me first

- **The DAG is the spec.** When code, ADRs, FR specs, and PR titles diverge, the DAG wins until updated.
- **Extend, don't rewrite.** Append-only: new phases go after P5, new FR-XXXX link to the layer they belong to.
- **Parallel-safe lanes** are the default. Each lane is independent unless its `deps:` line is non-empty.
- **Substrate-faithful rule:** any new system (godverb, tool, panel) routes through `sim.god_action` / MCP / JSON-RPC — never parallel paths.
- **Status icons:** ✅ done · 🟡 in flight · ⏸ queued · ❌ blocked · ◊ stretch.

## 1 · Layer model (P0 … Pn)

```
P0  Foundation            voxel · engine · simulation · phenotype-gfx · FixedI64
P1  Substrate             emergence phases · physics-coupling · save/load · JSON-RPC · godverbs
P2  Bridge                MCP (32 tools) · egui · 52 civ_* verbs · HolocronCmdK · click-to-fire e2e
P3  Pillar systems (FR-*) 22+ feature areas: market, language, religion, law, war, psyche, etc.
P4  Polish + Holocron     Holocron Keycap (Phases 3-5) · tutorial · trade v2 · QA hygiene
P5  Stretch / open-ended  ◊∞  (modding, multiplayer, web authoring L2, scenario editor)
```

Each layer is a self-contained DAG fragment. New FR-XXXX or ADR-XXXX can be inserted at any layer without breaking the graph.

## 2 · Phase ordering (deterministic substrate)

The `Simulation::tick` runs **29 phases** in fixed order (`crates/engine/src/engine.rs:2157-2185`). Adding a phase = append to the order; removing = mark `#[deprecated]`.

| # | phase | dep | lane |
|---|---|---|---|
| 1 | `planet` | — | core |
| 2 | `voxel` | planet | core |
| 3 | `compact` | voxel | core |
| 4 | `buildings` | compact | core |
| 5 | `life` | buildings | emergence |
| 6 | `tactics` | life | core |
| 7 | `production` | tactics | core |
| 8 | `citizen_lifecycle` | production | emergence |
| 9 | `research` | citizen_lifecycle | emergence |
| 10 | `tech` | research | emergence |
| 11 | `economic_focus_pre` | tech | emergence |
| 12 | `faction_decisions` | economic_focus_pre | emergence |
| 13 | `economy` | faction_decisions | core |
| 14 | `market` (FR-MARKET) | economy | pillar |
| 15 | `disasters` | market | pillar |
| 16 | `diplomacy` | disasters | pillar |
| 17 | `belief` | diplomacy | pillar |
| 18 | `unrest` | belief | pillar |
| 19 | `cohesion` | unrest | pillar |
| 20 | `social_mood` | cohesion | pillar |
| 21 | `stratification` | social_mood | pillar |
| 22 | `institutions` | stratification | pillar |
| 23 | `economic_focus` | institutions | emergence |
| 24 | `policy` | economic_focus | core |
| 25 | `military` | policy | core |
| 26 | `emergence` | military | emergence |
| 27 | `language` | emergence | pillar |
| 28 | `sentience` | language | pillar |
| 29 | `diffusion` | sentience | emergence |
| 30 | `audio` | diffusion | pillar |

**Extend hook:** append new `phase_X(&mut self)` + `phase_X,` in the order map. Phase stays deterministic.

## 3 · Pillar epics (P3) — all shipped ✅

| FR | name | lane | PRs | ADR |
|---|---|---|---|---|
| FR-MARKET | emergent markets | pillar | #804 | ADR-018 |
| FR-LANGUAGE | emergent lexicon | pillar | #829 | ADR-014 |
| FR-DIPLOMACY | emergent factions | pillar | #806 | ADR-015 |
| FR-ERA | era progression | pillar | #808 | ADR-016 |
| FR-RELIGION | belief wiring | pillar | #798+#801 | ADR-016 |
| FR-LAW | customary law | pillar | #832+#838 | ADR-016 |
| FR-CULTURE | drift culture | pillar | #839 | ADR-015 |
| FR-MUSIC | emergent motifs | pillar | #840 | ADR-018 |
| FR-DISASTER | disaster mechanics | pillar | #868+#794 | ADR-018 |
| FR-CIV-LIFE | citizen lifecycle | pillar | #858 | ADR-018 |
| FR-CIV-GOV | cohesion/unrest/inst/strat | pillar | #814+#787+#775 | ADR-016 |
| FR-CIV-SCALE | LOD + streaming | pillar | #857+#836 | ADR-019 |
| FR-TECH | tech gating | pillar | #904 | ADR-018 |
| FR-CIV-LEGENDS | legends engine | pillar | #909 | ADR-008 |
| FR-CIV-CA | material cellular automata | pillar | #907 | ADR-018 |
| FR-CIV-SPECIES | speciation | pillar | #888 | ADR-008 |
| FR-CIV-ARCH | emergent layouts | pillar | #884 | ADR-018 |
| FR-CIV-LLM | LLM garnish hook | pillar | #882 | ADR-006 |
| FR-CIV-LANG | lexicon drift | pillar | #885 | ADR-014 |
| FR-EMERGENCE | quality tests + dashboard | test | #783+#807+#790 | ADR-018 |
| FR-EMERGENCE-WIRING | wire dormant | substrate | #775+#776 | ADR-020 |
| FR-ECON | emergent trade | core | #723+#827 | ADR-018 |
| FR-MCP | 32 MCP tools | bridge | #769+#797 | ADR-009 |
| FR-CLIENT | click-to-fire + HUD + menu | bridge | #766+#788+#833+#757 | ADR-019 |
| FR-SAVELOAD | round-trip | core | #793+#834+#864 | ADR-003 |
| FR-INFRA | quality-infra + governance | ops | #863 | ADR-009 |
| FR-HOLOCRON | keycap palette | bridge | #845+#851+#866 | ADR-012 |
| FR-ACCESSIBILITY | l10n | bridge | #863 | ADR-021 |

**Extend hook:** add a new `FR-XXXX-NAME` row + ADR-XXXX ref. Add lane column if it's a new pillar.

## 4 · Pillar systems (P3) — what's queued / partial

| FR | name | state | ETA | lane |
|---|---|---|---|---|
| FR-VEHICLES | logistics | ⏸ | Q3 | pillar |
| FR-MODDING | mod SDK | ⏸ | Q4 | stretch |
| FR-MULTIPLAYER | co-op | ⏸ | Q4 | stretch |
| FR-WEB-L2 | web authoring | 🟡 | Q3 | stretch |
| FR-ORACLE | LLM oracle gate | 🟡 #926 | Q3 | bridge |
| FR-COMBAT | warfare depth | 🟡 #845 | Q3 | pillar |
| FR-TUTORIAL | onboarding | 🟡 #860 | Q3 | polish |

## 5 · P4 — Polish + Holocron Keycap sequence

| phase | PR | scope | state |
|---|---|---|---|
| 0 | #845 | `docs/design/HOLOCRON_KEYCAP_UI.md` | ✅ |
| 1 | #851 | `crates/holocron/` — VerbRegistry + VerbDescriptor | ✅ |
| 2 | #866 | `holocron_panel.rs` — Panel + CommandKOverlay | ✅ |
| 3 | (next) | context-aware ranking — sim-state → verb-boost fn | ⏸ |
| 4 | (next) | persistence — use_stats + recent → save/load | ⏸ |
| 5 | (stretch) | narrative — unlockable verbs + Holocron lore | ⏸ |

**Extend hook:** append Phase 6, 7, … each becomes a new FR row + a DAG node + a Holocron substrate extension.

## 6 · P5 — Stretch / open-ended ◊∞

The P5 layer is **deliberately unbounded**. It accepts:

- New FR-XXXX (any new feature area)
- New ADR-XXXX (new architecture decision)
- New client (e.g. mobile, console, web)
- New transport (WebSocket, gRPC, QUIC)
- New persistence backend (Postgres, S3, etc.)
- Modding SDK
- Scenario editor / worldgen
- Multiplayer / co-op
- Web L2 authoring per ADR-009

Each addition: append a `P5.<n>` sub-layer with the same fields (id, title, scope, deps, lanes, state, FR/ADR ref).

## 7 · Agent-dispatch lanes (parallel-safe)

Each lane is independent; agents self-report into `CIVIS_GAME_DAG.md` tick log. Mutex rules apply only where noted.

| lane | purpose | scope | mutex |
|---|---|---|---|
| `merge-harvest` | harvest MERGEABLE PRs | global | none (parallel-safe) |
| `dispatch-lanes` | worktree-add + parallel features | per-FR | partition by lane-id |
| `build-retry` | cargo check/test/clippy | workspace | exclusive on `.cargo/`, `target/` |
| `oracle-gate` | LLM/quality-gate reviews | per-PR | exclusive on `.github/workflows/` |
| `spec` | ADR / FR / design docs | per-doc | none |
| `qa` | clippy/test/deny | workspace | exclusive on `target/` |
| `cleanup` | stash/worktree/target hygiene | global | none |

**Concurrency rule:** at most 1 lane mutates `Cargo.toml` / `Cargo.lock` per tick. At most 1 lane pushes to `main`. All others self-report into the DAG tick log.

## 8 · Tick log (append-only, agents self-report here)

| tick | merged | opened | closed-superseded | open-net | notes |
|---|---|---|---|---|---|
| T0 | #845 | — | — | 1 | Holocron design doc |
| T1 | #851 | — | — | 1 | Holocron VerbRegistry substrate |
| T2 | #866 | — | — | 1 | HolocronPanel + CommandKOverlay |
| T3 (current) | — | — | — | 2 | #860 tutorial, #926 oracle gate — rebase on main |

**Tick lifecycle:** agents write row to tick log on commit. DAG stays canonical. Drop the `T0..Tn` rows from older logs only if the DAG rolls into a new v3+.

## 9 · Extendability hooks (how to add)

| you want to add | edit | re-run |
|---|---|---|
| New FR (feature) | row in §3 / §4 + node in §10 | `git commit` → `gh pr` |
| New ADR (decision) | new `docs/adr/ADR-XXX-*.md` + row in §10 | `git commit` → `gh pr` |
| New emergence phase | `phase_X(&mut self)` in engine.rs + row in §2 | cargo check |
| New godverb | add to `crates/civis-mcp/src/server.rs` + descriptor in `crates/holocron/src/verbs.rs` | cargo check |
| New client | new dir under `clients/` + workspace member | cargo check |
| New lane | row in §7 + dispatch rule | update build-retry agent |
| New P5 sub-layer | append `P5.<n>` block in §6 | update DAG header |

## 10 · Appendix — atomic dependencies

### 10.1 · Crates (28 total, alphabetical)

```
civ-agents · civ-allegiance · civ-arch · civ-assets · civ-audio · civ-climate
civ-clients · civ-combat · civ-disease · civ-emergence · civ-engine · civ-fx
civ-genes · civ-gfx · civ-holocron · civ-hud · civ-infra · civ-i18n
civ-integration · civ-laws · civ-migration · civ-notify · civ-out-of-band
civ-persist · civ-plot · civ-render · civ-server · civ-shell · civ-tick
```

### 10.2 · Clients (3)

```
clients/bevy-ref       — primary local UI (egui + holocron + click-to-fire)
clients/web/dashboard  — pages-deploy (ADR-009)
clients/cli            — ops/CI helpers
```

### 10.3 · ADRs (26, indexed)

```
ADR-001  rust-crate-structure
ADR-002  joule-economy-as-allocator
ADR-003  deterministic-replay
ADR-005  adaptive-voxel
ADR-006  llm-event-sourcing
ADR-007  three-renderers
ADR-008  algorithmic-genetics
ADR-009  web-client-strategy + amendment (L2 authoring)
ADR-010  ca-tick-budget-guard
ADR-011  n-series-emergence-coupling
ADR-012  keycap-palette-design-system
ADR-013  hexagonal-architecture
ADR-014  language-emergence-via-phoneme-drift
ADR-015  faction-emergence-via-k-means-ideology-clustering
ADR-016  religion-emergence-from-needs-vector
ADR-017  web-l2-authoring-amendment
ADR-018  emergence-systems-coupling
ADR-019  rendering-substrate-selection
ADR-020  wire-dormant-emergence-phases
ADR-021  accessibility-and-l10n-strategy
ADR-bevy-vulkan-primary-backend
ADR-determinism-dropped
ADR-emergence-charter
ADR-voxel-streaming-scale
```

### 10.4 · Open PRs (T3)

| PR | title | state |
|---|---|---|
| #860 | feat(client): tutorial onboarding | CONFLICTING (rebase needed) |
| #926 | feat(oracle): hard gate | CONFLICTING (rebase needed) |

### 10.5 · Stale worktrees (9)

All locked (Windows file-handle residue from external swarm). Clean-up: `taskkill /F /IM cargo.exe / rustc.exe` then `git worktree remove --force <path>`. Out of band — handled on the next idle tick.

## 11 · Versioning & changelog

- **v1** (2026-06-25): initial P0..P5 frame, 5-tick scope.
- **v2** (2026-06-26, this file): added phase ordering table, pillar epics matrix, lane-dispatch rules, tick log, extendability hooks, atomic-deps appendix. Forever-extendable: append `P5.<n>` or `FR-XXXX` without breaking the graph.

End of DAG v2.