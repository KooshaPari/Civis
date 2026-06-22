#![cfg(all(feature = "bevy", feature = "egui"))]
//! Era HUD: tracks CivEra from server snapshots, shows era in HUD, toasts on advance (FR-CIV-GAME-003).

use bevy::prelude::*;
use crate::menus::in_game;
use crate::event_feed::{EventFeed, EventKind as FeedKind};

#[derive(Resource, Default)]
pub struct EraState {
    pub current_era: String,
}

pub struct EraHudPlugin;
impl Plugin for EraHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EraState>()
           .add_systems(Update, poll_era.run_if(in_game));
    }
}

fn poll_era(
    mut era_state: ResMut<EraState>,
    hud: Res<crate::HudState>,
    mut feed: ResMut<EventFeed>,
) {
    let new_era = era_from_snapshot(&hud);
    if !new_era.is_empty() && new_era != era_state.current_era {
        if !era_state.current_era.is_empty() {
            feed.push(FeedKind::System, format!("Era advanced: Entered the {} Era!", new_era));
        }
        era_state.current_era = new_era;
    }
}

fn era_from_snapshot(hud: &crate::HudState) -> String {
    let pop = hud.snapshot.civilian_count as u64;
    let techs = hud.snapshot.tech_count;
    if techs >= 12 { "Modern".to_string() }
    else if pop >= 10_000 || techs >= 10 { "Renaissance".to_string() }
    else if pop >= 5_000 || techs >= 8 { "Medieval".to_string() }
    else if pop >= 2_000 || techs >= 5 { "Classical".to_string() }
    else if pop >= 500 || techs >= 2 { "Ancient".to_string() }
    else { "Prehistoric".to_string() }
}