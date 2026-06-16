//! HUD (Heads-Up Display) panel data substrate.
//!
//! Engine-agnostic, pure-data state models for the five primary HUD panels
//! spec'd in `agileplus-specs/civ-011-bevy-primary-client/spec.md` and
//! tracked under the `FR-CIV-HUD-001..005` family:
//!
//! - [`ToolPalette`] + [`ToolEntry`]       — `FR-CIV-HUD-001` (tool palette).
//! - [`TechTree`] + [`TechNode`]           — `FR-CIV-HUD-002` (tech tree panel).
//! - [`DiplomacyPanel`] + [`DiplomacyFsm`] — `FR-CIV-HUD-003` (diplomacy).
//! - [`EventFeed`] + [`EventFeedItem`]     — `FR-CIV-HUD-004` (event feed).
//! - [`MenuStack`] + [`MenuKind`]          — `FR-CIV-HUD-005` (menu system).
//!
//! The substrate deliberately holds NO rendering, NO `bevy` dependency, and
//! NO `Instant::now` / RNG. It is the single source of truth for "what is
//! currently in each panel?" and the Bevy-gated adapter is expected to bind
//! these state types to its widgets.
//!
//! ## Determinism
//!
//! Every public collection is `BTreeMap`-keyed where the natural key is
//! defined by the spec. Iteration order is therefore stable across runs and
//! the same panel state round-trips through `serde` byte-for-byte.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// FR-CIV-HUD-001 — Tool palette
// ---------------------------------------------------------------------------

/// What broad category a tool belongs to. Used by the panel renderer to
/// group entries into columns / tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// Build menu (roads, structures, zones).
    Build,
    /// Zone menu (housing, industry, agriculture).
    Zone,
    /// Citizen / agent inspection tools.
    Inspect,
    /// Disaster / emergency response tools.
    Disaster,
    /// Map / overlay tools.
    Map,
}

/// Whether the tool is currently usable by the player. Disabled entries are
/// rendered greyed-out and do not respond to clicks; the spec requires
/// "disabled when out of resources".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAvailability {
    /// Tool can be selected and used.
    Enabled,
    /// Tool cannot be used right now (insufficient resources, wrong era, etc.).
    Disabled,
}

/// One entry in the [`ToolPalette`]. The Bevy adapter renders one widget
/// per entry; the substrate stores the data the widget binds to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolEntry {
    /// Stable, unique id (also used for the keyboard shortcut table).
    pub id: String,
    /// Human-readable label shown next to the icon.
    pub label: String,
    /// Tier / era at which the tool unlocks (0 = always available).
    pub tier: u8,
    /// Single-character keyboard shortcut (`'B'`, `'R'`, etc.). `None` means
    /// "no shortcut bound".
    pub shortcut: Option<char>,
    /// Category used to group entries in the panel UI.
    pub category: ToolCategory,
    /// Whether the tool is currently selectable.
    pub availability: ToolAvailability,
}

impl ToolEntry {
    /// Construct a tier-0 enabled tool with no keyboard shortcut. Used as
    /// the common case for build / zone menus.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, category: ToolCategory) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tier: 0,
            shortcut: None,
            category,
            availability: ToolAvailability::Enabled,
        }
    }

    /// Builder helper: assign a tier (unlock era).
    #[must_use]
    pub fn with_tier(mut self, tier: u8) -> Self {
        self.tier = tier;
        self
    }

    /// Builder helper: assign a single-character keyboard shortcut.
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Builder helper: mark the entry as disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.availability = ToolAvailability::Disabled;
        self
    }
}

/// Errors returned by [`ToolPalette`] mutators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPaletteError {
    /// `set_selected` was called with a tool id that is not present in the
    /// palette, or that is currently `Disabled`. The spec requires the
    /// selected tool to always be highlightable; selecting a disabled tool
    /// is a programmer error.
    UnknownOrDisabledTool(String),
}

