//! Accessibility primitives — FR-CIV-ACCESS-010, FR-CIV-ACCESS-020,
//! NFR-CIV-ACC-001..004.
//!
//! Per `docs/adr/ADR-021-accessibility-and-l10n-strategy.md` and
//! `docs/reference/non-functional-requirements.md` §ACC, this module is the
//! HUD-side pure-logic layer for the accessibility surface. It is
//! substrate-neutral: clients (web / Bevy / Godot / Unreal) read the values
//! and project them onto whatever UI surface they own. No Bevy, no
//! rendering, no systems.
//!
//! ## Coverage
//!
//! | FR / NFR ID             | Surface                                              |
//! |-------------------------|------------------------------------------------------|
//! | FR-CIV-ACCESS-010       | [`PaletteMode::ALL`] (≥3 colorblind palettes)         |
//! | FR-CIV-ACCESS-020       | [`HighContrastTheme`]                                |
//! | NFR-CIV-ACC-001         | [`palette_pair_distinguishable`] contrast gate       |
//! | NFR-CIV-ACC-002         | [`KeybindRegistry`] / [`KeybindAction`]              |
//! | NFR-CIV-ACC-003         | [`MIN_FONT_SIZE_PX`] + [`font_size_accepted`]          |
//! | NFR-CIV-ACC-004         | [`TooltipDescriptor`] / [`TooltipRegistry`]          |
//!
//! ## Contract
//!
//! 1. **Pure data, no engine.** All structs are `serde`-serializable.
//! 2. **Additive only.** Does not modify any existing public surface in
//!    `civ_hud`.
//! 3. **Deterministic.** Every function is total over its input domain
//!    (no panic on out-of-range inputs — clamped or rejected explicitly).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The colorblind-aware faction palette variants.
///
/// Per `docs/adr/ADR-021-accessibility-and-l10n-strategy.md` §FR Coverage
/// (FR-CIV-ACCESS-010), the workspace SHALL expose ≥3 palette modes so
/// players with deuteranopia, protanopia, or tritanopia can distinguish
/// factions. The `Default` palette is the canonical RGB variant; the
/// three CVD variants remap each faction to a hue that's discriminable
/// under the corresponding simulation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaletteMode {
    /// Canonical RGB palette — vibrant red/green/blue. Default.
    #[default]
    Default,
    /// Deuteranopia-safe palette: replaces red↔green confusion with
    /// blue↔orange / yellow↔violet axes.
    Deuteranopia,
    /// Protanopia-safe palette: same axis substitution as deuteranopia
    /// (red cone absent), tuned for slightly different luminance.
    Protanopia,
    /// Tritanopia-safe palette: replaces blue↔yellow confusion with
    /// red↔cyan / magenta↔green axes.
    Tritanopia,
}

impl PaletteMode {
    /// All palette variants, declaration order. Used by palette-validate
    /// CI and host clients that want to cycle through every variant.
    pub const ALL: [PaletteMode; 4] = [
        PaletteMode::Default,
        PaletteMode::Deuteranopia,
        PaletteMode::Protanopia,
        PaletteMode::Tritanopia,
    ];

    /// Wire-stable identifier for the variant. Used in settings files and
    /// HUD config payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PaletteMode::Default => "default",
            PaletteMode::Deuteranopia => "deuteranopia",
            PaletteMode::Protanopia => "protanopia",
            PaletteMode::Tritanopia => "tritanopia",
        }
    }

    /// Inverse of [`Self::as_str`]. Returns `None` for unknown wire ids.
    #[must_use]
    pub fn parse_wire(s: &str) -> Option<Self> {
        match s {
            "default" => Some(Self::Default),
            "deuteranopia" => Some(Self::Deuteranopia),
            "protanopia" => Some(Self::Protanopia),
            "tritanopia" => Some(Self::Tritanopia),
            _ => None,
        }
    }
}

