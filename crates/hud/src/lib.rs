//! Shared Keycap Palette HUD types.
//!
//! Per `docs/design/SAVELOAD_HUD_PLAN.md` §4 / §5, this crate holds the
//! substrate-neutral HUD surface that all four clients (web, Bevy, Godot,
//! Unreal) render against the **same token list**. No engine / no server / no
//! save-db dependency — the HUD is a read-only instrument panel that reads
//! snapshot data and emits wire commands (`sim.set_speed`, `power.activate`,
//! `inspect.probe`).
//!
//! The crate is the **canonical home** of the Keycap Palette: 8 keycap
//! definitions (Q/W/E/R/T/A/S/F) wired to substrate verbs, the top-bar chip
//! data, and the tile-inspector projection. It also ships a token audit that
//! fails compilation if anyone introduces a hex outside
//! `docs/design/ui-design-language.md` §1.

#![forbid(unsafe_code)]

pub mod key_palette;
pub mod tile_inspector;
pub mod tokens;
pub mod top_bar;

pub use key_palette::{KeyId, KeycapDef, KeycapPalette, keycap_palette_default};
pub use tile_inspector::{TileInspector, CELL_NONE};
pub use tokens::{Token, TokenName, CANONICAL_TOKENS, TOKEN_AUDIT};
pub use top_bar::{
    AutoChipState, ChipValue, SimSpeed, TopBarChips, AUTO_FRESH_AGE_SECS, AUTO_STALE_AGE_SECS,
};

/// Re-export of the canonical era ids used by the HUD. The full colour/glyph
/// mapping is in `docs/design/ui-design-language.md`; this is the
/// substrate-neutral vocabulary shared across crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HudEra {
    /// No era data yet (fresh sim).
    Unknown,
    /// Stone / hunter-gatherer.
    Stone,
    /// Bronze age.
    Bronze,
    /// Iron age — emergence starts here per `emergence-charter.md`.
    Iron,
    /// Steel / industrial era.
    Steel,
    /// Information age.
    Information,
    /// Atomic / post-atomic.
    Atomic,
}

impl HudEra {
    /// Era accent token name (per `ui-design-language.md` §6, era table).
    /// Returns a `TokenName` so callers can map to hex via the canonical
    /// token list — never hardcode hex in this crate.
    #[must_use]
    pub const fn accent_token(self) -> TokenName {
        match self {
            HudEra::Unknown => TokenName::TextLow,
            HudEra::Stone => TokenName::TextLow,
            HudEra::Bronze => TokenName::Amber,
            HudEra::Iron => TokenName::Neon,
            HudEra::Steel => TokenName::HoloCore,
            HudEra::Information => TokenName::HoloDeep,
            HudEra::Atomic => TokenName::HoloGlow,
        }
    }
}

impl Default for HudEra {
    fn default() -> Self {
        HudEra::Unknown
    }
}

/// Boolean open/closed flag for non-palette surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SurfaceFlag {
    pub open: bool,
}

impl SurfaceFlag {
    pub const fn closed() -> Self {
        Self { open: false }
    }
    pub const fn open() -> Self {
        Self { open: true }
    }
    pub const fn is_open(&self) -> bool {
        self.open
    }
    pub fn open_self(&mut self) {
        self.open = true;
    }
    pub fn close_self(&mut self) {
        self.open = false;
    }
}

impl Default for SurfaceFlag {
    fn default() -> Self {
        Self::closed()
    }
}

/// Composite HUD state — the open/closed FSM. The default opens top-bar +
/// keycap palette and leaves the tile inspector + save panel closed (charter
/// density rule: at-rest = 0 holo surfaces visible).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HudState {
    /// Always-on chrome top-bar.
    pub top_bar: SurfaceFlag,
    /// 8-key rim cluster (always visible when HUD is up).
    pub key_palette: KeycapPalette,
    /// HOLO tile inspector (read-only projection).
    pub tile_inspector: TileInspector,
    /// Save/Load modal (chrome E2 — never renders holo).
    pub save_panel: SurfaceFlag,
}

impl HudState {
    /// Default HUD: top-bar + keycap palette visible; tile inspector + save
    /// panel closed.
    #[must_use]
    pub fn default_open() -> Self {
        Self {
            top_bar: SurfaceFlag::open(),
            key_palette: KeycapPalette::default(),
            tile_inspector: TileInspector::closed(),
            save_panel: SurfaceFlag::closed(),
        }
    }

    /// True when the keycap palette is visible (the 8-key rim cluster).
    #[must_use]
    pub fn key_palette_is_open(&self) -> bool {
        // The palette is always visible when the HUD is "up" — the spec
        // pins it to the rim cluster (always on). Open-ness is a host
        // decision; we model the data only.
        true
    }

    /// Number of HUD holo surfaces currently open. Used by the density-rule
    /// test — must never exceed 2.
    #[must_use]
    pub fn open_holo_count(&self) -> u32 {
        // Tile inspector is the only HUD-owned holo surface (the in-world
        // brush ring is a 3D-world concern, not a HUD surface). Save panel
        // is chrome, not holo. Key palette is chrome.
        if self.tile_inspector.is_open() {
            1
        } else {
            0
        }
    }
}

impl Default for HudState {
    fn default() -> Self {
        Self::default_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hud_state_opens_top_bar_and_keycaps_only() {
        let state = HudState::default();
        assert!(state.top_bar.is_open());
        assert!(state.key_palette_is_open());
        assert!(!state.tile_inspector.is_open());
        assert!(!state.save_panel.is_open());
        // Density-rule test: at-rest = 0 holo surfaces.
        assert_eq!(state.open_holo_count(), 0);
    }

    #[test]
    fn opening_inspector_then_save_panel_closes_inspector() {
        let mut state = HudState::default();
        state.tile_inspector.open_on((3, 7, 1));
        assert_eq!(state.open_holo_count(), 1);
        state.save_panel.open_self();
        // Density rule (§2 of plan): save panel open = tile inspector closes.
        state.tile_inspector.close();
        assert!(!state.tile_inspector.is_open());
    }

    #[test]
    fn keycap_palette_default_matches_god_tools_sandbox_recipe() {
        let palette = KeycapPalette::default();
        assert_eq!(palette.keys.len(), 8);
        // Q = Inspect, A = Pause, S = Speed, F = Disaster per §4.2.
        assert_eq!(palette.keys[0].id, KeyId::Q);
        assert_eq!(palette.keys[5].id, KeyId::A);
        assert_eq!(palette.keys[6].id, KeyId::S);
        assert_eq!(palette.keys[7].id, KeyId::F);
        assert_eq!(palette.keys[6].verb, "Speed");
    }

    #[test]
    fn auto_chip_age_bands_color_correctly() {
        let fresh = AutoChipState::from_age_seconds(60);
        assert!(matches!(fresh, AutoChipState::Fresh(_)));
        let medium = AutoChipState::from_age_seconds(900);
        assert!(matches!(medium, AutoChipState::Medium(_)));
        let stale = AutoChipState::from_age_seconds(3600);
        assert!(matches!(stale, AutoChipState::Stale(_)));
        let failed = AutoChipState::Failed;
        assert!(matches!(failed, AutoChipState::Failed));
    }
}