#import bevy_pbr::forward_io::VertexOutput

struct TriplanarParams {
    scale: f32,
    normal_strength: f32,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    vertex_color_blend: f32,
    sun_strength: f32,
    ambient: f32,
    light_dir: vec3<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: TriplanarParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var albedo_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var albedo_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var normal_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var normal_sampler: sampler;

fn triplanar_weights(n: vec3<f32>) -> vec3<f32> {
    let an = abs(n);
    let s = an.x + an.y + an.z + 1e-5;
    return an / s;
}

fn sample_albedo_triplanar(pos: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
    let w = triplanar_weights(n);
    let s = params.scale;
    let cx = textureSample(albedo_tex, albedo_sampler, pos.yz * s);
    let cy = textureSample(albedo_tex, albedo_sampler, pos.xz * s);
    let cz = textureSample(albedo_tex, albedo_sampler, pos.xy * s);
    return cx * w.x + cy * w.y + cz * w.z;
}

fn sample_normal_triplanar(pos: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let w = triplanar_weights(n);
    let s = params.scale;
    let nx = textureSample(normal_tex, normal_sampler, pos.yz * s).xyz * 2.0 - 1.0;
    let ny = textureSample(normal_tex, normal_sampler, pos.xz * s).xyz * 2.0 - 1.0;
    let nz = textureSample(normal_tex, normal_sampler, pos.xy * s).xyz * 2.0 - 1.0;
    let bx = vec3(0.0, nx.y, nx.z) * w.x;
    let by = vec3(ny.x, 0.0, ny.z) * w.y;
    let bz = vec3(nz.x, nz.y, 0.0) * w.z;
    return normalize(n + (bx + by + bz) * params.normal_strength);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position.xyz;
    let geom_n = normalize(in.world_normal);
    let detail_n = sample_normal_triplanar(world_pos, geom_n);
    let tex = sample_albedo_triplanar(world_pos, geom_n);
#ifdef VERTEX_COLORS
    let vert = in.color;
#else
    let vert = vec4(1.0);
#endif
    let flat = vec4(vert.rgb, 1.0);
    let base = mix(tex, flat, params.vertex_color_blend);
    let lit = params.ambient + params.sun_strength * max(dot(detail_n, normalize(params.light_dir)), 0.0);
    let spec = pow(max(dot(detail_n, normalize(params.light_dir)), 0.0), mix(8.0, 128.0, 1.0 - params.perceptual_roughness));
    let spec_col = spec * params.reflectance * (1.0 - params.metallic);
    return vec4(base.rgb * lit + vec3(spec_col), base.a);
}
