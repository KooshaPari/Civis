//! Headless scene + sim state dump for machine-level verification.
//!
//! When `CIVIS_DUMP=<path>` is set, the app warms up (so chunk meshes, GLTF
//! scenes, and the first sim ticks land), then writes a single JSON document
//! describing the AUTHORITATIVE scene graph + sim counters and exits.
//!
//! This is the ground-truth introspection that lets a verifier find:
//!   * floating actors/buildings  -> entity Transform.y vs terrain surface_y
//!   * dissolved / fragmented terrain -> chunk mesh entity count + AABB spread
//!   * T-pose actors -> animation clip playing yes/no
//!   * wrong pop / era / resources -> raw sim integers, not pixels
//!   * material-panel-always-on etc. -> system run-condition flags
//!
//! It reads the same data the renderer consumes, so it cannot "hallucinate"
//! the way a screenshot read can. Pixels are never inspected.
//!
//! AUTHORITATIVE for (render-independent state): terrain mesh count/bounds,
//! voxel census, sim counters (tick/pop/citizen_count/resources), actor/building
//! LOGICAL positions vs terrain surface_y (seating check).
//!
//! NOT AUTHORITATIVE for (render-world-gated state): GLTF scene-graph
//! instantiation and therefore AnimationPlayer presence. In a headless dump run
//! (no window/GPU) Bevy does NOT instantiate GLTF SceneRoot child hierarchies,
//! so `animation.players` reads 0 even when a windowed run would have them.
//! Verify animation/material/lighting in a WINDOWED run, not from this dump.

use bevy::prelude::*;

/// Marks the app to dump scene+sim JSON to `path` after `warmup`, then exit.
#[derive(Resource)]
pub struct SceneDump {
    pub path: String,
    pub armed_at: std::time::Instant,
    pub warmup: std::time::Duration,
    pub done: bool,
}

/// Install the dump hook if `CIVIS_DUMP` is set. Returns true if armed.
pub fn arm_from_env(app: &mut App) -> bool {
    let Ok(path) = std::env::var("CIVIS_DUMP") else {
        return false;
    };
    let warmup_seconds = std::env::var("CIVIS_DUMP_WARMUP")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(6.0);
    info!("[scene_dump] armed: path={path} warmup={warmup_seconds:.1}s");
    app.insert_resource(SceneDump {
        path,
        armed_at: std::time::Instant::now(),
        warmup: std::time::Duration::from_secs_f32(warmup_seconds),
        done: false,
    })
    .add_systems(Update, dump_scene_system);
    true
}

