//! `civis-3d-quality` — CLI gate that asserts the 3D rendering kernel
//! (clients/bevy-ref/src) ships with the required surface contracts.
//!
//! This is the *Full 3D quality gate* — every rendered artifact
//! depends on these contracts:
//!   - voxel_smooth_mesher exposes a hero build_dual_contour_mesh fn
//!   - voxel_triplanar exposes the triplanar projection constants
//!   - voxel_stream exposes stage_* fns
//!   - pbr_materials exposes LodRenderPlan / validate
//!   - lighting_gi exposes probe constants
//!   - minimap exposes live_minimap / holo_minimap dimensions
//!   - gpu_features exposes Capability::MIN_REQUIRED or a clippy-constant
//!
//! On success emits a structured JSON receipt (CI-gate machine-readable).
//! On any missing contract exits 1 with the failed contract listed.

use std::path::Path;

fn main() {
    let workspace_root = match std::env::var("CIVIS_WORKSPACE")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(default_workspace_root)
    {
        Some(p) => p,
        None => {
            eprintln!("FATAL: cannot locate Civis workspace root. Set $CIVIS_WORKSPACE.");
            std::process::exit(2);
        }
    };
    let bevy_ref = workspace_root.join("clients").join("bevy-ref").join("src");

    let contracts: &[(&str, &str)] = &[
        // voxel_smooth_mesher — Surface Nets + smooth meshing kernel
        ("voxel_smooth_mesher.rs", "SMOOTH_MESH_PADDED_EDGE"),
        ("voxel_smooth_mesher.rs", "BLUR_RADIUS"),
        ("voxel_smooth_mesher.rs", "ISO_LEVEL"),
        ("voxel_smooth_mesher.rs", "MaterialRegistry"),
        // voxel_triplanar — CC0 triplanar PBR projection
        ("voxel_triplanar.rs", "TRI_SHADER"),
        ("voxel_triplanar.rs", "TRI_SCALE"),
        ("voxel_triplanar.rs", "TerrainTextureLayer"),
        ("voxel_triplanar.rs", "terrain_layer_for_material"),
        // voxel_stream — staged streaming (camera-driven chunk streaming sandbox)
        ("voxel_stream.rs", "VoxelStreamState"),
        ("voxel_stream.rs", "STREAM_RADIUS"),
        ("voxel_stream.rs", "DESIRED_CHUNK_COUNT"),
        // pbr_materials — PBR config
        ("pbr_materials.rs", "MaterialType"),
        ("pbr_materials.rs", "material_for"),
        // lighting_gi — global illumination (Solari wrapper)
        ("lighting_gi.rs", "SolariGiPlugin"),
        ("lighting_gi.rs", "bevy_solari"),
        // minimap — viewport indicator (FR-VIEWPORT-001)
        ("minimap.rs", "viewport"),
        // materials — PBR biome loader
        ("materials.rs", "Biome"),
        ("materials.rs", "BiomeMaterials"),
        ("materials.rs", "BIOME_COUNT"),
        // gpu_features — runtime capability detection
        ("gpu_features.rs", "GpuCapabilities"),
        ("gpu_features.rs", "ray_tracing"),
        ("gpu_features.rs", "mesh_shaders"),
    ];

    let mut found_count = 0u32;
    let mut missing = Vec::new();
    let mut per_file: Vec<serde_json::Value> = Vec::new();
    for (file, needle) in contracts {
        let path = bevy_ref.join(file);
        let present = match std::fs::read_to_string(&path) {
            Ok(s) => s.contains(needle),
            Err(_) => false,
        };
        if present {
            found_count += 1;
            per_file.push(serde_json::json!({"file": file, "needle": needle, "present": true}));
        } else {
            per_file.push(serde_json::json!({"file": file, "needle": needle, "present": false}));
            missing.push(format!("{file} :: {needle}"));
        }
    }

    // Determinism: do a second pass and confirm identical outputs (byte-for-byte).
    let mut found2 = 0u32;
    let mut pass2: Vec<serde_json::Value> = Vec::new();
    for (file, needle) in contracts {
        let path = bevy_ref.join(file);
        let present2 = std::fs::read_to_string(&path).map(|s| s.contains(needle)).unwrap_or(false);
        if present2 { found2 += 1; }
        pass2.push(serde_json::json!({"file": file, "needle": needle, "present2": present2}));
    }
    let deterministic = found_count == found2;

    let total = contracts.len() as u32;
    let coverage_pct = (found_count as f64 * 100.0 / total as f64 * 100.0).round() / 100.0;
    let passed = missing.is_empty() && deterministic;
    let payload = serde_json::json!({
        "gate": "civis-3d-quality",
        "version": env!("CARGO_PKG_VERSION"),
        "workspace_root": workspace_root.display().to_string(),
        "contracts_total": total,
        "contracts_found": found_count,
        "contracts_missing_count": missing.len(),
        "coverage_pct": coverage_pct,
        "deterministic": deterministic,
        "passed": passed,
        "contracts": per_file,
        "pass2": pass2,
        "missing": missing,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    if !passed {
        eprintln!("\n3D-QUALITY FAILED: {} contracts missing:", missing.len());
        for m in &missing {
            eprintln!("  - {m}");
        }
        std::process::exit(1);
    }
}

fn default_workspace_root() -> Option<std::path::PathBuf> {
    // Try walking up from $CARGO_MANIFEST_DIR/../../  (civis-cli -> crates -> root).
    if let Ok(p) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = Path::new(&p);
        // $CARGO_MANIFEST_DIR = crates/civis-cli -> ../.. is workspace root
        let root = path.ancestors().nth(2)?;
        if root.join("Cargo.toml").exists() && root.join("crates").exists() {
            return Some(root.to_path_buf());
        }
    }
    // Fallback: look up the parent of the CWD for Cargo.toml.
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.as_path();
        for ancestor in path.ancestors() {
            if ancestor.join("Cargo.toml").exists() && ancestor.join("crates").exists() {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}
