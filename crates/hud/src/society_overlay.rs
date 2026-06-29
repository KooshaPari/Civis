//! FR-CIV-INFOVIEW-913 — Society overlay data sources.
//!
//! Per [`docs/specs/requirements/FR-CIV-INFOVIEW.md`] §913 and the
//! priority-12 rank-1 overlay catalog in [`docs/design/info-views.md`]
//! §3, this module provides the **pure data / query backing layer** for
//! the CS2-class *society* overlays:
//!
//! * **Ideology / culture clusters** — emergent ideology-tagging per agent
//!   aggregated into a dominant-cluster *id* per cell (categorical
//!   scalar, normalised to `0.0..=1.0` for the legend ramp).
//! * **Language / dialect regions** — emergent language-tagging per
//!   agent aggregated into a dominant-language region id per cell
//!   (categorical scalar, normalised to `0.0..=1.0`).
//! * **Kinship / contact density** — emergent per-cell density of social
//!   ties (kinship edges + non-kin contacts) per cell, normalised to
//!   `0.0..=1.0` against the configured saturation. This is the
//!   *continuous* society field (the FR calls it the overlap field).
//!
//! Design contract (charter: emergence, not invented categories):
//!
//! 1. **One data source = one normalized scalar per cell.** Every
//!    [`SocietyDataSource::sample`] returns a value in `0.0..=1.0`
//!    that *tracks* the underlying raw field. The acceptance test is
//!    encoded as [`SocietyDataSource::dominant_cluster_id`] (dominant
//!    cluster id, e.g. `0`, `1`, `2`) — the panel-hover / inspector path
//!    the four clients (web / Bevy / Godot / Unreal) read.
//! 2. **Raw fields are substrate-owned.** The producing crate
//!    (`civ-agents` for the social graph + emergent culture tagging) is
//!    the canonical home of the raw `f32` / `u32` (no `faction:u32`
//!    stand-in — the FR explicitly forbids that substitution). This
//!    module is read-only — it copies raw values into its registry and
//!    normalises on query.
//! 3. **Pure-logic, no engine.** No Bevy, no rendering, no systems.
//!    The renderer / panel / hotkey wiring reads the normalised scalars
//!    via the [`SocietyOverlayRegistry::query`] and
//!    [`SocietyOverlayRegistry::query_raw`] accessors and feeds them
//!    into the existing `InfoOverlay` colour functions / ramps. This
//!    module is the substrate-neutral backing store the four clients
//!    all bind to.
//! 4. **Additive only.** This module is *additive* to `civ-hud`; it
//!    does not touch any other crate and does not modify any existing
//!    public surface in `lib.rs`. The two `mod` / `pub use` lines in
//!    `lib.rs` are the only `lib.rs` diffs.
//!
//! The registry is intentionally a plain `HashMap`-backed store (no
//! engine features, no globals) so unit tests can build a small
//! per-cell field and assert that the normalised scalar tracks the raw
//! field at every cell. This is the acceptance test
//! (`society_overlay_returns_dominant_cluster_id_per_cell`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Identifier for a cell in the simulation lattice. Lattice coordinates
/// are signed integers (substrate-neutral — the producing crate decides
/// the origin / extent). Mirrors the `(i32, i32, i32)` cell tuple used
/// by `TileInspector` in `tile_inspector.rs` and by `env_overlay.rs` so
/// the four clients (web / Bevy / Godot / Unreal) can pass a probed
/// cell through unchanged.
pub type CellId = (i32, i32, i32);

/// Normalized scalar in `0.0..=1.0` produced by a [`SocietyDataSource`]
/// for the legend ramp.
///
/// The contract is *tracking*, not *interpretation*: the scalar must
/// rise when the underlying raw field rises (and fall when it falls),
/// saturating to the configured min/max so the legend ramp is stable
/// across snapshots.
///
/// Categorical overlays (ideology, language) project their dominant
/// cluster id into this same range — the legend ramp is a colour
/// gradient, but the *hover text* / inspector path reads the raw
/// `dominant_cluster_id` (see the acceptance test).
pub type NormalizedScalar = f32;

/// Categorical cluster id (ideology tag, language tag). `u32` so the
/// producing crate can use any stable hash of the emergent tag name;
/// `0` is reserved as "no-data / missing tag" (matches the
/// `TileInspector::CELL_NONE` convention).
///
/// The FR-CIV-INFOVIEW-913 acceptance test reads this id directly off
/// the data source via [`SocietyDataSource::dominant_cluster_id`].
pub type ClusterId = u32;

