#![cfg(feature = "bevy")]

//! Shared HUD snapshot resource used by the Bevy client binary and the
//! library-side overlays that read live simulation state.

use bevy::prelude::{Entity, Resource};

use crate::LiveHudSnapshot;

/// Live HUD resource mirrored from the streaming client state.
#[derive(Resource)]
pub struct HudState {
    /// Latest streamed HUD snapshot.
    pub snapshot: LiveHudSnapshot,
    /// Overlay text entity updated each frame.
    pub text: Entity,
}
