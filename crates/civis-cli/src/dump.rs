//! Pure CIVIS_DUMP scene+sim JSON validation for render-frame regression.
//!
//! Parses the JSON document emitted by `clients/bevy-ref` [`scene_dump`] after
//! `CIVIS_DUMP=<path>` is set. Gates on machine-readable invariants the dump
//! is authoritative for: terrain mesh spread, floating actors/buildings, sim
//! counters, voxel census — not pixels.
//!
//! [`scene_dump`]: ../../../clients/bevy-ref/src/scene_dump.rs

use serde::{Deserialize, Serialize};

/// Marker lines wrapping dump JSON on stdout (see `scene_dump.rs`).
pub const DUMP_BEGIN_MARKER: &str = "=== CIVIS_DUMP BEGIN ===";
/// Closing marker for stdout-wrapped dumps.
pub const DUMP_END_MARKER: &str = "=== CIVIS_DUMP END ===";

/// Top-level CIVIS_DUMP document (matches `scene_dump.rs` wire shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneDump {
    /// Live sim counters, or `null` when no bridge is attached.
    pub sim: Option<SimSection>,
    /// Voxel grid census, or `null` when voxel sim is absent.
    pub voxel: Option<VoxelSection>,
    /// Chunk mesh entity aggregate (dissolved-terrain check).
    pub meshes: MeshesSection,
    /// Civilian actor positions + floating tally.
    pub actors: EntityPlacementSection,
    /// Building positions + floating tally.
    pub buildings: EntityPlacementSection,
    /// AnimationPlayer census (headless runs may read zero — see policy).
    pub animation: AnimationSection,
    /// Total ECS entity count (sanity).
    pub total_entities: usize,
}

/// Raw sim integers from the bridge snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimSection {
    /// Current sim tick.
    pub tick: u64,
    /// World population.
    pub population: u64,
    /// Spawned citizen count.
    pub citizen_count: u64,
    /// Building count.
    pub building_count: u64,
    /// Food resource.
    pub food: f64,
    /// Energy resource.
    pub energy: f64,
    /// Wood + metal combined.
    pub materials: f64,
}

/// Voxel grid census proving the data layer is populated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoxelSection {
    /// Grid dimensions `[x, y, z]`.
    pub dims: [u32; 3],
    /// Cells that are not air.
    pub non_air_cells: usize,
    /// Water cell count.
    pub water_cells: usize,
    /// Percent of cells that are water.
    pub water_pct: f64,
    /// Surface height samples at fixed grid fractions.
    pub surface_y_samples: Vec<f64>,
}

/// Chunk mesh entity spread (continuous terrain vs collapsed blobs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshesSection {
    /// Number of chunk mesh entities.
    pub count: usize,
    /// Minimum chunk-origin translation, or `null` when `count == 0`.
    pub origin_min: Option<[f64; 3]>,
    /// Maximum chunk-origin translation, or `null` when `count == 0`.
    pub origin_max: Option<[f64; 3]>,
}

/// Actor or building placement summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityPlacementSection {
    /// Total entities with the marker component.
    pub count: usize,
    /// Sample rows whose `|dy| > 1.0` (floating above/below terrain).
    pub floating: usize,
    /// Up to 20 sample placements.
    pub sample: Vec<PlacementSample>,
}

/// One sampled world position vs terrain surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementSample {
    /// World X.
    pub x: f64,
    /// World Y (transform translation).
    pub y: f64,
    /// World Z.
    pub z: f64,
    /// Terrain surface Y at `(x, z)`.
    pub surface_y: f64,
    /// `y - surface_y`.
    pub dy: f64,
}

/// AnimationPlayer playing-state census.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSection {
    /// Total AnimationPlayer components.
    pub players: usize,
    /// Players with at least one clip playing.
    pub playing: usize,
    /// `players - playing` (T-pose indicator when actors exist).
    pub t_posed: usize,
}

/// Regression gate configuration for a dump check run.
#[derive(Debug, Clone, PartialEq)]
pub struct DumpPolicy {
    /// Fail when `sim` is null.
    pub require_sim: bool,
    /// Fail when `voxel` is null.
    pub require_voxel: bool,
    /// Minimum chunk mesh entities (terrain present).
    pub min_mesh_count: usize,
    /// Minimum XZ extent of mesh origins (world not collapsed).
    pub min_mesh_spread_xz: f64,
    /// Minimum non-air voxel cells.
    pub min_non_air_cells: usize,
    /// Maximum allowed floating actors in the sample window.
    pub max_floating_actors: usize,
    /// Maximum allowed floating buildings in the sample window.
    pub max_floating_buildings: usize,
    /// Minimum total ECS entities.
    pub min_total_entities: usize,
    /// When true, fail if actors exist but animation players are zero (headful).
    pub require_animation_when_actors: bool,
    /// Maximum tolerated T-posed players when animation gate is on.
    pub max_t_posed: usize,
}

