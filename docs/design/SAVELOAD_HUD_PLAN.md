# Save/Load UI + In-Game HUD — Holocron Keycap Plan

> Design doc. Authored against the live `crates/` tree at the time of writing.
> Loads/saves are the **one place the substrate must round-trip a snapshot
> atomically** — losing the active session on `save.load` is a charter breach
> (see `emergence-charter.md:24-27`). The HUD is the player's instrument
> panel; the save/load panel is the **console** that operates the instrument
> panel. Both surfaces share the **Keycap Palette holocron** visual recipe
> established in `docs/design/ui-design-language.md` §1.5 + ADR-012
> (midnight substrate + teal active edge + `HOLO_CORE` brackets).

---

## 0. Scope

This plan covers **two** UI surfaces and **one** set of supporting crates:

| Surface | Today | Target | Crate(s) added |
|---------|-------|--------|----------------|
| **Save / Load panel** | Wire-only (`save.slot` / `save.load` / `save.list` in `crates/server/src/jsonrpc.rs`). No UI. No autosave eviction exposed. No rename, no per-slot delete, no thumbnail/preview, no confirmation modal, no "what changed since last save" diff. | Holocron Keycap modal: **Slots tab** (manual + autosave), **Restore** (load + preview + confirm), **Manage** (rename / delete / evict), **Save As…** flow, **Import/Export `.civsave`**. Wire is already there; the panel is the missing front-end. | `crates/save-ui` (new, server-rendered HTML/JS via `crates/watch` + native widget crate for Bevy/Godot/Unreal). |
| **In-game HUD (top-bar + tile inspector + sim-speed cluster)** | `crates/voxel/src/hud.rs:1` (a **mono resource string**, no chrome) + `clients/bevy-ref/src/game_ui.rs` (a partial chrome top-bar with POP/TREASURY/ERA/YEAR/TIME but no tile inspector or sim-speed cluster). Web dashboard has a terrain viewer with no game-time HUD. Godot/Unreal clients have no HUD at all. | A unified **Keycap Palette** chrome HUD: era chip, population chip, treasury chip, sim-speed keycap cluster (pause/±/step), **tile inspector** (the holo readout when a brush/inspect tool is armed), and the always-on slot-fingerprint chip ("AUTO · 14:32 · slot 03"). Style: midnight + teal per ADR-012 + `GOD_TOOLS_SANDBOX.md` §2.2. | `crates/hud` (new shared crate) + `crates/hud-web` (HTML/JS dashboard widget) + thin per-client wrappers in Bevy/Godot/Unreal. |
| **Wire (save-db + server saves module)** | `crates/save-db/src/lib.rs` already ships `record_slot_save`, `record_autosave`, `list_for_session`, `evict_autosaves`, plus `SaveBrowserEntry`. `crates/server/src/saves.rs` exposes the JSON-RPC surface. Gaps: **no in-memory cache for `list_for_session`**, **no thumbnail/preview blob column**, **no `rename` row method**, **no `delete_one` row method**, **no `diff_to(other)` for the "what changed" panel**, **no mod-aware slot filtering**. | Extend `crates/save-db` with: `rename(slot_id, new_label)`, `delete_slot(slot_id)`, `get_archive(slot_id) -> SaveArchive` (for the inspector preview), `diff_summary(slot_a, slot_b) -> SaveDiffSummary` (added/removed tiles, Δ pop, Δ era, mod list), `last_n_for_session(n)` (the "fingerprint chip" hot path), `attach_thumbnail(slot_id, png_bytes)`. | `crates/save-db` (extend in place — no new crate). |

**Out of scope** (do not implement in this plan; record as future work):
- Multiplayer save coordination / quorum (`docs/development-guide/fr-mp-lobby.md` roadmap).
- Cloud sync (Phase 6 stubs only).
- Mod-aware slot eviction policy beyond a hard `max_autosaves` cap.

**Cross-references:**
- Visual style: `docs/design/ui-design-language.md` §1.5 (two-tier chrome/holo), §5.2 (blade button), §6 (top resource bar recipe).
- Holocron deck context: `docs/design/GOD_TOOLS_SANDBOX.md` §2.1–2.3 (Keycap Palette rim + Deck), §6.1 (default HUD layout when deck is closed).
- ADR-012 (Keycap Palette design system): `docs/adr/ADR-012-keycap-palette-design-system.md`.
- Substrate save/load path: `crates/engine/src/save.rs` (`save_archive` / `load_archive`), `crates/engine/src/era.rs` (`CivEra::evaluate`).
- Existing JSON-RPC: `crates/server/src/jsonrpc.rs` + `crates/server/src/saves.rs` (already exposes `save.slot`, `save.load`, `save.list`, `save.list_browser`, `save.evict`, `save.rename`, `save.delete`).
- Existing HUD stubs: `crates/voxel/src/hud.rs:1-200`, `clients/bevy-ref/src/game_ui.rs:1-260`.

---

## 1. The two-tier discipline (why this plan looks the way it does)

Per `ui-design-language.md` §0, every UI pixel is either **CHROME** (≈92%, graphite/glass/Geist, neon only on active edge) or **HOLO** (≈8%, scanlines/brackets/aberration). The two-tier rule for *this* plan:

- **CHROME tier** (the *Console* — neutral graphite glass, player issues commands):
  - The whole Save/Load panel surface, slot rows, rename inputs, confirmation modal chrome, the export/import strip.
  - The top HUD bar (era/pop/treasury/year/time chips).
  - The sim-speed keycap cluster (`A` pause, `S` slow, `D` normal, `F` fast, `G` step).
