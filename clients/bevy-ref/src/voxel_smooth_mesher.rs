//! Terrain surface meshing for bevy-ref voxel chunks.
//!
//! The internal CA material grid stays untouched; this module only transforms a
//! dense 16³ chunk into a render mesh using a smooth extractor. The default path
//! is `Surface Nets` via the `surface-nets` crate (`v0.1.0`) with a per-material
//! density callback so material/physics signals can eventually drive softness.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

use civ_voxel::{material::AIR, MaterialId, MeshBuffer, MeshVertex};
use civ_voxel::material::MaterialRegistry;

// Must match voxel_sim::CHUNK_EDGE — both must be updated together.
const CHUNK_EDGE: usize = 32;
/// Apron ring (in voxels) carried around each chunk so the blur and Surface Nets
/// can see neighbour-chunk voxels and round across seams. A 2-voxel apron is what
/// a 5x5x5 (`BLUR_RADIUS = 2`) blur needs; with only 1 the outer blur ring falls
/// out of bounds and chunk faces step. Kept in sync with the apron built by
/// `voxel_sim::slice_chunk_with_apron` via `SMOOTH_MESH_PADDED_EDGE`.
// Increased from 2 to 3: wider apron gives more cross-chunk density context,
// reducing visible gaps at chunk seams. BLUR_RADIUS=2 stays <= APRON-1=2. OK.
const APRON: usize = 3;
const CHUNK_EDGE_PADDED: usize = CHUNK_EDGE + 2 * APRON;
pub const SMOOTH_MESH_PADDED_EDGE: usize = CHUNK_EDGE_PADDED;
/// Blur half-width. 2 -> a 5x5x5 Gaussian neighbourhood (125 samples) for finer edge
/// rounding on the rolling relief. Temporarily dropped to 1 (3x3x3) when chunk meshing
/// was SYNCHRONOUS on the main thread (radius 2 cost ~14s on load and blocked the
/// autoshot window); restored to 2 now that meshing is async/off-thread (task #5,
/// mesh dispatch ~0.5s, no main-thread hitch) so the 5³ cost is off the critical path.
/// Needs `APRON >= 2` (satisfied) so the outer blur ring stays inside the chunk apron.
const BLUR_RADIUS: isize = 2;
const SOLID_CENTER_BIAS: f32 = 0.08;
// Iso-surface tuning. The surface is the shared `solid_occ` contour at `ISO_LEVEL`.
// `ISO_LEVEL` near 0.5 keeps the surface gently rounded; a low value over-favors
// solid and re-hardens the field back to near-binary/cubic. `ISO_SPAN` is the
// gradient steepness across the boundary: a gentle span (~1.0) keeps the extracted
// surface SMOOTH — a steep span snaps Surface Nets vertices to the grid and reads
// cubic/stepped. Watertightness comes from the SHARED `solid_occ` field, not from a
// hard iso bias. The blur stays 3x3x3 to match the 1-voxel chunk apron.
const ISO_LEVEL: f32 = 0.47;
const ISO_SPAN: f32 = 1.0;
// Slope-aware softening: on steep transitions (near-vertical faces, where the local
// `solid_occ` is mid-range rather than saturated) the span eases toward
// `ISO_SPAN_STEEP` so those faces round instead of stepping. Saturated interior /
// pure-air keep the base `ISO_SPAN`.
const ISO_SPAN_STEEP: f32 = 0.8;
/// Signed-distance gain. The field value is `(ISO_LEVEL - solid_occ) * span * GAIN`.
/// Without it the magnitude is bounded by `max(ISO_LEVEL, 1-ISO_LEVEL) ≈ 0.53`, so a
/// fully-solid interior only reaches ~-0.53 and never reads as confidently solid (it
/// regressed the `density_field_varies` invariant: interior must be < -0.75). The
/// pre-refactor density used a `*2.0`; this restores it. Result is still clamped to
/// [-1,1], so the only effect is steeper saturation away from the iso contour —
/// interior → -1, open air → +1 — with no change to the zero-crossing surface shape.
const DENSITY_GAIN: f32 = 2.0;