/// The tool palette panel state. Entries are stored in a `BTreeMap` keyed by
/// the entry's stable `id` so iteration order matches the spec's "listing
/// buildable structure types by tier" requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolPalette {
    /// Map of `id -> entry`. Iteration is in `id` ascending order.
    pub entries: BTreeMap<String, ToolEntry>,
    /// Currently selected tool id. `None` means "no tool selected".
    pub selected: Option<String>,
    /// Schema version of this panel state shape.
    pub schema_version: String,
}

/// Schema version of the [`ToolPalette`] data shape. Bump on breaking
/// changes to the struct.
pub const HUB_PALETTE_SCHEMA_VERSION: &str = "0.1.0-hub-palette";

impl ToolPalette {
    /// Construct an empty palette bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            selected: None,
            schema_version: HUB_PALETTE_SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert or replace an entry. Returns `&mut self` for chaining.
    pub fn insert(&mut self, entry: ToolEntry) -> &mut Self {
        self.entries.insert(entry.id.clone(), entry);
        self
    }

    /// Mark a tool as the currently-selected one. Fails if the id is
    /// missing or the entry is `Disabled`.
    pub fn set_selected(&mut self, id: &str) -> Result<&mut Self, ToolPaletteError> {
        match self.entries.get(id) {
            Some(entry) if entry.availability == ToolAvailability::Enabled => {
                self.selected = Some(id.to_string());
                Ok(self)
            }
            Some(_) => Err(ToolPaletteError::UnknownOrDisabledTool(id.to_string())),
            None => Err(ToolPaletteError::UnknownOrDisabledTool(id.to_string())),
        }
    }

    /// Mark every entry as `Enabled` whose `tier <= current_tier`. The
    /// caller (panel renderer or game-state adapter) is responsible for
    /// calling this when the local polity's tier changes.
    pub fn refresh_for_tier(&mut self, current_tier: u8) -> &mut Self {
        for entry in self.entries.values_mut() {
            entry.availability = if entry.tier <= current_tier {
                ToolAvailability::Enabled
            } else {
                ToolAvailability::Disabled
            };
        }
        // If the currently-selected tool just became disabled, drop the
        // selection rather than leave a dangling pointer.
        if let Some(id) = self.selected.as_ref() {
            let is_enabled = self
                .entries
                .get(id)
                .is_some_and(|e| e.availability == ToolAvailability::Enabled);
            if !is_enabled {
                self.selected = None;
            }
        }
        self
    }

    /// Number of entries the palette currently holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when the palette is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ToolPalette {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-HUD-002 — Tech tree panel
// ---------------------------------------------------------------------------

/// What state a research node is in relative to the local polity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechStatus {
    /// Locked: prerequisites not met, or polity tier too low.
    Locked,
    /// Unlocked: prerequisites met, the player can queue research.
    Available,
    /// Queued for research (waiting on the research tick).
    InProgress,
    /// Researched.
    Researched,
}

/// One research node in the [`TechTree`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechNode {
    /// Stable id (e.g. `"tech_masonry"`).
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Cost to research, in some resource unit (the substrate is
    /// resource-agnostic; the research crate interprets the units).
    pub cost: u32,
    /// Ids of nodes that must be `Researched` before this one becomes
    /// `Available`. Stored in a `BTreeSet` for determinism.
    pub prerequisites: std::collections::BTreeSet<String>,
    /// Current state of this node.
    pub status: TechStatus,
}

impl TechNode {
    /// Construct a node with no prerequisites and `Locked` status.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, cost: u32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            cost,
            prerequisites: std::collections::BTreeSet::new(),
            status: TechStatus::Locked,
        }
    }

    /// Builder helper: declare a single prerequisite.
    #[must_use]
    pub fn requires(mut self, prereq: impl Into<String>) -> Self {
        self.prerequisites.insert(prereq.into());
        self
    }

    /// `true` when every prerequisite is `Researched`.
    #[must_use]
    pub fn prerequisites_met(&self, tree: &TechTree) -> bool {
        self.prerequisites.iter().all(|p| {
            tree.nodes
                .get(p)
                .is_some_and(|n| n.status == TechStatus::Researched)
        })
    }

    /// `true` when the node is researchable *right now* (i.e. `Available`
    /// and the player can afford the cost — cost is checked by the caller,
    /// this method only checks status + prereqs).
    #[must_use]
    pub fn can_queue(&self, tree: &TechTree) -> bool {
        matches!(self.status, TechStatus::Available) && self.prerequisites_met(tree)
    }
}