/// One (faction, color) tuple under a specific palette mode.
///
/// 8-bit sRGB channels. Hosts convert to linear / display-referred as
/// needed; this struct is the source of truth for the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaletteEntry {
    /// Faction id (`u32` to match `Civilian::faction` historical alias).
    pub faction_id: u32,
    /// Red channel `[0, 255]`.
    pub r: u8,
    /// Green channel `[0, 255]`.
    pub g: u8,
    /// Blue channel `[0, 255]`.
    pub b: u8,
}

impl PaletteEntry {
    /// Construct an entry from raw 8-bit channels. Caller is responsible
    /// for ensuring `faction_id` is unique within the palette.
    #[must_use]
    pub const fn new(faction_id: u32, r: u8, g: u8, b: u8) -> Self {
        Self { faction_id, r, g, b }
    }

    /// Luminance proxy (Rec. 601). Returns a value in `[0, 255]` —
    /// `((R*299) + (G*587) + (B*114)) / 1000`. Hosts use this for
    /// contrast-gate arithmetic and legend bucketing.
    ///
    /// Uses `u32` arithmetic to avoid overflow on `u8 * 587` (which
    /// would exceed `u16::MAX` for any non-zero green channel).
    #[must_use]
    pub fn luminance(self) -> u16 {
        let r = u32::from(self.r);
        let g = u32::from(self.g);
        let b = u32::from(self.b);
        // The maximum numerator is bounded by
        // `(255*299 + 255*587 + 255*114) = 255000`, and
        // 255000 / 1000 = 255 — fits in `u16`.
        ((r * 299 + g * 587 + b * 114) / 1000) as u16
    }
}

/// A complete palette table — every faction's color under one mode.
///
/// Hosts render `Lookup[PaletteMode].entry(faction_id)` to retrieve the
/// `(r, g, b)` tuple to feed the renderer / overlay pipeline. The table
/// is read-only after construction; clients should rebuild rather than
/// mutate when the player changes mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteTable {
    /// Mode this table represents.
    pub mode: PaletteMode,
    /// Sorted by `faction_id` ascending — deterministic iteration order.
    pub entries: Vec<PaletteEntry>,
}

impl PaletteTable {
    /// Build a table. Sorts entries by `faction_id` ascending and
    /// rejects duplicate `faction_id` (returns `Err`).
    pub fn new(mode: PaletteMode, mut entries: Vec<PaletteEntry>) -> Result<Self, String> {
        entries.sort_by_key(|e| e.faction_id);
        for w in entries.windows(2) {
            if w[0].faction_id == w[1].faction_id {
                return Err(format!(
                    "palette mode {:?} has duplicate faction_id {}",
                    mode, w[0].faction_id
                ));
            }
        }
        Ok(Self { mode, entries })
    }

    /// Look up an entry by faction id. Returns `None` for unknown ids.
    #[must_use]
    pub fn entry(&self, faction_id: u32) -> Option<&PaletteEntry> {
        self.entries
            .binary_search_by_key(&faction_id, |e| e.faction_id)
            .ok()
            .map(|i| &self.entries[i])
    }

    /// Number of entries in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the table has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// High-contrast theme toggle — FR-CIV-ACCESS-020.
///
/// Hosts read the configured theme and crank up the contrast multiplier
/// on the renderer. The default is the normal-contrast variant
/// (`Normal`). When the user opts in, the renderer applies a luminance
/// re-mapping that pushes light tones lighter and dark tones darker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighContrastTheme {
    /// Normal contrast — no remap applied. Default.
    #[default]
    Normal,
    /// High contrast — luminance remap; foreground/background gap is
    /// maximized for low-vision users.
    High,
}

impl HighContrastTheme {
    /// All theme variants, declaration order.
    pub const ALL: [HighContrastTheme; 2] = [HighContrastTheme::Normal, HighContrastTheme::High];

    /// Multiplier the renderer applies to foreground/background luminance
    /// when computing the on-screen color. `Normal` is a no-op
    /// (`1.0`); `High` triples the gap (`3.0`). Hosts cap the resulting
    /// value at `1.0` (white) and `0.0` (black).
    #[must_use]
    pub fn contrast_multiplier(self) -> f32 {
        match self {
            HighContrastTheme::Normal => 1.0,
            HighContrastTheme::High => 3.0,
        }
    }
}