/// After warmup, collect scene + sim state into JSON, write it, and exit.
#[allow(clippy::too_many_arguments)]
fn dump_scene_system(
    mut dump: ResMut<SceneDump>,
    transforms: Query<(Entity, &GlobalTransform)>,
    meshes_q: Query<&GlobalTransform, With<crate::voxel_sim::ChunkMeshTag>>,
    civilians: Query<&GlobalTransform, With<crate::sim_bridge::SimCivilianMarkerPublic>>,
    buildings: Query<&GlobalTransform, With<crate::sim_bridge::SimBuildingMarkerPublic>>,
    anim_players: Query<&AnimationPlayer>,
    voxel: Option<Res<crate::voxel_sim::VoxelSimState>>,
    sim: Option<Res<crate::sim_bridge::SimState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if dump.done {
        return;
    }
    if dump.armed_at.elapsed() < dump.warmup {
        return;
    }
    // Wait until the world is actually populated before dumping — the windowed
    // menu->Playing->worldgen->spawn chain finishes later than a fixed timer.
    // Once warmup elapses, keep waiting (up to a hard ceiling) until either
    // terrain meshes or actors exist, so we never dump an empty pre-spawn scene.
    let populated = meshes_q.iter().count() > 0 || civilians.iter().count() > 0;
    let hard_ceiling = dump.warmup + std::time::Duration::from_secs(30);
    if !populated && dump.armed_at.elapsed() < hard_ceiling {
        return;
    }
    dump.done = true;

    let mut out = String::new();
    out.push_str("{\n");

    // --- Sim counters (raw integers — the authoritative numbers) -----------
    if let Some(sim) = sim.as_ref() {
        let snap = sim.0.snapshot();
        out.push_str(&format!("  \"sim\": {{\n"));
        out.push_str(&format!("    \"tick\": {},\n", snap.tick));
        out.push_str(&format!("    \"population\": {},\n", snap.population));
        out.push_str(&format!("    \"citizen_count\": {},\n", snap.citizen_count));
        out.push_str(&format!(
            "    \"building_count\": {},\n",
            snap.building_count
        ));
        out.push_str(&format!(
            "    \"food\": {:.1},\n",
            snap.resources.food.to_f64()
        ));
        out.push_str(&format!(
            "    \"energy\": {:.1},\n",
            snap.resources.energy.to_f64()
        ));
        out.push_str(&format!(
            "    \"materials\": {:.1}\n",
            snap.resources.wood.to_f64() + snap.resources.metal.to_f64()
        ));
        out.push_str("  },\n");
    } else {
        out.push_str("  \"sim\": null,\n");
    }

    // --- Voxel grid census (proves the data layer is populated) ------------
    if let Some(voxel) = voxel.as_ref() {
        let dims = voxel.grid.dims;
        let air = civ_voxel::material::AIR;
        let water = civ_voxel::material::WATER;
        let non_air = voxel.grid.cells.iter().filter(|c| **c != air).count();
        let water_cells = voxel.grid.cells.iter().filter(|c| **c == water).count();
        let total = voxel.grid.cells.len().max(1);
        out.push_str("  \"voxel\": {\n");
        out.push_str(&format!(
            "    \"dims\": [{}, {}, {}],\n",
            dims[0], dims[1], dims[2]
        ));
        out.push_str(&format!("    \"non_air_cells\": {non_air},\n"));
        out.push_str(&format!(
            "    \"water_cells\": {water_cells},\n    \"water_pct\": {:.2},\n",
            100.0 * water_cells as f64 / total as f64
        ));
        // Surface heights at a few sample columns prove relief exists.
        let mut samples = Vec::new();
        for &(fx, fz) in &[(0.25, 0.25), (0.5, 0.5), (0.75, 0.75)] {
            let gx = fx * (dims[0] as f32 - 1.0);
            let gz = fz * (dims[2] as f32 - 1.0);
            let h = crate::voxel_sim::voxel_surface_y(&voxel.grid, gx, gz);
            samples.push(format!("{h:.1}"));
        }
        out.push_str(&format!(
            "    \"surface_y_samples\": [{}]\n",
            samples.join(", ")
        ));
        out.push_str("  },\n");
    } else {
        out.push_str("  \"voxel\": null,\n");
    }

    // --- Mesh entities: count + translation spread (dissolved-terrain check) -
    // Chunk meshes are positioned at their chunk-origin translations, so the
    // spread of translations reveals whether terrain covers the world extent
    // (continuous) or collapsed to a few clustered blobs (dissolved/fragmented).
    let mut mesh_count = 0usize;
    let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
    let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
    for gt in meshes_q.iter() {
        mesh_count += 1;
        let t = gt.translation();
        min_x = min_x.min(t.x);
        min_y = min_y.min(t.y);
        min_z = min_z.min(t.z);
        max_x = max_x.max(t.x);
        max_y = max_y.max(t.y);
        max_z = max_z.max(t.z);
    }
    out.push_str("  \"meshes\": {\n");
    out.push_str(&format!("    \"count\": {mesh_count},\n"));
    if mesh_count > 0 {
        out.push_str(&format!(
            "    \"origin_min\": [{min_x:.1}, {min_y:.1}, {min_z:.1}],\n"
        ));
        out.push_str(&format!(
            "    \"origin_max\": [{max_x:.1}, {max_y:.1}, {max_z:.1}]\n"
        ));
    } else {
        out.push_str("    \"origin_min\": null,\n    \"origin_max\": null\n");
    }
    out.push_str("  },\n");

    // --- Actors: position + floating check (y vs terrain surface) ----------
    let mut actor_rows = Vec::new();
    let mut floating_actors = 0usize;
    for gt in civilians.iter().take(20) {
        let p = gt.translation();
        let surface = voxel
            .as_ref()
            .map(|v| crate::voxel_sim::voxel_surface_y(&v.grid, p.x, p.z))
            .unwrap_or(0.0);
        let dy = p.y - surface;
        if dy.abs() > 1.0 {
            floating_actors += 1;
        }
        actor_rows.push(format!(
            "    {{\"x\": {:.1}, \"y\": {:.1}, \"z\": {:.1}, \"surface_y\": {:.1}, \"dy\": {:.2}}}",
            p.x, p.y, p.z, surface, dy
        ));
    }
    out.push_str(&format!(
        "  \"actors\": {{\n    \"count\": {},\n    \"floating\": {floating_actors},\n    \"sample\": [\n{}\n    ]\n  }},\n",
        civilians.iter().count(),
        actor_rows.join(",\n")
    ));

    // --- Buildings: position + floating check ------------------------------
    let mut building_rows = Vec::new();
    let mut floating_buildings = 0usize;
    for gt in buildings.iter().take(20) {
        let p = gt.translation();
        let surface = voxel
            .as_ref()
            .map(|v| crate::voxel_sim::voxel_surface_y(&v.grid, p.x, p.z))
            .unwrap_or(0.0);
        let dy = p.y - surface;
        if dy.abs() > 1.0 {
            floating_buildings += 1;
        }
        building_rows.push(format!(
            "    {{\"x\": {:.1}, \"y\": {:.1}, \"z\": {:.1}, \"surface_y\": {:.1}, \"dy\": {:.2}}}",
            p.x, p.y, p.z, surface, dy
        ));
    }
    out.push_str(&format!(
        "  \"buildings\": {{\n    \"count\": {},\n    \"floating\": {floating_buildings},\n    \"sample\": [\n{}\n    ]\n  }},\n",
        buildings.iter().count(),
        building_rows.join(",\n")
    ));

    // --- Animation: are any clips actually playing (T-pose check)? ----------
    // players==0 with actors>0 means actors spawned as PRIMITIVE capsules (no
    // GLTF scene = no AnimationPlayer), i.e. the model fallback fired and never
    // got swapped to a SceneRoot. That is the real T-pose root cause.
    let total_players = anim_players.iter().count();
    let playing = anim_players
        .iter()
        .filter(|p| p.playing_animations().count() > 0)
        .count();
    out.push_str(&format!(
        "  \"animation\": {{\"players\": {total_players}, \"playing\": {playing}, \"t_posed\": {}}},\n",
        total_players.saturating_sub(playing)
    ));

    // --- Total entity count (sanity) ---------------------------------------
    out.push_str(&format!(
        "  \"total_entities\": {}\n",
        transforms.iter().count()
    ));
    out.push_str("}\n");

    match std::fs::write(&dump.path, &out) {
        Ok(()) => info!("[scene_dump] wrote {} bytes to {}", out.len(), dump.path),
        Err(e) => error!("[scene_dump] write failed: {e}"),
    }
    // Also echo to stdout so a headless run captures it even without file access.
    println!("=== CIVIS_DUMP BEGIN ===\n{out}=== CIVIS_DUMP END ===");
    // HEADFUL-WITH-HEADLESS-ACCESS: when CIVIS_DUMP_KEEP_ALIVE=1, run the game
    // WINDOWED (GLTF scenes instantiate -> AnimationPlayers real -> animation
    // state is authoritative) and write the dump WITHOUT exiting. Otherwise the
    // headless one-shot path exits after writing (render-world-gated state is a
    // false-negative there — see module docs).
    if std::env::var("CIVIS_DUMP_KEEP_ALIVE").as_deref() != Ok("1") {
        exit.write(AppExit::Success);
    }
}