/// Errors returned by [`TechTree`] mutators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechTreeError {
    /// The node id is not present in the tree.
    UnknownNode(String),
    /// The player tried to queue a node whose prerequisites are not met or
    /// whose status is not `Available`.
    NotQueueable(String),
    /// The player tried to declare a prerequisite that would create a cycle.
    Cycle(String),
}

/// The tech tree panel state. Nodes are keyed by stable id; iteration is
/// in id-ascending order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechTree {
    /// Map of `id -> node`. Iteration is in `id` ascending order.
    pub nodes: BTreeMap<String, TechNode>,
    /// Schema version of this panel state shape.
    pub schema_version: String,
}

/// Schema version of the [`TechTree`] data shape.
pub const HUB_TECH_SCHEMA_VERSION: &str = "0.1.0-hub-tech";

impl TechTree {
    /// Construct an empty tree bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            schema_version: HUB_TECH_SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert or replace a node. Returns `&mut self` for chaining.
    pub fn insert(&mut self, node: TechNode) -> &mut Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Recompute every node's `status` from its prerequisites. Run this
    /// after any node's status changes (e.g. after research completes) so
    /// newly-unlocked nodes flip from `Locked` to `Available`.
    pub fn refresh_statuses(&mut self) -> &mut Self {
        // First pass: mark `Locked` nodes that have all prereqs met as
        // `Available`. We do NOT touch `InProgress` or `Researched` —
        // those states are managed by the research crate.
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in ids {
            let prereqs_met = {
                let node = &self.nodes[&id];
                node.status == TechStatus::Locked && node.prerequisites_met(self)
            };
            if prereqs_met {
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.status = TechStatus::Available;
                }
            }
        }
        self
    }

    /// Mark a node as `InProgress` (queue research). Fails if the node is
    /// unknown or not currently queueable.
    pub fn queue(&mut self, id: &str) -> Result<&mut Self, TechTreeError> {
        let can = self.nodes.get(id).is_some_and(|n| n.can_queue(self));
        if !can {
            return Err(TechTreeError::NotQueueable(id.to_string()));
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = TechStatus::InProgress;
        }
        Ok(self)
    }

    /// Mark a node as `Researched` and re-derive downstream statuses. This
    /// is the "research tick succeeded" path.
    pub fn complete(&mut self, id: &str) -> Result<&mut Self, TechTreeError> {
        if !self.nodes.contains_key(id) {
            return Err(TechTreeError::UnknownNode(id.to_string()));
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = TechStatus::Researched;
        }
        self.refresh_statuses();
        Ok(self)
    }

    /// Return every node whose status is `Available`, in id-ascending order.
    pub fn available_nodes(&self) -> impl Iterator<Item = &TechNode> {
        self.nodes
            .values()
            .filter(|n| n.status == TechStatus::Available)
    }
}

impl Default for TechTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-HUD-003 — Diplomacy panel
// ---------------------------------------------------------------------------

/// The diplomatic state machine the spec wires to a "FSM state badge".
/// Values match the FSM states the DIPLO crate exposes; the substrate
/// stores the current value so the panel can render the badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomacyFsm {
    /// No diplomatic contact yet.
    Neutral,
    /// Open trade / open borders, no formal alliance.
    Open,
    /// Formal defensive / offensive alliance.
    Allied,
    /// Active sanctions (trade freeze, no diplomacy).
    Sanctioned,
    /// Active war.
    AtWar,
}

