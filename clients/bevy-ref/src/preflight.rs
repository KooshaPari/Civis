//! Startup preflight diagnostics for the Bevy standalone client (P1.1.1).
//!
//! Validates critical dependencies before the main app loop. On failure, emits
//! structured `[preflight]` errors naming what failed and exits non-zero — no
//! silent degradation.

use bevy::mesh::Indices;
use bevy::prelude::*;
use civ_engine::Simulation;

use crate::native_backend::native_only_backends;
use crate::terrain::{self, WORLD_SIZE};
use crate::AttachMode;

/// Run all startup preflight checks. Returns `Ok(())` after printing the success line.
pub fn run_startup_preflight(attach_mode: AttachMode) -> Result<(), String> {
    check_terrain().map_err(|reason| format!("[preflight] Terrain x: {reason}"))?;

    if attach_mode == AttachMode::Standalone {
        check_simulation().map_err(|reason| format!("[preflight] Sim x: {reason}"))?;
    }

    let gpu_ok = check_gpu_adapter()
        .map_err(|reason| format!("[preflight] GPU x: {reason}"))?;

    let mut line = String::from("[preflight] Terrain ✓");
    if attach_mode == AttachMode::Standalone {
        line.push_str(" | Sim ✓");
    }
    line.push_str(&format!(" | GPU ✓ ({gpu_ok})"));
    eprintln!("{line}");
    Ok(())
}

fn check_terrain() -> Result<(), String> {
    let mesh = terrain::terrain_mesh();
    let vertex_count = mesh.count_vertices();
    if vertex_count == 0 {
        return Err("terrain_mesh produced zero vertices".to_string());
    }

    let expected = terrain::GRID * terrain::GRID;
    if vertex_count != expected {
        return Err(format!(
            "terrain_mesh vertex count {vertex_count} != expected {expected}"
        ));
    }

    match mesh.indices() {
        Some(Indices::U32(indices)) if !indices.is_empty() => {}
        Some(Indices::U16(indices)) if !indices.is_empty() => {}
        _ => return Err("terrain_mesh missing triangle indices".to_string()),
    }

    let centre_h = terrain::terrain_height(WORLD_SIZE * 0.5, WORLD_SIZE * 0.5);
    if !centre_h.is_finite() {
        return Err(format!("terrain_height at centre is non-finite: {centre_h}"));
    }
    if !(0.0..=terrain::HEIGHT_SCALE).contains(&centre_h) {
        return Err(format!(
            "terrain_height at centre out of range: {centre_h} (max {})",
            terrain::HEIGHT_SCALE
        ));
    }

    Ok(())
}

fn check_simulation() -> Result<(), String> {
    let sim = Simulation::new();
    if sim.state.tick != 0 {
        return Err(format!(
            "Simulation::new tick={} (expected 0)",
            sim.state.tick
        ));
    }

    const PREFLIGHT_SEED: u64 = 0x5EED_C171;
    let seeded = Simulation::with_seed(PREFLIGHT_SEED);
    if seeded.state.tick != 0 {
        return Err(format!(
            "Simulation::with_seed({PREFLIGHT_SEED}) tick={} (expected 0)",
            seeded.state.tick
        ));
    }
    if seeded.state.rng_seed != PREFLIGHT_SEED {
        return Err(format!(
            "Simulation::with_seed rng_seed={} (expected {PREFLIGHT_SEED})",
            seeded.state.rng_seed
        ));
    }

    Ok(())
}

fn check_gpu_adapter() -> Result<String, String> {
    let backends: wgpu::Backends = native_only_backends().into();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(backends);
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or_else(|| "no wgpu adapter for native backends".to_string())?;

    let info = adapter.get_info();
    Ok(format!("{} {:?}", info.name, info.backend))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_preflight_passes() {
        check_terrain().expect("terrain preflight");
    }

    #[test]
    fn simulation_preflight_passes() {
        check_simulation().expect("simulation preflight");
    }

    #[test]
    fn startup_preflight_success_line_standalone() {
        run_startup_preflight(AttachMode::Standalone).expect("preflight");
    }
}
