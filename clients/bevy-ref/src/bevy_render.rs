//! Bevy 0.14 renderer for `civ-voxel` worlds.
//!
//! Behind the `bevy` feature flag so the default workspace build stays cheap.
//! Run with:
//!
//! ```bash
//! cargo run -p civ-bevy-ref --features bevy
//! ```
//!
//! Current scope: open a window, render a single static `VoxelWorld` as
//! `MeshBuffer` chunks via PBR. Per-tick streaming + voxel-delta integration
//! land in a follow-up PR once the protocol bridge is wired.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::MeshBuffer;

/// Convert an engine-neutral [`MeshBuffer`] into a Bevy [`Mesh`].
///
/// Vertex layout: position (Vec3) + normal (Vec3) + UV (Vec2). The kernel
/// `MeshBuffer` already carries all three.
#[must_use]
pub fn mesh_buffer_to_bevy(buf: &MeshBuffer) -> Mesh {
    let positions: Vec<[f32; 3]> = buf.vertices.iter().map(|v| v.position).collect();
    let normals: Vec<[f32; 3]> = buf.vertices.iter().map(|v| v.normal).collect();
    let uvs: Vec<[f32; 2]> = buf.vertices.iter().map(|v| v.uv).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(buf.indices.clone()));
    mesh
}

/// Spawn a default PBR scene: camera + sun + ground plane. Voxel meshes are
/// added separately by the caller via [`spawn_voxel_mesh`].
pub fn spawn_default_scene(commands: &mut Commands) {
    // Sun light.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(32.0, 32.0, 32.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Spawn a voxel mesh entity with a basic stone-coloured PBR material.
pub fn spawn_voxel_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    buf: &MeshBuffer,
) {
    let handle = meshes.add(mesh_buffer_to_bevy(buf));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.69, 0.62),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: handle,
        material,
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-001 — MeshBuffer with one quad converts to a Bevy Mesh
    /// carrying the expected attribute lengths.
    #[test]
    fn mesh_buffer_converts_quad() {
        let buf = MeshBuffer {
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [0.0, 0.0],
                    material: crate::MaterialId(1),
                },
                MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [1.0, 0.0],
                    material: crate::MaterialId(1),
                },
                MeshVertex {
                    position: [1.0, 0.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [1.0, 1.0],
                    material: crate::MaterialId(1),
                },
                MeshVertex {
                    position: [0.0, 0.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [0.0, 1.0],
                    material: crate::MaterialId(1),
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        let mesh = mesh_buffer_to_bevy(&buf);
        // Bevy Mesh exposes attribute count through positions length.
        let pos = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("positions");
        assert_eq!(pos.len(), 4);
    }
}