impl DumpPolicy {
    /// Headless `CIVIS_DUMP` run: sim/voxel/mesh/terrain gates on; animation off.
    #[must_use]
    pub fn headless() -> Self {
        Self {
            require_sim: true,
            require_voxel: true,
            min_mesh_count: 1,
            min_mesh_spread_xz: 16.0,
            min_non_air_cells: 1_000,
            max_floating_actors: 0,
            max_floating_buildings: 0,
            min_total_entities: 10,
            require_animation_when_actors: false,
            max_t_posed: usize::MAX,
        }
    }

    /// Windowed run with GLTF scenes: same as headless plus animation gate.
    #[must_use]
    pub fn headful() -> Self {
        let mut policy = Self::headless();
        policy.require_animation_when_actors = true;
        policy.max_t_posed = 0;
        policy
    }

    /// Parse policy name (`headless` | `headful`).
    pub fn from_name(name: &str) -> Result<Self, DumpError> {
        match name {
            "headless" => Ok(Self::headless()),
            "headful" => Ok(Self::headful()),
            other => Err(DumpError::UnknownPolicy(other.to_string())),
        }
    }
}

/// Numeric tolerances when diffing actual vs baseline dumps.
#[derive(Debug, Clone, PartialEq)]
pub struct DumpTolerance {
    /// Absolute tolerance for resource floats (`food`, `energy`, `materials`).
    pub resource_abs: f64,
    /// Absolute tolerance for placement/sample floats.
    pub placement_abs: f64,
    /// Allowed delta for integer counts (`tick`, mesh count, etc.).
    pub count_delta: u64,
}

impl Default for DumpTolerance {
    fn default() -> Self {
        Self {
            resource_abs: 1.0,
            placement_abs: 0.5,
            count_delta: 0,
        }
    }
}

/// One failed gate with a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DumpViolation {
    /// Gate identifier (stable for CI logs).
    pub gate: String,
    /// Actionable failure message.
    pub message: String,
}

/// Outcome of [`validate_dump`] or [`compare_to_baseline`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DumpReport {
    /// True when `violations` is empty.
    pub passed: bool,
    /// Failed gates (empty when passed).
    pub violations: Vec<DumpViolation>,
    /// Policy name when running invariant checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    /// Baseline path label when diffing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
}

impl DumpReport {
    /// Convenience constructor for a clean pass.
    #[must_use]
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            policy: None,
            baseline: None,
        }
    }

    /// Attach policy label for JSON output.
    #[must_use]
    pub fn with_policy(mut self, policy: impl Into<String>) -> Self {
        self.policy = Some(policy.into());
        self
    }

    /// Attach baseline label for JSON output.
    #[must_use]
    pub fn with_baseline(mut self, baseline: impl Into<String>) -> Self {
        self.baseline = Some(baseline.into());
        self
    }
}

