# Civis Onboarding & QoL — The AAA-Completeness Bells-and-Whistles Suite

> **Status:** Design spec (2026-05-30). Owned by Design (Planner stance — specs / AC / UX-flow only, no implementation code).
> Companion to [`docs/research/game-rnd.md`](../research/game-rnd.md) §4 (Polish/QoL table + adopt-now top-10), [`docs/research/competitive-benchmark.md`](../research/competitive-benchmark.md) §3–4 (legibility = #1 credibility gap), [`docs/design/info-views.md`](./info-views.md) (overlay suite), and [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) (only laws authored; everything else emerges → polish *surfaces* emergence, never *authors* it).
> Requirements live under `FR-CIV-QOL-*` (this doc) and cross-reference the shipped `FR-CIV-NOTIFY-900..921` (event feed, stats, onboarding, rebinding) and `FR-CIV-INSPECT-*` (inspect-anything).

---

## 0. Thesis & charter alignment

The benchmark verdict is unambiguous (§3–5): Civis's substrate is ambitious but **only partially surfaced**, and the two worst-scoring axes are *visual fidelity* and *UX/UI polish*. This suite is the **AAA-completeness layer** — the "bells and whistles" that turn a tech demo into a *game a player trusts*. None of it authors emergent outcomes; per the charter, every feature here either **surfaces** measured emergent state (onboarding, tooltips, stats, replay, achievements), **gives the player better authoring ergonomics** (undo, blueprints, hotkeys, camera, time, photo mode), or **adapts presentation** (accessibility, notifications, settings). Zero charter risk.

**Grounding in existing systems (read-only survey).** This spec wires to what already exists in `clients/bevy-ref/src/`:

| Existing module | Resource / type | This suite reads / extends |
|---|---|---|
| `game_ui.rs` | `GameSpeed { multiplier }`, `handle_speed_shortcuts`, `speed_control_ui` | Time controls (§7) extend this; do not duplicate. |
| `notifications.rs` | `NotificationKind`, ring buffer (`NOTIFICATION_CAP=64`), toast stack | Notifications polish (§11) extends this; alerts gain severity + camera-jump. |
| `settings_ui.rs` | `GameSettings` (serde → `settings.ron`), Graphics/Audio/Gameplay/Keybinds groups | Settings depth (§12), accessibility (§10), hotkey rebinding (§5) live here. Keybinds tab is **read-only today** → becomes editable. |
| `spawn_tools` | `ActiveTool`, `SelectedEntity` | Undo/redo (§3), blueprints (§4), tutorial guided steps (§1) read these. |
| `tool_categories` | `Category`, `SubTool`, `CATEGORIES` | Tooltips (§2) and onboarding tool-discovery (§1) read the taxonomy. |
| `info_views.rs` | `InfoOverlay` registry (31 overlays per info-views.md) | Photo mode (§8) hides them; stats (§9) cross-link; colorblind palettes (§10) recolor ramps. |
| `menus.rs` | `GameUiMode { MainMenu, WorldSetup, Loading, Playing, Paused }` | Every QoL overlay gates on `Playing`; onboarding hooks `Loading→Playing`. |
| `ui_theme.rs` | palette + `accent_frame`/`inner_glow`/`panel_shadow` painters | All new panels reuse these; accessibility scaling multiplies the type scale. |
| `crates/watch` + Legends saga-graph (game-rnd §1.2) | event stream | Notifications, achievements, replay, stats all consume this one stream. |

**Shared design rules (apply to every feature below):**
1. **Wrap > hand-roll** — every feature cites the crate from game-rnd §4 (`undo_2`, `leafwing-input-manager`, `egui_plot`, `egui_graphs`); no bespoke charting, keymaps, or command stacks.
2. **One event source** — notifications, achievements, replay scrubber, and stats all read the **same** `crates/watch` + Legends stream. No feature invents a parallel event bus.
3. **Data-driven, not hardcoded** — tutorial steps, achievement defs, alert thresholds, and hotkey defaults live in RON (`template > hardcode`), so they are moddable and the charter's "no authored content" stays honoured for *gameplay* (only the *tutorial chrome* is authored).
4. **Loud-not-silent** (repo CLAUDE.md) — a tutorial step whose target system is BLIND shows a *named* "coming soon" skip, not a soft-lock; an achievement whose producing event never fires logs a gap.
5. **Gate on `GameUiMode::Playing`** unless explicitly a menu/photo feature.

---

## 1. Tutorial / Onboarding — teach by guided play (WorldBox/god-game style)
**FR-CIV-QOL-100..109.** Extends `FR-CIV-NOTIFY-920`. Priority: **8 (must-ship)**.

**Design stance:** god-games teach by *doing*, never walls of text (WorldBox: drop a creature, watch it live; Black & White: the hand guides by gesture). Civis's tutorial is a **progressive-disclosure guided-play tour** — a sequence of short, skippable, *action-gated* steps each anchored to a real HUD element, where the step **completes when the player performs the action**, not when they click "Next".

**Data model (RON, data-driven per game-rnd §4 "data-driven steps in RON"):**
```
TutorialStep {
  id, title, body (≤2 lines),
  anchor: HudAnchor (toolbar category | top-bar chip | inspector | minimap | overlay-button | free),
  completion: Trigger (ToolUsed(tool) | EntitySelected | OverlayOpened(id) | SpeedChanged | CameraMoved | SettlementFounded | Manual),
  spotlight: bool   // dim rest of screen, ring the anchor
  optional: bool    // skippable without blocking the chain
  gated_on: Option<SystemFlag>  // if target system BLIND → show "coming soon", auto-skip (loud)
}
TutorialTour { id, name, steps: [TutorialStep] }
```

**UX flow (first-run, `Loading → Playing` hook in `menus.rs`):**
1. On first `Playing` entry (persisted flag in `GameSettings`, §12), a non-modal **welcome card** offers *Start Guided Tour / Skip / Never show*. Skipping is one click and respected forever.
2. **Guided "First Civilization" tour** (the canonical onboarding arc, ~8 steps, teach-by-play):
   - *Look around* — completion: `CameraMoved` (pan + zoom). Spotlight: minimap.
   - *Read the world* — open one info-view overlay (Elevation/Biome). completion: `OverlayOpened`. Spotlight: overlay button. (Reinforces the legibility moat from step one.)
   - *Seed life* — pick the Life category → spawn tool → place agents. completion: `ToolUsed(spawn)`. Spotlight: Life toolbar drawer.
   - *Inspect anything* — click a spawned agent → read its inspector card. completion: `EntitySelected`. Spotlight: right inspector (ties to `FR-CIV-INSPECT-900`).
   - *Shape the land* — Terraform/Material brush one stroke. completion: `ToolUsed(terraform|material)`.
   - *Control time* — speed up to watch needs/economy move. completion: `SpeedChanged` (reads `GameSpeed`). Spotlight: speed control.
   - *Watch it emerge* — let the sim run until the first emergent event fires on the feed (a birth / settlement founding). completion: feed event of kind ∈ {Birth, Founding}. Spotlight: event feed. (This is the **payoff moment** — the player *sees* emergence they didn't script.)
   - *Read the story* — open the Legends/chronicle browser on that event. completion: `OverlayOpened(legends)`. Spotlight: stats/legends button.
3. **Contextual micro-tips (tool discovery):** the *first* time a player opens each tool category, a one-line flyout tip appears (dismissible, shown-once flag per category in `GameSettings`). This is the WorldBox "every tab teaches itself" pattern and reuses the `tool_categories` taxonomy.
4. **Replayable:** the tour is re-launchable from Settings → Gameplay → "Replay Tutorial". Individual tours (Economy, Diplomacy, God-tools) unlock as those systems surface.

**Reads:** `GameUiMode`, `GameSpeed`, `ActiveTool`/`ActiveSubTool`/`SelectedEntity`, `CATEGORIES`, the `crates/watch` event feed, `GameSettings` (seen-flags).
**AC:** step advances *only* on the real action (not a Next button) for non-optional steps; Skip/Never are one-click and persisted; a BLIND-gated step shows a named "coming soon" and auto-skips with a log line; tour fully replayable; zero modal walls of text (every body ≤2 lines).

---

## 2. Tooltips Everywhere
**FR-CIV-QOL-110..115.** Priority: **8 (must-ship)** — directly closes legibility gap #2.

**Stance:** *every* interactive element and *every* surfaced number has a tooltip. Two tiers: **chrome tooltips** (what a button/chip does + its hotkey) and **data tooltips** (what a number *means* + how it's derived, since emergent numbers are meaningless without provenance).

**UX flow:**
- **Chrome tooltips:** every toolbar category/sub-tool, top-bar chip, overlay button, and settings control shows on hover: label + one-line description + bound hotkey (pulled live from the §5 rebinding map, so it stays correct after a rebind). `tool_categories` already carries labels+hotkeys; this formalizes coverage as a lint ("no interactive widget without a tooltip").
- **Data tooltips (provenance):** hovering a stat chip (population, era, treasury) or an inspector field shows the value **plus its source** ("Population: 1,204 — live count from `agents`") and, where it's a rate, the trend arrow (▲/▼ from the §9 time-series). This is the charter-honest move: numbers say where they came from.
- **Rich tooltips:** hovering an info-view legend stop shows the band's exact range; hovering a notification shows full event detail + "jump to" hint.
- **Delay & pinning:** short hover delay (config in §12); `Shift`-hover pins a tooltip so the player can read a long one or copy a value.

**Reads:** `tool_categories`, top-bar stat resources, `info_views` legends, `SelectedEntity` components, §5 keybind map, §9 time-series (for trend arrows).
**AC:** 100% of interactive widgets have a chrome tooltip; every displayed stat has a provenance data-tooltip; hotkeys in tooltips reflect the *current* binding; `Shift`-pin works; delay configurable.

---

## 3. Undo / Redo (god-tools)
**FR-CIV-QOL-120..124.** Priority: **7**. Crate: **`undo_2`** (returns a command sequence the app interprets — better fit for Bevy editor-style than `undo`).

**Stance:** CS2 *lacks* undo and it's a documented pain → this is a differentiator. Scope is **player authoring actions only** (god-tool edits), **never the autonomous sim** — you cannot "undo" an emergent death; you *can* undo a terraform stroke, a spawn, a brush, a blueprint stamp.

**Command model:** each god-tool action emits an `EditCommand` capturing the **inverse patch** (terraform: prior voxel column heights; material brush: prior material ids; spawn: spawned entity ids to despawn; blueprint stamp: prior region snapshot). Commands push onto an `undo_2` history (cap N, configurable). The sim continues running underneath — undo applies the inverse *as a new edit at the current tick* (it is not time-travel; that's Replay §6).

**UX flow:** `Ctrl+Z` / `Ctrl+Y` (rebindable §5); a small history affordance in the tool panel shows the last action label + count; undo of a stamp that the sim has since built upon undoes only the *authored placement*, not emergent consequences (and a tooltip says so — loud, honest).

**Reads/writes:** `ActiveTool`, voxel grid, entity spawns, blueprint stamps (§4). **Boundary rule:** only player-originated `EditCommand`s are undoable; sim-originated mutations never enter the stack.
**AC:** every god-tool edit is invertible; undo/redo bounded + rebindable; sim mutations excluded; undoing a stamp leaves emergent consequences intact with an honest tooltip; no desync (apply-as-new-edit, not rollback).

---

## 4. Blueprints / Copy-Paste (infra)
**FR-CIV-QOL-130..134.** Priority: **6**. Approach: serialize a selection region (voxel band + entity/structure tags) → re-stamp via the placement tool; **reuse the save-serialization** (game-rnd §3.2) and the WFC stamp tool (§1.1). Pairs with Undo (§3) and the WFC "stamp a plausible district" authoring tool.

**Stance:** charter-safe because blueprints are a *player authoring* convenience over *infrastructure the player places* (roads, building shells, district layouts) — not a way to clone emergent society. You copy the *bricks*, the *life* re-emerges.

**UX flow:**
1. **Marquee select** a region (rectangular or painted) in a Blueprint sub-tool.
2. **Copy** (`Ctrl+C`) serializes the region's authored content (voxel materials + structure tags + road segments) into a named **Blueprint** (in-memory + saveable to a blueprint library RON).
3. **Paste** (`Ctrl+V`) enters a ghost-preview placement mode: the blueprint follows the cursor, rotatable (`R`) and flippable, snapping to terrain/road grid (reuses road-tool snapping); left-click stamps; the stamp is one `EditCommand` (undoable §3).
4. **Library panel:** saved blueprints listed with thumbnails (rendered via photo-mode capture §8), renamable, deletable, shareable as files.

**Reads/writes:** voxel grid, structure/road tags, save-serialization layer, WFC stamp tool, `EditCommand` stack.
**AC:** copy captures only authored infra (no agents/emergent state); paste previews + snaps + rotates before commit; stamp is a single undoable command; blueprints persist to a library and round-trip through save-serialization (fuzzed per game-rnd §2.3).

---

## 5. Hotkey Rebinding
**FR-CIV-QOL-140..145.** Extends `FR-CIV-NOTIFY-921` (rebindable hotkey map). Priority: **7**. Crate: **`leafwing-input-manager`** (Bevy gold-standard: action-based, rebindable, gamepad+kb). Wrap, don't hand-roll a keymap.

**Stance:** today `settings_ui.rs` Keybinds is a **read-only reference list** — this makes it editable and authoritative. All input migrates to a `leafwing` `Actionlike` enum (`OpenSettings`, `Pause`, `Speed1x..10x`, `Undo`, `Redo`, `Copy`, `Paste`, `PhotoMode`, `CameraBookmark{1..9}`, `ToggleOverlay`, per-category tool selects). Existing ad-hoc `ButtonInput<KeyCode>` checks (`handle_speed_shortcuts`, the `O`/`Esc` opens) route through the action map.

**UX flow (Settings → Keybinds tab, now editable):**
- Grouped, searchable action list (Camera / Time / Tools / Overlays / UI / Editing).
- Click a binding → "press a key" capture; **conflict detection** highlights duplicates and offers swap/clear; primary + secondary binding per action; gamepad column where applicable.
- **Reset to defaults** (per-group + global); bindings persist into `GameSettings`/`settings.ron`. Defaults seed from a `keybinds.ron`.
- Because tooltips (§2) read this map live, rebinds immediately reflect everywhere.

**Reads/writes:** `GameSettings`, `leafwing` `InputMap`, all consumers of input actions.
**AC:** every action rebindable with conflict detection; primary+secondary+gamepad; persists + reloads; reset-to-defaults; no remaining hardcoded `KeyCode` checks for player actions; tooltips reflect live bindings.

---

## 6. Camera Bookmarks
**FR-CIV-QOL-150..153.** Priority: **5**. ~30 LOC (game-rnd §4). 

**UX flow:** `Ctrl+1..9` stores the current camera transform into slot N; `1..9` (or a configurable modifier, rebindable §5) flies the camera there with a smooth eased interpolation (not a teleport). A thin bookmark strip (optional, toggle) shows occupied slots with mini-thumbnails (photo-mode capture §8). Bookmarks persist per-save. **Follow-cam** (`FR-CIV-INSPECT-920`, lock camera to selected agent) is the dynamic cousin — bookmarks are static, follow-cam tracks; both share the camera-control system.

**Reads/writes:** camera transform, per-save bookmark store, §5 action map.
**AC:** store/recall 9 slots; smooth eased fly-to; persists per-save; integrates with follow-cam without fighting it; thumbnails optional.

---

## 7. Time Controls + Speed
**FR-CIV-QOL-160..164.** Priority: **8 (must-ship)** — **extends existing `GameSpeed`** in `game_ui.rs`; do NOT duplicate.

**Stance:** `GameSpeed { multiplier }` + `speed_control_ui` (pause/1x/2x/5x/10x) already exists. This formalizes and deepens it. Sim tick rate is **decoupled from render** (separate sim schedule per game-rnd §3.3) so pause freezes the sim while the camera/UI stay live and inspectable.

**UX flow:**
- Segmented control (exists): Pause · 1× · 2× · 5× · 10× (+ a "fast-forward to next event" affordance that runs until the next feed-worthy event, then auto-pauses — RimWorld-style).
- Hotkeys (rebindable §5): `Space` pause-toggle, `1/2/3/4` speed steps, `+`/`-` step.
- **Pause is fully interactive** — inspect, open overlays, queue god-tool edits (applied on resume); a clear "PAUSED" banner (loud).
- The active multiplier surfaces in the top bar + a data-tooltip explaining sim-seconds-per-real-second; ties to the frame-time guard (game-rnd §3.3) — under load the HUD shows *effective* sim speed if the sim can't keep up (honest, not silent).

**Reads/writes:** `GameSpeed`, the decoupled sim schedule, event feed (for fast-forward target), perf guard (effective-speed readout).
**AC:** pause freezes sim, leaves UI/camera live; all speeds rebindable; fast-forward-to-event works + auto-pauses; effective-speed shown under load; no behavioural change to existing `GameSpeed` consumers.

---

## 8. Photo Mode
**FR-CIV-QOL-170..174.** Priority: **5**. Approach (game-rnd §4): hide-UI + free-cam + screenshot (reuses existing `screenshot-automation.md`) + optional DoF/grade toggle.

**UX flow:** a hotkey (rebindable §5) enters Photo Mode — HUD + info-view overlays + gizmos hidden (reads `info_views` to suppress), an unconstrained free-cam (decoupled from the gameplay camera so exiting restores position), and a minimal photo toolbar: depth-of-field toggle + focus slider, exposure/grade nudges (reuses the art-direction ACES grade), time-of-day scrub if `planet` exposes a clock, hide/show specific overlays, and a framing grid + level. Capture writes a timestamped PNG via the screenshot automation; an optional "high-res" multiplier renders above display resolution. Photo Mode auto-pauses (or not — toggle) so the frame holds. Doubles as the thumbnail source for blueprints (§4) and camera bookmarks (§6).

**Reads:** render camera, `info_views` (to hide), art-direction grade params, screenshot automation, `planet` clock.
**AC:** all chrome hideable; free-cam restores on exit; DoF + grade + (optional) time-of-day; PNG capture incl. high-res; auto-pause optional; reused as thumbnail source.

---

## 9. Statistics / Graphs Panels
**FR-CIV-QOL-180..186.** Extends `FR-CIV-NOTIFY-910/911`. Priority: **7**. Crates: **`egui_plot`** (time-series) + **`egui_graphs`** (egui+petgraph, to visualize the Legends saga graph directly).

**Stance:** Songs-of-Syx empire-scale dashboards — **aggregate + drill-down**, reading the `civ-engine` timeseries (CIV-0103) and the Legends graph. Charter-honest: every chart is a *measurement* of emergent state over time, with no authored target lines.

**UX flow (a dockable Stats panel, tabbed):**
- **Population** — total + births/deaths + age pyramid + per-region drill-down (line/area, `egui_plot`).
- **Economy** — treasury, prices (driven by tâtonnement, game-rnd §1.3), production/consumption, trade volume.
- **Society/Ideology/Culture** — ideology share over time, culture/language drift (gated BLIND until the producing crate surfaces; specified now, lights up later).
- **Legends graph** — the saga DAG rendered with `egui_graphs`: click an entity node → its events → causal neighbors (the "why did X happen" walk). This is the DF-Legends moat made visual.
- **Conflict** — battles, casualties, territory change.
- **Interactions:** pannable/zoomable; aggregate↔per-region drill-down (renders 100k+ aggregates without stalling — sampled/binned, not per-agent); hover a point = exact value + epoch; click a Legends node = camera-jump + inspector (shared with §11).
- All charts use **colorblind-safe ramps** (§10) and share the §2 provenance tooltips.

**Reads:** CIV-0103 timeseries, Legends saga graph (`petgraph` via `crates/watch`), region index.
**AC:** time-series for population/economy/ideology/resources/conflict; aggregate + drill-down at empire scale without stall; Legends graph browsable + click-to-jump; BLIND tabs gated + labeled; CB-safe palettes.

---

## 10. Accessibility
**FR-CIV-QOL-190..196.** Priority: **8 (must-ship)** — palettes + scaling are adopt-NOW (game-rnd §4). Crate: **`accesskit`** (Bevy a11y) for screen-reader where feasible (NEXT).

**Stance:** accessibility is a completeness *requirement*, not a nicety, and it's cheap. Three immediate wins + one deeper:
- **Colorblind-safe palettes (NOW):** all info-view ramps (§9 charts too) default to **perceptually-uniform, CB-safe** ramps (viridis/cividis). A Settings → Accessibility "Color vision" selector (None / Deuteranopia / Protanopia / Tritanopia) swaps the ramp set; categorical overlays add **redundant encoding** (patterns/labels, not color-only) so a territory map is readable without hue discrimination.
- **UI scaling (NOW):** an egui-native global UI scale slider (0.75×–2.0×) multiplying the `ui_theme` type/spacing scale; persists in `GameSettings`. Honours OS DPI as the baseline.
- **Full key remap (NOW):** delivered by §5 — listed here as an accessibility guarantee (every action rebindable, no input locked to a fixed key).
- **Screen-reader / reduced-motion / dyslexia-friendly font (NEXT):** `accesskit` exposes the HUD tree where feasible; a "reduce motion" toggle disables camera eases/animated transitions (affects §6 fly-to, toast animations); optional high-readability font.

**Reads/writes:** `GameSettings`, `info_views` ramps, `ui_theme` scale, §5 action map.
**AC:** CB-safe default ramps + 3 CVD modes + redundant encoding on categorical overlays; UI scale 0.75–2.0× persisted; every action rebindable; reduce-motion toggle; `accesskit` HUD tree (NEXT, gated).

---

## 11. Notifications Polish
**FR-CIV-QOL-200..205.** Extends existing `notifications.rs` + `FR-CIV-NOTIFY-900/901`. Priority: **7**.

**Stance:** the ring-buffer + toast stack exists (`NotificationKind`, `NOTIFICATION_CAP=64`, `TOAST_STACK=6`). This adds **severity, camera-jump, filtering, and grouping** — the RimWorld-letter / CS2-chirper polish — all consuming the **one** Legends/`crates/watch` stream (no parallel bus).

**UX flow:**
- **Severity tiers** (Info / Notice / Warning / Crisis) drive toast color (reuse `ui_theme` GREEN/GOLD/RED/VIOLET), sound cue, and dwell time; Crisis pins until acknowledged.
- **Click-to-jump:** clicking a notification flies the camera (eased, §6) to the event location and selects the subject (opens inspector). Implements `FR-CIV-NOTIFY-900`'s camera-jump.
- **Data-driven thresholds** (`FR-CIV-NOTIFY-901`): alert rules in RON (happiness < X, population collapse, hyperinflation) — measured, not scripted.
- **Feed panel:** the full ring buffer as a scrollable, **filterable** (by kind/severity) history; **grouping/throttling** collapses bursts ("12 births in Riverhold") so a busy world doesn't spam.
- **Do-not-disturb / per-kind mutes** in Settings (§12).

**Reads:** Legends/`crates/watch` stream, region/entity index (for jump), `GameSettings` (mutes/thresholds), §6 camera.
**AC:** 4 severity tiers w/ color+sound+dwell; Crisis sticky; click jumps camera + selects subject; thresholds in RON; feed filterable; bursts grouped/throttled; per-kind mute.

---

## 12. Settings Depth
**FR-CIV-QOL-210..216.** Extends existing `settings_ui.rs` `GameSettings`. Priority: **6**.

**Stance:** the four standard groups exist (Graphics/Audio/Gameplay/Keybinds). This deepens them into an AAA-complete settings surface, all serde-round-tripping to `settings.ron`.

**UX flow (tabbed, extends current):**
- **Graphics** — existing presets + per-feature toggles; add GI/upscaler (DLSS/FSR) controls, bloom/grade/vignette sliders (art-direction), render-scale, FPS cap, HDR.
- **Audio** — existing volumes + per-bus (UI/ambience/music/SFX) + adaptive-music intensity (RND-007 Kira).
- **Gameplay** — sim speed default + autosave interval (exist) + autosave count/rotation, default LOD/perf preset, "Replay Tutorial", tutorial/tip seen-flags reset, alert-threshold editor (links §11).
- **Accessibility (NEW tab, §10)** — color-vision mode, UI scale, reduce-motion, font, redundant-encoding toggle.
- **Controls/Keybinds (now editable, §5)** — full rebind UI.
- **Cross-cutting:** search box across all settings; per-setting tooltip (§2) explaining it; **reset-to-default** per group + global; settings are **validated on load** with a loud named fallback for any unreadable field (CLAUDE.md loud-not-silent), never a silent default-wipe of the whole file.

**Reads/writes:** `GameSettings` ⇄ `settings.ron`; consumed by render, audio, sim, input, a11y, tutorial subsystems.
**AC:** all groups deepened + Accessibility tab added; search + per-setting tooltips + reset-to-default; loud per-field fallback on load (named), no whole-file wipe; round-trips through serde.

---

## 13. Replay / Timelapse
**FR-CIV-QOL-220..224.** Priority: **3 (LATER)**. Approach (game-rnd §4): snapshot-interval capture → playback scrubber. **Charter: snapshots, not seed-replay** (determinism dropped); timelapse = play snapshots fast. Depends on save-versioning (game-rnd G2/§3.2).

**UX flow:** a background snapshotter writes world snapshots at a configurable interval into a session ring (capped, on E:/ per the build-disk note). A **Replay panel** offers a scrubber timeline with snapshot ticks; scrub jumps the view to a snapshot (read-only "ghost" view — the *live* sim keeps running separately, or is paused). Play/pause/speed over snapshots = **timelapse**; pairs with Photo Mode (§8) for shareable clips (export a frame range as PNG sequence / capture). Legends events mark the timeline so you can scrub *to* a notable moment.
**Reads:** snapshot store (save-serialization), Legends event marks, photo mode, frame export.
**AC:** interval snapshots to disk (E:/ target); scrubber with event marks; timelapse playback; read-only ghost view (no sim corruption); PNG-sequence export; gated on save-versioning.

---

## 14. Achievements
**FR-CIV-QOL-230..234.** Priority: **4 (NEXT)**. Approach (game-rnd §4): data-driven RON defs checked against Legends events/metrics — **emergent-friendly**.

**Stance:** charter-honest achievements reward *emergent* milestones, not scripted goals — "a lineage crossed a sentience threshold", "a market hit hyperinflation", "a settlement survived 100 epochs", "two cultures merged". Definitions are **measured predicates** over the Legends graph + CIV-0103 metrics, in RON (moddable). They observe; they never *cause*.

**UX flow:** an `AchievementDef { id, name, desc, icon, predicate: MetricOrLegendQuery }` set loaded from RON; an evaluator subsystem watches the event/metric stream and fires on first satisfaction (idempotent, persisted per-profile). Unlock → a special-severity notification (§11) + an Achievements panel (locked/unlocked grid with progress bars for countable ones). **Loud-not-silent:** an achievement whose producing event never fires logs a gap (the predicate references a BLIND system) rather than silently never unlocking.
**Reads:** Legends graph, CIV-0103 metrics, `crates/watch` stream, per-profile unlock store.
**AC:** RON-defined predicates over emergent state (no scripted triggers); first-satisfaction fire, persisted/idempotent; unlock toast + panel w/ progress; gap logged for BLIND-referencing predicates.

---

## 15. Priority roster (the "priority-8" must-ship set)

Priority scale: **8 = must-ship (closes a named credibility gap or is table-stakes legibility)** down to **1**.

| Pri | Feature | FR-CIV-QOL | Why this tier | Crate / extends |
|:---:|---|---|---|---|
| **8** | **Tooltips everywhere** (§2) | 110 | Closes legibility gap #2; emergent numbers are worthless without provenance. | bevy_egui; `tool_categories` |
| **8** | **Tutorial / onboarding (guided play)** (§1) | 100 | First-run trust; teaches the legibility moat by *doing*; the "watch it emerge" payoff. | RON steps; ext. NOTIFY-920 |
| **8** | **Time controls + speed** (§7) | 160 | Table-stakes; pause-to-inspect is how emergence is *read*. | **extends `GameSpeed`** |
| **8** | **Accessibility (palettes + UI scale + remap)** (§10) | 190 | Cheap, adopt-NOW; CB-safe ramps are required for every overlay/chart to be legible. | viridis/cividis; egui scale; §5 |
| **7** | Stats / graphs panels (§9) | 180 | Makes empire-scale emergence + Legends saga visible over time. | `egui_plot`, `egui_graphs`; ext. NOTIFY-910/911 |
| **7** | Hotkey rebinding (§5) | 140 | Accessibility + power-user table-stakes; makes Keybinds tab authoritative. | `leafwing-input-manager`; ext. NOTIFY-921 |
| **7** | Notifications polish (§11) | 200 | Severity + camera-jump turn the feed from log into alerting. | **extends `notifications.rs`**; ext. NOTIFY-900/901 |
| **7** | Undo/redo (§3) | 120 | CS2-lacks-it differentiator for god-tool authoring. | `undo_2` |
| **6** | Settings depth (§12) | 210 | AAA-complete options surface; hosts a11y + rebind + thresholds. | **extends `GameSettings`** |
| **6** | Blueprints / copy-paste (§4) | 130 | Infra authoring ergonomics; reuses save-serial + WFC stamp. | save-serial + WFC; +§3 |
| **5** | Camera bookmarks (§6) | 150 | Cheap navigation QoL; thumbnail + follow-cam synergy. | ~30 LOC; +`INSPECT-920` |
| **5** | Photo mode (§8) | 170 | Shareability + thumbnail source for §4/§6. | screenshot-automation; art grade |
| **4** | Achievements (§14) | 230 | Emergent-milestone meta layer; RON-defined. | RON; Legends graph |
| **3** | Replay / timelapse (§13) | 220 | LATER; depends on save-versioning; shareable clips. | snapshots; +§8 |

**The priority-8 (must-ship) four:** **Tooltips-everywhere (§2)**, **Tutorial/onboarding guided-play (§1)**, **Time controls + speed (§7)**, **Accessibility — CB palettes + UI scale + key remap (§10)**. Together they make the emergent depth *perceptible, navigable, and trustworthy on first run* — which the benchmark names as the project's single biggest gap.

---

## 16. Phased WBS (maps to game-rnd §8 + AgilePlus CIV-W6)

| Phase | Tasks | Depends on |
|---|---|---|
| **Q1 Legibility-now (pri-8)** | §2 Tooltips · §7 Time controls (extend `GameSpeed`) · §10 a11y palettes+scale · §1 Tutorial guided-play | info_views, GameSpeed, GameSettings, watch stream |
| **Q2 Power + alerting (pri-7)** | §5 Rebinding (`leafwing`) · §11 Notifications polish · §9 Stats/graphs · §3 Undo/redo | Legends graph (game-rnd G3), CIV-0103, §5 before §2-tooltip-hotkeys finalize |
| **Q3 Authoring + depth (pri 6–5)** | §12 Settings depth · §4 Blueprints · §6 Camera bookmarks · §8 Photo mode | save-serial (G2), WFC stamp, screenshot-automation |
| **Q4 Meta + shareable (pri 4–3)** | §14 Achievements · §13 Replay/timelapse | Legends graph, save-versioning (G2) |

**Cross-project reuse (Phenotype org, confirm before extraction):** the `undo_2` command-stack wrapper, `leafwing` action-map scaffold, the RON tutorial-tour engine, the data-driven achievement evaluator, and the snapshot-scrubber are all charter-agnostic Bevy QoL utilities — candidates to fold into a shared `phenotype-bevy-qol` alongside the `phenotype-bevy-hardening` proposed in game-rnd §9.