/// One row in the diplomacy panel: a treaty slot, the FSM state, and the
/// influence-capital bar the spec calls for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreatySlot {
    /// Stable id of the other polity.
    pub polity_id: String,
    /// Display name for the panel.
    pub polity_label: String,
    /// Current FSM state.
    pub fsm: DiplomacyFsm,
    /// Influence capital in `[0, 1]`. The panel renders this as a bar.
    /// `Eq` is intentionally not derived because `f32` does not implement
    /// it (NaN handling); `PartialEq` is enough for tests + serde.
    pub influence_capital: f32,
}

impl TreatySlot {
    /// Construct a neutral slot at zero influence.
    #[must_use]
    pub fn new(polity_id: impl Into<String>, polity_label: impl Into<String>) -> Self {
        Self {
            polity_id: polity_id.into(),
            polity_label: polity_label.into(),
            fsm: DiplomacyFsm::Neutral,
            influence_capital: 0.0,
        }
    }

    /// Builder helper: assign an FSM state.
    #[must_use]
    pub fn with_fsm(mut self, fsm: DiplomacyFsm) -> Self {
        self.fsm = fsm;
        self
    }

    /// Builder helper: assign an influence capital in `[0, 1]`. Values
    /// outside the range are clamped so the renderer never has to deal
    /// with a bar that would overflow.
    #[must_use]
    pub fn with_influence(mut self, influence: f32) -> Self {
        self.influence_capital = influence.clamp(0.0, 1.0);
        self
    }
}

/// The diplomacy panel state. Slots are stored in a `BTreeMap` keyed by the
/// other polity's id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyPanel {
    /// Map of `polity_id -> slot`. Iteration is in `polity_id` ascending order.
    pub slots: BTreeMap<String, TreatySlot>,
    /// Schema version of this panel state shape.
    pub schema_version: String,
}

impl DiplomacyPanel {
    /// Schema version constant.
    pub const SCHEMA_VERSION: &'static str = "0.1.0-hub-diplomacy";

    /// Construct an empty panel bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: BTreeMap::new(),
            schema_version: Self::SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert or replace a slot. Returns `&mut self` for chaining.
    pub fn insert(&mut self, slot: TreatySlot) -> &mut Self {
        self.slots.insert(slot.polity_id.clone(), slot);
        self
    }

    /// Look up a slot by polity id.
    #[must_use]
    pub fn get(&self, polity_id: &str) -> Option<&TreatySlot> {
        self.slots.get(polity_id)
    }
}

impl Default for DiplomacyPanel {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-HUD-004 — Event feed
// ---------------------------------------------------------------------------

/// How severe an event is. The spec calls for color-coding by severity; the
/// substrate stores the bucket, the renderer maps to a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSeverity {
    /// Cosmetic / flavor (a festival, a rumor).
    Info,
    /// Notable but not disruptive (a treaty signed).
    Notable,
    /// Disruptive (a market crash, an election upset).
    Warning,
    /// Crisis (a war, a disaster).
    Critical,
}

/// One item in the event feed. `tick` is the simulation tick at which the
/// event occurred; `region` is the world region the renderer should focus
/// the camera on when the player clicks the entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFeedItem {
    /// Stable id (used for dedup across feeds).
    pub id: String,
    /// Simulation tick the event happened at.
    pub tick: u64,
    /// Severity bucket.
    pub severity: EventSeverity,
    /// Short headline shown in the feed row.
    pub headline: String,
    /// Optional longer description; the panel can show this on hover.
    pub body: Option<String>,
    /// Optional region id the camera should focus on click. `None` means
    /// "no specific focus".
    pub region: Option<String>,
}

impl EventFeedItem {
    /// Construct a feed item at the given tick with the given severity.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        tick: u64,
        severity: EventSeverity,
        headline: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            tick,
            severity,
            headline: headline.into(),
            body: None,
            region: None,
        }
    }

    /// Builder helper: attach a body / description.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Builder helper: attach a focus region.
    #[must_use]
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
}