static SMOOTH_CHUNKS: AtomicU64 = AtomicU64::new(0);
static CUBIC_CHUNKS: AtomicU64 = AtomicU64::new(0);

use surface_nets::surface_net;

/// Output mesh path for terrain generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainMesherMode {
    /// Surface Nets / Surface Nets + density field.
    Smooth,
    /// Legacy cubic mesher for fallback/legacy chunks.
    Cubic,
}

/// Run-time selectable mesher mode (env override).
///
/// `CIVIS_VOXEL_MESHER`:
/// - `cubic` -> legacy cubic faces
/// - anything else / unset -> smooth surface extractor
#[must_use]
pub fn resolved_mesher_mode() -> TerrainMesherMode {
    match std::env::var("CIVIS_VOXEL_MESHER")
        .unwrap_or_else(|_| "smooth".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "cubic" => TerrainMesherMode::Cubic,
        "smooth" | "" => TerrainMesherMode::Smooth,
        _ => TerrainMesherMode::Smooth,
    }
}

/// Internal hook for density shaping. Returns signed distance value where positive
/// means empty/air and negative means solid.

/// Number of sample positions per axis the mesher evaluates density at. Surface
/// Nets memoizes density over `resolution + 1` corners per axis.
const SAMPLE_EDGE: usize = CHUNK_EDGE_PADDED + 1;

/// Per-chunk blurred fields that are MATERIAL-INDEPENDENT, so they are computed
/// ONCE per chunk and shared across every material mesh instead of being re-blurred
/// per material (the per-material redundancy was the ~33s synchronous-mesh stall:
/// the 5x5x5 solid_occ blur ran once per material × ~5 materials × 144 chunks).
struct ChunkBlurField {
    /// Blurred occupancy of ANY solid (non-AIR) at each sample corner, `[0,1]`.
    solid: Vec<f32>,
    /// Blurred saturation at each sample corner, `[0,1]`.
    sat: Vec<f32>,
}

/// Build a single smooth terrain buffer shared across all materials.
pub fn build_smooth_meshes(
    voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    registry: &MaterialRegistry,
) -> Vec<MeshBuffer> {
    SMOOTH_CHUNKS.fetch_add(1, Ordering::Relaxed);
    // Shared, material-independent solid/saturation blur — blurred ONCE per chunk.
    let field = std::sync::Arc::new(build_chunk_blur_field(padded_voxels, saturation));
    let _ = (voxels, registry);
    let density = build_shared_solid_density(std::sync::Arc::clone(&field));
    let material_id = dominant_material_lookup(
        *padded_voxels,
        CHUNK_EDGE_PADDED as isize,
        CHUNK_EDGE_PADDED as isize,
        CHUNK_EDGE_PADDED as isize,
    );
    vec![build_surface_nets(density, material_id)]
}

/// Precompute the blurred solid + saturation fields over every sample corner once.
fn build_chunk_blur_field(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
) -> ChunkBlurField {
    let mut solid = vec![0.0f32; SAMPLE_EDGE * SAMPLE_EDGE * SAMPLE_EDGE];
    let mut sat = vec![0.0f32; SAMPLE_EDGE * SAMPLE_EDGE * SAMPLE_EDGE];
    for z in 0..SAMPLE_EDGE {
        for y in 0..SAMPLE_EDGE {
            for x in 0..SAMPLE_EDGE {
                let (s, a) = sample_blurred_solid(padded_voxels, saturation, x, y, z);
                let idx = x + y * SAMPLE_EDGE + z * SAMPLE_EDGE * SAMPLE_EDGE;
                solid[idx] = s;
                sat[idx] = a;
            }
        }
    }
    ChunkBlurField { solid, sat }
}

#[must_use]
pub fn mesher_chunk_stats() -> (u64, u64) {
    (
        SMOOTH_CHUNKS.load(Ordering::Relaxed),
        CUBIC_CHUNKS.load(Ordering::Relaxed),
    )
}

/// Record that a chunk took the legacy cubic path (the smooth path self-records
/// in `build_smooth_meshes`). Lets `mesher_chunk_stats` report a true smooth-vs-cubic
/// split so a runtime diagnostic can tell whether terrain is actually meshing smooth.
pub fn record_cubic_chunk() {
    CUBIC_CHUNKS.fetch_add(1, Ordering::Relaxed);
}

