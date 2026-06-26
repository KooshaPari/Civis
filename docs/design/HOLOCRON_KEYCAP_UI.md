# Holocron Keycap UI — Design Pass

**Status:** Draft (2026-06-25)
**Branch:** `feat/holocron-keycap-design`
**Scope:** UI / UX substrate layer (no sim changes)
**Depends on:** `crates/hud/src/key_palette.rs` (keycap palette tokens), `clients/bevy-ref/src/god_panel.rs` (god verbs panel), ADR-012 (keycap palette design system)

---

## Why this exists

The Civis godgame has accumulated a **dense, well-named godgame substrate**:

- **22/23 emergence phases** wired into `Simulation::tick` (cohesion, stratification, social_mood, institutions, belief, unrest, market, trade, religion, law, faction, era, psyche, disasters, …)
- **Save/load round-trip** functional (`crates/engine/src/save.rs`)
- **MCP surface** (~32 tools) for agentic godgame access
- **JSON-RPC + WebSocket** bridge with psyche read-API
- **egui god_panel** with keycap-palette-styled buttons (501 lines in `clients/bevy-ref/src/god_panel.rs`)

But the **discoverability layer** for human players is still a wall of buttons. Players see ~20 godgame categories in the panel but don't have a fast way to *find* a specific verb, *preview* what it will do, or *chain* it with the world state. That's what the **Holocron Keycap UI** is for.

## Naming

- **Holocron** (Star Wars reference): the in-fiction container of Jedi wisdom — here, the player's **personal index** of godgame verbs, accumulated through play
- **Keycap**: the visual + interaction idiom — physical-keycap-shaped UI tokens (per ADR-012), one verb per key
- **UI**: the UX surface that ties it together

Three layers:
1. **Keycap palette** (existing) — the visual tokens (per ADR-012)
2. **Holocron panel** (new) — the player's in-world verb registry, paginated/categorized, persistent across sessions
3. **Command-K palette** (new) — the fast-launcher overlay (Spotlight/Command-K pattern)

## UX goals

1. **< 200 ms** to launch any verb (command-K → type → enter)
2. **Discoverable**: verbs are listed by category, with provenance ("first discovered when X happened at tick N")
3. **Persistent**: every verb the player has ever used shows up in their Holocron, with last-used timestamp
4. **Context-aware**: in "disaster" tick state, surface `calm_disaster`, `divine_warning`, etc. at the top
5. **Substrate-faithful**: every verb is bound to a `sim.god_action` endpoint; no UI-only verbs
6. **Replayable**: Holocron is part of save/load — the player's relationship with their verbs is part of the game

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│ HolocronKeycapUi (top-level widget, dockable, toggleable)     │
│ ┌────────────────────────────┬──────────────────────────────┐ │
│ │ HolocronPanel              │ CommandKOverlay              │ │
│ │ - verb registry            │ - cmd-line input             │ │
│ │ - per-verb usage stats     │ - fuzzy match                │ │
│ │ - per-verb provenance      │ - arg scaffolding            │ │
│ │ - per-tick context rank    │ - arg preview                │ │
│ └────────────────────────────┴──────────────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
                          │
                          ▼
              HolocronKeycapBridge (MCP + JSON-RPC)
                          │
                          ▼
              sim.god_action(verb, args, tick, provenance)
```

### Module layout

```
crates/hud/src/
├── key_palette.rs          # existing: visual tokens
├── holocron/
│   ├── mod.rs              # HolocronKeycapUi root widget
│   ├── panel.rs            # HolocronPanel — verb registry view
│   ├── command_k.rs        # CommandKOverlay — fast-launcher
│   ├── registry.rs         # verb_registry: HashMap<VerbId, VerbEntry>
│   ├── provenance.rs       # Provenance: when/why verb was discovered
│   └── bridge.rs           # HolocronKeycapBridge — MCP/JSON-RPC adapter
```

### Data model

```rust
pub struct VerbEntry {
    pub id: VerbId,                    // e.g. "disaster.calm"
    pub label: String,                 // "Calm Disaster"
    pub category: Category,            // Disaster, Religion, Faction, Era, …
    pub args: Vec<ArgSpec>,            // { name, type, default, description }
    pub first_used_tick: Option<Tick>,
    pub last_used_tick: Option<Tick>,
    pub use_count: u32,
    pub provenance: Vec<Provenance>,   // (tick, context, summary)
    pub keycap: KeyCap,                // per ADR-012 palette
}

