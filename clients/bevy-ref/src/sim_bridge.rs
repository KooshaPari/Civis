//! Minimal in-process sim bridge plugin for the standalone Bevy client.

use bevy::prelude::*;

/// No-op bridge plugin placeholder that preserves the standalone wiring.
#[derive(Default)]
pub struct SimBridgePlugin;

impl Plugin for SimBridgePlugin {
    fn build(&self, _app: &mut App) {}
}
