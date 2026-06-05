#![cfg(all(feature = "bevy", feature = "egui"))]

//! Category-based god-game tool taxonomy for the Civis HUD.
//!
//! Modelled on Cities Skylines (a clean top-level toolbar where each button
//! opens a flyout of related sub-tools), WorldBox (chunky icon palette with
//! sub-tool drawers) and Empire at War (command groups). Every *category* is a
//! top-level toolbar slot; selecting it opens a flyout drawer of *sub-tools*.
//!
//! ## Coordination contract
//!
//! `spawn_tools.rs` (owned by the Infra Lead) exposes a small [`SpawnTool`] enum
//! (`Select`, `SpawnCivilian`, `SpawnBuilding`, `Terraform`, `Destroy`). The UI
//! must not edit that file, so this module defines a richer [`SubTool`] taxonomy
//! and *maps* each sub-tool onto an existing `SpawnTool` variant via
//! [`SubTool::spawn_tool`]. When the enum grows (e.g. `PaintMaterial`,
//! `Disaster(kind)`), only that one mapping table changes — the whole flyout UI
//! and category layout stay put. Sub-tools whose closest mapping is `None` are
//! shown lit-but-inert (a no-op click) until the backing variant lands, so the
//! palette reads as the full design today.
//!
//! [`ActiveSubTool`] is the UI-side source of truth for *which* sub-tool the
//! player picked (carrying full semantic identity); it is kept in lockstep with
//! `spawn_tools::ActiveTool` (the coarse backing variant) by the HUD.

use bevy::prelude::Resource;

use crate::spawn_tools::SpawnTool;
use crate::ui_theme;
use bevy_egui::egui;

/// A leaf tool inside a category flyout. Unit variants keep the taxonomy cheap
/// to copy/compare and let the UI light the exact picked sub-tool.
///
/// Variants are deliberately undocumented individually: each is a self-evident
/// god-game verb (`House`, `Meteor`, `Bless`, …) and its player-facing meaning
/// is the canonical `label()`/`icon()`. Per-variant doc lines would only restate
/// the name, so `missing_docs` is suppressed for the variant list here.
#[allow(missing_docs)] // self-documenting verb variants; see label()/icon()
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubTool {
    // Select / Inspect
    Select,
    Inspect,
    // Life
    SpawnOrganism,
    SpawnHerd,
    Bless,
    Curse,
    Disease,
    Kill,
    // Structure
    House,
    Farm,
    Workshop,
    Market,
    Wall,
    Tower,
    Monument,
    // Infrastructure (placement coordinated with Infra Lead's spawn_tools)
    Road,
    Trail,
    Highway,
    Bridge,
    Canal,
    // Terraform
    Raise,
    Lower,
    Flatten,
    PaintBiome,
    // Material paint
    Water,
    Sand,
    Dirt,
    Stone,
    Lava,
    Gas,
    // Disaster
    Meteor,
    Flood,
    Quake,
    Storm,
    Wildfire,
    Plague,
    // Diplomacy
    Alliance,
    War,
    Trade,
    // Policy
    Tax,
    Edict,
    Religion,
}

impl SubTool {
    /// Map this sub-tool onto the coarse backing [`SpawnTool`] variant.
    ///
    /// `None` = no backing variant exists yet; the UI renders the button lit but
    /// treats the click as a no-op. Extending `SpawnTool` later only touches the
    /// arms here, never the flyout/layout code.
    pub fn spawn_tool(self) -> Option<SpawnTool> {
        use SubTool::*;
        match self {
            Select => Some(SpawnTool::Select),
            Inspect => Some(SpawnTool::Select),
            SpawnOrganism | SpawnHerd => Some(SpawnTool::SpawnCivilian),
            House | Farm | Workshop | Market | Wall | Tower | Monument => {
                Some(SpawnTool::SpawnBuilding)
            }
            Raise | Lower | Flatten | PaintBiome => Some(SpawnTool::Terraform),
            // Kill despawns an actor; the env disasters are handled by
            // `disaster_tools` (they mutate the voxel world, not actors), so they
            // intentionally have NO actor-side SpawnTool — `is_active_capable`
            // still reports them selectable via `is_disaster`.
            Kill => Some(SpawnTool::Destroy),
            // Material-paint sub-tools arm the material brush; the painted
            // material is set from `paint_material_name` by a sync system.
            Water | Sand | Dirt | Stone | Lava | Gas => Some(SpawnTool::PaintMaterial),
            // No backing variant yet (await Infra Lead enum growth).
            _ => None,
        }
    }

