#![cfg(feature = "bevy")]

use bevy::prelude::*;

use civ_engine::godtools::{
    DisasterOp, DisasterRequest, GodToolReceipt, GodToolRequest, InspectOp, InspectRequest,
    LifeRequest, MaterialRequest, SpawnOrganism, TerraformOp, TerraformRequest,
};
use civ_voxel::{MaterialId, WorldCoord};

use crate::sim_bridge::SimState;

#[derive(Debug, Clone, Message)]
pub struct GodToolRequestEvent {
    pub request: GodToolRequest,
}

#[derive(Debug, Resource, Default)]
pub struct GodToolEventLog {
    entries: Vec<GodToolReceipt>,
    capacity: usize,
}

impl GodToolEventLog {
    pub const DEFAULT_CAPACITY: usize = 16;

    #[must_use]
    pub fn new() -> Self {
        Self { entries: Vec::new(), capacity: Self::DEFAULT_CAPACITY }
    }

    #[must_use]
    pub fn entries(&self) -> &[GodToolReceipt] {
        &self.entries
    }

    fn push(&mut self, receipt: GodToolReceipt) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(receipt);
    }
}

#[derive(Debug, Default)]
pub struct GodToolsPlugin;

impl Plugin for GodToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodToolEventLog>()
            .add_message::<GodToolRequestEvent>()
            .add_systems(Update, dispatch_god_tool_requests);
    }
}

pub fn dispatch_god_tool_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<GodToolRequestEvent>,
    mut log: ResMut<GodToolEventLog>,
) {
    for event in requests.read() {
        match sim.0.apply_god_tool(event.request.clone()) {
            Ok(receipt) => log.push(receipt),
            Err(err) => tracing::warn!(error = %err, "god-tool request rejected"),
        }
    }
}

pub fn raise_terrain(world: &mut World, center: WorldCoord, delta: i32, radius: i32) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Terraform(TerraformRequest { op: TerraformOp::Raise, center, delta, target_height: 0, radius }) });
}

pub fn lower_terrain(world: &mut World, center: WorldCoord, delta: i32, radius: i32) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Terraform(TerraformRequest { op: TerraformOp::Lower, center, delta, target_height: 0, radius }) });
}

pub fn level_terrain(world: &mut World, center: WorldCoord, target_height: i32, radius: i32) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Terraform(TerraformRequest { op: TerraformOp::Level, center, delta: 0, target_height, radius }) });
}

pub fn replace_material(world: &mut World, center: WorldCoord, material: MaterialId, radius: i32, depth: i32) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Material(MaterialRequest { center, material, radius, depth }) });
}

pub fn spawn_organism(world: &mut World, civilian_id: u64, alignment: civ_agents::Alignment, x: f32, y: f32, visual: civ_agents::ActorVisualKind) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Life(LifeRequest { spawn: SpawnOrganism { civilian_id, alignment, x, y, visual } }) });
}

pub fn cast_meteor(world: &mut World, center: WorldCoord) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Disaster(DisasterRequest { op: DisasterOp::Meteor, center }) });
}

pub fn probe(world: &mut World, coord: WorldCoord) {
    world.write_message(GodToolRequestEvent { request: GodToolRequest::Inspect(InspectRequest { op: InspectOp::Probe, coord }) });
}