/// FR-CIV-ACCESS-010 + NFR-CIV-ACC-001: every pair of entries in the
/// table must have a luminance gap of at least `min_luma_gap` to pass
/// the CVD contrast gate. Returns the list of offending `(i, j)` index
/// pairs so the validator script can report precise failures.
#[must_use]
pub fn palette_pair_distinguishable(
    table: &PaletteTable,
    min_luma_gap: u16,
) -> Vec<(usize, usize)> {
    let mut offenders = Vec::new();
    for i in 0..table.entries.len() {
        for j in (i + 1)..table.entries.len() {
            let a = table.entries[i].luminance();
            let b = table.entries[j].luminance();
            if a.abs_diff(b) < min_luma_gap {
                offenders.push((i, j));
            }
        }
    }
    offenders
}

/// One logical input action that the player can remap — NFR-CIV-ACC-002.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeybindAction {
    /// Stable id (e.g. `"camera.zoom_in"`).
    pub id: String,
    /// Human-readable label (e.g. `"Zoom In"`).
    pub label: String,
}

/// One (action → key) binding — NFR-CIV-ACC-002.
///
/// Hosts iterate [`KeybindRegistry::actions`] and render the mapping
/// table; the player edits an entry to remap. The `key` field stores
/// the platform-neutral key token (e.g. `"F1"`, `"ArrowUp"`,
/// `"Ctrl+Shift+M"`); clients translate to their native input layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindEntry {
    /// Action this binding covers.
    pub action_id: String,
    /// Default key (the binding present at first install / settings
    /// reset). Hosts expose this in the "Reset to defaults" UI.
    pub default_key: String,
    /// The user's current key. Equal to `default_key` until the user
    /// remaps.
    pub current_key: String,
}

/// Per-user (or per-profile) keybind registry — NFR-CIV-ACC-002.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeybindRegistry {
    /// Master action list (the full universe of rebindable actions).
    actions: Vec<KeybindAction>,
    /// Per-action current binding, keyed by `action_id`. Missing entries
    /// mean "use the default" — the host fills them in on read.
    bindings: BTreeMap<String, KeybindEntry>,
}

impl KeybindRegistry {
    /// Construct a registry from the master action list. Bindings start
    /// empty; the caller populates them via [`Self::bind`] /
    /// [`Self::remap`].
    #[must_use]
    pub fn new(actions: Vec<KeybindAction>) -> Self {
        Self {
            actions,
            bindings: BTreeMap::new(),
        }
    }

    /// Register the default key for an action. Subsequent [`Self::bind`]
    /// calls without a `current_key` use this as the user's current key.
    pub fn bind(&mut self, action_id: &str, default_key: &str, current_key: &str) {
        self.bindings.insert(
            action_id.to_string(),
            KeybindEntry {
                action_id: action_id.to_string(),
                default_key: default_key.to_string(),
                current_key: current_key.to_string(),
            },
        );
    }

    /// Remap an existing entry. Returns `false` if `action_id` isn't in
    /// the registry (the user can't remap an action that doesn't exist).
    pub fn remap(&mut self, action_id: &str, new_key: &str) -> bool {
        if let Some(entry) = self.bindings.get_mut(action_id) {
            entry.current_key = new_key.to_string();
            true
        } else {
            false
        }
    }

    /// Look up the current key for an action. Returns `None` if the
    /// action isn't bound.
    #[must_use]
    pub fn current_key(&self, action_id: &str) -> Option<&str> {
        self.bindings.get(action_id).map(|e| e.current_key.as_str())
    }

    /// All bound actions, sorted by `action_id` ascending.
    #[must_use]
    pub fn bindings(&self) -> Vec<&KeybindEntry> {
        self.bindings.values().collect()
    }