/// The event feed state. Items are stored in a `BTreeMap` keyed by id so
/// duplicates collapse. Iteration is in id-ascending order; the renderer is
/// expected to sort by `tick` descending when it draws the scrollable feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFeed {
    /// Map of `id -> item`.
    pub items: BTreeMap<String, EventFeedItem>,
    /// Schema version of this panel state shape.
    pub schema_version: String,
}

impl EventFeed {
    /// Schema version constant.
    pub const SCHEMA_VERSION: &'static str = "0.1.0-hub-events";

    /// Construct an empty feed bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            schema_version: Self::SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert (or replace, deduplicating by id) an event. Returns
    /// `&mut self` for chaining.
    pub fn insert(&mut self, item: EventFeedItem) -> &mut Self {
        self.items.insert(item.id.clone(), item);
        self
    }

    /// Return the highest-severity item currently in the feed, or `None`
    /// when the feed is empty. Used by the renderer to flash a border on
    /// the feed widget when a new critical event lands.
    #[must_use]
    pub fn highest_severity(&self) -> Option<EventSeverity> {
        self.items.values().map(|i| i.severity).max()
    }

    /// Return the items in `tick` descending order, so the renderer can
    /// draw newest-first without re-sorting.
    pub fn items_newest_first(&self) -> Vec<&EventFeedItem> {
        let mut out: Vec<&EventFeedItem> = self.items.values().collect();
        out.sort_unstable_by(|a, b| b.tick.cmp(&a.tick).then_with(|| a.id.cmp(&b.id)));
        out
    }
}

impl Default for EventFeed {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-HUD-005 — Menu system
// ---------------------------------------------------------------------------

/// Which top-level menu the player is currently looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuKind {
    /// Pre-game (New / Load / Settings / Quit).
    Main,
    /// Pause overlay (Resume / Save / Settings / Quit).
    Pause,
    /// Settings sub-panel (graphics, DLSS, key bindings).
    Settings,
}

/// Errors returned by [`MenuStack`] mutators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuStackError {
    /// `pop` was called on an empty stack.
    EmptyStack,
}

/// A small LIFO menu stack. The spec calls for "main menu, pause menu,
/// settings panel" as the top-level surfaces; nested sub-menus (e.g.
/// "Settings → Key Bindings") are stacked on top so the back button pops
/// the most recent one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuStack {
    /// Stack of currently-open menus, bottom = first-opened.
    pub stack: Vec<MenuKind>,
    /// Schema version of this panel state shape.
    pub schema_version: String,
}

impl MenuStack {
    /// Schema version constant.
    pub const SCHEMA_VERSION: &'static str = "0.1.0-hub-menu";

