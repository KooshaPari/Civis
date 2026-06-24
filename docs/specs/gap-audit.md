# Civis Gap Audit — Everything Still Missing to Ship a Complete AAA/Manor-Lords-Grade Game

> **Status:** Honest gap audit (2026-05-31). Owner: Design/Audit. **Read-only on code** — this is an
> assessment artifact, no implementation. Companion to [`feature-matrix.md`](./feature-matrix.md),
> [`../research/competitive-benchmark.md`](../research/competitive-benchmark.md), and
> [`../design/master-roadmap.md`](../design/master-roadmap.md).
>
> **Stance: brutally honest.** The playable build today is **basic** and has **active render bugs** (the
> most recent commits are still fixing a stray center sphere + a water-billboard plane). The design corpus
> is broad and high-quality — **17 design docs** — but the gap between *designed* and *wired-into-the-engine
> *is the dominant fact of the project. Most of the moat (emergent depth) is **specced, not built**.

## How to read this

Every gap is tagged:

- **Status:** `designed-not-built` (spec exists, no/inert code) · `partially-built` (substrate exists, not
  surfaced/wired) · `not-even-designed` (no spec, no code).
- **Severity:** `BLOCKER` (cannot ship / pitch collapses without it) · `MAJOR` (clearly sub-AAA, players
  notice) · `MINOR` (polish).
- **Effort (agent terms):** S = ≤1 subagent wave · M = 2–3 parallel subagents · L = 3–5 subagent waves ·
  XL = multi-wave initiative.

### Method / evidence base

Derived from: the 17 `docs/design/*.md` docs; `crates/*` source sizes + `Cargo.toml` dependency edges;
`clients/bevy-ref/src/*` (52 files, ~20.6k LOC); `assets/` tree; `.github/workflows`; `git log`. Key
structural finding repeated throughout: **`civ-legends`, `civ-ai`, `civ-research`, `civ-species`,
`civ-genetics`, `civ-laws` are NOT in the engine or server dependency graph** — they compile in isolation
but do not run in the tick. `psyche.rs`/`social.rs` exist in `crates/agents` but **nothing in
`crates/engine` references them**. This is *the* implementation gap.

---

## 1. IMPLEMENTATION GAP — designed moat not wired into the engine

The 17 design docs and their wiring status. "Wired" = referenced in the engine tick *and* surfaced in-game.

| # | Design doc | Crate/substrate | Wired into engine? | Surfaced in-game? | Status | Severity | Effort |
|---|---|---|---|---|---|---|---|
| 1 | `legends-engine.md` | `crates/legends` (1.4k LOC, graph/query/worker) | **No** — not in engine/server deps | No | partially-built | **BLOCKER** | L |
| 2 | `psyche-social.md` | `crates/agents/{psyche,social}.rs` | **No** — engine doesn't call psyche/social | No | partially-built | **BLOCKER** | L |
| 3 | `civ-ai-crate.md` | `crates/ai` (1.1k LOC, provider/cache/pool) | **No** — not in engine/server deps | No (no narrator/naming) | partially-built | MAJOR | L |
| 4 | `polities-markets.md` | `crates/economy` ledger; shadow diplomacy | Economy: yes; polity emergence: no | Partial (diplomacy_ui) | partially-built | **BLOCKER** | XL |
| 5 | `warfare.md` | `crates/tactics` (2.7k LOC) | In engine deps | Not surfaced as RTS loop | partially-built | MAJOR | XL |
| 6 | `vehicles-logistics.md` | `crates/civ-traffic` (848 LOC) | In engine deps | Not surfaced / no desire-paths | partially-built | MAJOR | L |
| 7 | `species-sentience.md` | `crates/species`, `crates/genetics` | **No** — species/genetics not in engine deps | No | partially-built | MAJOR | L |
| 8 | `tech-engineering.md` | `crates/research`, `crates/laws` | **No** — research/laws not in engine deps | tech_tree_ui.rs exists (client) | partially-built | MAJOR | L |
| 9 | `audio-direction.md` | `crates/?` + `client/audio.rs` (286 LOC) | Client only | **No audio asset files at all** | partially-built | MAJOR | M |
| 10 | `onboarding-qol.md` | — | No | **No tutorial/onboarding in client** | designed-not-built | MAJOR | L |
| 11 | GPU-CA (voxel) | `crates/voxel` (2.1k LOC) + client voxel_sim | Yes (CPU); GPU-CA not confirmed | Partial (render bugs) | partially-built | MAJOR | L |
| 12 | `modding-platform.md` | `crates/mod-host` (3.3k LOC), `civlab-sdk` | In server deps | No mod UI / no workshop | partially-built | MINOR | L |
| 13 | `brush-tool-system.md` | client `terraform_brush`, `material_brush_ui` | Client | Partial | partially-built | MINOR | M |
| 14 | `ui-design-language.md` | client `ui_theme`, `ui_holo` | Client | Yes (themed) | partially-built | MINOR | S |
| 15 | `info-views.md` | client `info_views.rs` (755 LOC, ~26 refs) | Client | Scaffolded; most overlays BLIND (no producer fields) | partially-built | **BLOCKER** | L |
| 16 | `lighting-biomes-art.md` | client lighting_gi/atmosphere/materials | Client | Partial (warm/cool + biome surface) | partially-built | MAJOR | L |
| 17 | (no doc) general legibility/inspect | client `inspect.rs` (398 LOC) | Client | Partial; not fed by sim depth | partially-built | MAJOR | M |

