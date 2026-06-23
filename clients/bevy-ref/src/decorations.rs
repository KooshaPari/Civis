use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use crate::terrain::terrain_height;

pub fn spawn_decorations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tree_trunk = meshes.add(Mesh::from(bevy::math::primitives::Cylinder::new(0.35, 3.2)));
    let tree_foliage = meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 1.6 }));
    let rock = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
        2.0, 1.3, 1.8,
    )));

    let tree_trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.22, 0.12),
        perceptual_roughness: 1.0,
        ..default()
    });
    let tree_foliage_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.34, 0.12),
        perceptual_roughness: 1.0,
        ..default()
    });
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.50, 0.50, 0.52),
        perceptual_roughness: 1.0,
        ..default()
    });

    for x in (24..232).step_by(28) {
        for z in (24..232).step_by(28) {
            let world_x = x as f32 - 128.0;
            let world_z = z as f32 - 128.0;
            let height = terrain_height(x as f32, z as f32);
            let noise = crate::terrain::hash(x as f32 * 0.31, z as f32 * 0.17);
            if height > 10.0 && noise > 0.58 {
                commands.spawn((
                    Mesh3d(tree_trunk.clone()),
                    MeshMaterial3d(tree_trunk_mat.clone()),
                    Transform::from_xyz(world_x, height + 1.6, world_z),
                ));
                commands.spawn((
                    Mesh3d(tree_foliage.clone()),
                    MeshMaterial3d(tree_foliage_mat.clone()),
                    Transform::from_xyz(world_x, height + 4.0, world_z)
                        .with_scale(Vec3::new(1.0, 1.1, 1.0)),
                ));
            } else if height > 8.0 && noise < 0.26 {
                commands.spawn((
                    Mesh3d(rock.clone()),
                    MeshMaterial3d(rock_mat.clone()),
                    Transform::from_xyz(world_x, height + 0.9, world_z)
                        .with_rotation(Quat::from_rotation_y(noise * std::f32::consts::TAU))
                        .with_scale(Vec3::new(1.0 + noise, 0.8 + noise * 0.3, 1.0)),
                ));
            }
        }
    }
}