    /// True for the environmental disaster verbs handled by `disaster_tools`
    /// (Meteor/Flood/Quake/Storm/Wildfire/Plague). These mutate the voxel world
    /// directly rather than through a `SpawnTool`.
    #[must_use]
    pub fn is_disaster(self) -> bool {
        use SubTool::*;
        matches!(self, Meteor | Flood | Quake | Storm | Wildfire | Plague)
    }

    /// For the Material-paint sub-tools, the `civ-voxel` material name the brush
    /// should paint. `None` for non-material sub-tools. Drives the material brush
    /// so selecting "Water"/"Lava"/… actually arms paint with that material
    /// (previously these were inert no-ops).
    #[must_use]
    pub fn paint_material_name(self) -> Option<&'static str> {
        use SubTool::*;
        Some(match self {
            Water => "Water",
            Sand => "Sand",
            Dirt => "Dirt",
            Stone => "Stone",
            Lava => "Lava",
            Gas => "Steam",
            _ => return None,
        })
    }

    /// Glyph shown on the sub-tool button (unicode fallback; SVG icons in
    /// `assets/ui/` can't be loaded by Bevy yet).
    pub fn icon(self) -> &'static str {
        use SubTool::*;
        match self {
            Select => "\u{1f446}",
            Inspect => "\u{1f50d}",
            SpawnOrganism => "\u{1f9cd}",
            SpawnHerd => "\u{1f404}",
            Bless => "\u{2728}",
            Curse => "\u{1f480}",
            Disease => "\u{1f9a0}",
            Kill => "\u{2620}",
            House => "\u{1f3e0}",
            Farm => "\u{1f33e}",
            Workshop => "\u{1f528}",
            Market => "\u{1f3ea}",
            Wall => "\u{1f9f1}",
            Tower => "\u{1f5fc}",
            Monument => "\u{1f5ff}",
            Road => "\u{1f6e3}",
            Trail => "\u{1f43e}",
            Highway => "\u{1f6e3}",
            Bridge => "\u{1f309}",
            Canal => "\u{1f30a}",
            Raise => "\u{2b06}",
            Lower => "\u{2b07}",
            Flatten => "\u{1f7f0}",
            PaintBiome => "\u{1f3a8}",
            Water => "\u{1f4a7}",
            Sand => "\u{1f3d6}",
            Dirt => "\u{1fa93}",
            Stone => "\u{1faa8}",
            Lava => "\u{1f30b}",
            Gas => "\u{1f4a8}",
            Meteor => "\u{2604}",
            Flood => "\u{1f30a}",
            Quake => "\u{1f30b}",
            Storm => "\u{1f329}",
            Wildfire => "\u{1f525}",
            Plague => "\u{2620}",
            Alliance => "\u{1f91d}",
            War => "\u{2694}",
            Trade => "\u{1f4b1}",
            Tax => "\u{1f4b0}",
            Edict => "\u{1f4dc}",
            Religion => "\u{26ea}",
        }
    }

    /// Human-readable label for the sub-tool button + tooltip.
    pub fn label(self) -> &'static str {
        use SubTool::*;
        match self {
            Select => "Select",
            Inspect => "Inspect",
            SpawnOrganism => "Organism",
            SpawnHerd => "Herd",
            Bless => "Bless",
            Curse => "Curse",
            Disease => "Disease",
            Kill => "Kill",
            House => "House",
            Farm => "Farm",
            Workshop => "Workshop",
            Market => "Market",
            Wall => "Wall",
            Tower => "Tower",
            Monument => "Monument",
            Road => "Road",
            Trail => "Trail",
            Highway => "Highway",
            Bridge => "Bridge",
            Canal => "Canal",
            Raise => "Raise",
            Lower => "Lower",
            Flatten => "Flatten",
            PaintBiome => "Biome",
            Water => "Water",
            Sand => "Sand",
            Dirt => "Dirt",
            Stone => "Stone",
            Lava => "Lava",
            Gas => "Gas",
            Meteor => "Meteor",
            Flood => "Flood",
            Quake => "Quake",
            Storm => "Storm",
            Wildfire => "Wildfire",
            Plague => "Plague",
            Alliance => "Alliance",
            War => "War",
            Trade => "Trade",
            Tax => "Tax",
            Edict => "Edict",
            Religion => "Religion",
        }
    }

    /// Whether the sub-tool does something when clicked — either it has a
    /// backing `SpawnTool` or it is an environmental disaster handled by
    /// `disaster_tools` (which has no `SpawnTool` but is still fully active).
    pub fn is_active_capable(self) -> bool {
        self.spawn_tool().is_some() || self.is_disaster()
    }
}

