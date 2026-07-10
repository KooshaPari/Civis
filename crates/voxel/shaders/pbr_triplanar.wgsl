// Civis PBR — triplanar fragment shader for voxel chunk surfaces.
//
// Targets bevy 0.14+ PBR pipeline convention (compatible with bevy 0.18 in this
// workspace — the WGSL surface itself is engine-agnostic and bevy's auto-converted
// `mesh_functions` / `pbr_functions` can be omitted when used as a custom shader).
//
// WHAT THIS SHADER DOES
// ---------------------
// Samples albedo / normal / ORM (R=AO,G=Roughness,B=Metallic) from THREE axis-aligned
// 2D texture-array slices (X / Y / Z) using world-space triplanar projection, then
// blends the three samples with per-fragment axis weights derived from the world-space
// normal. Output is a PBR fragment struct that a downstream pipeline (or a test
// harness) can hand to a lighting pass.
//
// ATLAS SLICING SCHEME
// --------------------
// `ATLAS_SIZE` defines the width and height of every layer (in texels). The full
// atlas is one `texture_2d_array` with three row-blocks of one layer each:
//
//   layer 0  →  X-axis albedo
//   layer 1  →  Y-axis albedo
//   layer 2  →  Z-axis albedo
//   layer 3  →  X-axis normal (tangent-space, .rgb / .a=1)
//   layer 4  →  Y-axis normal
//   layer 5  →  Z-axis normal
//   layer 6  →  X-axis ORM (R=AO, G=roughness, B=metallic)
//   layer 7  →  Y-axis ORM
//   layer 8  →  Z-axis ORM
//
// ROM/RM splitting (ORM combined vs MR + AO separate) is decided CPU-side by
// `GreedyAtlas` and `TriplanarPbrMaterial`; the shader always reads R/G/B from
// the ORM layer for simplicity.
//
// UV PROJECTION (TRIPLANAR)
// -------------------------
// For a world-space point P and world-space normal N:
//   uvX = (P.yz) / tile
//   uvY = (P.xz) / tile
//   uvZ = (P.xy) / tile
// Each axis sample is fetched from its corresponding array layer, and the three
// samples are blended by the per-fragment axis weights:
//
//   w = vec3(|N.x|, |N.y|, |N.z|)
//   w = w / max(sum(w), 1e-4)
//
// The result is renormalised and returned with `baseColor` stored in linear RGB.
// The caller decides whether the albedo texture was loaded sRGB and converts on
// the CPU (behaviour matches `ColorSpacePolicy::strict` in `material_pbr.rs`).
//
// COMPATIBILITY
// -------------
// Entry point is `@fragment` (no `@compute`, no `@vertex`) so it slots into
// bevy's PBR pipeline as a custom material using
// `Shader::from_wgsl_with_path` + `MaterialDescriptor`. Vertex inputs are the
// minimum the shader needs; bevy's `mesh_vertex_output` struct provides them.

#define ATLAS_SIZE 1024
#define ATLAS_LAYERS 9

struct PbrVertexInput {
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal:   vec3<f32>,
    @location(2) uv:             vec2<f32>,
    @location(3) matid:          u32,
};

struct PbrFragmentOutput {
    @location(0) baseColor:    vec4<f32>, // linear RGBA albedo
    @location(1) normal:       vec4<f32>, // world-space normal (xyz), alpha=1
    @location(2) metallic:     f32,       // 0..1
    @location(3) roughness:    f32,       // 0..1
    @location(4) occlusion:    f32,       // 0..1 (multiplicative; 1 = un-occluded)
    @location(5) emissive:     vec4<f32>, // linear RGBA emissive
};

// 2-D array texture atlas; layer index encodes axis (X/Y/Z) and channel
// (albedo/normal/orm) per the slicing scheme above. Sampler is filterable.
@group(0) @binding(0) var atlas:          texture_2d_array<f32>;
@group(0) @binding(1) var atlas_sampler:  sampler;

// Uniforms — driven from `TriplanarPbrMaterial` on the bevy side.
struct PbrParams {
    world_tile:        f32, // voxel units per tile; 1.0 = one tile per voxel
    texture_scale:     f32, // multiplier on the projected UV before sample
    vertex_color_blend: f32, // 0=full PBR, 1=vertex-color fallback (FR-005)
    has_emissive:      f32, // 1.0 if the emissive channel should be on, else 0
};
@group(0) @binding(2) var<uniform> params: PbrParams;