    /// Construct an empty stack bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            schema_version: Self::SCHEMA_VERSION.to_string(),
        }
    }

    /// Push a new menu on top. Returns `&mut self` for chaining.
    pub fn push(&mut self, kind: MenuKind) -> &mut Self {
        self.stack.push(kind);
        self
    }

    /// Pop the topmost menu. Returns the popped kind or `EmptyStack` when
    /// the stack was empty.
    pub fn pop(&mut self) -> Result<MenuKind, MenuStackError> {
        self.stack.pop().ok_or(MenuStackError::EmptyStack)
    }

    /// Currently-focused menu (top of the stack), or `None` if the player
    /// is in-game with no menu open.
    #[must_use]
    pub fn top(&self) -> Option<MenuKind> {
        self.stack.last().copied()
    }

    /// `true` when at least one menu is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Number of menus on the stack.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Whether the menu stack is empty (no menus open).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl Default for MenuStack {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- FR-CIV-HUD-001 ------------------------------------------------

    /// FR-CIV-HUD-001 — `ToolPalette::set_selected` succeeds for an
    /// enabled tool and fails for a disabled or unknown one.
    #[test]
    fn fr_hub_001_tool_palette_set_selected_respects_availability() {
        let mut p = ToolPalette::new();
        p.insert(ToolEntry::new(
            "build_road",
            "Build Road",
            ToolCategory::Build,
        ));
        p.insert(
            ToolEntry::new("build_castle", "Build Castle", ToolCategory::Build)
                .with_tier(3)
                .disabled(),
        );
        assert!(p.set_selected("build_road").is_ok());
        assert_eq!(p.selected.as_deref(), Some("build_road"));
        assert!(p.set_selected("build_castle").is_err());
        assert!(p.set_selected("does_not_exist").is_err());
    }

    /// FR-CIV-HUD-001 — `refresh_for_tier` flips a tier-3 entry from
    /// `Disabled` to `Enabled` when the polity reaches tier 3, and clears
    /// the selection if the previously-selected tool just got locked.
    #[test]
    fn fr_hub_001_refresh_for_tier_enables_and_drops_selection() {
        let mut p = ToolPalette::new();
        p.insert(ToolEntry::new("build_road", "Road", ToolCategory::Build).with_tier(1));
        p.insert(ToolEntry::new("build_castle", "Castle", ToolCategory::Build).with_tier(3));
        // At tier 0 only tier-0 entries would be enabled; both are > tier 0
        // so the initial refresh_for_tier(0) disables both. Re-selecting a
        // disabled tool must fail.
        p.refresh_for_tier(0);
        assert!(p.set_selected("build_road").is_err());
        assert!(p.set_selected("build_castle").is_err());
        assert_eq!(p.selected, None);
        // Tier 3 unlocks both; selection can now succeed.
        p.refresh_for_tier(3);
        p.set_selected("build_road").unwrap();
        p.set_selected("build_castle").unwrap();
        assert_eq!(p.selected.as_deref(), Some("build_castle"));
        // Tier regression back to 0: the selected entry becomes Disabled
        // and `refresh_for_tier` drops the dangling selection.
        p.refresh_for_tier(0);
        assert_eq!(p.selected, None, "selection should drop when locked");
    }

    #[test]
    fn tool_entry_with_shortcut_sets_key() {
        let e = ToolEntry::new("build_road", "Road", ToolCategory::Build).with_shortcut('r');
        assert_eq!(e.shortcut, Some('r'));
    }

    // -- FR-CIV-HUD-002 ------------------------------------------------

    /// FR-CIV-HUD-002 — a node stays `Locked` until every prerequisite is
    /// `Researched`; after `complete`, `refresh_statuses` flips downstream
    /// nodes from `Locked` to `Available`.
    #[test]
    fn fr_hub_002_tech_tree_unlock_propagates_through_prereqs() {
        let mut t = TechTree::new();
        t.insert(TechNode::new("masonry", "Masonry", 100));
        t.insert(TechNode::new("castles", "Castles", 200).requires("masonry"));
        t.insert(
            TechNode::new("royal_court", "Royal Court", 400)
                .requires("masonry")
                .requires("castles"),
        );
        // Initial state: every node is Locked.
        assert_eq!(t.nodes["masonry"].status, TechStatus::Locked);
        assert_eq!(t.nodes["castles"].status, TechStatus::Locked);
        // First, complete masonry. The downstream nodes should not unlock
        // yet because castles still needs masonry AND the player hasn't
        // queued masonry's downstream.
        t.complete("masonry").unwrap();
        t.refresh_statuses();
        assert_eq!(t.nodes["masonry"].status, TechStatus::Researched);
        assert_eq!(t.nodes["castles"].status, TechStatus::Available);
        assert_eq!(t.nodes["royal_court"].status, TechStatus::Locked);
        // Now finish castles; royal_court becomes Available.
        t.complete("castles").unwrap();
        assert_eq!(t.nodes["royal_court"].status, TechStatus::Available);
    }

    /// FR-CIV-HUD-002 — `queue` refuses a node whose prerequisites are not
    /// met and refuses a node that is already `Researched`.
    #[test]
    fn fr_hub_002_queue_refuses_locked_or_done_nodes() {
        let mut t = TechTree::new();
        t.insert(TechNode::new("a", "A", 10));
        t.insert(TechNode::new("b", "B", 20).requires("a"));
        // 'b' is still Locked.
        assert!(t.queue("b").is_err());
        t.complete("a").unwrap();
        // 'b' is Available; queue succeeds.
        t.queue("b").unwrap();
        assert_eq!(t.nodes["b"].status, TechStatus::InProgress);
        // Cannot queue again — InProgress is not Available.
        assert!(t.queue("b").is_err());
        t.complete("b").unwrap();
        // Cannot queue a Researched node.
        assert!(t.queue("b").is_err());
    }

    #[test]
    fn tech_node_prerequisites_met_requires_all_researched() {
        let mut t = TechTree::new();
        t.insert(TechNode::new("a", "A", 100));
        t.insert(TechNode::new("b", "B", 200).requires("a"));
        assert!(!t.nodes["b"].prerequisites_met(&t));
        assert!(t.nodes["a"].prerequisites_met(&t));
        t.complete("a").unwrap();
        assert!(t.nodes["b"].prerequisites_met(&t));
    }

    #[test]
    fn tech_node_can_queue_needs_available_and_prereqs() {
        let mut t = TechTree::new();
        t.insert(TechNode::new("a", "A", 100));
        t.insert(TechNode::new("b", "B", 200).requires("a"));
        t.refresh_statuses();
        assert!(t.nodes["a"].can_queue(&t));
        assert!(!t.nodes["b"].can_queue(&t));
        t.complete("a").unwrap();
        assert!(t.nodes["b"].can_queue(&t));
    }

    #[test]
    fn tech_tree_available_nodes_lists_only_available() {
        let mut t = TechTree::new();
        t.insert(TechNode::new("a", "A", 100));
        t.insert(TechNode::new("b", "B", 200).requires("a"));
        t.refresh_statuses();
        let avail: Vec<&str> = t.available_nodes().map(|n| n.id.as_str()).collect();
        assert!(avail.contains(&"a"));
        assert!(!avail.contains(&"b"));
        assert!(t
            .available_nodes()
            .all(|n| n.status == TechStatus::Available));
    }

    // -- FR-CIV-HUD-003 ------------------------------------------------

    /// FR-CIV-HUD-003 — `TreatySlot::with_influence` clamps out-of-range
    /// values so the panel's bar never overflows.
    #[test]
    fn fr_hub_003_treaty_slot_clamps_influence_to_unit_interval() {
        let slot = TreatySlot::new("p1", "Polity 1").with_influence(2.5);
        assert_eq!(slot.influence_capital, 1.0);
        let slot = TreatySlot::new("p1", "Polity 1").with_influence(-0.5);
        assert_eq!(slot.influence_capital, 0.0);
    }

    /// FR-CIV-HUD-003 — `DiplomacyPanel` round-trips through `ron` so
    /// saves / replays can persist the panel state alongside the polity
    /// ledger.
    #[test]
    fn fr_hub_003_diplomacy_panel_ron_round_trip() {
        let mut p = DiplomacyPanel::new();
        p.insert(
            TreatySlot::new("a", "A")
                .with_fsm(DiplomacyFsm::Allied)
                .with_influence(0.75),
        );
        p.insert(
            TreatySlot::new("b", "B")
                .with_fsm(DiplomacyFsm::AtWar)
                .with_influence(0.1),
        );
        let encoded = ron::to_string(&p).expect("serialize");
        let decoded: DiplomacyPanel = ron::from_str(&encoded).expect("deserialize");
        assert_eq!(p, decoded);
    }

    #[test]
    fn diplomacy_panel_get_finds_inserted_slot() {
        let mut panel = DiplomacyPanel::new();
        panel.insert(TreatySlot::new("p1", "Polity One"));
        assert!(panel.get("p1").is_some());
        assert!(panel.get("absent").is_none());
    }

    // -- FR-CIV-HUD-004 ------------------------------------------------

    /// FR-CIV-HUD-004 — `EventFeed::insert` deduplicates by id so a
    /// producer cannot double-log the same event.
    #[test]
    fn fr_hub_004_event_feed_dedupes_by_id() {
        let mut f = EventFeed::new();
        f.insert(EventFeedItem::new(
            "evt-1",
            10,
            EventSeverity::Info,
            "Hello",
        ));
        f.insert(EventFeedItem::new(
            "evt-1",
            99,
            EventSeverity::Critical,
            "Replaced",
        ));
        assert_eq!(f.items.len(), 1);
        // The second insert wins (replace, not append).
        assert_eq!(f.items["evt-1"].severity, EventSeverity::Critical);
    }

    /// FR-CIV-HUD-004 — `items_newest_first` orders by tick descending and
    /// breaks ties by id ascending so the order is fully deterministic.
    #[test]
    fn fr_hub_004_event_feed_newest_first_is_deterministic() {
        let mut f = EventFeed::new();
        f.insert(EventFeedItem::new("a", 10, EventSeverity::Info, "a"));
        f.insert(EventFeedItem::new("b", 30, EventSeverity::Info, "b"));
        f.insert(EventFeedItem::new("c", 20, EventSeverity::Info, "c"));
        f.insert(EventFeedItem::new("d", 30, EventSeverity::Info, "d"));
        let order: Vec<&str> = f
            .items_newest_first()
            .into_iter()
            .map(|i| i.id.as_str())
            .collect();
        assert_eq!(order, vec!["b", "d", "c", "a"]);
    }

    /// FR-CIV-HUD-004 — `highest_severity` returns the maximum across
    /// every item in the feed.
    #[test]
    fn fr_hub_004_event_feed_highest_severity_picks_max() {
        let mut f = EventFeed::new();
        f.insert(EventFeedItem::new("a", 1, EventSeverity::Info, "a"));
        f.insert(EventFeedItem::new("b", 2, EventSeverity::Warning, "b"));
        f.insert(EventFeedItem::new("c", 3, EventSeverity::Critical, "c"));
        assert_eq!(f.highest_severity(), Some(EventSeverity::Critical));
    }

    #[test]
    fn event_feed_item_builders_set_body_and_region() {
        let item = EventFeedItem::new("e1", 10, EventSeverity::Info, "Headline")
            .with_body("details")
            .with_region("north");
        assert_eq!(item.body.as_deref(), Some("details"));
        assert_eq!(item.region.as_deref(), Some("north"));
    }

    // -- FR-CIV-HUD-005 ------------------------------------------------

    /// FR-CIV-HUD-005 — `MenuStack` is LIFO: `push`/`pop` semantics are
    /// stack-shaped, and `pop` on an empty stack returns `EmptyStack`
    /// rather than panicking.
    #[test]
    fn fr_hub_005_menu_stack_is_lifo_and_pops_safely() {
        let mut s = MenuStack::new();
        assert_eq!(s.pop(), Err(MenuStackError::EmptyStack));
        s.push(MenuKind::Main);
        s.push(MenuKind::Settings);
        assert_eq!(s.top(), Some(MenuKind::Settings));
        assert_eq!(s.pop().unwrap(), MenuKind::Settings);
        assert_eq!(s.top(), Some(MenuKind::Main));
        assert_eq!(s.pop().unwrap(), MenuKind::Main);
        assert_eq!(s.pop(), Err(MenuStackError::EmptyStack));
        assert!(!s.is_open());
    }

    /// FR-CIV-HUD-005 — `MenuStack` round-trips through `ron` so the
    /// pause menu survives a save/load.
    #[test]
    fn fr_hub_005_menu_stack_ron_round_trip() {
        let mut s = MenuStack::new();
        s.push(MenuKind::Pause);
        s.push(MenuKind::Settings);
        let encoded = ron::to_string(&s).expect("serialize");
        let decoded: MenuStack = ron::from_str(&encoded).expect("deserialize");
        assert_eq!(s, decoded);
    }
}