/// A top-level toolbar category with its flyout of sub-tools.
pub struct Category {
    /// Glyph shown on the toolbar slot.
    pub icon: &'static str,
    /// Category name.
    pub label: &'static str,
    /// Hotkey letter shown in the tooltip + lit on press.
    pub hotkey: &'static str,
    /// Accent colour used to theme the category + its flyout.
    pub accent: egui::Color32,
    /// Sub-tools shown in the flyout drawer.
    pub subtools: &'static [SubTool],
}

impl Category {
    /// Filename stem (under `assets/ui/tool-icons/`) of the rasterized PNG icon
    /// for this category, or `None` when no dedicated PNG exists (the unicode
    /// glyph in [`Self::icon`] is then used as the fallback).
    #[must_use]
    pub fn icon_key(&self) -> Option<&'static str> {
        Some(match self.label {
            "Select" => "select",
            "Life" => "spawn-life",
            "Structure" => "spawn-structure",
            "Terraform" => "terraform",
            "Material" => "spawn-material",
            "Infra" => "infra",
            "Disaster" => "disaster",
            "Diplomacy" => "diplomacy",
            "Policy" => "policy",
            _ => return None,
        })
    }
}

use SubTool::*;

/// The full category taxonomy. Order = left-to-right toolbar order.
pub const CATEGORIES: &[Category] = &[
    Category {
        icon: "\u{1f446}",
        label: "Select",
        hotkey: "Q",
        accent: ui_theme::ACCENT,
        subtools: &[Select, Inspect],
    },
    Category {
        icon: "\u{1f9cd}",
        label: "Life",
        hotkey: "E",
        accent: ui_theme::GREEN,
        subtools: &[SpawnOrganism, SpawnHerd, Bless, Curse, Disease, Kill],
    },
    Category {
        icon: "\u{1f3db}",
        label: "Structure",
        hotkey: "R",
        accent: ui_theme::GOLD,
        subtools: &[House, Farm, Workshop, Market, Wall, Tower, Monument],
    },
    Category {
        icon: "\u{1f6e3}",
        label: "Infra",
        hotkey: "C",
        accent: ui_theme::ACCENT,
        subtools: &[Road, Trail, Highway, Bridge, Canal],
    },
    Category {
        icon: "\u{26f0}",
        label: "Terraform",
        hotkey: "T",
        accent: ui_theme::GOLD,
        subtools: &[Raise, Lower, Flatten, PaintBiome],
    },
    Category {
        icon: "\u{1faa8}",
        label: "Material",
        hotkey: "A",
        accent: ui_theme::ACCENT,
        subtools: &[Water, Sand, Dirt, Stone, Lava, Gas],
    },
    Category {
        icon: "\u{1f4a5}",
        label: "Disaster",
        hotkey: "X",
        accent: ui_theme::RED,
        subtools: &[Meteor, Flood, Quake, Storm, Wildfire, Plague],
    },
    Category {
        icon: "\u{1f91d}",
        label: "Diplomacy",
        hotkey: "D",
        accent: ui_theme::VIOLET,
        subtools: &[Alliance, War, Trade],
    },
    Category {
        icon: "\u{1f4dc}",
        label: "Policy",
        hotkey: "F",
        accent: ui_theme::VIOLET,
        subtools: &[Tax, Edict, Religion],
    },
];