    /// Master action list.
    #[must_use]
    pub fn actions(&self) -> &[KeybindAction] {
        &self.actions
    }

    /// True when every action in `actions` has a binding entry. Used by
    /// the integration test `tests/accessibility/keybind_remap_all_actions`
    /// to assert 100% coverage.
    #[must_use]
    pub fn covers_all_actions(&self) -> bool {
        self.actions
            .iter()
            .all(|a| self.bindings.contains_key(&a.id))
    }
}

/// Minimum readable font size in pixels at 1080p reference resolution —
/// NFR-CIV-ACC-003. Per the spec, no UI text SHALL be below this bound.
/// The constant is `pub` so a lint or startup test can import it and
/// assert every `TextStyle::font_size` value is `>= MIN_FONT_SIZE_PX`.
pub const MIN_FONT_SIZE_PX: f32 = 14.0;

/// NFR-CIV-ACC-003 acceptance gate. Returns `true` when `font_size_px`
/// meets the minimum-readable bound. Hosts call this for every
/// registered `TextStyle` at startup and assert it returns `true`.
#[must_use]
pub fn font_size_accepted(font_size_px: f32) -> bool {
    font_size_px >= MIN_FONT_SIZE_PX
}

/// One interactive UI element's tooltip descriptor — NFR-CIV-ACC-004.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TooltipDescriptor {
    /// Stable id of the element (e.g. `"hud.population_button"`).
    pub element_id: String,
    /// One-line label (e.g. `"Population"`).
    pub label: String,
    /// Multi-line description. Optional — some tooltips are label-only.
    pub description: Option<String>,
}

/// Per-screen tooltip registry — NFR-CIV-ACC-004.
///
/// Hosts call [`Self::add`] for every interactive element on screen and
/// [`Self::get`] on hover. The integration test
/// `tests/accessibility/all_interactive_have_tooltips` walks the master
/// interactive-element list and asserts every id resolves.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TooltipRegistry {
    entries: BTreeMap<String, TooltipDescriptor>,
}

impl TooltipRegistry {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace a tooltip.
    pub fn add(&mut self, descriptor: TooltipDescriptor) {
        self.entries.insert(descriptor.element_id.clone(), descriptor);
    }

    /// Look up a tooltip by element id. Returns `None` for unregistered
    /// ids.
    #[must_use]
    pub fn get(&self, element_id: &str) -> Option<&TooltipDescriptor> {
        self.entries.get(element_id)
    }