/// Reserved "no-data" cluster id (`0`). Mirrors
/// `TileInspector::CELL_NONE` so the four clients can pattern-match on
/// missing cells without inventing a separate sentinel.
pub const CLUSTER_NONE: ClusterId = 0;

/// Raw, substrate-owned field value. Semantics differ per data source
/// (count for kinship, id for cluster fields) — callers that need the
/// substrate units use [`SocietyOverlayRegistry::query_raw`].
pub type RawScalar = f32;

// ---------------------------------------------------------------------------
// Data-source identity.
// ---------------------------------------------------------------------------

/// Machine id for a society data source. One variant per
/// FR-CIV-INFOVIEW-913 row in the overlay catalog.
///
/// Note: "polity-membership cluster overlap" is modelled as a single
/// derived continuous field ([`SocietyDataSourceId::PolityOverlap`])
/// because the FR describes it as the *overlap* of clusters (a
/// continuous density), not as a categorical cluster id of its own.
/// See `docs/design/info-views.md` §3 for the catalog split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocietyDataSourceId {
    /// Dominant ideology / culture cluster id per cell (categorical).
    Ideology,
    /// Dominant language / dialect region id per cell (categorical).
    Language,
    /// Kinship / contact density per cell (continuous).
    Kinship,
    /// Polity-membership cluster overlap per cell (continuous; FR's
    /// "overlap visualized as continuous field").
    PolityOverlap,
}

impl SocietyDataSourceId {
    /// Stable string id (e.g. `"society.ideology"`). Used by the four
    /// clients to bind the panel / hotkey groups without re-deriving
    /// the label.
    #[must_use]
    pub fn stable_id(self) -> &'static str {
        match self {
            SocietyDataSourceId::Ideology => "society.ideology",
            SocietyDataSourceId::Language => "society.language",
            SocietyDataSourceId::Kinship => "society.kinship",
            SocietyDataSourceId::PolityOverlap => "society.polity_overlap",
        }
    }

    /// Display label for the registry / panel.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SocietyDataSourceId::Ideology => "Ideology / culture clusters",
            SocietyDataSourceId::Language => "Language / dialect regions",
            SocietyDataSourceId::Kinship => "Kinship / contact density",
            SocietyDataSourceId::PolityOverlap => "Polity-membership overlap",
        }
    }

    /// All four variants — for iteration in the registry / legend panel.
    #[must_use]
    pub const fn all() -> [SocietyDataSourceId; 4] {
        [
            SocietyDataSourceId::Ideology,
            SocietyDataSourceId::Language,
            SocietyDataSourceId::Kinship,
            SocietyDataSourceId::PolityOverlap,
        ]
    }
}

// ---------------------------------------------------------------------------
// Per-source field store (raw → normalized).
// ---------------------------------------------------------------------------

/// Categorical cluster field — one entry per cell storing the
/// dominant cluster id at that cell (ideology tag or language tag).
///
/// The raw value is a `ClusterId`; the normalised scalar (`0.0..=1.0`)
/// is `id / max_known_id` (saturating clamp). This lets the legend
/// ramp render a stable colour gradient even as the producing crate
/// adds new clusters — `max_known_id` advances monotonically and the
/// colour ramp spans `[0.0, 1.0]`.
///
/// The FR's "clusters NOT from `faction:u32`" caveat is enforced at the
/// contract level: this module is *not* a faction registry, it is a
/// *culture / language* registry. The two are deliberately split so the
/// producing crate (`civ-agents`) cannot accidentally bind a polity
/// `faction:u32` here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterField {
    /// Raw cluster id per cell. Missing cells are
    /// [`CLUSTER_NONE`] (no-data).
    pub cluster: HashMap<CellId, ClusterId>,
    /// Largest known cluster id observed so far. Used to normalise the
    /// raw id into `0.0..=1.0` so the colour ramp is stable as new
    /// clusters emerge. Defaults to `1` so `CLUSTER_NONE → 0.0` and the
    /// first real cluster id (`1`) maps to `1.0`. The producing crate
    /// MUST bump `max_known_id` whenever a new cluster id is observed;
    /// [`Self::observe`] does this automatically.
    pub max_known_id: ClusterId,
}