**The moat is unbuilt.** The project's stated differentiator — *emergent psyche + social graph + legends/
histories + emergent polities/markets, made legible* — exists as **6 crates + 5 design docs that do not run
in the live simulation**. The engine ticks substrate (voxel/needs/economy/tactics/traffic/diffusion) but
**none of the depth-moat layers**. Until `civ-legends`, `civ-ai`, `species`, `genetics`, and the
`psyche/social` modules are added to the engine dependency graph and called each tick, the headline pitch
("everything emerges") is **invisible in the actual game**.

---

## 2. PLAYABLE FUNDAMENTALS — is the loop engaging or "basic"?

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Terrain renders correctly | partially-built | **BLOCKER** | M | Latest commits still fixing stray center sphere + water-billboard; "looks flat/unfinished" is the #1 dismissal risk (benchmark §3). |
| God-tools actually do something | partially-built | MAJOR | M | spawn_tools/terraform_brush exist; need verified instant-feedback loop (vision-verify, not log-verify). |
| Agent visibility | partially-built | MAJOR | M | civilian.glb exists; agents must be visible, selectable, inspectable, and *legibly doing* needs-driven behavior. |
| Core loop is engaging (not a tech demo) | partially-built | **BLOCKER** | XL | No goal/feedback/progression surfaced. Poking an inert world ≠ a game. The loop only becomes engaging once the depth-moat (§1) is wired and legible (§15). |
| Stable 60fps at any scale | not-validated | MAJOR | L | Perf designed (CIV-0500), never benchmarked. |

**Honest verdict:** the build is a **substrate viewer with render bugs**, not yet a game. Fundamentals
(render correctness → visible agents → working tools → a reason to keep playing) must land before any
breadth work pays off.

---

## 3. ART / BRANDING — does the world read AAA?

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| App icon | designed-not-built | MAJOR | S | `assets/icon/` dir exists but is **empty** (icon being added now). |
| Iconography (tool/material/HUD) | partially-built | MINOR | M | `ui/tool-icons`, `ui/material-icons`, `operations/iconography` exist — decent. |
| 3D model variety + quality | partially-built | MAJOR | L | 19 `.glb` (Kenney-style buildings/carts/civilian). Thin for AAA; need biome props, agent variety, orientation/scale QA. |
| Model orientation/scale correctness | partially-built | MAJOR | M | Recurring WSM3D-class risk (scale/orientation); needs a vision-verify pass. |
| VFX | partially-built | MINOR | M | `vfx.rs` exists; coverage/quality unknown. |
| Animation (agent skeletal/locomotion) | not-even-designed | MAJOR | XL | No animation system evident; static/billboard agents read as prototype. |
| Splash / main menu polish | partially-built | MINOR | M | `menus.rs` exists; AAA splash/menu not confirmed. |
| Marketing key art | not-even-designed | MAJOR | M | None. Required for any store page. |
| Trailer | not-even-designed | MAJOR | L | None. Required to sell an emergent game (it must be *shown*). |