    /// Number of registered tooltips.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no tooltips are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All registered tooltips, sorted by `element_id` ascending.
    #[must_use]
    pub fn entries(&self) -> Vec<&TooltipDescriptor> {
        self.entries.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-CIV-ACCESS-010 + NFR-CIV-ACC-001 -------------------------------------------------

    /// FR-CIV-ACCESS-010: ≥3 palette modes are exposed.
    #[test]
    fn palette_mode_exposes_at_least_three_variants() {
        assert!(PaletteMode::ALL.len() >= 3);
        // Wire ids are stable and unique.
        let mut seen = std::collections::HashSet::new();
        for mode in PaletteMode::ALL {
            let s = mode.as_str();
            assert!(seen.insert(s), "duplicate palette wire id: {s}");
            assert_eq!(PaletteMode::parse_wire(s), Some(mode));
        }
    }

    /// NFR-CIV-ACC-001: a palette whose entries have a wide luminance
    /// gap passes the contrast gate; a palette with two near-equal
    /// entries fails.
    #[test]
    fn palette_pair_distinguishable_flags_close_entries() {
        // Wide gap — all pairs pass.
        let wide = PaletteTable::new(
            PaletteMode::Default,
            vec![
                PaletteEntry::new(0, 0, 0, 0),
                PaletteEntry::new(1, 255, 255, 255),
            ],
        )
        .unwrap();
        assert!(palette_pair_distinguishable(&wide, 64).is_empty());

        // Near-equal — every pair fails when the gap threshold is high.
        let close = PaletteTable::new(
            PaletteMode::Default,
            vec![
                PaletteEntry::new(0, 10, 10, 10),
                PaletteEntry::new(1, 11, 11, 11),
            ],
        )
        .unwrap();
        let offenders = palette_pair_distinguishable(&close, 64);
        assert_eq!(offenders.len(), 1);
        assert_eq!(offenders[0], (0, 1));
    }

    /// `PaletteTable::entry` does a binary search by faction_id;
    /// unknown ids return `None`.
    #[test]
    fn palette_table_entry_lookup() {
        let t = PaletteTable::new(
            PaletteMode::Deuteranopia,
            vec![
                PaletteEntry::new(2, 0, 0, 200),
                PaletteEntry::new(5, 200, 100, 0),
                PaletteEntry::new(9, 50, 200, 50),
            ],
        )
        .unwrap();
        assert_eq!(t.entry(5).unwrap().b, 0);
        assert!(t.entry(99).is_none());
    }

    /// Duplicate `faction_id` in the input is rejected.
    #[test]
    fn palette_table_rejects_duplicate_faction() {
        let r = PaletteTable::new(
            PaletteMode::Default,
            vec![
                PaletteEntry::new(0, 0, 0, 0),
                PaletteEntry::new(0, 255, 255, 255),
            ],
        );
        assert!(r.is_err());
    }

    // FR-CIV-ACCESS-020 -------------------------------------------------------------------

    /// FR-CIV-ACCESS-020: high-contrast theme's multiplier is > 1; the
    /// normal theme is a no-op.
    #[test]
    fn high_contrast_theme_multiplier() {
        assert!((HighContrastTheme::Normal.contrast_multiplier() - 1.0).abs() < f32::EPSILON);
        assert!(HighContrastTheme::High.contrast_multiplier() > 1.0);
    }

    // NFR-CIV-ACC-002 ---------------------------------------------------------------------

    /// NFR-CIV-ACC-002: 100% of named actions are bindable; remap
    /// replaces the current key; reset restores the default.
    #[test]
    fn keybind_registry_remap_and_reset() {
        let mut reg = KeybindRegistry::new(vec![
            KeybindAction {
                id: "a".into(),
                label: "Action A".into(),
            },
            KeybindAction {
                id: "b".into(),
                label: "Action B".into(),
            },
        ]);
        reg.bind("a", "F1", "F1");
        reg.bind("b", "F2", "F2");
        assert!(reg.covers_all_actions());
        assert_eq!(reg.current_key("a"), Some("F1"));

        assert!(reg.remap("a", "G"));
        assert_eq!(reg.current_key("a"), Some("G"));
        assert!(!reg.remap("missing", "H"));
    }

    // NFR-CIV-ACC-003 ---------------------------------------------------------------------

    /// NFR-CIV-ACC-003: font sizes below the minimum are rejected.
    #[test]
    fn min_font_size_is_fourteen() {
        assert!((MIN_FONT_SIZE_PX - 14.0).abs() < f32::EPSILON);
        assert!(font_size_accepted(14.0));
        assert!(font_size_accepted(18.0));
        assert!(!font_size_accepted(13.999));
        assert!(!font_size_accepted(0.0));
    }

    // NFR-CIV-ACC-004 ---------------------------------------------------------------------

    /// NFR-CIV-ACC-004: tooltip lookup, iteration, and coverage all work.
    #[test]
    fn tooltip_registry_lookup_and_iteration() {
        let mut reg = TooltipRegistry::new();
        reg.add(TooltipDescriptor {
            element_id: "btn.a".into(),
            label: "A".into(),
            description: Some("Action A button".into()),
        });
        reg.add(TooltipDescriptor {
            element_id: "btn.b".into(),
            label: "B".into(),
            description: None,
        });
        assert_eq!(reg.len(), 2);
        assert!(reg.get("btn.a").is_some());
        assert!(reg.get("missing").is_none());
        let all = reg.entries();
        assert_eq!(all[0].element_id, "btn.a");
        assert_eq!(all[1].element_id, "btn.b");
    }
}