impl Default for ClusterField {
    fn default() -> Self {
        Self {
            cluster: HashMap::new(),
            max_known_id: 1,
        }
    }
}

impl ClusterField {
    /// Build an empty cluster field with the canonical default
    /// (`max_known_id = 1`; first real cluster id `1` maps to `1.0`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cluster id at `cell`. If the id exceeds the current
    /// `max_known_id`, advance `max_known_id` so the normalised scalar
    /// stays in `0.0..=1.0` for every observed id.
    pub fn observe(&mut self, cell: CellId, id: ClusterId) {
        if id > self.max_known_id {
            self.max_known_id = id;
        }
        self.cluster.insert(cell, id);
    }

    /// Read the cluster id at `cell`. Missing cells return
    /// [`CLUSTER_NONE`].
    #[must_use]
    pub fn dominant_cluster_id(&self, cell: CellId) -> ClusterId {
        self.cluster.get(&cell).copied().unwrap_or(CLUSTER_NONE)
    }

    /// Override `max_known_id` (substrate-specific). Must be `>= 1`.
    pub fn set_max_known_id(&mut self, max: ClusterId) {
        if max >= 1 {
            self.max_known_id = max;
        }
    }

    /// Normalise a cluster id to `0.0..=1.0`. `CLUSTER_NONE` (`0`)
    /// maps to `0.0`; ids `> max_known_id` saturate to `1.0`.
    #[must_use]
    pub fn normalize(&self, id: ClusterId) -> NormalizedScalar {
        if id == CLUSTER_NONE {
            return 0.0;
        }
        let span = self.max_known_id.max(1) as f32;
        (id as f32 / span).clamp(0.0, 1.0)
    }

    /// Normalised scalar at `cell` (legend-ramp path). Returns
    /// `0.0` for missing cells.
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.dominant_cluster_id(cell))
    }
}

/// Continuous society field — one entry per cell storing a raw
/// `f32` (kinship count / contact count / polity-overlap density).
///
/// Missing cells default to `0.0` (no ties). The normalised scalar is
/// `raw / max_raw` (saturating clamp), used by the legend ramp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityField {
    /// Raw per-cell density (units depend on the data source — number
    /// of social ties for kinship, fractional overlap for polity).
    pub raw: HashMap<CellId, RawScalar>,
    /// `min` raw (no ties / no overlap). Defaults to `0.0`.
    pub min_raw: RawScalar,
    /// `max` raw (saturating dense). Defaults to `1.0` so the field is
    /// already in `0.0..=1.0`; the producing crate may override (e.g.
    /// a census count).
    pub max_raw: RawScalar,
}

impl Default for DensityField {
    fn default() -> Self {
        Self {
            raw: HashMap::new(),
            min_raw: 0.0,
            max_raw: 1.0,
        }
    }
}

impl DensityField {
    /// Build a density field with the canonical `0.0..=1.0` saturation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the saturation (substrate-specific). `max_raw` must
    /// exceed `min_raw`; if not, the field falls back to the canonical
    /// `0.0..=1.0` to avoid divide-by-zero in [`Self::normalize`].
    pub fn set_saturation(&mut self, min: RawScalar, max: RawScalar) {
        if max > min {
            self.min_raw = min;
            self.max_raw = max;
        }
    }

    /// Push a raw sample. Missing cells stay at the clean baseline.
    pub fn insert(&mut self, cell: CellId, value: RawScalar) {
        self.raw.insert(cell, value);
    }

    /// Read the raw value at `cell`. Missing cells return `min_raw`.
    #[must_use]
    pub fn get_raw(&self, cell: CellId) -> RawScalar {
        self.raw.get(&cell).copied().unwrap_or(self.min_raw)
    }

    /// Normalize a raw value to `0.0..=1.0` using the configured
    /// saturation. Saturating clamp — values outside `[min_raw, max_raw]`
    /// pin to the nearest endpoint (never panic, never wrap).
    #[must_use]
    pub fn normalize(&self, raw: RawScalar) -> NormalizedScalar {
        let span = (self.max_raw - self.min_raw).max(f32::EPSILON);
        ((raw - self.min_raw) / span).clamp(0.0, 1.0)
    }

    /// Normalised scalar at `cell` (legend-ramp path).
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.get_raw(cell))
    }
}

// ---------------------------------------------------------------------------
// Registry — the four clients bind here.
// ---------------------------------------------------------------------------

