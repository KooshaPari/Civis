//! Voxel/material inspector — FR-CIV-INSPECT-903.
//!
//! The data + query backing layer for the **voxel-specific** half of the
//! click-to-inspect world flow. While [`crate::inspect`] resolves *any* world
//! element (agent/vehicle/structure/settlement/voxel) to a tagged
//! [`Summary`](crate::inspect::Summary), this module owns the deeper
//! voxel/material query: given a picked world position, return the voxel's
//! material, temperature, pressure, mass, and phase.
//!
//! # Scope (per FR-CIV-INSPECT-903)
//!
//! "Voxel/material inspector SHALL show material, temperature, pressure,
//!  mass, phase. Reads `civ-voxel`/`civ-laws`; values match CA state."
//!
//! This module is a **pure-logic data/query layer** with no Bevy, no
//! rendering, and no coupling to `civ-voxel` / `civ-laws`. It owns:
//!
//! - Small, dependency-free data structs ([`VoxelKey`], [`VoxelState`],
//!   [`Phase`], [`VoxelInspectorSummary`]).
//! - An in-memory index ([`VoxelInspectorIndex`]) that maps `VoxelKey → VoxelState`.
//! - A registry/query ([`VoxelInspector::summary`]) that returns a
//!   [`VoxelInspectorSummary`] for a picked position, or `None` for empty
//!   space (a "hole" with no registered material cell).
//!
//! A renderer (Bevy, egui, web, …) populates the index from whatever CA
//! state it currently has — typically a thin subscriber on `civ-voxel` or
//! `civ-laws` — and then calls [`VoxelInspector::summary`] for each pick.
//! The values it returns *are* the inspector panel fields, matching the
//! acceptance contract of FR-CIV-INSPECT-903.
//!
//! # Resolution contract
//!
//! - [`VoxelInspector::summary`] takes a picked position and returns
//!   `Some(VoxelInspectorSummary)` when a material cell is registered at
//!   that key, or `None` when the position is empty (unregistered /
//!   outside CA bounds).
//! - Mass is **per-cell material mass**, in a deterministic scalar unit
//!   (see [`VoxelState::mass_units`]). Phase is tagged, not a stringly-
//!   typed enum-keyed-by-convention; the renderer can render it however
//!   it wants, but a [`Phase::name`] helper is provided.
//! - Temperature/pressure units are intentionally simple scalars
//!   (Kelvin / hPa, see field docs) so the inspector can display them
//!   without a unit-conversion round-trip.
//!
//! # Additive contract
//!
//! This module is **additive** with respect to the rest of the crate and to
//! the rest of the workspace. It only adds types and queries; it never
//! mutates existing logic, never touches any other crate, never depends on
//! Bevy, and never replaces the existing [`crate::inspect`] voxel fields.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

/// Stable, hashable identifier for a single voxel cell.
///
/// `VoxelKey` is intentionally a separate type from
/// [`crate::inspect::WorldPos`] so this module is independent of the
/// generic inspect plumbing: a renderer that only renders the voxel
/// inspector does not have to construct a `WorldPos`. The two types are
/// trivially convertible (`From` impls below) so callers that already
/// work in `WorldPos` can pass picks in directly.
///
/// Coordinates are voxel-space integers (column / row / layer), matching
/// the dense world storage used everywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VoxelKey {
    /// X axis (voxel column).
    pub x: i32,
    /// Y axis (voxel row).
    pub y: i32,
    /// Z axis (voxel layer — vertical).
    pub z: i32,
}

impl VoxelKey {
    /// Construct a `VoxelKey`. `const` so test data and `static` indexes
    /// can build keys without a runtime helper.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl From<crate::inspect::WorldPos> for VoxelKey {
    fn from(pos: crate::inspect::WorldPos) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }
}