/// Total number of leaf sub-tools across all categories (for tests/telemetry).
pub fn subtool_count() -> usize {
    CATEGORIES.iter().map(|c| c.subtools.len()).sum()
}

/// HUD-side source of truth for the picked sub-tool + the open flyout category.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveSubTool {
    /// The semantic sub-tool the player last selected.
    pub current: SubTool,
    /// Index into [`CATEGORIES`] whose flyout is currently open, if any.
    pub open_category: Option<usize>,
}

impl Default for ActiveSubTool {
    fn default() -> Self {
        Self {
            current: SubTool::Select,
            open_category: None,
        }
    }
}

impl ActiveSubTool {
    /// Index of the category that owns the currently-selected sub-tool.
    pub fn active_category(&self) -> Option<usize> {
        CATEGORIES
            .iter()
            .position(|c| c.subtools.contains(&self.current))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_is_rich() {
        // The overhaul brief asks for "MORE TOOLS heavily".
        assert!(CATEGORIES.len() >= 9, "expected >=9 categories");
        assert!(
            subtool_count() >= 40,
            "expected >=40 sub-tools, got {}",
            subtool_count()
        );
    }

    #[test]
    fn every_subtool_has_glyph_and_label() {
        for cat in CATEGORIES {
            assert!(!cat.label.is_empty() && !cat.icon.is_empty() && !cat.hotkey.is_empty());
            for &st in cat.subtools {
                assert!(!st.icon().is_empty(), "{:?} missing icon", st);
                assert!(!st.label().is_empty(), "{:?} missing label", st);
            }
        }
    }

    #[test]
    fn core_subtools_map_to_spawn_tools() {
        assert_eq!(SubTool::Select.spawn_tool(), Some(SpawnTool::Select));
        assert_eq!(
            SubTool::SpawnOrganism.spawn_tool(),
            Some(SpawnTool::SpawnCivilian)
        );
        assert_eq!(SubTool::House.spawn_tool(), Some(SpawnTool::SpawnBuilding));
        assert_eq!(SubTool::Raise.spawn_tool(), Some(SpawnTool::Terraform));
        assert_eq!(SubTool::Kill.spawn_tool(), Some(SpawnTool::Destroy));
    }

    #[test]
    fn disasters_are_active_but_have_no_actor_spawn_tool() {
        // Env disasters are handled by `disaster_tools` (voxel-world mutation),
        // so they have NO SpawnTool but ARE selectable/active.
        for d in [
            SubTool::Meteor,
            SubTool::Flood,
            SubTool::Quake,
            SubTool::Storm,
            SubTool::Wildfire,
            SubTool::Plague,
        ] {
            assert!(d.is_disaster(), "{d:?} should be a disaster");
            assert_eq!(
                d.spawn_tool(),
                None,
                "{d:?} must not map to an actor SpawnTool"
            );
            assert!(
                d.is_active_capable(),
                "{d:?} must still be active/selectable"
            );
        }
        assert!(!SubTool::House.is_disaster());
    }

    #[test]
    fn inert_subtools_have_no_backing_variant_yet() {
        // Diplomacy/Policy still await infra enum growth; material brushes
        // now map to `SpawnTool::PaintMaterial`.
        assert_eq!(SubTool::Alliance.spawn_tool(), None);
        assert_eq!(SubTool::Tax.spawn_tool(), None);
        assert!(!SubTool::Tax.is_active_capable());
    }

    #[test]
    fn active_category_resolves_from_current() {
        let st = ActiveSubTool {
            current: SubTool::Farm,
            open_category: None,
        };
        let idx = st.active_category().unwrap();
        assert_eq!(CATEGORIES[idx].label, "Structure");
    }

    #[test]
    fn default_subtool_is_select() {
        assert_eq!(ActiveSubTool::default().current, SubTool::Select);
        assert!(ActiveSubTool::default().open_category.is_none());
    }
}