/// Combined registry of all society data sources. Owns one
/// [`ClusterField`] per categorical source (ideology, language) and
/// one [`DensityField`] per continuous source (kinship, polity
/// overlap). Missing sources are valid (an empty registry returns
/// `0.0` / `CLUSTER_NONE` for any query — the four clients treat the
/// overlay as "data not yet populated").
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SocietyOverlayRegistry {
    /// Dominant ideology / culture cluster field.
    pub ideology: ClusterField,
    /// Dominant language / dialect region field.
    pub language: ClusterField,
    /// Kinship / contact density field.
    pub kinship: DensityField,
    /// Polity-membership cluster overlap field (continuous).
    pub polity_overlap: DensityField,
}

impl SocietyOverlayRegistry {
    /// Build an empty registry. All sources default to "no-data" /
    /// clean baseline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalised scalar for `source` at `cell` across any of the four
    /// [`SocietyDataSourceId`] variants. Returns `Some(scalar)` for
    /// populated sources and `None` only when the *id* is unknown
    /// (which today cannot happen — all four ids map to owned fields;
    /// the `Option` is forward-compatible with future sources).
    #[must_use]
    pub fn query(
        &self,
        source: SocietyDataSourceId,
        cell: CellId,
    ) -> Option<NormalizedScalar> {
        match source {
            SocietyDataSourceId::Ideology => Some(self.ideology_sample(cell)),
            SocietyDataSourceId::Language => Some(self.language_sample(cell)),
            SocietyDataSourceId::Kinship => Some(self.kinship_sample(cell)),
            SocietyDataSourceId::PolityOverlap => Some(self.polity_overlap_sample(cell)),
        }
    }

    /// Raw, substrate-owned value for `source` at `cell`. Used by the
    /// panel hover-text / inspector path to display unit-bearing
    /// values alongside the legend ramp position. Categorical sources
    /// return the [`ClusterId`] as `f32`; callers that need the id
    /// directly use the typed accessors
    /// ([`Self::ideology_cluster_id`] / [`Self::language_cluster_id`]).
    #[must_use]
    pub fn query_raw(
        &self,
        source: SocietyDataSourceId,
        cell: CellId,
    ) -> Option<RawScalar> {
        match source {
            SocietyDataSourceId::Ideology => Some(self.ideology_cluster_id(cell) as RawScalar),
            SocietyDataSourceId::Language => Some(self.language_cluster_id(cell) as RawScalar),
            SocietyDataSourceId::Kinship => Some(self.kinship.get_raw(cell)),
            SocietyDataSourceId::PolityOverlap => Some(self.polity_overlap.get_raw(cell)),
        }
    }

    /// Dominant cluster id (categorical, NOT normalised). Used by the
    /// tile inspector / hover-text path. **Acceptance test path:**
    /// FR-CIV-INFOVIEW-913 requires "data-source returns per-cell
    /// society scalar (e.g. dominant cluster id)".
    #[must_use]
    pub fn ideology_cluster_id(&self, cell: CellId) -> ClusterId {
        self.ideology.dominant_cluster_id(cell)
    }

    /// Dominant cluster id for the language overlay.
    #[must_use]
    pub fn language_cluster_id(&self, cell: CellId) -> ClusterId {
        self.language.dominant_cluster_id(cell)
    }

    /// Normalised scalar (legend ramp) for the ideology overlay.
    #[must_use]
    pub fn ideology_sample(&self, cell: CellId) -> NormalizedScalar {
        self.ideology.sample(cell)
    }

    /// Normalised scalar for the language overlay.
    #[must_use]
    pub fn language_sample(&self, cell: CellId) -> NormalizedScalar {
        self.language.sample(cell)
    }

    /// Normalised scalar for the kinship overlay.
    #[must_use]
    pub fn kinship_sample(&self, cell: CellId) -> NormalizedScalar {
        self.kinship.sample(cell)
    }

    /// Normalised scalar for the polity-overlap overlay.
    #[must_use]
    pub fn polity_overlap_sample(&self, cell: CellId) -> NormalizedScalar {
        self.polity_overlap.sample(cell)
    }

    /// Number of distinct data sources owned by this registry. Today
    /// always `4` (one per [`SocietyDataSourceId`]). Future-proof
    /// accessor so the four clients can size their panel dropdown
    /// without hard-coding the count.
    #[must_use]
    pub fn data_source_count(&self) -> usize {
        SocietyDataSourceId::all().len()
    }
}