fn build_surface_nets(
    density: impl Fn(usize, usize, usize) -> f32 + 'static,
    material_id_at: impl Fn(f32, f32, f32) -> MaterialId + 'static,
) -> MeshBuffer {
    let (positions, normals, indices) = surface_net(CHUNK_EDGE_PADDED, &density, true);
    let mut vertices = Vec::with_capacity(positions.len());
    for (position, normal) in positions.iter().zip(normals.iter()) {
        let material = material_id_at(position[0], position[1], position[2]);
        let normal = normalize_or_unit_up(*normal);
        let position = [
            position[0] - APRON as f32,
            position[1] - APRON as f32,
            position[2] - APRON as f32,
        ];
        let uv = [
            (position[0].clamp(0.0, CHUNK_EDGE as f32 - 1.0)) / CHUNK_EDGE as f32,
            (position[2].clamp(0.0, CHUNK_EDGE as f32 - 1.0)) / CHUNK_EDGE as f32,
        ];
        vertices.push(MeshVertex {
            position,
            normal,
            uv,
            material,
        });
    }
    let indices = indices
        .into_iter()
        .filter_map(|i| u32::try_from(i).ok())
        .collect();
    MeshBuffer { vertices, indices }
}

fn build_shared_solid_density(
    field: std::sync::Arc<ChunkBlurField>,
) -> impl Fn(usize, usize, usize) -> f32 + 'static {
    move |x, y, z| {
        let idx = x + y * SAMPLE_EDGE + z * SAMPLE_EDGE * SAMPLE_EDGE;
        let solid_occ = field.solid[idx];
        let sat = field.sat[idx];
        let span = slope_aware_span(solid_occ);
        let density = (ISO_LEVEL - solid_occ) * span * DENSITY_GAIN;
        let saturation_soften = 1.0 - sat * 0.25;
        (density * saturation_soften).clamp(-1.0, 1.0)
    }
}

/// Pick a dominant non-AIR material near a vertex sample position.
/// Uses a cheap 3×3×3 neighbour weighted vote in padded-voxel space to avoid
/// hard seams and keep one shared surface mesh continuous.
#[inline]
fn dominant_material_lookup(
    padded_voxels: [MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    grid_w: isize,
    grid_h: isize,
    grid_d: isize,
) -> impl Fn(f32, f32, f32) -> MaterialId + 'static {
    move |x, y, z| {
        let fx = x.floor() as isize;
        let fy = y.floor() as isize;
        let fz = z.floor() as isize;
        let mut best_material = AIR;
        let mut best_score = -1.0f32;
        let mut all_air = true;
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let sx = fx + dx;
                    let sy = fy + dy;
                    let sz = fz + dz;
                    if sx < 0
                        || sy < 0
                        || sz < 0
                        || sx >= grid_w
                        || sy >= grid_h
                        || sz >= grid_d
                    {
                        continue;
                    }
                    let idx = sx as usize
                        + sy as usize * CHUNK_EDGE_PADDED
                        + sz as usize * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
                    let material = padded_voxels[idx];
                    if material == AIR {
                        continue;
                    }
                    all_air = false;
                    let dxp = sx as f32 - x;
                    let dyp = sy as f32 - y;
                    let dzp = sz as f32 - z;
                    let dist_sq = dxp * dxp + dyp * dyp + dzp * dzp;
                    let weight = 1.0 / (1.0 + dist_sq);
                    if weight > best_score {
                        best_score = weight;
                        best_material = material;
                    }
                }
            }
        }
        if all_air { AIR } else { best_material }
    }
}
/// Span used to scale the signed distance, eased on steep transitions.
///
/// Cells whose blurred `solid_occ` sits near `ISO_LEVEL` are the surface-defining
/// (slope/boundary) cells — exactly where a steep span snaps Surface Nets vertices
/// to the grid and reads stepped. Near the iso contour the span eases toward
/// `ISO_SPAN_STEEP` so those faces round; well inside solid or air it keeps the
/// base `ISO_SPAN`. `t` is 1.0 at the contour and falls to 0.0 by half a band away.
#[inline]
fn slope_aware_span(solid_occ: f32) -> f32 {
    const BAND: f32 = 0.5;
    let t = (1.0 - (solid_occ - ISO_LEVEL).abs() / BAND).clamp(0.0, 1.0);
    ISO_SPAN + (ISO_SPAN_STEEP - ISO_SPAN) * t
}