impl From<VoxelKey> for crate::inspect::WorldPos {
    fn from(k: VoxelKey) -> Self {
        Self {
            x: k.x,
            y: k.y,
            z: k.z,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

/// Discrete phase of a voxel cell.
///
/// Tagged, not a `String`, so the inspector cannot accidentally compare
/// "liquid" against "Liquid" across crates. Each variant has a stable
/// lower-case [`name`](Self::name) for display, and an [`index`](Self::index)
/// for sortability and for tests that want a stable ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Solid phase (e.g. rock, ice, wood).
    Solid,
    /// Liquid phase (e.g. water, magma).
    Liquid,
    /// Gaseous phase (e.g. air, steam, smoke).
    Gas,
    /// Plasma / exotic (anything that doesn't fit solid/liquid/gas).
    Plasma,
}

impl Phase {
    /// All phases in canonical (ascending) order.
    pub const ALL: [Phase; 4] = [Phase::Solid, Phase::Liquid, Phase::Gas, Phase::Plasma];

    /// Stable lower-case display name for the phase (e.g. `"solid"`).
    pub const fn name(self) -> &'static str {
        match self {
            Phase::Solid => "solid",
            Phase::Liquid => "liquid",
            Phase::Gas => "gas",
            Phase::Plasma => "plasma",
        }
    }

    /// Stable integer index (Solid=0, Liquid=1, Gas=2, Plasma=3).
    /// Useful for sort/filter UI and tests.
    pub const fn index(self) -> u8 {
        match self {
            Phase::Solid => 0,
            Phase::Liquid => 1,
            Phase::Gas => 2,
            Phase::Plasma => 3,
        }
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::Solid
    }
}

// ---------------------------------------------------------------------------
// VoxelState
// ---------------------------------------------------------------------------

/// Per-cell material + thermodynamic state.
///
/// This is the *raw* CA-style state the inspector displays — exactly the
/// shape FR-CIV-INSPECT-903 names: `material`, `temperature`, `pressure`,
/// `mass`, `phase`. A subscriber on `civ-voxel` / `civ-laws` produces a
/// `VoxelState` per touched cell; the inspector then exposes them.
///
/// # Field units (deliberate)
///
/// The quantities are stored as plain scalars in the most common
/// engineering unit for an inspector panel:
///
/// - `material` — canonical material identifier (e.g. `"water"`,
///   `"basalt"`, `"oak"`). Stored as `String` for forward-compatibility
///   with modded material ids.
/// - `temperature_kelvin` — Kelvin. Panel display can convert to °C.
/// - `pressure_hpa` — hectopascals (matches the existing
///   `VoxelSummary::pressure` in [`crate::inspect`]). Panel display can
///   convert to kPa / atm / psi.
/// - `mass_units` — per-cell material mass in a deterministic scalar
///   unit ("mass units"). Avoids carrying a unit enum at this stage;
///   the panel picks whatever display unit the player prefers.
/// - `phase` — tagged [`Phase`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoxelState {
    /// Stable identifier of the cell this state belongs to.
    pub key: VoxelKey,
    /// Canonical material identifier (`"water"`, `"basalt"`, …).
    pub material: String,
    /// Temperature in Kelvin.
    pub temperature_kelvin: i32,
    /// Pressure in hectopascals (hPa).
    pub pressure_hpa: i32,
    /// Per-cell mass, in CA mass units.
    pub mass_units: u32,
    /// Discrete phase of the cell.
    pub phase: Phase,
}

impl VoxelState {
    /// Construct a `VoxelState`. Provided for ergonomic test/seed data.
    pub fn new(
        key: VoxelKey,
        material: &'static str,
        temperature_kelvin: i32,
        pressure_hpa: i32,
        mass_units: u32,
        phase: Phase,
    ) -> Self {
        Self {
            key,
            material: String::from(material),
            temperature_kelvin,
            pressure_hpa,
            mass_units,
            phase,
        }
    }