// --- Triplanar helpers ----------------------------------------------------

fn triplanar_axis_weights(n: vec3<f32>) -> vec3<f32> {
    let abs_n = vec3<f32>(abs(n.x), abs(n.y), abs(n.z));
    let sum = abs_n.x + abs_n.y + abs_n.z;
    // Degenerate-normal fallback: equal weights, matching
    // `triplanar_axis_weights` in `material_pbr.rs`.
    return select(abs_n / max(sum, 1e-4), vec3<f32>(1.0 / 3.0), sum <= 1e-4);
}

fn triplanar_uv(uv: vec2<f32>, tile: f32) -> vec2<f32> {
    return uv * (params.texture_scale / max(tile, 1e-4));
}

// Sample one axis-aligned slice of the atlas for a given point+channel.
// `channel_offset` maps the 3-layer axis pack to the offset within the
// atlas: albedo=0, normal=3, orm=6.
fn triplanar_sample(
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    channel_offset: i32,
) -> vec4<f32> {
    let tile = max(params.world_tile, 1e-4);
    let uv_x = triplanar_uv(vec2<f32>(world_pos.y, world_pos.z), tile);
    let uv_y = triplanar_uv(vec2<f32>(world_pos.x, world_pos.z), tile);
    let uv_z = triplanar_uv(vec2<f32>(world_pos.x, world_pos.y), tile);

    let sx = textureSample(atlas, atlas_sampler, uv_x, f32(channel_offset + 0));
    let sy = textureSample(atlas, atlas_sampler, uv_y, f32(channel_offset + 1));
    let sz = textureSample(atlas, atlas_sampler, uv_z, f32(channel_offset + 2));

    let w = triplanar_axis_weights(world_normal);
    return sx * w.x + sy * w.y + sz * w.z;
}

// --- Fragment entry point -------------------------------------------------

@fragment
fn pbr_triplanar_fragment(in: PbrVertexInput) -> PbrFragmentOutput {
    var out: PbrFragmentOutput;

    // Albedo (linear RGB; sRGB decode happens CPU-side per ColorSpacePolicy).
    let albedo = triplanar_sample(in.world_position, in.world_normal, 0);
    out.baseColor = vec4<f32>(albedo.rgb, 1.0);

    // Tangent-space normal map → world-space. Without per-pixel tangent data
    // we approximate by re-orienting the sampled normal against the dominant
    // axis of the world normal. This is the standard low-cost triplanar
    // technique (matches the FR-009 blend helper in `material_pbr.rs`).
    var sampled_normal = triplanar_sample(in.world_position, in.world_normal, 3);
    let nxy = sampled_normal.xy * 2.0 - vec2<f32>(1.0);
    let nz  = sqrt(max(1.0 - dot(nxy, nxy), 0.0));
    let abs_n = vec3<f32>(abs(in.world_normal.x), abs(in.world_normal.y), abs(in.world_normal.z));
    let dominant = max(abs_n.x, max(abs_n.y, abs_n.z));
    let tangent_n = select(
        vec3<f32>(nz, nxy.x, nxy.y),
        select(
            vec3<f32>(nxy.x, nz, nxy.y),
            vec3<f32>(nxy.x, nxy.y, nz),
            abs_n.y > abs_n.x
        ),
        dominant == abs_n.x
    );
    let blended_n = normalize(tangent_n + in.world_normal);
    out.normal = vec4<f32>(blended_n, 1.0);

    // ORM (R=AO, G=Roughness, B=Metallic).
    let orm = triplanar_sample(in.world_position, in.world_normal, 6);
    out.occlusion = orm.r;
    out.roughness = clamp(orm.g, 0.0, 1.0);
    out.metallic  = clamp(orm.b, 0.0, 1.0);

    // Emissive: kept off by default; flipped on via `params.has_emissive` so
    // voxel chunks that need a glow (lava, crystal) can override CPU-side.
    out.emissive = vec4<f32>(0.0, 0.0, 0.0, params.has_emissive);

    return out;
}
