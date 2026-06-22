//! Thin Civis-side bridge for the shared `phenotype-voxel` kernel.
//!
//! The bridge keeps the kernel world type at arm's length and translates dirty
//! chunk notifications into Bevy ECS entity churn.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{CommandQueue, Commands},
    world::World,
};
use bevy_math::IVec3;
use phenotype_voxel::{ChunkId, MaterialId, VoxelWorld};

/// Kernel schema version supported by this bridge.
const SUPPORTED_KERNEL_SCHEMA_VERSION: u32 = 1;

/// Thin Civis-side wrapper around the shared voxel world plus the active mesher choice.
pub struct CivisVoxelBridge {
    world: VoxelWorld<MaterialId>,
    mesher: MesherKind,
}

impl CivisVoxelBridge {
    /// Create a new bridge for a world that uses the given fixed-point voxel span.
    #[must_use]
    pub fn new(voxel_span: i64) -> Self {
        Self {
            world: VoxelWorld::new(voxel_span),
            mesher: MesherKind::default(),
        }
    }

    /// Borrow the wrapped voxel world.
    #[must_use]
    pub fn world(&self) -> &VoxelWorld<MaterialId> {
        &self.world
    }

    /// Borrow the wrapped voxel world mutably.
    #[must_use]
    pub fn world_mut(&mut self) -> &mut VoxelWorld<MaterialId> {
        &mut self.world
    }

    /// Return the active mesher kind.
    #[must_use]
    pub fn mesher(&self) -> MesherKind {
        self.mesher
    }
}

/// Mesher selection mirrored by Civis-side renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MesherKind {
    /// Greedy block meshing.
    Greedy,
    /// Surface nets meshing.
    SurfaceNet,
    /// Marching cubes meshing.
    Marching,
}

impl Default for MesherKind {
    fn default() -> Self {
        Self::Greedy
    }
}

/// Drain dirty chunks from the kernel world and schedule ECS remesh work.
///
/// Each dirty chunk despawns its previous entity, if any, then gets a fresh
/// placeholder entity so downstream Bevy systems can attach a rebuilt mesh on
/// the next pass.
pub fn drain_and_schedule_remesh(
    bridge: &mut CivisVoxelBridge,
    commands: &mut Commands<'_, '_>,
    chunk_entities: &mut HashMap<IVec3, Entity>,
) {
    let dirty = bridge.world.drain_dirty();
    let mut remesh_keys: Vec<IVec3> = dirty
        .into_iter()
        .map(|event| chunk_key_from_chunk_id(event.chunk_id))
        .collect();
    remesh_keys.sort_unstable();
    remesh_keys.dedup();

    for key in remesh_keys {
        if let Some(entity) = chunk_entities.remove(&key) {
            commands.entity(entity).despawn();
        }
        let entity = commands.spawn_empty().id();
        chunk_entities.insert(key, entity);
    }
}

/// Ensure the bridge and kernel agree on the public schema contract.
#[must_use]
pub fn check_version_compat(bridge: &CivisVoxelBridge) -> Result<(), String> {
    let _ = bridge;
    let kernel_schema = phenotype_voxel::SCHEMA_VERSION;
    if kernel_schema == SUPPORTED_KERNEL_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(format!(
            "voxel schema mismatch: bridge supports {SUPPORTED_KERNEL_SCHEMA_VERSION}, kernel exposes {kernel_schema}"
        ))
    }
}

fn chunk_key_from_chunk_id(chunk_id: ChunkId) -> IVec3 {
    let packed = chunk_id.0;
    let mut cx = ((packed >> 40) & 0x00ff_ffff) as i32;
    let mut cy = ((packed >> 16) & 0x00ff_ffff) as i32;
    let mut cz = (packed & 0x0000_ffff) as i32;
    if cx & 0x0080_0000 != 0 {
        cx |= !0x00ff_ffff;
    }
    if cy & 0x0080_0000 != 0 {
        cy |= !0x00ff_ffff;
    }
    if cz & 0x0000_8000 != 0 {
        cz |= !0x0000_ffff;
    }
    IVec3::new(cx, cy, cz)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_commands() -> (World, CommandQueue) {
        (World::new(), CommandQueue::default())
    }

    fn bridge_with_dirty_chunk() -> CivisVoxelBridge {
        let mut bridge = CivisVoxelBridge::new(1_000_000);
        bridge.world_mut().write(
            phenotype_voxel::WorldCoord { x: 0, y: 0, z: 0 },
            MaterialId(7),
        );
        bridge
    }

    /// FR-CIV-VOXEL-BRIDGE-001 — the bridge defaults to the greedy mesher.
    #[test]
    fn default_mesher_is_greedy() {
        let bridge = CivisVoxelBridge::new(1_000_000);
        assert_eq!(bridge.mesher(), MesherKind::Greedy);
    }

    /// FR-CIV-VOXEL-BRIDGE-002 — dirty chunks despawn the old entity and get a
    /// replacement entity queued for the remesh pass.
    #[test]
    fn dirty_chunk_replaces_entity() {
        let mut bridge = bridge_with_dirty_chunk();
        let (mut world, mut queue) = test_commands();
        let key = IVec3::new(0, 0, 0);
        let stale = world.spawn_empty().id();
        let mut chunk_entities = HashMap::from([(key, stale)]);
        let mut commands = Commands::new(&mut queue, &world);

        drain_and_schedule_remesh(&mut bridge, &mut commands, &mut chunk_entities);
        drop(commands);
        queue.apply(&mut world);

        assert!(!world.entities().contains(stale));
        assert_eq!(chunk_entities.len(), 1);
        let replacement = chunk_entities[&key];
        assert!(world.entities().contains(replacement));
        assert_ne!(replacement, stale);
    }

    /// FR-CIV-VOXEL-BRIDGE-003 — the bridge accepts the current kernel schema.
    #[test]
    fn version_compat_passes_for_current_kernel() {
        let bridge = CivisVoxelBridge::new(1_000_000);
        assert!(check_version_compat(&bridge).is_ok());
    }
}