    /// One-line description for tooltips and the panel header — preserves
    /// the same shape used by [`crate::inspect::VoxelSummary::one_line`]
    /// so the renderer can stay uniform across the two layers.
    pub fn one_line(&self) -> String {
        format!(
            "{} @ {}K, {}hPa (m={}u, {})",
            self.material,
            self.temperature_kelvin,
            self.pressure_hpa,
            self.mass_units,
            self.phase.name(),
        )
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

/// Resolved inspector fields for a picked voxel — the FR-CIV-INSPECT-903
/// panel payload.
///
/// This is a *display-facing* view of a [`VoxelState`]; it carries exactly
/// the five fields the FR names (material, temperature, pressure, mass,
/// phase) plus the [`VoxelKey`] so the panel header can show the picked
/// coordinate. We do not include any derived/CA-side fields here —
/// derived metrics (energy, density, conductivity) belong to a separate
/// inspector extension if the FR ever grows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoxelInspectorSummary {
    /// Picked cell.
    pub key: VoxelKey,
    /// Canonical material identifier.
    pub material: String,
    /// Temperature in Kelvin.
    pub temperature_kelvin: i32,
    /// Pressure in hectopascals (hPa).
    pub pressure_hpa: i32,
    /// Per-cell mass in CA mass units.
    pub mass_units: u32,
    /// Discrete phase of the cell.
    pub phase: Phase,
}

impl VoxelInspectorSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        format!(
            "{} @ {}K, {}hPa (m={}u, {})",
            self.material,
            self.temperature_kelvin,
            self.pressure_hpa,
            self.mass_units,
            self.phase.name(),
        )
    }

    /// Borrow the underlying phase as a stable lower-case name. Mirrors
    /// [`Phase::name`] at the summary level for renderer convenience.
    pub fn phase_name(&self) -> &'static str {
        self.phase.name()
    }
}

impl From<VoxelState> for VoxelInspectorSummary {
    fn from(s: VoxelState) -> Self {
        Self {
            key: s.key,
            material: s.material,
            temperature_kelvin: s.temperature_kelvin,
            pressure_hpa: s.pressure_hpa,
            mass_units: s.mass_units,
            phase: s.phase,
        }
    }
}

// ---------------------------------------------------------------------------
// Index
// ---------------------------------------------------------------------------

/// In-memory index of voxel/material states keyed by [`VoxelKey`].
///
/// Backed by a single `BTreeMap` so `summary(key)` is O(log n) and the
/// index is trivially cloneable (the substrate rebuilds it cheaply when
/// a CA tick changes the world).
///
/// Populating the index is the renderer's / substrate subscriber's job —
/// this struct only owns the storage and the lookup. It does not own any
/// CA evolution logic; FR-CIV-INSPECT-903 is data-and-query only.
#[derive(Debug, Clone, Default)]
pub struct VoxelInspectorIndex {
    cells: BTreeMap<VoxelKey, VoxelState>,
}

impl VoxelInspectorIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace) a voxel cell without reporting the previous state.
    ///
    /// This is the bulk-load path used when the substrate rebuilds the index.
    /// Use [`VoxelInspector::set`] when a caller needs the replaced state.
    pub fn insert(&mut self, cell: VoxelState) -> Option<VoxelState> {
        self.cells.insert(cell.key, cell);
        None
    }

    /// Remove a voxel cell. Returns the removed state, if any.
    pub fn remove(&mut self, key: VoxelKey) -> Option<VoxelState> {
        self.cells.remove(&key)
    }

    /// Look up a voxel cell by key.
    pub fn get(&self, key: VoxelKey) -> Option<&VoxelState> {
        self.cells.get(&key)
    }

    /// Number of voxel cells in the index.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// True if no voxel cells are registered.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Iterate all `(key, state)` pairs in key (i.e. coordinate) order.
    /// Coordinates are visited in lexicographic `(x, y, z)` order.
    pub fn iter(&self) -> impl Iterator<Item = (VoxelKey, &VoxelState)> + '_ {
        self.cells.iter().map(|(k, v)| (*k, v))
    }
}

// ---------------------------------------------------------------------------
// Registry / query
// ---------------------------------------------------------------------------

/// Front door for the voxel/material inspector.
///
/// Typical use:
///
/// ```text
/// // substrate tick:
/// inspector.set(VoxelState { key, material, ... });
///
/// // pick handler:
/// if let Some(summary) = inspector.summary(picked_key) {
///     render_panel(&summary);
/// }
/// ```
///
/// `summary` is the single acceptance-test query: it takes a picked
/// `VoxelKey` and returns the FR-CIV-INSPECT-903 panel payload, or
/// `None` if no material cell is registered at that key (empty space).
#[derive(Debug, Clone, Default)]
pub struct VoxelInspector {
    index: VoxelInspectorIndex,
}

impl VoxelInspector {
    /// Construct an inspector with no cells.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an inspector pre-populated with the given index.
    pub fn from_index(index: VoxelInspectorIndex) -> Self {
        Self { index }
    }

    // --- mutable state setters ---

    /// Insert (or replace) the state for a single voxel cell.
    pub fn set(&mut self, cell: VoxelState) -> Option<VoxelState> {
        self.index.cells.insert(cell.key, cell)
    }