fn normalize_or_unit_up(mut normal: [f32; 3]) -> [f32; 3] {
    let mag_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    if mag_sq <= f32::EPSILON {
        return [0.0, 1.0, 0.0];
    }
    let inv = 1.0 / mag_sq.sqrt();
    normal[0] *= inv;
    normal[1] *= inv;
    normal[2] *= inv;
    normal
}

/// Blur the material-INDEPENDENT solid occupancy + saturation at one sample corner.
/// `solid_occ` = weighted fraction of any non-AIR neighbour (the shared surface
/// field); `sat` = weighted saturation. No per-material work — material ownership
/// is assigned after extraction.
fn sample_blurred_solid(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    x: usize,
    y: usize,
    z: usize,
) -> (f32, f32) {
    const DEFAULT_SAT: u8 = 0;
    let mut solid_occ = 0.0f32;
    let mut sat_acc = 0.0f32;
    let mut weight_sum = 0.0f32;
    for dz in -BLUR_RADIUS..=BLUR_RADIUS {
        for dy in -BLUR_RADIUS..=BLUR_RADIUS {
            for dx in -BLUR_RADIUS..=BLUR_RADIUS {
                let sx = x as isize + dx;
                let sy = y as isize + dy;
                let sz = z as isize + dz;
                let mut w = weight_for_offset(dx, dy, dz);
                let in_bounds = !(sx < 0
                    || sy < 0
                    || sz < 0
                    || sx >= CHUNK_EDGE_PADDED as isize
                    || sy >= CHUNK_EDGE_PADDED as isize
                    || sz >= CHUNK_EDGE_PADDED as isize);
                if !in_bounds {
                    // Out-of-world is air, but still keep full kernel sample
                    // weight for boundary consistency across chunk seams.
                    weight_sum += w;
                    continue;
                }
                let idx = sx as usize
                    + sy as usize * CHUNK_EDGE_PADDED
                    + sz as usize * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
                if dx == 0 && dy == 0 && dz == 0 {
                    w += SOLID_CENTER_BIAS;
                }
                if padded_voxels[idx] != AIR {
                    solid_occ += w;
                }
                let sat = saturation
                    .and_then(|arr| arr.get(idx))
                    .copied()
                    .unwrap_or(DEFAULT_SAT);
                sat_acc += f32::from(sat) / 255.0 * w;
                weight_sum += w;
            }
        }
    }
    let inv = if weight_sum > 0.0 { 1.0 / weight_sum } else { 0.0 };
    (solid_occ * inv, sat_acc * inv)
}

