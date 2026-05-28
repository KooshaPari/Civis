use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub const GRID: usize = 256;
pub const WORLD_SIZE: f32 = 256.0;
pub const HEIGHT_SCALE: f32 = 120.0;
pub const WATER_LEVEL: f32 = 0.18 * HEIGHT_SCALE;

pub fn terrain_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(GRID * GRID);
    let mut normals = Vec::with_capacity(GRID * GRID);
    let mut colors = Vec::with_capacity(GRID * GRID);
    let half = WORLD_SIZE * 0.5;

    for z in 0..GRID {
        for x in 0..GRID {
            let fx = x as f32 / (GRID - 1) as f32;
            let fz = z as f32 / (GRID - 1) as f32;
            let wx = fx * WORLD_SIZE;
            let wz = fz * WORLD_SIZE;
            let height = terrain_height(wx, wz);
            positions.push([wx - half, height, wz - half]);
            normals.push([0.0, 1.0, 0.0]);
            colors.push(color_for_height(height));
        }
    }

    let mut indices = Vec::with_capacity((GRID - 1) * (GRID - 1) * 6);
    for z in 0..GRID - 1 {
        for x in 0..GRID - 1 {
            let i = (z * GRID + x) as u32;
            indices.extend_from_slice(&[
                i,
                i + GRID as u32,
                i + 1,
                i + 1,
                i + GRID as u32,
                i + GRID as u32 + 1,
            ]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Sample procedural terrain surface height at mesh-space XZ (0..[`WORLD_SIZE`]).
#[must_use]
pub fn terrain_surface_y(x: f32, z: f32) -> f32 {
    terrain_height(x.clamp(0.0, WORLD_SIZE), z.clamp(0.0, WORLD_SIZE))
}

pub fn terrain_height(x: f32, z: f32) -> f32 {
    let nx = x / WORLD_SIZE - 0.5;
    let nz = z / WORLD_SIZE - 0.5;
    let mut h = 0.0;
    let mut amp = 1.0;
    let mut freq = 0.018;
    for _ in 0..5 {
        h += value_noise(nx * freq, nz * freq) * amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    h = h / 1.9375;
    let ridge = (1.0 - (nx.abs() * 1.55).min(1.0)) * (1.0 - (nz.abs() * 1.55).min(1.0));
    let island = 1.0 - ((nx * nx + nz * nz).sqrt() * 1.85).clamp(0.0, 1.0);
    ((h * 0.62 + ridge * 0.18 + island * 0.20) - 0.18).clamp(0.0, 1.0) * HEIGHT_SCALE
}

pub fn color_for_height(height: f32) -> [f32; 4] {
    let t = height / HEIGHT_SCALE;
    if t < 0.18 {
        [0.20, 0.40, 0.86, 1.0]
    } else if t < 0.24 {
        [0.86, 0.78, 0.52, 1.0]
    } else if t < 0.48 {
        [0.28, 0.58, 0.24, 1.0]
    } else if t < 0.68 {
        [0.12, 0.34, 0.12, 1.0]
    } else if t < 0.85 {
        [0.50, 0.50, 0.52, 1.0]
    } else {
        [0.97, 0.97, 0.97, 1.0]
    }
}

pub fn value_noise(x: f32, z: f32) -> f32 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = x - xi;
    let zf = z - zi;
    let u = smooth(xf);
    let v = smooth(zf);
    let h00 = hash(xi, zi);
    let h10 = hash(xi + 1.0, zi);
    let h01 = hash(xi, zi + 1.0);
    let h11 = hash(xi + 1.0, zi + 1.0);
    let a = lerp(h00, h10, u);
    let b = lerp(h01, h11, u);
    lerp(a, b, v)
}

pub fn hash(x: f32, z: f32) -> f32 {
    ((x * 127.1 + z * 311.7).sin() * 43_758.547).fract().abs()
}

pub fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_surface_y_matches_height_inside_bounds() {
        let y = terrain_surface_y(64.0, 128.0);
        assert_eq!(y, terrain_height(64.0, 128.0));
    }

    #[test]
    fn terrain_surface_y_clamps_out_of_bounds() {
        let y = terrain_surface_y(-10.0, 999.0);
        assert_eq!(y, terrain_height(0.0, WORLD_SIZE));
    }
}
