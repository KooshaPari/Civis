//! Bevy 0.18 renderer for `civ-voxel` worlds.
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
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::MeshMaterial3d;

use crate::{
    chunk_fade_alpha, chunk_fade_color, chunk_fade_complete, presentation_ambient_brightness,
    presentation_ambient_color_rgb, presentation_clear_color_rgb, CameraTarget, MeshBuffer,
    DEBUG_WIREFRAME_OVERLAY_ALPHA,
};

/// Wireframe line colour for chunk debug overlay.
pub const CHUNK_WIREFRAME_LINE_COLOR: Color = Color::srgba(0.92, 0.94, 0.98, 0.85);

/// Apply chunk PBR settings for normal, fade-in, or wireframe debug modes.
pub fn apply_chunk_material(
    material: &mut StandardMaterial,
    base_rgb: [f32; 3],
    wireframe_debug: bool,
    fade_elapsed: Option<f32>,
) {
    if wireframe_debug {
        material.base_color = Color::srgba(
            base_rgb[0],
            base_rgb[1],
            base_rgb[2],
            DEBUG_WIREFRAME_OVERLAY_ALPHA,
        );
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = true;
        material.perceptual_roughness = 0.85;
        material.metallic = 0.0;
        return;
    }

    material.unlit = false;
    if let Some(elapsed) = fade_elapsed {
        if chunk_fade_complete(elapsed) {
            material.base_color = Color::srgb(base_rgb[0], base_rgb[1], base_rgb[2]);
            material.alpha_mode = AlphaMode::Opaque;
        } else {
            let alpha = chunk_fade_alpha(elapsed);
            let rgba = chunk_fade_color(base_rgb, alpha);
            material.base_color = Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3]);
            material.alpha_mode = AlphaMode::Blend;
        }
    } else {
        material.base_color = Color::srgb(base_rgb[0], base_rgb[1], base_rgb[2]);
        material.alpha_mode = AlphaMode::Opaque;
    }
}

/// Convert an engine-neutral [`MeshBuffer`] into a Bevy [`Mesh`].
///
/// Vertex layout: position (Vec3) + normal (Vec3) + UV (Vec2). The kernel
/// `MeshBuffer` already carries all three. Uses a single pass with pre-sized
/// attribute vectors to avoid repeated iterator allocations.
#[must_use]
pub fn mesh_buffer_to_bevy(buf: &MeshBuffer) -> Mesh {
    let vertex_count = buf.vertices.len();
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    for vertex in &buf.vertices {
        positions.push(vertex.position);
        normals.push(vertex.normal);
        uvs.push(vertex.uv);
    }

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

/// Spawn a default PBR scene: camera + sun. Voxel meshes are added separately
/// by the caller via [`spawn_voxel_mesh`].
///
/// Camera placement follows [`CameraTarget::default`] (orbit around the first
/// chunk centre). Drag-to-orbit is not wired yet.
pub fn spawn_default_scene(commands: &mut Commands) {
    let camera = CameraTarget::default();
    let eye = camera.orbit_position();
    let centre = Vec3::from_array(camera.centre);

    let day_factor = 1.0_f32;
    let clear_rgb = presentation_clear_color_rgb(day_factor);
    let ambient_rgb = presentation_ambient_color_rgb(day_factor);
    commands.insert_resource(ClearColor(Color::srgb(
        clear_rgb[0],
        clear_rgb[1],
        clear_rgb[2],
    )));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(ambient_rgb[0], ambient_rgb[1], ambient_rgb[2]),
        brightness: presentation_ambient_brightness(day_factor),
        affects_lightmapped_meshes: true,
    });

    // Sun light — offset from the camera azimuth so voxels pick up depth.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(eye[0] + 12.0, eye[1] + 20.0, eye[2] + 8.0).looking_at(centre, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(eye[0], eye[1], eye[2]).looking_at(centre, Vec3::Y),
    ));
}

/// Spawn a voxel mesh entity with a basic stone-coloured PBR material.
pub fn spawn_voxel_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    buf: &MeshBuffer,
    chunk_id: crate::ChunkId,
    camera_eye: [f32; 3],
    max_dist: f32,
) {
    if !crate::should_render_chunk(chunk_id, camera_eye, max_dist) {
        return;
    }
    let _lod = crate::mesh_lod_level(max_dist);
    let handle = meshes.add(mesh_buffer_to_bevy(buf));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.69, 0.62),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((Mesh3d(handle), MeshMaterial3d(material)));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-001 — MeshBuffer with one quad converts to a Bevy Mesh
    /// carrying the expected attribute lengths.
    #[test]
    fn apply_chunk_material_wireframe_uses_unlit_low_alpha() {
        let mut material = StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            unlit: false,
            ..default()
        };
        apply_chunk_material(&mut material, [0.72, 0.69, 0.62], true, None);
        assert!(material.unlit);
        assert_eq!(material.alpha_mode, AlphaMode::Blend);
        assert!((material.base_color.alpha() - DEBUG_WIREFRAME_OVERLAY_ALPHA).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_chunk_material_restores_opaque_when_wireframe_off() {
        let mut material = StandardMaterial::default();
        apply_chunk_material(&mut material, [0.72, 0.69, 0.62], true, None);
        apply_chunk_material(&mut material, [0.72, 0.69, 0.62], false, None);
        assert!(!material.unlit);
        assert_eq!(material.alpha_mode, AlphaMode::Opaque);
    }

    #[test]
    fn mesh_buffer_converts_quad() {
        let buf = MeshBuffer {
            vertices: vec![
                crate::MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [0.0, 0.0],
                    material: crate::MaterialId(1),
                },
                crate::MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [1.0, 0.0],
                    material: crate::MaterialId(1),
                },
                crate::MeshVertex {
                    position: [1.0, 0.0, 1.0],
                    normal: [0.0, 1.0, 0.0],
                    uv: [1.0, 1.0],
                    material: crate::MaterialId(1),
                },
                crate::MeshVertex {
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