**Verdict:** flat single-roughness world + 19 placeholder models + empty icon = reads as prototype. The
cheap, high-wow lighting/PBR closeout (`lighting-biomes-art.md`) is the highest-ROI art fix; bespoke models/
animation are the expensive long pole.

---

## 4. AUDIO — crate exists, content + wiring missing

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Music (adaptive score) | designed-not-built | MAJOR | M | `audio-direction.md` + kira (RND-007) + `client/audio.rs` exist; **zero `.ogg/.wav/.mp3` files in repo.** |
| SFX | designed-not-built | MAJOR | M | No sample content; no event→SFX wiring confirmed. |
| Ambience (biome/weather beds) | designed-not-built | MINOR | M | Designed only. |
| Spatial audio | designed-not-built | MINOR | M | Designed only. |

**Verdict:** audio is a **content + wiring** gap, not an architecture gap. A game that ships silent reads as
unfinished. Sourcing CC0/licensed beds + SFX and wiring the existing `audio.rs` is a self-contained M wave.

---

## 5. UX / QoL

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Onboarding / tutorial / first-run | designed-not-built | MAJOR | L | `onboarding-qol.md` exists; **no tutorial/onboard code in client.** |
| Tooltips / contextual help | partially-built | MAJOR | M | Inspect panel exists; rich tooltips minimal. |
| Undo / redo | designed-not-built | MAJOR | M | Benchmarked as a CS2 pain point to *beat*; absent. |
| Accessibility (colorblind/scaling/contrast) | not-even-designed | MAJOR | M | Not designed. Table-stakes for shipping. |
| Settings depth (graphics/audio/controls) | partially-built | MINOR | M | `settings_ui.rs` exists; depth unknown. |
| Localization / i18n | **not-even-designed** | MAJOR | L | **No i18n in client; no fluent/locale infra.** Strings hardcoded. Retrofit cost grows daily. |
| Controller support | not-even-designed | MINOR | L | Not designed. |
| Save-slot UI | partially-built | MINOR | M | `crates/save-db` + `server/saves.rs` exist; player-facing slot UI unconfirmed. |
| Rebindable hotkeys | designed-not-built | MINOR | M | Designed (S5.W2); not built. |
| Notifications / alert feed | partially-built | MINOR | M | `notifications.rs`/`event_feed.rs` exist; routing/camera-jump unconfirmed. |

**Standout:** **i18n is not even designed** — and string externalization is exponentially cheaper *before*
the UI grows. Flag now.

---

## 6. CONTENT BREADTH

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Scenarios / starts | not-even-designed | MAJOR | L | No scenario system; WorldBox/CS2 ship many. |
| Presets (worldgen/biome/difficulty) | not-even-designed | MINOR | M | None surfaced. |
| Achievements | not-even-designed | MINOR | M | None. |
| God-tool palette breadth | partially-built | MAJOR | L | Far from WorldBox's ~374 powers; thin palette. |
| Disasters / spawn-anything variety | partially-built | MAJOR | L | Partial (place_voxel/spawn/damage); needs breadth. |

---

## 7. DISTRIBUTION / SHIPPING

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Installer / packaging | partially-built | MAJOR | M | `.github/workflows/release.yml` + release-drafter exist; no per-OS installer/bundle confirmed. |
| Steam / itch store page + assets | not-even-designed | **BLOCKER (to ship)** | L | No store page, capsule art, screenshots, description. |
| Build pipeline / release automation | partially-built | MINOR | M | release workflows exist; artifact build/sign unconfirmed. |
| Auto-update | not-even-designed | MINOR | L | None. |
| Crash reporting / telemetry | partially-built | MAJOR | M | `civ-panic.log` panic-hook exists (local only); no aggregated crash reporting (e.g. Sentry-class, self-hosted). |
| EULA / credits / licenses | not-even-designed | MAJOR | S | None. Required (esp. for any third-party assets/models). |

---

## 8. MULTIPLAYER / NETWORK

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Co-op / spectator | not-even-designed | MINOR | XL | `crates/protocol-3d` + `server/ws_bridge.rs` are a **client↔server transport** (single-player streaming), NOT a multiplayer netcode layer. Determinism was *dropped* (memory), so lockstep MP is off the table; would need state-sync/rollback. **Recommend: explicitly out of scope for v1** unless the user states co-op/spectator is a goal. |