- **HOLO tier** (the *Projection* — read-only instrument readout):
  - The tile inspector panel when an inspect tool is armed (it's the "measurement", per `GOD_TOOLS_SANDBOX.md` §1).
  - The thumbnail preview pane inside each slot row (it's a "what you'll restore" projection, not a save control).
  - The "what changed since last save" diff strip (it's a measurement of state Δ, not a write).

**Density discipline:** at most **two HUD holo surfaces** visible at any moment (per `ui-design-language.md:469-471`). The default at-rest layout: **Inspector closed = zero HUD holo surfaces visible**; **Inspector open = exactly one (the tile inspector)**, even when the save panel is open (the panel is chrome, not holo). The save panel **never** opens a holo projection itself.

**Token discipline (no new hex):** the Keycap Palette uses only tokens already defined in `ui-design-language.md`:

| Token | Hex | Where it appears |
|-------|-----|------------------|
| `GRAPHITE_900` | `#0F131A` | Panel substrate, slot row background, HUD top bar base |
| `GRAPHITE_700` | `#1E242E` | Unselected slot fill, unselected keycap fill |
| `STEEL_400` | `#4A5564` | Hover bevel, active keycap bevel highlight |
| `HOLO_GLOW` | `#2FBFE6` | Active keycap 1px edge + 3px halo |
| `HOLO_CORE` | `#7FE9FF` | Active keycap glyph, tile-inspector brackets, "AUTO" chip border |
| `HOLO_DEEP` | `#0E3A4A` | Tile-inspector translucent fill (0.18 alpha) |
| `TEXT_MID` | `#9AA4B2` | Slot label, chip label (uppercase) |
| `TEXT_LOW` | `#646F7E` | Mono numerics (slot time, pop, Δ tick) |
| `WARN` | `#F2C14E` | Stale-autosave chip (≥ N minutes old) |
| `NEON` | `#9CFF6E` | "Saved just now" confirmation pulse |

No new tokens. No new palette swatches. **Adding new tokens is forbidden by the design language discipline** (per `GOD_TOOLS_SANDBOX.md:103`).

---

## 2. Save/Load UI flow

### 2.1 Trigger surfaces

The Save/Load panel opens from **four** triggers (all map to the same modal — the *modal* is the affordance, the trigger is the entrypoint):

1. **`Esc` → "Save & Quit"** in the pause menu (charter-required per `emergence-charter.md:24-27`: no silent data loss on quit).
2. **`F5` → Quick Save** (slot 0, no modal, just a toast: "Saved → slot 00 · 14:32:08").
3. **`F9` → Quick Load** (modal opens pre-filtered to "most recent manual slot").
4. **Main menu → "Continue" / "Load Game"** (modal opens on the Slots tab).

The panel itself is a **CHROME E2 modal** (per `ui-design-language.md` §3): `INK_0` (`#06080C`) scrim @ 0.72 alpha behind, `GRAPHITE_900` panel at 96% opacity, no scanlines (chrome, not holo).

### 2.2 Layout (panel-open HUD)

```
   ░░░░░░░░░░░░░░░░░░░░░ INK_0 scrim @0.72 ░░░░░░░░░░░░░░░░░░░
   
   ╭───── CIVIS · SAVE / LOAD ─────╮    [Esc close]   [F5 quick-save]
   │                                │
   │  ╭─ TABS ─────────────────╮    │
   │  │ [SLOTS]  [RESTORE]     │    │
   │  │ [MANAGE] [IMPORT/EXPORT]    │
   │  ╰────────────────────────╯    │
   │                                │
   │  ╭── search slots... ────╮    │
   │  ╰──────────────────────╯    │
   │                                │
   │  ┌─ slot 07 · "Iron Tide" ───── MANUAL · 14:32:08 · 12.4K pop · era IRON ─┐
   │  │ [thumbnail 96×54]   slot 07           label: "Iron Tide"               │
   │  │                   SAVED 2m ago         size: 4.2 MB                      │
   │  │                   mods: core, example-policy                            │
   │  │                                                  [Restore] [⋯ menu]    │
   │  └────────────────────────────────────────────────────────────────────────┘
   │  ┌─ AUTO 14 · 14:18:02 ──────────── AUTO · 18m ago · 12.3K pop · era IRON ──┐
   │  │ [thumbnail 96×54]   autosave #14     ⚠ stale                            │
   │  │                   18m ago            size: 4.1 MB                       │
   │  │                                                  [Restore] [⋯ menu]    │
   │  └────────────────────────────────────────────────────────────────────────┘
   │  ...                                                                  │
   │                                                                        │
   ╰────────────────────────────────────────────────────────────────────────╯
   
   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

The **HUD top-bar stays visible** through the panel scrim (so the player can see the era/year/speed even while loading). The top-bar is **never** a holo surface; it stays in CHROME tier behind the scrim.

### 2.3 Tab 1 — SLOTS (default tab)

A scrollable list of slot rows. Row anatomy:

```
[ thumbnail 96×54 ]  slot 0N · label · MANUAL/AUTO · ISO timestamp · age
                     pop count · era chip · mod list · size · "what changed" delta vs previous slot
                                                       [Restore]  [⋯ menu]
```

- **Thumbnail:** 96×54 PNG, captured at save-time by the engine (or by `crates/save-ui` calling into `crates/save-db::attach_thumbnail`). Per `ui-design-language.md` §5.2, the thumbnail is a small keycap blade chip — not a holo projection.
- **Mod list:** comma-separated mod ids that were active at save-time. The Restore button is **disabled** with a tooltip if the active mods ≠ the saved mods (`docs/guides/mod-sandbox-security.md:73-79`).
- **"What changed" delta:** the row right-clicks → "Diff vs selected" → opens a holo strip showing Δ pop, Δ era, Δ tile count, Δ treasury, added/removed buildings — *measured*, not invented (charter: emergence, not scripted deltas).
- **Empty state:** "No saves yet. Press **F5** to save into slot 00."
- **Sort:** Manual first (newest first), then autosaves (newest first). Toggle: `▾ sort: time | name | era`.

### 2.4 Tab 2 — RESTORE

The Restore tab is **selected slot preview + confirm**. Two-column:

```
┌─ SLOT 07 · "Iron Tide" ─────────────────┐  ┌─ CONFIRM RESTORE ────────────┐
│  [ large thumbnail 320×180 ]            │  │  ⚠ current session will be   │
│                                         │  │     replaced. autosave first? │
│  era   IRON    (prev: BRONZE, +12 min)  │  │                               │
│  pop   12,431  (Δ +127 vs slot 06)       │  │  [ Save current → AUTO 15 ]   │
│  year  −420    treasury 8,214           │  │  [ Discard + Restore ]        │
│  mods  core, example-policy              │  │  [ Cancel ]                   │
│                                         │  │                               │
│  diff vs slot 06:                       │  │  Diff: +127 pop, +3 buildings,│
│   ▸ 12 new farms, 4 new roads            │  │  +1 era. 4m 12s of sim time.   │
│   ▸ no removals                         │  │                               │
╰─────────────────────────────────────────╯  ╰───────────────────────────────╯
```

- **Confirmation is mandatory** (no silent overwrite of the live session). Three buttons: Save-current-and-restore (the safe path, default focus), Discard-and-restore (the fast path, red `WARN` border), Cancel.
- **The "discard and restore" button is a `WARN`-accented keycap** per `ui-design-language.md` §6 "destructive actions get a `WARN` keycap with a 220ms scan-sweep animation on hover".
- **Save-current-and-restore** calls `save.slot` on `AUTO` first, then `save.load` on the selected slot. The two operations are wrapped server-side in a single `save.restore_with_autosave` JSON-RPC method to keep them atomic from the client's perspective.
- **Loading is async** — the panel shows a chrome spinner (the existing `civ-watch` spinner widget) with the label "Restoring snapshot …". The HUD top-bar shows a `LOAD` chip (chrome, `WARN` color) while the restore is in flight.

### 2.5 Tab 3 — MANAGE

| Action | Wire | UI affordance |
|--------|------|---------------|
| **Rename** slot | `save.rename { slot_id, new_label }` → `crates/save-db::rename` (extend, see §3) | Inline text input on the row (single-click label to edit). Validation: 1–64 chars, no path separators. |
| **Delete** slot | `save.delete { slot_id }` → `crates/save-db::delete_slot` (extend) | `⋯` menu → Delete → confirmation modal ("Delete slot 07? This cannot be undone."). The **Delete button is `WARN`-accented**. |
| **Evict old autosaves** | `save.evict { keep_n: 8 }` → existing `crates/save-db::evict_autosaves` | Footer button: "Keep last **8** autosaves" with a number stepper. Default `8`, configurable via scenario. |
| **Pin slot** | new wire `save.pin { slot_id, pinned: bool }` (extend, see §3) | `⋯` menu → Pin/Unpin. Pinned slots are exempt from `evict_autosaves` and from any future "trim to free disk" sweep. |

### 2.6 Tab 4 — IMPORT / EXPORT

- **Export `.civsave`** — bundles `SaveArchive` + metadata + mods-manifest + thumbnail into a single zip. Wire: `save.export { slot_id } → { bytes_b64 }`. Browser downloads via `Blob`; native clients write to `<userData>/saves/`.
- **Import `.civsave`** — drag-drop on the panel, or click "Import" → file picker. Wire: `save.import { bytes_b64, label? }` → returns new `slot_id`. Validates the mod manifest; if mods missing, opens a one-line warning chip "missing: example-economic (will restore without it)".
- **Format spec** lives in `docs/specs/SAVE_FORMAT.md` (to be authored as part of Phase 4 — currently lives as inline comments in `crates/engine/src/save.rs`).

### 2.7 Autosave policy (the "fingerprint chip" in the HUD)

Every `N` ticks (default 600 = 10 min @ 1× speed; scenario-configurable), the server calls `crates/save-db::record_autosave`. Eviction keeps the **last 8** autosaves by default (per `crates/save-db::evict_autosaves`); pinned slots are exempt. **No prompt**, no modal — autosave is silent and the only HUD signal is the **slot fingerprint chip** (see §3.1).

If a save fails (disk full, serialization error, mod crash mid-snapshot), the chip turns `WARN` and shows "AUTOSAVE FAILED — 14:32:08". On the **next successful save** the chip clears.

### 2.8 Acceptance criteria (save/load UI)

- **AC-SL-1:** `F5` saves to slot 0 and shows a chrome toast that auto-dismisses after 1.6s (`ui-design-language.md` §3.5 chrome toast recipe). No modal.
- **AC-SL-2:** `F9` opens the panel on the Slots tab with slot 0 (or the most recent manual slot) pre-selected.
- **AC-SL-3:** `Esc → Save & Quit` from the pause menu opens the panel on the Slots tab and **focuses the Save-current-and-quit button by default** (one keypress → done).
- **AC-SL-4:** Restoring a slot while mods are mismatched disables the Restore button and shows the missing-mods chip (no silent partial restore).
- **AC-SL-5:** The "what changed" diff shows **only measured values** — never invented text. Δ pop, Δ era, Δ tile count, added/removed building ids, +sim ticks. No "the civilization is thriving!" strings.
- **AC-SL-6:** Autosaves silently evict to keep 8 (configurable). Pinned slots never evict. Stale-autosave chip (≥ 30 min) renders in `WARN` color.
- **AC-SL-7:** The slot row thumbnail is captured at save-time (≤ 16 KB PNG) and attached via `crates/save-db::attach_thumbnail`. The wire never blocks on thumbnail generation.
- **AC-SL-8:** Export round-trips: `save.export → file → save.import` produces a byte-identical `SaveArchive` and the same `slot_id` could be re-derived. Tested via `crates/save-db` integration tests.
- **AC-SL-9:** The save panel **never** renders a holo surface. Inspector stays closed when the panel is open (charter density rule).
- **AC-SL-10:** The "Save current → AUTO 15" button is the **default focus** on the Restore tab — one `Enter` confirms the safe path.

---

## 3. `crates/save-db` extensions (the wire half)

`crates/save-db` already exposes `record_slot_save`, `record_autosave`, `list_for_session`, `evict_autosaves`, `SaveEntry`, `SaveKind`, `SaveBrowserEntry`. Extensions below land in-place — no new crate, no schema break (additive only — existing rows with `NULL` thumbnail / `pinned = 0` stay valid).

| New API | Signature | Purpose | Test |
|---------|-----------|---------|------|
| `rename` | `pub fn rename(&self, slot_id: i64, new_label: &str) -> Result<(), SaveDbError>` | Rename slot label (≤ 64 chars, validated). | `crates/save-db/tests/rename_test.rs` |
| `delete_slot` | `pub fn delete_slot(&self, slot_id: i64) -> Result<(), SaveDbError>` | Delete one slot (NOT autosave-evict — explicit delete). | `crates/save-db/tests/delete_test.rs` |
| `get_archive` | `pub fn get_archive(&self, slot_id: i64) -> Result<SaveArchive, SaveDbError>` | Fetch the saved snapshot bytes (for the diff & for the export). | `crates/save-db/tests/get_archive_test.rs` |
| `diff_summary` | `pub fn diff_summary(&self, a: i64, b: i64) -> Result<SaveDiffSummary, SaveDbError>` | Compute measured Δ between two slots: pop Δ, era Δ, tile Δ, treasury Δ, added/removed building ids, sim tick Δ, mod set Δ. | `crates/save-db/tests/diff_summary_test.rs` |
| `attach_thumbnail` | `pub fn attach_thumbnail(&self, slot_id: i64, png: &[u8]) -> Result<(), SaveDbError>` | Store ≤16 KB PNG alongside the row. | `crates/save-db/tests/thumb_test.rs` |
| `pin` | `pub fn pin(&self, slot_id: i64, pinned: bool) -> Result<(), SaveDbError>` | Toggle pin; pinned rows exempt from `evict_autosaves`. | `crates/save-db/tests/pin_test.rs` |
| `last_n_for_session` | `pub fn last_n_for_session(&self, session_id: &str, n: u32) -> Result<Vec<SaveBrowserEntry>, SaveDbError>` | Bounded list query — the HUD fingerprint chip hot path (≤ 200µs at p99). | `crates/save-db/tests/last_n_test.rs` |

**New types:**
```rust
pub struct SaveDiffSummary {
    pub pop_delta: i64,
    pub era_a: CivEra,
    pub era_b: CivEra,
    pub tile_count_delta: i64,
    pub treasury_delta: i64,
    pub sim_tick_delta: i64,
    pub added_building_ids: Vec<BuildingId>,
    pub removed_building_ids: Vec<BuildingId>,
    pub added_mod_ids: Vec<String>,
    pub removed_mod_ids: Vec<String>,
}
```

**Schema migration** (additive — no breaking change):
```sql
-- 0007_save_ui_columns.sql (new migration in crates/save-db/migrations/)
ALTER TABLE saves ADD COLUMN label TEXT;
ALTER TABLE saves ADD COLUMN thumbnail BLOB;                 -- ≤16KB PNG
ALTER TABLE saves ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE saves ADD COLUMN size_bytes INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_saves_session_kind_time
  ON saves (session_id, kind, created_at DESC);
```

**JSON-RPC additions** (extend `crates/server/src/jsonrpc.rs` + `crates/server/src/saves.rs`):
- `save.rename { slot_id, new_label } → { ok: bool }`
- `save.delete { slot_id } → { ok: bool }`
- `save.pin { slot_id, pinned } → { ok: bool }`
- `save.thumbnail { slot_id, png_b64 } → { ok: bool }`
- `save.diff { slot_a, slot_b } → { summary: SaveDiffSummary }`
- `save.export { slot_id } → { bytes_b64: string, filename: string }`
- `save.import { bytes_b64, label? } → { slot_id: i64 }`
- `save.last_n { session_id, n } → { entries: Vec<SaveBrowserEntry> }`

All methods gated by the same **autosave-on-restore** wrapper that already wraps `save.load` server-side, so restore-with-autosave is atomic from the client.

---

## 4. In-game HUD (Keycap Palette holocron style)

The HUD is **always visible** when the save/load panel is closed. When the panel is open, only the top-bar persists (see §2.2). The HUD has three regions:

```
┌─[● LIVE 1×]── top resource bar (CHROME E0) ─────────────────────────────────────┐
│ ◆ POP 12.4K   ⌖ ERA  IRON   ⛁ TREAS 8.2K   ◷ YEAR −420   ☼ 14:32   💾 AUTO 14   │
└──────────────────────────────────────────────────────────────────────────────────┘
   ┌─ KEYCAP PALETTE (chrome dock, midnight + teal active) ─────────────────────┐
   │  [Q SEL]  [W RAI]  [E DRP]  [R SPO]  [T T+G]  [A PAU]  [S SPD]  [F LGT]   │
   │                                       ▲                                     │
   │                                  (teal active edge)                         │
   └────────────────────────────────────────────────────────────────────────────┘
   (whenever a god-tool is armed OR Q is tapped)
        ┌── HOLO TILE INSPECTOR (read-only) ──┐
        │ cell (12, 7, 3) · height 14 · mat Rock │
        │ biome Temperate · slope 0.7            │
        │ agent_count 0 · faction — · mood —     │
        ╰────────────────────────────────────────╯
```

The "tile inspector" lives in the **`crates/hud` shared crate** and is rendered by each client. The web dashboard renders it as a fixed-position panel; the Bevy client renders it via `bevy_egui`; Godot/Unreal render it via their native UI subsystems.

### 4.1 Top resource bar (CHROME E0, always-on)

Following `ui-design-language.md` §6 "Top resource bar" recipe and `GOD_TOOLS_SANDBOX.md` §6.1:

| Chip | Glyph | Token | Source | Notes |
|------|-------|-------|--------|-------|
| POP | `◆` | `TEXT_MID` for label, `TEXT_HIGH` for value | `crates/agents::pop_count(sim)` | Mono numeric, `K`/`M` formatter |
| ERA | `⌖` | era-specific accent (BRONZE = `WARN`, IRON = `NEON`, STEEL = `HOLO_CORE`, INFORMATION = `HOLO_DEEP`, ATOMIC = `HOLO_GLOW`) | `crates/engine::era::CivEra::evaluate(sim)` | The accent is the era name color, **not** an invented color. |
| TREASURY | `⛁` | `TEXT_MID` / `TEXT_HIGH` | `crates/economy::treasury(sim)` | Mono numeric, `K`/`M` formatter |
| YEAR | `◷` | `TEXT_LOW` | `crates/calendar::year(sim)` | Negative years in BC suffix |
| TIME | `☼` | `TEXT_LOW` | sim clock | `HH:MM` 24-hour, sim-time |
| SPEED | `●` | `NEON` when LIVE, `WARN` when PAUSED, `TEXT_LOW` when stepping | `crates/engine::GameSpeed::multiplier(sim)` | Toggles `1×`/`2×`/`5×`/`10×`/`PAUSE`/`STEP` |
| AUTO | `💾` | `HOLO_GLOW` when last autosave ≤ 5 min, `TEXT_LOW` when 5–30 min, `WARN` when ≥ 30 min or last save failed | `crates/save-db::last_n_for_session(session, 1)` | The "fingerprint chip" |

**Tapping a chip** opens a **holo readout strip** (a thin holo bar above the chip row — this is the **second** HUD holo surface, paired with the tile inspector when both are open; the density rule permits two). E.g. tapping ERA shows the era-transition timeline (a `HOLO_CORE` ribbon with markers at past transitions).

### 4.2 Keycap Palette (rim cluster)

The 8-key ring from `GOD_TOOLS_SANDBOX.md` §2.1 — extended with the **save/load verb on the rim**:

| # | Key | Verb | Source | Crate binding |
|---|-----|------|--------|---------------|
| 1 | `Q` | Select / Inspect | `crates/hud::key_palette::Q` | binds to `PowerId::Inspect.Probe` |
| 2 | `W` | Raise / Lower toggle | `crates/hud::key_palette::W` | binds to `PowerId::Terrain.Raise` |
| 3 | `E` | Material — AdditiveDrop | `crates/hud::key_palette::E` | binds to `PowerId::Material.AdditiveDrop` |
| 4 | `R` | Spawn — Organism | `crates/hud::key_palette::R` | binds to `PowerId::Life.SpawnOrganism` |
| 5 | `T` | Terraform God — AddLand/DigOcean toggle | `crates/hud::key_palette::T` | binds to `PowerId::Terrain.AddLand` |
| 6 | `A` | Time — Pause | `crates/hud::key_palette::A` | binds to `PowerId::Time.Pause` (server `sim.set_speed{0}`) |
| 7 | `S` | Speed +/– | `crates/hud::key_palette::S` | rotates `GameSpeed.multiplier ∈ {1, 2, 5, 10}` |
| 8 | `F` | Disaster — Lightning | `crates/hud::key_palette::F` | binds to `PowerId::Disaster.Lightning` |

(The brief's "sim-speed" maps to `A` + `S` together — pause and rate. The `S` keycap is the speed key.)

**Visual recipe** (per `GOD_TOOLS_SANDBOX.md` §2.2 — verbatim, no edits):

| Element | Token | Hex |
|---------|-------|-----|
| Disc face | `GRAPHITE_900` | `#0F131A` |
| Disc shadow / bottom bevel | `INK_1` | `#0A0D12` |
| Unselected key fill | `GRAPHITE_700` | `#1E242E` |
| Unselected key text | `TEXT_MID` | `#9AA4B2` |
| Active key bevel highlight | `STEEL_400` | `#4A5564` |
| Active key teal edge (1px) | `HOLO_GLOW` @0.8 | `#2FBFE6` |
| Active key teal halo (3px) | `HOLO_GLOW` @0.35 | `#2FBFE6` |
| Active key text | `HOLO_CORE` | `#7FE9FF` |
| Hotkey mono character | `Numeric` Mono `TEXT_LOW` | `#646F7E` |
| Disc label | `Label` UPPERCASE +0.6 | — |

The keycap cluster is rendered by `crates/hud::KeycapPalette` (a server-rendered HTML/JS widget for the web dashboard, and a `bevy_egui` panel for the Bevy client). Godot and Unreal clients implement the same recipe against their UI subsystems — the **token list is the contract**.

### 4.3 Tile inspector (HOLO, on-demand)

The tile inspector is **read-only** (charter-required per `FR-CIV-GODTOOL-910`). It opens when:

- A god-tool is armed and the cursor hovers a tile (the in-world brush ring + the inspector sync).
- `Q` is pressed (Select / Inspect).
- An event of kind `actor.selected` or `cell.probed` lands on the watch event bus.

Anatomy:
```
┌── TILE INSPECTOR (HOLO) ──────────────┐
│ cell (12, 7, 3) · height 14 · mat Rock │   ← `HOLO_CORE` brackets
│ biome Temperate · slope 0.7            │   ← `HOLO_DEEP` @0.18 fill
│ agent_count 0 · faction — · mood —     │   ← Mono numerics
│ ↳ right-click → Probe (I1) → full Inspector  │
╰────────────────────────────────────────╯
```

When the **save panel is open**, the tile inspector **closes** (density rule: 2 HUD holo surfaces max, and the save panel + tile inspector would be 2 holo surfaces stacked on chrome — we trade down to keep the panel scannable). The save panel + the keycap palette + the top-bar is **one chrome surface stack** (still ≤ 2 holo).

### 4.4 Sim-speed cluster (the `A` + `S` keycaps)

The sim-speed controls are **part of the Keycap Palette** (positions 6 and 7 in §4.2) — not a separate widget. Their behaviour:

- `A` (Pause) — toggles `GameSpeed.multiplier` between `{current}` and `0`. Active when paused (`WARN` dot, `ui-design-language.md` §6 "Pause = WARN dot, Play = NEON dot").
- `S` (Speed) — rotates the multiplier through `{1, 2, 5, 10}` (or `{0.25, 0.5, 1}` if `Shift` held — slow band). The current value appears in the SPEED chip on the top-bar (`● LIVE 1×`).
- `Shift+S` — open the speed **picker modal** (a small chrome E1 popover listing `{0.25, 0.5, 1, 2, 5, 10}` with mono numerics). The active value is highlighted in `HOLO_CORE`.
- Right-click the SPEED chip → "Custom rate…" → opens a single-line input (the existing `civ-watch` numeric widget).

No other verb in the Keycap Palette mutates sim speed; the camera verbs (`C1`–`C5`) and the wheel always control camera, never speed (per `GOD_TOOLS_SANDBOX.md` §3.7 camera-never-mutates-sim guarantee).

### 4.5 Acceptance criteria (HUD)

- **AC-HUD-1:** The top-bar shows all six chips (`POP`, `ERA`, `TREASURY`, `YEAR`, `TIME`, `SPEED`) plus the `AUTO` fingerprint chip. Tapping a chip opens a holo readout (≤ 200ms transition).
- **AC-HUD-2:** The AUTO chip turns `WARN` if the most recent autosave is ≥ 30 min old, or if the last autosave attempt failed. The chip's tooltip shows the ISO timestamp and the slot id.
- **AC-HUD-3:** The Keycap Palette renders 8 keycaps using the verbatim recipe from `GOD_TOOLS_SANDBOX.md` §2.2. **No new hex tokens** are introduced (verified by `crates/hud/tests/token_audit.rs`).
- **AC-HUD-4:** The active keycap has a 1px `HOLO_GLOW` edge + a 3px teal halo. The unselected 7 keycaps are pure graphite.
- **AC-HUD-5:** The tile inspector opens on tool-arm and on `Q` press; closes when the save panel opens (density rule).
- **AC-HUD-6:** Sim-speed `A` and `S` mutate `GameSpeed.multiplier` only — never camera, never save. A camera move + a speed change produces identical sim hash before/after the camera move (c.f. AC-GT-7 in `GOD_TOOLS_SANDBOX.md:516`).
- **AC-HUD-7:** The HUD **never** mutates the substrate directly. Every HUD action routes through the existing JSON-RPC catalog (verified by `crates/hud/tests/wire_only_test.rs`).

---

## 5. Crates to add / extend

| Crate | Status | Lines (target) | Owner |
|-------|--------|----------------|-------|
| `crates/save-db` | **extend in place** | +400 (new APIs + migration + tests) | save-db owner |
| `crates/save-ui` | **new** (HTML/JS widget served by `crates/watch`) | +1200 (panel markup + state machine + slot-row component + import/export) | save-ui owner |
| `crates/hud` | **new** (shared keycap palette + tile inspector types + token audit) | +600 (KeycapPalette struct, TileInspector, HudState, token audit) | hud owner |
| `crates/hud-web` | **new** (renders `crates/hud` types into the `civ-watch` HTML/JS layer) | +800 (vanilla TS, no framework — extends the existing watch app) | hud-web owner |
| `crates/server/src/saves.rs` | **extend** | +200 (new JSON-RPC methods: `save.rename`, `save.delete`, `save.pin`, `save.thumbnail`, `save.diff`, `save.export`, `save.import`, `save.last_n`) | server owner |
| `crates/server/src/jsonrpc.rs` | **extend** | +60 (method table) | server owner |
| `crates/voxel/src/hud.rs` | **deprecate** (replaced by `crates/hud`) | −200 (move types, keep one re-export for compat) | voxel owner |
| `clients/bevy-ref/src/game_ui.rs` | **extend** | +400 (mount `KeycapPalette` + `TileInspector` via `bevy_egui`) | bevy-ref owner |
| `clients/godot-ref/` | **extend** | +400 (Godot Control nodes matching the token list) | godot owner |
| `clients/unreal-show/` | **extend** | +400 (UMG widgets matching the token list) | unreal owner |
| `web/` | **extend** | +600 (Save/Load modal + HUD integration in the existing Svelte/TS dashboard) | web owner |

**No new `crates/save-db` clones, no parallel save paths.** The single substrate is `crates/engine::save::SaveArchive` → serialized → row in `crates/save-db`. The wire is `crates/server::saves` → JSON-RPC. The UI is `crates/save-ui` + `crates/hud-web` + per-client wrappers.

---

## 6. Phased WBS

### Phase 0 — Foundations (no UI; substrate only)

**Goal:** widen `crates/save-db` + `crates/server/src/saves.rs` so the UI has something to talk to. Pure backend, fully testable, ships behind `civis-3d-verify`.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 0.1 | Migration `0007_save_ui_columns.sql` (label, thumbnail, pinned, size_bytes, index) | `crates/save-db/migrations/` | `just civis-3d-catalog-check` passes; new columns nullable / defaulted |
| 0.2 | Add `SaveDiffSummary` struct + `diff_summary(a, b)` | `crates/save-db/src/lib.rs` | `crates/save-db/tests/diff_summary_test.rs` green |
| 0.3 | Add `rename(slot_id, new_label)` + `delete_slot(slot_id)` | `crates/save-db/src/lib.rs` | rename_test + delete_test green |
| 0.4 | Add `attach_thumbnail(slot_id, png)` + `get_archive(slot_id)` | `crates/save-db/src/lib.rs` | thumb_test + get_archive_test green |
| 0.5 | Add `pin(slot_id, pinned)` (extend `evict_autosaves` to skip pinned) | `crates/save-db/src/lib.rs` | pin_test green; `evict_autosaves` test still green |
| 0.6 | Add `last_n_for_session(session_id, n)` (bounded query) | `crates/save-db/src/lib.rs` | last_n_test green; p99 ≤ 200µs at 10k rows |
| 0.7 | JSON-RPC: `save.rename`, `save.delete`, `save.pin`, `save.thumbnail` | `crates/server/src/saves.rs` + `jsonrpc.rs` | JSON-RPC catalog test green |
| 0.8 | JSON-RPC: `save.diff`, `save.last_n` | `crates/server/src/saves.rs` + `jsonrpc.rs` | JSON-RPC catalog test green |
| 0.9 | JSON-RPC: `save.export`, `save.import` (zip envelope; spec inline in code comments; full spec deferred to `docs/specs/SAVE_FORMAT.md` in Phase 4) | `crates/server/src/saves.rs` + `jsonrpc.rs` | round-trip export→import test green |

**Verify:** `just civis-3d-verify` + `just civis-3d-catalog-check`.

### Phase 1 — Shared `crates/hud` types + token audit

**Goal:** define `KeycapPalette`, `TileInspector`, `HudState` in a substrate-neutral crate, with a **token-audit test** that fails if anyone introduces a new hex outside `ui-design-language.md`.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 1.1 | New crate `crates/hud` with `Cargo.toml`, `lib.rs`, `key_palette.rs`, `tile_inspector.rs`, `tokens.rs` | `crates/hud/` | `cargo check -p civ-hud` green |
| 1.2 | `KeycapPalette { keys: [KeycapDef; 8] }` — verbatim §4.2 recipe | `crates/hud/src/key_palette.rs` | `KeycapPalette::default()` matches `GOD_TOOLS_SANDBOX.md:85-95` |
| 1.3 | `TileInspector { cell, height, mat, biome, slope, agent_count, faction, mood }` | `crates/hud/src/tile_inspector.rs` | `TileInspector::from(sim, cell)` round-trip test |
| 1.4 | `HudState { top_bar, key_palette, tile_inspector, save_panel }` — the open/closed FSM | `crates/hud/src/lib.rs` | `HudState::default()` opens top-bar + key_palette only |
| 1.5 | `tokens::audit(hex) -> bool` — **fail if a hex outside `ui-design-language.md` is referenced** | `crates/hud/src/tokens.rs` | `crates/hud/tests/token_audit.rs` enumerates every hex in the crate and asserts membership in the canonical token list |
| 1.6 | Wire `crates/hud` → `crates/server` (read-only state from `sim.snapshot`) | `crates/hud/src/wire.rs` | `HudState::from_snapshot(snapshot)` test |

**Verify:** `cargo test -p civ-hud`.

### Phase 2 — `crates/save-ui` panel (web-first)

**Goal:** the Save/Load panel rendered by `crates/watch` as a server-served HTML/JS widget. This is the path-of-least-resistance: no native UI work, immediate parity with the wire.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 2.1 | New crate `crates/save-ui` (`Cargo.toml`, `lib.rs`, markup templates) | `crates/save-ui/` | `cargo check -p civ-save-ui` green |
| 2.2 | Markup: panel shell + 4 tabs (`SLOTS`, `RESTORE`, `MANAGE`, `IMPORT/EXPORT`) | `crates/save-ui/src/markup.rs` | rendered DOM matches §2.2 ASCII; snapshot test |
| 2.3 | `slot_row(slot: SaveBrowserEntry)` component | `crates/save-ui/src/slot_row.rs` | renders thumbnail + label + age + mods + Δ; snapshot test |
| 2.4 | `restore_panel(slot: SaveBrowserEntry, diff: SaveDiffSummary)` | `crates/save-ui/src/restore_panel.rs` | three-button confirm flow; focus-on-Save-current-and-restore |
| 2.5 | `manage_panel(rows: Vec<SaveBrowserEntry>)` (rename inline, delete confirm, pin toggle, evict stepper) | `crates/save-ui/src/manage_panel.rs` | keyboard-accessible; `WARN` accent on destructive |
| 2.6 | `import_export_panel` (drag-drop + file picker) | `crates/save-ui/src/import_export.rs` | file ingest + `save.export`/`save.import` round-trip |
| 2.7 | Toast widget for `F5` quick-save feedback | `crates/save-ui/src/toast.rs` | auto-dismiss @ 1.6s; chrome (not holo) |
| 2.8 | State machine: `PanelTab × SelectedSlot × ConfirmingAction` | `crates/save-ui/src/state.rs` | exhaustive `match`; no illegal states |
| 2.9 | Mount in `crates/watch` at `/saveload` route | `crates/watch/src/routes.rs` | `curl localhost:9090/saveload` returns the panel HTML |
| 2.10 | Wire `save-ui` to JSON-RPC: every button → corresponding method | `crates/save-ui/src/wire.rs` | integration test against a stub server |

**Verify:** `cd web && npm test && cd web && npm run build` + `cargo test -p civ-save-ui`.

### Phase 3 — `crates/hud-web` + dashboard HUD integration

**Goal:** the in-game HUD (top-bar + Keycap Palette + tile inspector) on the web dashboard. Reuses `crates/hud` types.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 3.1 | New crate `crates/hud-web` (`Cargo.toml`, `lib.rs`) | `crates/hud-web/` | `cargo check -p civ-hud-web` green |
| 3.2 | Top-bar chip components: `POP`, `ERA`, `TREASURY`, `YEAR`, `TIME`, `SPEED`, `AUTO` | `crates/hud-web/src/chips.rs` | each chip styled per §4.1 token table |
| 3.3 | `KeycapPalette` component (8 keycaps, verbatim recipe) | `crates/hud-web/src/keycap.rs` | snapshot test; token audit (no new hex) |
| 3.4 | `TileInspector` component (HOLO; opens on tool-arm + Q; closes when save panel opens) | `crates/hud-web/src/tile_inspector.rs` | density-rule test (≤ 2 holo surfaces visible) |
| 3.5 | `SpeedPicker` popover (`Shift+S`) | `crates/hud-web/src/speed_picker.rs` | keyboard + mouse; mono numerics |
| 3.6 | Mount HUD on the dashboard at `/play` (existing route) | `web/src/play.ts` | HUD persists through save panel open |
| 3.7 | Wire HUD actions to JSON-RPC: `sim.set_speed`, `inspect.probe`, `power.activate` | `crates/hud-web/src/wire.rs` | no-direct-substrate-mutation test |
| 3.8 | AUTO chip wired to `save.last_n` (200ms poll) | `crates/hud-web/src/auto_chip.rs` | chip color changes per age band |

**Verify:** `cd web && npm test && cd web && npm run build` + `cargo test -p civ-hud-web`.

### Phase 4 — Native clients (Bevy, Godot, Unreal)

**Goal:** the HUD on the native reference clients; the save/load panel on at least the Bevy reference.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 4.1 | Bevy: mount `KeycapPalette` + `TileInspector` + `TopBar` via `bevy_egui` | `clients/bevy-ref/src/game_ui.rs` | existing game_ui.rs (1–260) extended; HUD visible in PIE |
| 4.2 | Bevy: mount `SaveLoadPanel` via `bevy_egui` (re-using `crates/save-ui` markup via a `bevy_egui` adapter) | `clients/bevy-ref/src/save_load_panel.rs` | `F5`/`F9` open the panel; round-trip save→load works |
| 4.3 | Godot: HUD scene (`HUD.tscn`) with Control nodes per the token list | `clients/godot-ref/` | `just godot-test` green; HUD visible |
| 4.4 | Unreal: UMG widgets per the token list | `clients/unreal-show/` | `clients/unreal-show/scripts/build.ps1` green |
| 4.5 | Format spec `docs/specs/SAVE_FORMAT.md` (extract from `crates/engine/src/save.rs` comments) | `docs/specs/` | doc matches code byte-for-byte (snapshot test) |

**Verify:** `.\scripts\agent-smoke.ps1` + `just godot-test` + `clients/unreal-show/scripts/build.ps1`.

### Phase 5 — Polish + density-rule enforcement

**Goal:** AC-SL-9 (save panel never renders holo), AC-HUD-3 (token audit), and the ≤ 2 HUD holo surfaces rule hold across all clients.

| ID | Task | Crate | Acceptance |
|----|------|-------|------------|
| 5.1 | Vision-verify rule on the save panel: scanlines? → fail. Holo fill? → fail. | `scripts/quality/` (extend) | `CIVIS_QUALITY_UNREAL=1` + `emit-quality-manifest.ps1` shows save panel as 0% holo |
| 5.2 | Vision-verify rule on the HUD: ≤ 2 holo surfaces visible across the 4 HUD states (closed / panel-open / inspector-open / panel+inspector-open) | `scripts/quality/` | 4 screenshots captured; pixel scan passes |
| 5.3 | Token audit across all 5 crates (`save-ui`, `hud`, `hud-web`, bevy-ref, godot-ref, unreal-show) | `scripts/quality/token_audit.py` | zero new hex tokens introduced |
| 5.4 | End-to-end test: load game → tick 5 min → autosave fires → quit → reload → state matches | `crates/save-db/tests/e2e_test.rs` | e2e_test green |
| 5.5 | End-to-end test: load a slot with mismatched mods → Restore disabled + chip | `crates/save-ui/tests/mods_mismatch_test.rs` | green |
| 5.6 | README cross-links from `docs/design/ui-design-language.md` §6 (top resource bar) and `GOD_TOOLS_SANDBOX.md` §2.1–2.2 to this doc | `docs/design/` | n/a (doc-only) |

**Verify:** `just civis-3d-verify` + `.\scripts\agent-smoke.ps1` + `scripts/quality/`.

### Phase 6 — Future work (NOT in this PR)

- Cloud sync (the panel stub: a "Cloud" tab behind a feature flag; off by default).
- Multiplayer save coordination (see `docs/development-guide/fr-mp-lobby.md`).
- Save format versioning (`save_format_version` column; migration tests).
- Cross-slot "diff vs current session" (live diff while playing) — deferred to a separate doc.

---

## 7. Risks + mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Holo density breach** — adding the tile inspector while the save panel is open pushes to 2 holo surfaces, leaving no room for the in-world brush ring. | M | M | Density-rule test (5.2) + spec rule: tile inspector closes when save panel opens (AC-HUD-5). |
| **Restore loses unsaved session** — the charter forbids silent data loss on quit/restore. | L | H | `Save-current-and-restore` is the default focus (AC-SL-10); the bare `Discard-and-restore` is `WARN`-accented and requires explicit click. |
| **Mod mismatch on restore** — a slot saved with mod X is loaded without X → broken state. | M | H | `save.load` validates the mod manifest; the Restore button disables if mods mismatch (AC-SL-4). |
| **Thumbnail bloat** — 16 KB PNG × 100s of slots = MBs. | L | L | Hard cap at 16 KB + `attach_thumbnail` enforces size; eviction runs on autosaves only (manual slots keep their thumbnails). |
| **Token creep** — a contributor introduces a new hex outside `ui-design-language.md`. | M | M | `crates/hud/tests/token_audit.rs` + `scripts/quality/token_audit.py` (5.3). **Adding new tokens is forbidden by the design language discipline.** |
| **Spec drift** — `crates/engine/src/save.rs` changes without updating `docs/specs/SAVE_FORMAT.md` (deferred to Phase 4.5). | M | M | Defer the spec doc to Phase 4 (after the wire stabilizes in Phase 0); the inline comments in `save.rs` are the source of truth until then. |
| **HUD scope creep** — the HUD wants to show "everything" (every agent, every diff, every event). | M | M | The HUD is **read-only**. The Holocron Deck (existing, see `GOD_TOOLS_SANDBOX.md`) is the verb surface. The HUD is the instrument panel. |

---

## 8. Open questions (resolve before Phase 1 ships)

1. **Save format versioning.** Do we add `save_format_version: u32` to `SaveArchive` now (Phase 0) or defer to Phase 6? — *Recommend: defer to Phase 6; the format has not changed since v1.*
2. **Autosave interval default.** `600` ticks (10 min @ 1×) — confirm with scenario config. — *Default OK; scenario-configurable.*
3. **Autosave eviction default.** `8` autosaves — confirm. — *Default OK; scenario-configurable.*
4. **HUD web polling.** 200ms poll on `save.last_n` for the AUTO chip — confirm this is acceptable for the dashboard. — *200ms is acceptable; the watch event bus already emits `mod.loaded.v1` and `snapshot.saved` events we could subscribe to instead — defer to a Phase 6 optimization.*
5. **Save panel hotkey conflict.** `F5` is the browser refresh on the web dashboard. — *Use `Ctrl+S` instead on web; `F5` only on native. Note in the panel's keyboard cheatsheet.*

---

## 9. Out-of-scope (explicit non-goals)

- Cloud sync (Phase 6 stubs only).
- Multiplayer save coordination (separate doc).
- Save format versioning (Phase 6).
- Save diff against the *current live session* while playing (separate doc).
- Autosave on every tick (charter violation — too noisy; the chip exists to make autosaves legible, not to fire them every second).
- A "cloud browser" tab — deferred.
- Save thumbnails for the **imported** slots — they'll show a default `civis-icon` placeholder; full thumbnail only on native saves.

---

## 10. Single-line summary

**Two UI surfaces, one shared crate, one backend extension, six phases.** Save/load flows from `F5`/`F9`/`Esc` into a chrome Keycap modal (4 tabs: Slots, Restore, Manage, Import/Export) backed by `crates/save-db` extensions (`rename`, `delete_slot`, `attach_thumbnail`, `diff_summary`, `pin`, `last_n_for_session`); the in-game HUD (top-bar + Keycap Palette + tile inspector + sim-speed keycaps) lives in a new shared `crates/hud` crate that all four clients (web, Bevy, Godot, Unreal) render against the verbatim `ui-design-language.md` token list — no new hex, ≤ 2 HUD holo surfaces visible at any moment, autosave policy keeps 8 by default, restore-with-autosave is atomic server-side.