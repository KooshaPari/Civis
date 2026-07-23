//! Public seam between voxel simulation and streaming/render clients.
//!
//! The individual feature modules are intentionally small contracts while
//! their implementations are being migrated from the historical prototype.

pub use civ_voxel::{ChunkId, MaterialId};

pub mod boundary;
pub mod fluid_ca;
pub mod hud;
pub mod lod;
pub mod material;
pub mod material_pbr;
pub mod reactions;
pub mod scale_budget;
pub mod stream;
pub mod window;
pub mod worldgen;

#[cfg(test)]
mod tests {
    use super::window::{ring_distance, WindowPolicy};
    use civ_voxel::ChunkCoord;

    #[test]
    fn window_adapter_uses_kernel_policy() {
        let policy = WindowPolicy::default();
        let anchor = ChunkCoord {
            cx: 0,
            cy: 0,
            cz: 0,
        };
        let neighbor = ChunkCoord {
            cx: 2,
            cy: 0,
            cz: 0,
        };

        assert_eq!(ring_distance(neighbor, anchor, policy.vy_weight), 2);
        assert_eq!(policy.sim_cohort(anchor, anchor), civ_voxel::SimCohort::FullSim);
    }
}