    /// Remove a voxel cell from the index.
    pub fn clear_cell(&mut self, key: VoxelKey) -> Option<VoxelState> {
        self.index.remove(key)
    }

    /// Mutable handle to the underlying index (bulk rebuilds, tests).
    pub fn index_mut(&mut self) -> &mut VoxelInspectorIndex {
        &mut self.index
    }

    // --- read-only accessors ---

    /// Borrow the underlying index.
    pub fn index(&self) -> &VoxelInspectorIndex {
        &self.index
    }

    /// Number of voxel cells currently tracked.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// True if no cells are tracked.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    // --- core query (the acceptance-test surface) ---

    /// FR-CIV-INSPECT-903 acceptance: given a picked voxel key, return
    /// the voxel's material + thermo fields as a
    /// [`VoxelInspectorSummary`], or `None` if no cell is registered at
    /// that key (the pick landed in empty space / outside CA bounds).
    ///
    /// The returned summary carries exactly the fields the FR names —
    /// `material`, `temperature`, `pressure`, `mass`, `phase` — so the
    /// acceptance test can assert field-for-field equality against the
    /// stored [`VoxelState`].
    pub fn summary<K: Into<VoxelKey>>(&self, picked: K) -> Option<VoxelInspectorSummary> {
        let key: VoxelKey = picked.into();
        self.index
            .get(key)
            .cloned()
            .map(VoxelInspectorSummary::from)
    }

    // --- ergonomics: `WorldPos` passthrough ---