// ---------------------------------------------------------------------------
// `SocietyDataSource` trait — the acceptance-test surface.
// ---------------------------------------------------------------------------

/// Per-cell society data source. Implemented by each of the four
/// [`SocietyDataSourceId`] variants so the four clients can treat the
/// society overlay set polymorphically. The two required methods map
/// 1:1 to the FR-CIV-INFOVIEW-913 acceptance test:
///
/// * `dominant_cluster_id(&self, cell)` — categorical scalar (used by
///   the tile inspector / hover text).
/// * `sample(&self, cell)` — `0.0..=1.0` normalised scalar (used by
///   the legend ramp / recolor pass).
///
/// Density-only sources ([`SocietyDataSourceId::Kinship`],
/// [`SocietyDataSourceId::PolityOverlap`]) return
/// [`CLUSTER_NONE`] from `dominant_cluster_id` because they have no
/// per-cell category — they surface the continuous field only.
pub trait SocietyDataSource {
    /// Dominant cluster id at `cell`. `CLUSTER_NONE` for
    /// density-only sources or unobserved cells.
    fn dominant_cluster_id(&self, cell: CellId) -> ClusterId;

    /// Normalised scalar at `cell` (`0.0..=1.0`).
    fn sample(&self, cell: CellId) -> NormalizedScalar;
}

/// Trait extension tying a [`SocietyDataSourceId`] (variant) to its
/// data source. Lets `query_dominant` dispatch on the id without
/// losing the typed surface.
pub trait SocietyDataSourceDispatch {
    /// Run `dominant_cluster_id` for the variant.
    fn query_dominant(&self, source: SocietyDataSourceId, cell: CellId) -> ClusterId;
}

impl SocietyDataSourceDispatch for SocietyOverlayRegistry {
    fn query_dominant(&self, source: SocietyDataSourceId, cell: CellId) -> ClusterId {
        match source {
            SocietyDataSourceId::Ideology => self.ideology_cluster_id(cell),
            SocietyDataSourceId::Language => self.language_cluster_id(cell),
            SocietyDataSourceId::Kinship | SocietyDataSourceId::PolityOverlap => CLUSTER_NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for FR-CIV-INFOVIEW-913.
    //!
    //! The headline test is
    //! [`society_overlay_returns_dominant_cluster_id_per_cell`] (the
    //! FR acceptance test).
    use super::*;

    fn cell(x: i32, y: i32, z: i32) -> CellId {
        (x, y, z)
    }

    /// AC: data-source returns per-cell society scalar (e.g. dominant
    /// cluster id). Two cells, two distinct cluster ids — the registry
    /// must surface each one without confusion.
    #[test]
    fn society_overlay_returns_dominant_cluster_id_per_cell() {
        let mut reg = SocietyOverlayRegistry::new();
        // Cell A → ideology cluster 7, language cluster 3.
        reg.ideology.observe(cell(0, 0, 0), 7);
        reg.language.observe(cell(0, 0, 0), 3);
        // Cell B → ideology cluster 11, language cluster 5.
        reg.ideology.observe(cell(4, 0, 0), 11);
        reg.language.observe(cell(4, 0, 0), 5);

        assert_eq!(reg.ideology_cluster_id(cell(0, 0, 0)), 7);
        assert_eq!(reg.language_cluster_id(cell(0, 0, 0)), 3);
        assert_eq!(reg.ideology_cluster_id(cell(4, 0, 0)), 11);
        assert_eq!(reg.language_cluster_id(cell(4, 0, 0)), 5);

        // Dispatcher agrees.
        assert_eq!(
            reg.query_dominant(SocietyDataSourceId::Ideology, cell(0, 0, 0)),
            7
        );
        assert_eq!(
            reg.query_dominant(SocietyDataSourceId::Language, cell(4, 0, 0)),
            5
        );
        // Unobserved cell → CLUSTER_NONE.
        assert_eq!(
            reg.query_dominant(SocietyDataSourceId::Ideology, cell(99, 99, 99)),
            CLUSTER_NONE
        );
        // Density-only sources → CLUSTER_NONE.
        assert_eq!(
            reg.query_dominant(SocietyDataSourceId::Kinship, cell(0, 0, 0)),
            CLUSTER_NONE
        );
        assert_eq!(
            reg.query_dominant(SocietyDataSourceId::PolityOverlap, cell(0, 0, 0)),
            CLUSTER_NONE
        );
    }

    /// `observe` MUST advance `max_known_id` so the normalised scalar
    /// stays in `0.0..=1.0`. A registry that observed ids `7` and
    /// `11` MUST normalise them against `max_known_id = 11`, not the
    /// default `1`.
    #[test]
    fn society_overlay_cluster_field_auto_advances_max_id() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.ideology.observe(cell(0, 0, 0), 7);
        reg.ideology.observe(cell(1, 0, 0), 11);

        // max_known_id advanced to 11 → 7 / 11 ≈ 0.636, 11 / 11 = 1.0.
        let s7 = reg.ideology_sample(cell(0, 0, 0));
        let s11 = reg.ideology_sample(cell(1, 0, 0));
        assert!(
            (s7 - 7.0 / 11.0).abs() < 1e-6,
            "id 7 / max 11 must normalise to ~0.6364, got {s7}"
        );
        assert!(
            (s11 - 1.0).abs() < 1e-6,
            "max id 11 / max 11 must saturate to 1.0, got {s11}"
        );
        // CLUSTER_NONE always maps to 0.0.
        assert!((reg.ideology_sample(cell(99, 99, 99)) - 0.0).abs() < 1e-6);
    }

    /// Categorical `ClusterField::normalize` clamps ids larger than
    /// `max_known_id` to `1.0` (defensive — `observe` already advances
    /// `max_known_id`, but a manually-set id past the max must not
    /// produce a scalar > `1.0`).
    #[test]
    fn society_overlay_cluster_field_saturates_above_max() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.ideology.observe(cell(0, 0, 0), 5);
        reg.ideology.set_max_known_id(5);
        // Manually inject a cluster id past the saturation.
        reg.ideology.cluster.insert(cell(1, 0, 0), 9);
        let s = reg.ideology_sample(cell(1, 0, 0));
        assert!(
            (s - 1.0).abs() < 1e-6,
            "id 9 against max 5 must saturate to 1.0, got {s}"
        );
    }