**Open question for the user:** is co-op or spectator a v1 goal? If not, mark MP out-of-scope and stop
carrying it as a gap.

---

## 9. QA / STABILITY

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Test coverage of new crates | partially-built | MAJOR | M | 82 files with tests across crates, but the depth crates (`legends`, `ai`, `psyche/social`) are unwired and likely under-tested at integration level. Repo's own bar is ≥90% cov. |
| Automated playtesting / soak | not-even-designed | MAJOR | L | No headless long-run sim harness for stability. |
| Perf profiling at scale | not-even-designed | MAJOR | L | 20mi/100k-agent/60fps unproven (CIV-0500 designed only). |
| Resilience: NaN/overflow guards | not-even-designed | MAJOR | M | **Determinism dropped → real floats/RNG welcome, but that makes NaN/Inf/overflow guards MANDATORY.** No evidence of clamp/guard discipline on the new emergent float-heavy systems (psyche vectors, tâtonnement, doctrine GA). A single NaN can poison the whole sim. |
| Render-regression catching | partially-built | MAJOR | M | F9/F10 capture harness exists; no automated visual-diff gate (the stray-sphere/water bugs shipped to the build). |

**Standout:** dropping determinism is fine for emergent variety, but it **raises** the bar on numeric
resilience. Guard rails (clamp, `is_finite` checks, saturating arithmetic) on every emergent float system
are a MAJOR, currently-missing safety net.

---

## 10. DOCS / COMMUNITY

| Gap | Status | Severity | Effort | Notes |
|---|---|---|---|---|
| Player-facing docs / wiki | not-even-designed | MINOR | M | VitePress site is dev/governance docs, not a player manual. |
| Modding docs (player-facing) | partially-built | MINOR | M | `modding-platform.md` is a spec; no how-to-mod guide. |
| Community / Discord | not-even-designed | MINOR | S | None. |

---

## 11. PROCESS — what's slowing delivery

| Gap | Status | Severity | Notes |
|---|---|---|---|
| Build-lock / worktree-base contention | ongoing | MAJOR | Commit history shows a full-disk git-corruption recovery + worktree-base churn. Disk pressure (target dir) + worktree checkouts are a known delivery drag (memory: build→E: HDD, worktree isolation expensive). Keep `cargo target-dir` off C:; prefer disjoint-file main-dir agents over many worktree checkouts. |
| Render bugs reaching the build | ongoing | MAJOR | No visual-regression gate (see §9). Bugs are caught by the user, not CI. |
| Depth crates drifting from engine | ongoing | **MAJOR** | Building crates in isolation without wiring them into the tick creates a growing **integration debt** — the single biggest process risk. Wire-as-you-build, don't accumulate unwired crates. |

---

## TOP-15 — most important missing things, ranked

Ranked by *shippability impact* (how much closing it moves Civis from "basic tech demo with render bugs"
toward "complete, legible, attractive, emergent game").

1. **Render correctness + visible/legible agents (playable fundamentals).** The build has active render
   bugs and reads flat. Nothing else matters if the world looks broken. `BLOCKER`.
2. **Wire the depth-moat crates into the engine tick** — `legends`, `psyche/social`, `species/genetics`
   into the engine dependency graph and called each tick. The moat currently *does not run*. `BLOCKER`.
3. **Info-view overlays fed by real producer fields + inspect-anything.** Scaffold exists (755 LOC) but
   most overlays are BLIND because the producing sim fields don't exist/aren't wired. Legibility is the #1
   credibility gap. `BLOCKER`.
4. **An engaging core loop** (goal → feedback → progression surfaced). Currently a sandbox viewer. `BLOCKER`.
5. **Lighting/PBR/biome visual closeout** — emissive lava, wet water, warm/cool grade. Cheap, high-wow,
   clears the "looks like a prototype" dismissal. `MAJOR`.
6. **Emergent polities/markets surfaced** (the 4X/society payoff). `BLOCKER` for the pitch.
7. **NaN/overflow/clamp guards across all emergent float systems** — mandatory now that determinism is
   dropped; one NaN poisons the sim. `MAJOR`.
8. **Audio content + wiring** — repo ships **zero** audio files; a silent game reads unfinished. `MAJOR`.
9. **App icon + marketing key art + trailer** — branding + the means to *show* emergence. Icon dir is
   empty. `MAJOR`.