    /// Same as [`summary`](Self::summary) but accepting a
    /// [`crate::inspect::WorldPos`] directly, so callers that already
    /// work in world coordinates do not have to convert.
    pub fn summary_at_pos(
        &self,
        picked: crate::inspect::WorldPos,
    ) -> Option<VoxelInspectorSummary> {
        self.summary(VoxelKey::from(picked))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn water_cell() -> VoxelState {
        VoxelState {
            key: VoxelKey::new(0, 0, 0),
            material: "water".into(),
            temperature_kelvin: 293,
            pressure_hpa: 1013,
            mass_units: 1_000,
            phase: Phase::Liquid,
        }
    }

    fn basalt_cell() -> VoxelState {
        VoxelState {
            key: VoxelKey::new(1, 0, 0),
            material: "basalt".into(),
            temperature_kelvin: 1_200,
            pressure_hpa: 1013,
            mass_units: 2_700,
            phase: Phase::Solid,
        }
    }

    fn steam_cell() -> VoxelState {
        VoxelState {
            key: VoxelKey::new(2, 0, 0),
            material: "steam".into(),
            temperature_kelvin: 400,
            pressure_hpa: 1_013,
            mass_units: 0,
            phase: Phase::Gas,
        }
    }

    // --- key + phase primitives ---

    #[test]
    fn voxel_key_construction_and_equality() {
        let a = VoxelKey::new(3, 4, 5);
        let b = VoxelKey::new(3, 4, 5);
        let c = VoxelKey::new(5, 4, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.x, 3);
        assert_eq!(a.y, 4);
        assert_eq!(a.z, 5);
    }

    #[test]
    fn voxel_key_ordering_is_lexicographic() {
        let mut keys = [
            VoxelKey::new(0, 0, 1),
            VoxelKey::new(0, 0, 0),
            VoxelKey::new(1, 0, 0),
            VoxelKey::new(0, 1, 0),
        ];
        keys.sort();
        assert_eq!(
            keys,
            [
                VoxelKey::new(0, 0, 0),
                VoxelKey::new(0, 0, 1),
                VoxelKey::new(0, 1, 0),
                VoxelKey::new(1, 0, 0),
            ],
        );
    }

    #[test]
    fn phase_name_and_index_are_stable() {
        assert_eq!(Phase::Solid.name(), "solid");
        assert_eq!(Phase::Liquid.name(), "liquid");
        assert_eq!(Phase::Gas.name(), "gas");
        assert_eq!(Phase::Plasma.name(), "plasma");

        assert_eq!(Phase::Solid.index(), 0);
        assert_eq!(Phase::Liquid.index(), 1);
        assert_eq!(Phase::Gas.index(), 2);
        assert_eq!(Phase::Plasma.index(), 3);
    }

    #[test]
    fn phase_default_is_solid() {
        assert_eq!(Phase::default(), Phase::Solid);
    }

    // --- state + summary ---

    #[test]
    fn voxel_state_one_line_renders_all_fields() {
        let s = water_cell();
        let line = s.one_line();
        assert!(line.contains("water"), "{line}");
        assert!(line.contains("293"), "{line}");
        assert!(line.contains("1013"), "{line}");
        assert!(line.contains("1000"), "{line}");
        assert!(line.contains("liquid"), "{line}");
    }

    #[test]
    fn voxel_state_into_summary_preserves_every_field() {
        let s = water_cell();
        let summary: VoxelInspectorSummary = s.clone().into();
        assert_eq!(summary.key, s.key);
        assert_eq!(summary.material, s.material);
        assert_eq!(summary.temperature_kelvin, s.temperature_kelvin);
        assert_eq!(summary.pressure_hpa, s.pressure_hpa);
        assert_eq!(summary.mass_units, s.mass_units);
        assert_eq!(summary.phase, s.phase);
        assert_eq!(summary.phase_name(), "liquid");
    }

    #[test]
    fn summary_one_line_renders_all_fields() {
        let s: VoxelInspectorSummary = basalt_cell().into();
        let line = s.one_line();
        assert!(line.contains("basalt"), "{line}");
        assert!(line.contains("1200"), "{line}");
        assert!(line.contains("1013"), "{line}");
        assert!(line.contains("2700"), "{line}");
        assert!(line.contains("solid"), "{line}");
    }

    // --- index ---

    #[test]
    fn index_insert_and_get_replaces() {
        let mut idx = VoxelInspectorIndex::new();
        assert!(idx.is_empty());

        assert!(idx.insert(water_cell()).is_none());
        assert_eq!(idx.len(), 1);

        let prev = idx.insert(VoxelState {
            key: VoxelKey::new(0, 0, 0),
            material: "ice".into(),
            ..VoxelState {
                key: VoxelKey::new(99, 99, 99),
                material: "junk".into(),
                ..water_cell()
            }
        });
        assert!(prev.is_none(), "full replacement of an existing cell");
    }

    #[test]
    fn index_iterates_in_lexicographic_key_order() {
        let mut idx = VoxelInspectorIndex::new();
        idx.insert(water_cell()); // (0,0,0)
        idx.insert(basalt_cell()); // (1,0,0)
        idx.insert(steam_cell()); // (2,0,0)

        let order: Vec<VoxelKey> = idx.iter().map(|(k, _)| k).collect();
        assert_eq!(
            order,
            [
                VoxelKey::new(0, 0, 0),
                VoxelKey::new(1, 0, 0),
                VoxelKey::new(2, 0, 0),
            ],
        );
    }

    #[test]
    fn index_remove_returns_previous_state() {
        let mut idx = VoxelInspectorIndex::new();
        let cell = water_cell();
        idx.insert(cell.clone());
        let removed = idx.remove(VoxelKey::new(0, 0, 0));
        assert_eq!(removed, Some(cell));
        assert!(idx.get(VoxelKey::new(0, 0, 0)).is_none());
        assert!(idx.is_empty());
    }

    // --- inspector query (the FR-CIV-INSPECT-903 surface) ---

    fn populated_inspector() -> VoxelInspector {
        let mut insp = VoxelInspector::new();
        insp.set(water_cell());
        insp.set(basalt_cell());
        insp.set(steam_cell());
        insp
    }

    /// Acceptance test for FR-CIV-INSPECT-903: "summary returns the
    /// voxel's material+thermo fields".
    ///
    /// For each registered cell, the inspector's `summary(picked)` call
    /// returns a payload whose material, temperature, pressure, mass,
    /// and phase match the stored state field-for-field.
    #[test]
    fn fr_civ_inspect_903_summary_returns_voxel_material_and_thermo_fields() {
        let insp = populated_inspector();

        // water
        let s = insp
            .summary(VoxelKey::new(0, 0, 0))
            .expect("water cell registered");
        assert_eq!(s.key, VoxelKey::new(0, 0, 0));
        assert_eq!(s.material, "water");
        assert_eq!(s.temperature_kelvin, 293);
        assert_eq!(s.pressure_hpa, 1013);
        assert_eq!(s.mass_units, 1_000);
        assert_eq!(s.phase, Phase::Liquid);
        assert_eq!(s.phase_name(), "liquid");

        // basalt
        let s = insp
            .summary(VoxelKey::new(1, 0, 0))
            .expect("basalt cell registered");
        assert_eq!(s.material, "basalt");
        assert_eq!(s.temperature_kelvin, 1_200);
        assert_eq!(s.pressure_hpa, 1013);
        assert_eq!(s.mass_units, 2_700);
        assert_eq!(s.phase, Phase::Solid);

        // steam (gas phase + zero mass is fine)
        let s = insp
            .summary(VoxelKey::new(2, 0, 0))
            .expect("steam cell registered");
        assert_eq!(s.material, "steam");
        assert_eq!(s.phase, Phase::Gas);
        assert_eq!(s.mass_units, 0);
    }

    /// Empty space / unregistered key → `None`.
    #[test]
    fn fr_civ_inspect_903_empty_space_returns_none() {
        let insp = populated_inspector();
        assert!(insp.summary(VoxelKey::new(999, 999, 999)).is_none());
    }

    /// Empty inspector → every pick is `None`.
    #[test]
    fn empty_inspector_returns_none_for_any_pick() {
        let insp = VoxelInspector::new();
        assert!(insp.summary(VoxelKey::new(0, 0, 0)).is_none());
        assert!(insp.summary(VoxelKey::new(1, 2, 3)).is_none());
        assert!(insp.is_empty());
    }

    /// `set` replaces the state at an existing key (CA tick update).
    #[test]
    fn set_replaces_existing_cell_state() {
        let mut insp = VoxelInspector::new();
        insp.set(water_cell());
        // Same key, hotter water.
        let hotter = VoxelState {
            key: VoxelKey::new(0, 0, 0),
            material: "water".into(),
            temperature_kelvin: 373,
            pressure_hpa: 1013,
            mass_units: 1_000,
            phase: Phase::Liquid,
        };
        let prev = insp.set(hotter.clone());
        assert!(prev.is_some(), "must report the replaced state");
        let s = insp
            .summary(VoxelKey::new(0, 0, 0))
            .expect("cell still registered");
        assert_eq!(s.temperature_kelvin, 373);
    }

    /// `clear_cell` removes a cell, after which the pick returns `None`.
    #[test]
    fn clear_cell_removes_a_picked_cell() {
        let mut insp = populated_inspector();
        let prev = insp
            .clear_cell(VoxelKey::new(0, 0, 0))
            .expect("water cell existed");
        assert_eq!(prev.material, "water");
        assert!(insp.summary(VoxelKey::new(0, 0, 0)).is_none());
        // Other cells are still reachable.
        assert!(insp.summary(VoxelKey::new(1, 0, 0)).is_some());
        assert!(insp.summary(VoxelKey::new(2, 0, 0)).is_some());
    }

    /// `WorldPos` passthrough works without callers having to convert.
    #[test]
    fn summary_at_pos_accepts_world_pick_directly() {
        let insp = populated_inspector();
        let s = insp
            .summary_at_pos(crate::inspect::WorldPos::new(1, 0, 0))
            .expect("basalt registered at (1,0,0)");
        assert_eq!(s.material, "basalt");
    }

    /// Bidirectional `VoxelKey` ↔ `WorldPos` conversion is exact.
    #[test]
    fn voxel_key_world_pos_conversion_round_trips() {
        let wp = crate::inspect::WorldPos::new(3, 4, 5);
        let vk: VoxelKey = wp.into();
        assert_eq!(vk, VoxelKey::new(3, 4, 5));
        let wp2: crate::inspect::WorldPos = vk.into();
        assert_eq!(wp2, wp);
    }

    /// Summary is independent of underlying state — cloning the state
    /// out of the inspector does not invalidate subsequent queries.
    #[test]
    fn summary_is_value_typed_and_does_not_borrow() {
        let insp = populated_inspector();
        let s1 = insp.summary(VoxelKey::new(0, 0, 0)).unwrap();
        // Drop s1, query again — should still work and match.
        drop(s1);
        let s2 = insp.summary(VoxelKey::new(0, 0, 0)).unwrap();
        assert_eq!(s2.material, "water");
    }
}