/// Gaussian weight for a kernel offset, by squared distance.
///
/// PERF: the blur runs ~125 samples per density evaluation and the mesher
/// evaluates density at ~9k corners per material per chunk (memoized), so a naive
/// `exp()` here is ~10^8+ transcendental calls on a 144-chunk world load (the
/// observed ~37s stall). Squared distance over a radius-2 kernel only takes the
/// values {0,1,2,3,4,5,6,8,9,12}, so we return precomputed `exp(-d²/(2·1.1²))`
/// constants via a match — same Gaussian, no runtime `exp`.
#[inline]
fn weight_for_offset(dx: isize, dy: isize, dz: isize) -> f32 {
    match dx * dx + dy * dy + dz * dz {
        0 => 1.0,
        1 => 0.661_514_7,
        2 => 0.437_601_6,
        3 => 0.289_479_9,
        4 => 0.191_495_2,
        5 => 0.126_676_9,
        6 => 0.083_798_6,
        8 => 0.036_670_4,
        9 => 0.024_258_0,
        _ => 0.007_022_2, // 12 (kernel corner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::{material::MaterialRegistry, MaterialId};

    /// Reference Gaussian the `weight_for_offset` match table mirrors.
    fn gaussian_reference(dx: isize, dy: isize, dz: isize) -> f32 {
        let d2 = (dx * dx + dy * dy + dz * dz) as f32;
        (-d2 / (2.0 * 1.1 * 1.1)).exp()
    }

    /// Build a single material's density closure over a padded chunk, mirroring the
    /// per-chunk shared-blur path `build_smooth_meshes` uses.
    fn density_for(
        padded: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    ) -> impl Fn(usize, usize, usize) -> f32 {
        let field = std::sync::Arc::new(build_chunk_blur_field(padded, None));
        build_shared_solid_density(field)
    }

    #[test]
    fn weight_table_matches_gaussian() {
        for dz in -BLUR_RADIUS..=BLUR_RADIUS {
            for dy in -BLUR_RADIUS..=BLUR_RADIUS {
                for dx in -BLUR_RADIUS..=BLUR_RADIUS {
                    let table = weight_for_offset(dx, dy, dz);
                    let exact = gaussian_reference(dx, dy, dz);
                    assert!(
                        (table - exact).abs() < 1e-4,
                        "weight mismatch at ({dx},{dy},{dz}): table={table} exact={exact}"
                    );
                }
            }
        }
    }

    #[test]
    fn normalize_or_unit_up_returns_default_for_zero() {
        assert_eq!(normalize_or_unit_up([0.0, 0.0, 0.0]), [0.0, 1.0, 0.0]);
    }

    /// A solid feature of meaningful size must mesh to geometry.
    ///
    /// NOTE on the invariant: this is a SMOOTHING mesher (Surface Nets over a blurred
    /// occupancy field), not a 1:1 voxel renderer. A single isolated voxel's blurred
    /// `solid_occ` peaks around 0.05–0.09 — below `ISO_LEVEL`, so the isosurface never
    /// crosses and a lone speck correctly DISSOLVES rather than rendering as a blob
    /// (verified by the standalone density model + matches how a smooth surface should
    /// behave). The real invariant is therefore "a feature of at least the smoothing
    /// radius (~3³) produces geometry" — a 1-voxel test encoded a wrong expectation for
    /// a smoothing mesher. The renderer's voxel_sim path additionally has a cubic
    /// fallback for any non-empty chunk the smooth path drops, so nothing vanishes
    /// on screen even for sub-threshold specks.
    #[test]
    fn smooth_mesher_handles_solid_feature() {
        let mut chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        // A 3x3x3 solid block (>= the smoothing radius) so the isosurface crosses.
        for z in 1..4 {
            for y in 1..4 {
                for x in 1..4 {
                    padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                        MaterialId(1);
                }
            }
        }
        chunk[0] = MaterialId(1);
        let registry = MaterialRegistry::standard();
        let bufs = build_smooth_meshes(&chunk, &padded, None, &registry);
        assert!(!bufs.is_empty(), "a 3x3x3 solid feature must produce mesh buffers");
        assert!(
            bufs.iter().all(|buf| !buf.vertices.is_empty()),
            "every produced buffer must have vertices"
        );
    }

    #[test]
    fn density_field_varies_across_half_filled_boundary() {
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        for z in 0..CHUNK_EDGE_PADDED {
            for y in 0..CHUNK_EDGE_PADDED {
                for x in 0..CHUNK_EDGE_PADDED {
                    if x <= 8 {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(1);
                    }
                }
            }
        }
        let density = density_for(&padded);
        let air = density(13, 10, 10);
        let solid = density(4, 10, 10);
        let boundary = density(8, 10, 10);
        assert!(air > 0.75);
        assert!(solid < -0.75);
        assert!(solid < -0.5);
        assert!(boundary < 0.0);
        assert!(boundary > solid);
        assert!(boundary < air);
    }

    #[test]
    fn interface_does_not_create_material_discontinuity_mesh() {
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        for z in 0..CHUNK_EDGE_PADDED {
            for y in 0..CHUNK_EDGE_PADDED {
                for x in 0..CHUNK_EDGE_PADDED {
                    if x <= 8 {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(1);
                    } else {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(2);
                    }
                }
            }
        }
        let registry = MaterialRegistry::standard();
        let chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let bufs = build_smooth_meshes(&chunk, &padded, None, &registry);
        assert!(!bufs.is_empty(), "interface region should mesh");
        let mut materials = HashSet::new();
        for v in bufs.iter().flat_map(|b| b.vertices.iter()) {
            materials.insert(v.material);
        }
        assert!(materials.contains(&MaterialId(1)), "missing first material on interface surface");
        assert!(materials.contains(&MaterialId(2)), "missing second material on interface surface");
        assert!(materials.len() >= 2, "interface should expose at least two materials");
    }

    /// Smooth-mesher INVARIANT: Surface Nets over a soft density field must place
    /// vertices OFF the integer voxel grid (interpolated), not snapped to it. A
    /// grid-snapped mesh = cubic/blocky; a high off-grid fraction = molded/smooth.
    /// In-game we measured ~99% off-grid; this locks the property in a unit test so a
    /// regression to a hard/binary field (which re-snaps vertices to the grid → cubic)
    /// fails CI instead of only being caught by eyeballing a screenshot.
    #[test]
    fn smooth_mesh_vertices_are_interpolated_not_grid_snapped() {
        // A diagonal solid wedge -> a sloped boundary the mesher must round.
        let mut chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        for z in 0..CHUNK_EDGE_PADDED {
            for y in 0..CHUNK_EDGE_PADDED {
                for x in 0..CHUNK_EDGE_PADDED {
                    if x + y <= CHUNK_EDGE_PADDED {
                        let idx = x
                            + y * CHUNK_EDGE_PADDED
                            + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
                        padded[idx] = MaterialId(1);
                    }
                }
            }
        }
        for c in chunk.iter_mut() {
            *c = MaterialId(1);
        }
        let registry = MaterialRegistry::standard();
        let bufs = build_smooth_meshes(&chunk, &padded, None, &registry);
        let verts: usize = bufs.iter().map(|b| b.vertices.len()).sum();
        assert!(verts > 0, "mesher produced no vertices for a solid wedge");
        let off_grid = bufs
            .iter()
            .flat_map(|b| b.vertices.iter())
            .filter(|v| v.position.iter().any(|c| (c - c.round()).abs() > 0.01))
            .count();
        // Most boundary vertices must interpolate; a hard/binary field would snap
        // (near-0% off-grid). Require a clear majority to be off the integer grid.
        let pct = off_grid as f32 / verts as f32;
        assert!(
            pct > 0.5,
            "expected mostly interpolated verts (smooth), got {pct:.2} off-grid ({off_grid}/{verts}) — field may be grid-snapping (cubic)"
        );
    }

    /// Solid chunk must produce geometry near the chunk boundary face.
    /// Regression: with APRON=3 the density field extends far enough into neighbour
    /// context that seam faces get vertices, reducing visible gaps between chunks.
    #[test]
    fn seam_chunk_produces_continuous_boundary_surface() {
        let chunk = [MaterialId(1); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let mut padded = [AIR; SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE];
        for z in 0..CHUNK_EDGE {
            for y in 0..CHUNK_EDGE {
                for x in 0..CHUNK_EDGE {
                    let px = x + APRON;
                    let py = y + APRON;
                    let pz = z + APRON;
                    padded[px + py * SMOOTH_MESH_PADDED_EDGE + pz * SMOOTH_MESH_PADDED_EDGE * SMOOTH_MESH_PADDED_EDGE] = MaterialId(1);
                }
            }
        }
        let registry = MaterialRegistry::standard();
        let bufs = build_smooth_meshes(&chunk, &padded, None, &registry);
        assert!(!bufs.is_empty(), "solid chunk produced no mesh buffers");
        assert_eq!(bufs.len(), 1, "smooth terrain should be a single combined buffer");
        let vertex_count: usize = bufs.iter().map(|b| b.vertices.len()).sum();
        println!("[seam_test] vertex_count={vertex_count}");
        assert!(vertex_count > 1000, "solid chunk should produce a large connected surface, got {vertex_count}");
        assert!(
            bufs.iter().any(|b| b.vertices.iter().any(|v| v.position[0] >= 10.0)),
            "no vertex near chunk boundary (x>=10) — seam gap likely (vertex_count={vertex_count})"
        );
    }
}