10. **Onboarding / tutorial / tooltips / undo** — standard QoL entirely absent; required to be playable by
    anyone but the author. `MAJOR`.
11. **Visual-regression + headless-soak QA gates** — so render bugs and NaN crashes are caught by CI, not
    the user. `MAJOR`.
12. **Steam/itch store page + EULA/credits/licenses** — hard gate on actually shipping. `BLOCKER (to ship)`.
13. **Perf validation at scale** (20mi/100k agents/60fps captured benchmark). `MAJOR`.
14. **i18n/localization retrofit** — not even designed; cheapest if externalized now, before the UI grows.
    `MAJOR`.
15. **3D model variety + agent animation** — 19 placeholder models, no animation; the expensive long pole
    toward AAA look. `MAJOR`.

### The single most important gap

**#2 — the designed emergent-depth moat is not wired into the live engine.** `civ-legends`, `civ-ai`,
`civ-species`, `civ-genetics`, and the `psyche/social` modules compile in isolation but are **absent from
the engine/server dependency graph** and are never called in the tick. Civis's *entire reason to exist* —
a fully emergent civilization made legible — is therefore **invisible in the actual game**. Every other gap
(legibility, visuals, audio, loop) is in service of perceiving this depth; if the depth never runs, polishing
the frame around an empty world. **Wire the moat into the tick first** (closely paired with #1 render
correctness and #3 legibility so the now-running depth can actually be seen). This is also the biggest
*process* risk: each additional unwired crate increases integration debt.

---

## Recommended sequencing (what to build next to most increase shippability)

Aligned to the master-roadmap stage ladder, re-ordered for *shippability ROI* given the honest current state.

**Wave A — Make it not-broken and make the moat run (parallel, ~3 fleets).**
- A1: Fix render correctness; verify visible/selectable/inspectable agents (vision-verify, read the pixels).
- A2: Add `legends`, `species`, `genetics` to the engine deps and call `psyche/social/legends` in the tick.
- A3: Add `is_finite`/clamp/saturating guards to every emergent float system (psyche, market, doctrine GA).
- *Exit gate:* a world that renders correctly, ticks the depth-moat, and cannot NaN-crash.

**Wave B — Make the depth legible (depends on A).**
- B1: Wire real producer fields into the info-view overlays (light up the BLIND overlays).
- B2: Inspect-anything → mind/relationship/saga panels fed by the now-running psyche/legends data.
- B3: Surface emergent polities/markets read-outs.
- *Exit gate:* click any agent → its mind + saga; overlays show real data. The pitch becomes demonstrable.

**Wave C — Make it attractive + audible (parallel with B, shares no sim files).**
- C1: Lighting/PBR/biome closeout (emissive/wet/warm-cool).
- C2: Source + wire audio content (music beds + SFX) into existing `audio.rs`.
- C3: App icon + main-menu/splash polish.

**Wave D — Make it a game anyone can play.**
- D1: Onboarding/tutorial/tooltips/undo; settings depth; save-slot UI.
- D2: i18n string externalization (do early, cheap now).
- D3: Content breadth — scenarios/presets/god-tool palette toward WorldBox bar.

**Wave E — Make it shippable.**
- E1: Visual-regression + headless-soak CI gates; crash aggregation.
- E2: Perf validation at target scale.
- E3: Store page + capsule art + trailer + EULA/credits/licenses; installer/auto-update.

**Deferred / out-of-scope candidates:** multiplayer (confirm with user — determinism dropped, would need
state-sync); controller support; GI/DLSS (S4). Warfare-as-RTS-loop and vehicles/desire-paths sit in Wave D/E
breadth — valuable but worthless until the depth is legible and the world isn't broken.

---

## Cross-Project Reuse Opportunities (Phenotype org)

Per the reuse protocol (confirm destination with user before extraction):
- **`phenotype-history`** — generic event/causal-DAG core of `civ-legends` (reusable by WSM3D/DINOForge).
- **`phenotype-ai`** — `AiProvider` trait + cache + pool + preflight from `civ-ai` (domain-agnostic).
- **NaN/clamp guard utilities** — a shared `phenotype-num` resilience helper for all float-sim repos.
- **Visual-regression harness** — the F9/F10 capture + a diff gate generalizes across Bevy clients.
