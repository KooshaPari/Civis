//! Central HUD panel registry.
//!
//! **ADD NEW PANELS HERE (one place).**
//!
//! Both `bevy_window` and `standalone` bins pull in every HUD panel plugin
//! through a single `.add_plugins(HudPanelsPlugin)` call, so panel-registration
//! conflicts between bins are structurally impossible.
//!
//! Do **NOT** add panel plugins directly in the bin files.

use bevy::prelude::*;

// Panels available whenever the `bevy` feature is active.
use crate::{
    emergence_dashboard::EmergenceDashboardPlugin,
    faction_hud::FactionHudPlugin,
    game_laws::GameLawsPlugin,
    god_panel::GodPanelPlugin,
    live_pick::LivePickPlugin,
    minimap::MinimapPlugin,
    perf_hud::PerfHudPlugin,
    spawn_tools::SpawnToolsPlugin,
    tutorial::TutorialPlugin,
};

// Panels that additionally require the `egui` feature.
#[cfg(feature = "egui")]
use crate::{
    diplomacy_ui::DiplomacyUiPlugin,
    event_feed::EventFeedPlugin,
    game_ui::GameUiPlugin,
    god_actions::GodActionsPlugin,
    menus::MenusPlugin,
    save_load_ui::SaveLoadUiPlugin,
    tech_tree_ui::TechTreeUiPlugin,
};

/// Umbrella plugin that registers every HUD panel in the Civis 3D client.
///
/// Add new panel plugins here; both bins pick them up automatically.
pub struct HudPanelsPlugin;

impl Plugin for HudPanelsPlugin {
    fn build(&self, app: &mut App) {
        // ADD NEW PANELS HERE (one place) — do NOT add panel plugins in the bin files.

        // Core panels — active whenever the `bevy` feature is on.
        app.add_plugins((
            EmergenceDashboardPlugin,
            FactionHudPlugin,
            GameLawsPlugin,
            GodPanelPlugin,
            LivePickPlugin,
            MinimapPlugin,
            PerfHudPlugin,
            SpawnToolsPlugin,
            TutorialPlugin,
        ));

        // egui-gated panels.
        #[cfg(feature = "egui")]
        app.add_plugins((
            DiplomacyUiPlugin,
            EventFeedPlugin,
            GameUiPlugin,
            GodActionsPlugin,
            MenusPlugin,
            SaveLoadUiPlugin,
            TechTreeUiPlugin,
        ));
    }
}