pub struct Provenance {
    pub tick: Tick,
    pub context: String,               // "natural_disaster struck the Fields district"
    pub result: String,                // "disaster intensity -12%"
    pub sim_state_hash: Hash,          // for replay verification
}

pub struct Category {
    pub id: CategoryId,                // "disaster" | "religion" | …
    pub label: String,
    pub color: KeyCapColor,
    pub icon: KeyCapIcon,
    pub order: u8,
}
```

### Bridge protocol

```rust
pub trait HolocronKeycapBridge {
    /// Returns the full verb registry for this Holocron (player-specific)
    async fn list_verbs(&self) -> Result<Vec<VerbEntry>>;

    /// Fires a verb, returns the result + updated provenance
    async fn fire(&self, verb: VerbId, args: Value, provenance: Provenance) -> Result<FireResult>;

    /// Returns verbs relevant to the current sim state (context-aware ranking)
    async fn contextual_verbs(&self, sim_state: &SimSnapshot) -> Result<Vec<(VerbEntry, f32)>>;
}
```

Two impls:
- `McpBridge` — wraps `civis-mcp` tool surface (already has ~32 verbs)
- `JsonRpcBridge` — wraps `crates/server/src/jsonrpc.rs` (the `god_action` endpoint)

Both produce identical `VerbEntry` output — the UI never knows which substrate it's talking to.

### Persistence

The Holocron is part of save/load (`crates/engine/src/save.rs`). On `save`:
- Serialize `registry: HashMap<VerbId, VerbEntry>` to disk
- On `load`: rehydrate, replay `provenance` events from sim log if present

This makes the Holocron a **first-class saveable entity** — the player's verb history is preserved across sessions.

## UX flow

### Flow 1: First-time verb discovery

1. Player sees a new disaster (`disaster.flood` starts in `SimState::disasters`)
2. `HolocronPanel` automatically surfaces a "NEW" badge on `disaster.calm`
3. Tooltip explains: "First appeared when the Flood struck at tick 4127. Calms a natural disaster by 60% at the cost of 1 divine favor."
4. One-click → verb fires → provenance recorded → badge cleared

### Flow 2: Command-K fast-launch

1. Player hits `Ctrl+K` (or `Cmd+K` on macOS)
2. `CommandKOverlay` opens, takes input
3. Type "cal" → fuzzy match shows `disaster.calm`, `disaster.calm_gently`, `religion.calm_disbelievers`
4. Hit `Enter` → arg scaffolding appears (e.g. target district, intensity)
5. Hit `Enter` again → verb fires, provenance recorded, overlay closes

### Flow 3: Context-aware ranking

1. Sim tick: 5 disasters active, religion crisis in faction B, trade war between factions A & C
2. `contextual_verbs()` ranks:
   - Top: `disaster.calm`, `disaster.divert`, `religion.inspire_unity`, `faction.intervene`
   - Mid: `religion.calm_disbelievers`, `faction.send_envoy`
   - Bottom: `era.advance`, `trade.subsidize`, `market.price_fix`
3. UI surfaces top-N in a "now" panel above the Holocron

## Visual design (per ADR-012 keycap palette)

Each verb gets a **physical-keycap-shaped card**:
- **Top edge**: keycap glyph (`F1`-`F12`, `1`-`0`, or letter+modifier)
- **Body**: verb label, category color stripe, use-count badge
- **Bottom edge**: provenance indicator ("Last used: tick 8213", or "NEW")

Color tokens from `crates/hud/src/key_palette.rs`:
- **Disaster**: `palette::disaster_red` (#c8392b)
- **Religion**: `palette::religion_gold` (#d4a72c)
- **Faction**: `palette::faction_blue` (#3878c8)
- **Market/Trade**: `palette::market_green` (#3a9c5d)
- **Era**: `palette::era_violet` (#7c3fb5)
- **Psyche**: `palette::psyche_teal` (#2ea89c)
- **Tech**: `palette::tech_silver` (#a8b3c0)
- **Law**: `palette::law_bronze` (#a07c4f)

Typography: `key_palette::mono()` for category labels (high-contrast on dark UI), `key_palette::display()` for verb labels (large, readable).

## Phased implementation

### Phase 1: Substrate (1 PR)
- Add `crates/hud/src/holocron/` skeleton
- `registry.rs`: `VerbEntry` + `Provenance` + serialization
- `bridge.rs`: `HolocronKeycapBridge` trait + `McpBridge` impl
- Tests: registry round-trip, bridge contract

### Phase 2: Panel UI (1 PR)
- `panel.rs`: `HolocronPanel` widget with category-grouped verb registry
- `provenance.rs`: provenance timeline view
- Wire into `god_panel.rs` as a tab
- Tests: panel render snapshot

### Phase 3: Command-K (1 PR)
- `command_k.rs`: `CommandKOverlay` with fuzzy match + arg scaffolding
- Wire to `Ctrl+K` global hotkey
- Tests: input → match → fire flow

### Phase 4: Context-aware ranking (1 PR)
- `contextual_verbs()` impl in bridge
- "Now" panel above main Holocron
- Tests: ranking quality on canned sim states

### Phase 5: Persistence (1 PR)
- Wire Holocron into `save.rs` / `load.rs`
- On load: replay provenance events from sim log
- Tests: save/load round-trip preserves Holocron

### Phase 6: Holocron-as-narrative (stretch)
- Holocron learns from player behavior (e.g. uses `disaster.calm` 20 times → suggests `disaster.prevent` as next verb)
- Surface "verb relationship graph" — which verbs synergize / contradict
- Auto-generate "your godgame story" from provenance timeline

## Acceptance criteria

- [ ] Command-K launches any verb in < 200 ms (p95)
- [ ] Holocron persists across save/load
- [ ] All 22 wired emergence phases have at least one discoverable verb
- [ ] Context-aware ranking surfaces top-3 verbs within 1 tick of state change
- [ ] No UI-only verbs (all bound to `sim.god_action`)
- [ ] All UI text accessible (screen reader, high-contrast mode per ADR-012)
- [ ] Tests: registry, bridge, panel render, command-K flow, ranking, persistence

## Anti-goals (explicit non-targets)

- **NOT** a replacement for the egui `god_panel` — Holocron is an *index* and *launcher*, the panel is the *workspace*
- **NOT** a quest/mission system — Holocron is a verb registry, not a to-do list
- **NOT** a tutorial system — tutorials are separate (per FR-TUTORIAL)
- **NOT** moddable in this pass — verb set is fixed by sim substrate

## Open questions

1. Should the Holocron be **player-specific** (one per save) or **profile-wide** (shared across all saves)?
   - Recommendation: per-save (part of the save entity), with optional profile-wide "discoveries" badge
2. Should verbs be **unlockable** (gated by sim progression) or **always available** (just contextual)?
   - Recommendation: always available; context-aware ranking IS the gating
3. Should Command-K use **fuzzy match** (subsequence) or **prefix match** (start-of-string)?
   - Recommendation: subsequence (more forgiving, faster for partial recall)

## Out of scope

- 3D spatial UI (per ADR-013) — Holocron is panel-based only
- Voice control — separate feature
- Touch / mobile gestures — desktop-first per ADR-012
- Multiplayer Holocron sharing — separate feature