/// Errors parsing or configuring dump checks.
#[derive(Debug, thiserror::Error)]
pub enum DumpError {
    /// JSON decode failure.
    #[error("decode CIVIS_DUMP JSON: {0}")]
    Decode(#[from] serde_json::Error),
    /// stdout markers missing or malformed.
    #[error("extract CIVIS_DUMP markers: {0}")]
    Extract(String),
    /// Unknown policy name.
    #[error("unknown dump policy `{0}` (expected headless or headful)")]
    UnknownPolicy(String),
    /// A CLI tolerance was negative and would invert the diff semantics.
    #[error("invalid {name} `{value}` (expected a non-negative tolerance)")]
    InvalidTolerance {
        /// CLI field name.
        name: String,
        /// Raw value supplied by the operator.
        value: f64,
    },
    /// IO error reading a dump file (bin layer).
    #[error("read dump file {path}: {source}")]
    Io {
        /// Path that could not be read.
        path: std::path::PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// One or more regression gates failed (see JSON report on stdout).
    #[error("{count} regression gate(s) failed")]
    GatesFailed {
        /// Number of violations.
        count: usize,
    },
}

/// Parse a CIVIS_DUMP JSON document from text.
pub fn parse_dump_json(text: &str) -> Result<SceneDump, DumpError> {
    serde_json::from_str(text).map_err(DumpError::from)
}

/// Extract JSON between `=== CIVIS_DUMP BEGIN ===` / `END ===` markers.
pub fn extract_dump_from_markers(text: &str) -> Result<SceneDump, DumpError> {
    let begin = text
        .find(DUMP_BEGIN_MARKER)
        .ok_or_else(|| DumpError::Extract(format!("missing `{DUMP_BEGIN_MARKER}`")))?;
    let after_begin = &text[begin + DUMP_BEGIN_MARKER.len()..];
    let end = after_begin
        .find(DUMP_END_MARKER)
        .ok_or_else(|| DumpError::Extract(format!("missing `{DUMP_END_MARKER}`")))?;
    let json = after_begin[..end].trim();
    parse_dump_json(json)
}

/// Run invariant gates from [`DumpPolicy`] against a parsed dump.
#[must_use]
pub fn validate_dump(dump: &SceneDump, policy: &DumpPolicy) -> DumpReport {
    let mut violations = Vec::new();
    check_sim_voxel(dump, policy, &mut violations);
    check_meshes(dump, policy, &mut violations);
    check_placements(dump, policy, &mut violations);
    check_animation(dump, policy, &mut violations);
    check_entity_floor(dump, policy, &mut violations);
    DumpReport {
        passed: violations.is_empty(),
        violations,
        policy: None,
        baseline: None,
    }
}

/// Diff `actual` against `baseline` within [`DumpTolerance`].
#[must_use]
pub fn compare_to_baseline(
    actual: &SceneDump,
    baseline: &SceneDump,
    tolerance: &DumpTolerance,
) -> DumpReport {
    let mut violations = Vec::new();
    diff_sim(actual, baseline, tolerance, &mut violations);
    diff_voxel(actual, baseline, tolerance, &mut violations);
    diff_counts(actual, baseline, tolerance, &mut violations);
    diff_mesh_bounds(actual, baseline, tolerance, &mut violations);
    DumpReport {
        passed: violations.is_empty(),
        violations,
        policy: None,
        baseline: None,
    }
}

fn check_sim_voxel(dump: &SceneDump, policy: &DumpPolicy, out: &mut Vec<DumpViolation>) {
    if policy.require_sim && dump.sim.is_none() {
        push_violation(out, "sim.present", "sim section is null");
    }
    if policy.require_voxel && dump.voxel.is_none() {
        push_violation(out, "voxel.present", "voxel section is null");
    }
    if let Some(voxel) = &dump.voxel {
        if voxel.non_air_cells < policy.min_non_air_cells {
            push_violation(
                out,
                "voxel.non_air_cells",
                format!(
                    "non_air_cells {} < minimum {}",
                    voxel.non_air_cells, policy.min_non_air_cells
                ),
            );
        }
    }
}

fn check_meshes(dump: &SceneDump, policy: &DumpPolicy, out: &mut Vec<DumpViolation>) {
    if dump.meshes.count < policy.min_mesh_count {
        push_violation(
            out,
            "meshes.count",
            format!(
                "mesh count {} < minimum {}",
                dump.meshes.count, policy.min_mesh_count
            ),
        );
        return;
    }
    let Some(spread) = mesh_spread_xz(&dump.meshes) else {
        push_violation(
            out,
            "meshes.spread",
            "mesh bounds missing despite count > 0",
        );
        return;
    };
    if spread < policy.min_mesh_spread_xz {
        push_violation(
            out,
            "meshes.spread",
            format!(
                "mesh XZ spread {spread:.1} < minimum {}",
                policy.min_mesh_spread_xz
            ),
        );
    }
}

fn check_placements(dump: &SceneDump, policy: &DumpPolicy, out: &mut Vec<DumpViolation>) {
    if dump.actors.floating > policy.max_floating_actors {
        push_violation(
            out,
            "actors.floating",
            format!(
                "floating actors {} > maximum {}",
                dump.actors.floating, policy.max_floating_actors
            ),
        );
    }
    if dump.buildings.floating > policy.max_floating_buildings {
        push_violation(
            out,
            "buildings.floating",
            format!(
                "floating buildings {} > maximum {}",
                dump.buildings.floating, policy.max_floating_buildings
            ),
        );
    }
}

fn check_animation(dump: &SceneDump, policy: &DumpPolicy, out: &mut Vec<DumpViolation>) {
    if !policy.require_animation_when_actors || dump.actors.count == 0 {
        return;
    }
    if dump.animation.players == 0 {
        push_violation(
            out,
            "animation.players",
            "actors present but animation.players is 0 (GLTF scenes not instantiated?)",
        );
    }
    if dump.animation.t_posed > policy.max_t_posed {
        push_violation(
            out,
            "animation.t_posed",
            format!(
                "t_posed {} > maximum {}",
                dump.animation.t_posed, policy.max_t_posed
            ),
        );
    }
}

fn check_entity_floor(dump: &SceneDump, policy: &DumpPolicy, out: &mut Vec<DumpViolation>) {
    if dump.total_entities < policy.min_total_entities {
        push_violation(
            out,
            "total_entities",
            format!(
                "total_entities {} < minimum {}",
                dump.total_entities, policy.min_total_entities
            ),
        );
    }
}

fn mesh_spread_xz(meshes: &MeshesSection) -> Option<f64> {
    let min = meshes.origin_min?;
    let max = meshes.origin_max?;
    let dx = (max[0] - min[0]).abs();
    let dz = (max[2] - min[2]).abs();
    Some(dx.max(dz))
}

fn diff_sim(
    actual: &SceneDump,
    baseline: &SceneDump,
    tolerance: &DumpTolerance,
    out: &mut Vec<DumpViolation>,
) {
    match (&actual.sim, &baseline.sim) {
        (Some(a), Some(b)) => diff_sim_fields(a, b, tolerance, out),
        (None, None) => {}
        _ => push_violation(out, "sim.presence", "sim presence differs from baseline"),
    }
}

fn diff_sim_fields(
    actual: &SimSection,
    baseline: &SimSection,
    tolerance: &DumpTolerance,
    out: &mut Vec<DumpViolation>,
) {
    diff_u64(
        "sim.tick",
        actual.tick,
        baseline.tick,
        tolerance.count_delta,
        out,
    );
    diff_u64(
        "sim.population",
        actual.population,
        baseline.population,
        tolerance.count_delta,
        out,
    );
    diff_f64(
        "sim.food",
        actual.food,
        baseline.food,
        tolerance.resource_abs,
        out,
    );
    diff_f64(
        "sim.energy",
        actual.energy,
        baseline.energy,
        tolerance.resource_abs,
        out,
    );
    diff_f64(
        "sim.materials",
        actual.materials,
        baseline.materials,
        tolerance.resource_abs,
        out,
    );
}

fn diff_voxel(
    actual: &SceneDump,
    baseline: &SceneDump,
    tolerance: &DumpTolerance,
    out: &mut Vec<DumpViolation>,
) {
    match (&actual.voxel, &baseline.voxel) {
        (Some(a), Some(b)) => {
            if a.dims != b.dims {
                push_violation(out, "voxel.dims", "voxel dims differ from baseline");
            }
            diff_usize_count(
                "voxel.non_air_cells",
                a.non_air_cells,
                b.non_air_cells,
                tolerance.count_delta,
                out,
            );
        }
        (None, None) => {}
        _ => push_violation(
            out,
            "voxel.presence",
            "voxel presence differs from baseline",
        ),
    }
}

fn diff_counts(
    actual: &SceneDump,
    baseline: &SceneDump,
    tolerance: &DumpTolerance,
    out: &mut Vec<DumpViolation>,
) {
    diff_usize_count(
        "meshes.count",
        actual.meshes.count,
        baseline.meshes.count,
        tolerance.count_delta,
        out,
    );
    diff_usize_count(
        "actors.count",
        actual.actors.count,
        baseline.actors.count,
        tolerance.count_delta,
        out,
    );
    diff_usize_count(
        "actors.floating",
        actual.actors.floating,
        baseline.actors.floating,
        tolerance.count_delta,
        out,
    );
    diff_usize_count(
        "buildings.count",
        actual.buildings.count,
        baseline.buildings.count,
        tolerance.count_delta,
        out,
    );
}

fn diff_mesh_bounds(
    actual: &SceneDump,
    baseline: &SceneDump,
    tolerance: &DumpTolerance,
    out: &mut Vec<DumpViolation>,
) {
    let (Some(a_min), Some(a_max), Some(b_min), Some(b_max)) = (
        actual.meshes.origin_min,
        actual.meshes.origin_max,
        baseline.meshes.origin_min,
        baseline.meshes.origin_max,
    ) else {
        return;
    };
    for (axis, idx) in [("x", 0usize), ("y", 1), ("z", 2)] {
        diff_f64(
            &format!("meshes.origin_min.{axis}"),
            a_min[idx],
            b_min[idx],
            tolerance.placement_abs,
            out,
        );
        diff_f64(
            &format!("meshes.origin_max.{axis}"),
            a_max[idx],
            b_max[idx],
            tolerance.placement_abs,
            out,
        );
    }
}

fn diff_u64(gate: &str, actual: u64, baseline: u64, delta: u64, out: &mut Vec<DumpViolation>) {
    let diff = actual.abs_diff(baseline);
    if diff > delta {
        push_violation(
            out,
            gate,
            format!("actual {actual} vs baseline {baseline} (delta {diff} > {delta})"),
        );
    }
}

fn diff_usize_count(
    gate: &str,
    actual: usize,
    baseline: usize,
    delta: u64,
    out: &mut Vec<DumpViolation>,
) {
    diff_u64(gate, actual as u64, baseline as u64, delta, out);
}

fn diff_f64(gate: &str, actual: f64, baseline: f64, tol: f64, out: &mut Vec<DumpViolation>) {
    if (actual - baseline).abs() > tol {
        push_violation(
            out,
            gate,
            format!("actual {actual:.3} vs baseline {baseline:.3} (tol {tol})"),
        );
    }
}

fn push_violation(out: &mut Vec<DumpViolation>, gate: &str, message: impl Into<String>) {
    out.push(DumpViolation {
        gate: gate.to_string(),
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_HEADLESS: &str = include_str!("../fixtures/dump/good-headless.json");
    const BAD_FLOATING: &str = include_str!("../fixtures/dump/bad-floating-actors.json");
    const BAD_EMPTY: &str = include_str!("../fixtures/dump/bad-empty-scene.json");

    #[test]
    fn parse_good_fixture() {
        let dump = parse_dump_json(GOOD_HEADLESS).expect("parse");
        assert_eq!(dump.meshes.count, 64);
        assert!(dump.sim.is_some());
    }

    #[test]
    fn extract_from_markers() {
        let wrapped = format!("noise\n{DUMP_BEGIN_MARKER}\n{GOOD_HEADLESS}\n{DUMP_END_MARKER}\n");
        let dump = extract_dump_from_markers(&wrapped).expect("extract");
        assert_eq!(dump.actors.floating, 0);
    }

    #[test]
    fn headless_policy_passes_good_fixture() {
        let dump = parse_dump_json(GOOD_HEADLESS).expect("parse");
        let report = validate_dump(&dump, &DumpPolicy::headless());
        assert!(report.passed, "{report:?}");
    }

    #[test]
    fn headless_policy_fails_floating_actors() {
        let dump = parse_dump_json(BAD_FLOATING).expect("parse");
        let report = validate_dump(&dump, &DumpPolicy::headless());
        assert!(!report.passed);
        assert!(report
            .violations
            .iter()
            .any(|v| v.gate == "actors.floating"));
    }

    #[test]
    fn headless_policy_fails_empty_scene() {
        let dump = parse_dump_json(BAD_EMPTY).expect("parse");
        let report = validate_dump(&dump, &DumpPolicy::headless());
        assert!(!report.passed);
        assert!(report.violations.len() >= 3);
    }

    #[test]
    fn baseline_diff_is_clean_for_identical_fixture() {
        let dump = parse_dump_json(GOOD_HEADLESS).expect("parse");
        let report = compare_to_baseline(&dump, &dump, &DumpTolerance::default());
        assert!(report.passed);
    }

    #[test]
    fn baseline_diff_catches_sim_tick_drift() {
        let mut drifted = parse_dump_json(GOOD_HEADLESS).expect("parse");
        let baseline = parse_dump_json(GOOD_HEADLESS).expect("parse");
        drifted.sim.as_mut().expect("sim").tick = 99;
        let report = compare_to_baseline(&drifted, &baseline, &DumpTolerance::default());
        assert!(!report.passed);
        assert!(report.violations.iter().any(|v| v.gate == "sim.tick"));
    }

    #[test]
    fn headful_policy_requires_animation_players() {
        let dump = parse_dump_json(GOOD_HEADLESS).expect("parse");
        let report = validate_dump(&dump, &DumpPolicy::headful());
        assert!(!report.passed);
        assert!(report
            .violations
            .iter()
            .any(|v| v.gate == "animation.players"));
    }

    #[test]
    fn invalid_tolerance_is_constructible_for_cli_guard() {
        let err = DumpError::InvalidTolerance {
            name: "resource_tol".to_string(),
            value: -1.0,
        };
        assert_eq!(
            err.to_string(),
            "invalid resource_tol `-1` (expected a non-negative tolerance)"
        );
    }
}