    /// Continuous society fields (kinship, polity overlap) default to
    /// `0.0` (no ties / no overlap) at unobserved cells.
    #[test]
    fn society_overlay_density_default_is_zero() {
        let reg = SocietyOverlayRegistry::new();
        assert!((reg.kinship_sample(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
        assert!((reg.polity_overlap_sample(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
    }

    /// Continuous society fields track the raw value — insert a raw
    /// `0.42`, the normalised scalar MUST be `0.42`.
    #[test]
    fn society_overlay_density_tracks_raw_field() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.kinship.insert(cell(0, 0, 0), 0.42);
        reg.polity_overlap.insert(cell(0, 0, 0), 0.75);
        assert!((reg.kinship_sample(cell(0, 0, 0)) - 0.42).abs() < 1e-6);
        assert!((reg.polity_overlap_sample(cell(0, 0, 0)) - 0.75).abs() < 1e-6);
    }

    /// Saturation clamp — values outside `[min_raw, max_raw]` pin to
    /// the nearest endpoint (no panic, no wrap, no scalar past
    /// `0.0..=1.0`).
    #[test]
    fn society_overlay_density_saturates_outside_range() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.kinship.set_saturation(0.0, 10.0);
        reg.kinship.insert(cell(0, 0, 0), -999.0); // below min
        reg.kinship.insert(cell(1, 0, 0), 9999.0); // above max
        let lo = reg.kinship_sample(cell(0, 0, 0));
        let hi = reg.kinship_sample(cell(1, 0, 0));
        assert!(
            (lo - 0.0).abs() < 1e-6,
            "below-min must clamp to 0.0, got {lo}"
        );
        assert!(
            (hi - 1.0).abs() < 1e-6,
            "above-max must clamp to 1.0, got {hi}"
        );
    }

    /// Substrate-specific saturation overrides the canonical range —
    /// the field re-normalises against the new range, exactly as
    /// `env_overlay` does.
    #[test]
    fn society_overlay_density_supports_substrate_saturation() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.kinship.set_saturation(10.0, 20.0);
        reg.kinship.insert(cell(0, 0, 0), 15.0); // midpoint
        let s = reg.kinship_sample(cell(0, 0, 0));
        assert!(
            (s - 0.5).abs() < 1e-6,
            "15.0 in [10, 20] must normalise to 0.5, got {s}"
        );
    }

    /// Generic `query` dispatcher returns the same value as the typed
    /// accessor (cross-check: the registry is internally consistent
    /// across access paths).
    #[test]
    fn society_overlay_query_dispatcher_matches_typed() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.kinship.insert(cell(0, 0, 0), 0.6);
        let typed = reg.kinship_sample(cell(0, 0, 0));
        let dispatched = reg
            .query(SocietyDataSourceId::Kinship, cell(0, 0, 0))
            .expect("kinship must be present");
        assert!(
            (typed - dispatched).abs() < 1e-6,
            "typed ({typed}) and query ({dispatched}) must agree"
        );
    }

    /// `query_raw` returns substrate units (cluster id as `f32` for
    /// categorical sources, raw density for continuous sources).
    #[test]
    fn society_overlay_query_raw_returns_substrate_units() {
        let mut reg = SocietyOverlayRegistry::new();
        reg.ideology.observe(cell(0, 0, 0), 7);
        reg.kinship.insert(cell(0, 0, 0), 0.42);

        // Categorical → cluster id as f32.
        let raw_idea = reg
            .query_raw(SocietyDataSourceId::Ideology, cell(0, 0, 0))
            .expect("ideology raw must be present");
        assert!(
            (raw_idea - 7.0).abs() < 1e-6,
            "ideology raw at (0,0,0) must be 7.0, got {raw_idea}"
        );

        // Continuous → raw density.
        let raw_kin = reg
            .query_raw(SocietyDataSourceId::Kinship, cell(0, 0, 0))
            .expect("kinship raw must be present");
        assert!(
            (raw_kin - 0.42).abs() < 1e-6,
            "kinship raw at (0,0,0) must be 0.42, got {raw_kin}"
        );

        // Missing cell → CLUSTER_NONE (categorical) or min_raw (density).
        let raw_missing = reg
            .query_raw(SocietyDataSourceId::Language, cell(99, 99, 99))
            .expect("language raw must be present (returns CLUSTER_NONE)");
        assert!(
            (raw_missing - CLUSTER_NONE as f32).abs() < 1e-6,
            "language raw at unobserved cell must be CLUSTER_NONE, got {raw_missing}"
        );
    }

    /// Stable ids — the four [`SocietyDataSourceId`] variants produce
    /// the strings the four clients use to bind the panel / hotkey
    /// groups.
    #[test]
    fn society_overlay_stable_ids_match_design() {
        assert_eq!(SocietyDataSourceId::Ideology.stable_id(), "society.ideology");
        assert_eq!(SocietyDataSourceId::Language.stable_id(), "society.language");
        assert_eq!(SocietyDataSourceId::Kinship.stable_id(), "society.kinship");
        assert_eq!(
            SocietyDataSourceId::PolityOverlap.stable_id(),
            "society.polity_overlap"
        );
    }

    /// `data_source_count` MUST return `4` (one per
    /// [`SocietyDataSourceId`]).
    #[test]
    fn society_overlay_data_source_count_is_four() {
        let reg = SocietyOverlayRegistry::new();
        assert_eq!(reg.data_source_count(), 4);
        assert_eq!(SocietyDataSourceId::all().len(), 4);
    }

    /// `SocietyDataSource::all()` iterates the four variants in
    /// catalogue order.
    #[test]
    fn society_overlay_all_iterates_catalogue_order() {
        let all = SocietyDataSourceId::all();
        assert_eq!(all[0], SocietyDataSourceId::Ideology);
        assert_eq!(all[1], SocietyDataSourceId::Language);
        assert_eq!(all[2], SocietyDataSourceId::Kinship);
        assert_eq!(all[3], SocietyDataSourceId::PolityOverlap);
    }

    /// Cluster field `new` matches the canonical default
    /// (`max_known_id = 1`).
    #[test]
    fn society_overlay_cluster_field_canonical_default() {
        let f = ClusterField::new();
        assert_eq!(f.max_known_id, 1);
        assert_eq!(f.dominant_cluster_id(cell(0, 0, 0)), CLUSTER_NONE);
        assert!((f.sample(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
    }

    /// Density field `new` matches the canonical `0.0..=1.0` saturation.
    #[test]
    fn society_overlay_density_field_canonical_default() {
        let f = DensityField::new();
        assert_eq!(f.min_raw, 0.0);
        assert_eq!(f.max_raw, 1.0);
        assert!((f.get_raw(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
    }
}
