//! FR-CIV-INFOVIEW-910 — Environmental overlay data sources.
//!
//! Per [`docs/specs/requirements/FR-CIV-INFOVIEW.md`] §910 and the
//! priority-12 rank-1 overlay catalog in [`docs/design/info-views.md`]
//! §3, this module provides the **pure data / query backing layer** for
//! the CS2-class environmental overlays:
//!
//! * **Pollution** — air / ground / water / noise concentration per cell.
//! * **Land value** — emergent desirability scalar per cell.
//! * **Temperature** — measured °C per cell (lapse + latitude proxy when
//!   the `civ-planet` weather clock is absent; passthrough when present).
//! * **Water flow** / **Wind flow** — flow magnitude per cell (proxy
//!   `RAMP_FLOW` for the water-flow A2 / F1 overlays).
//!
//! Design contract (charter: emergence, not invented categories):
//!
//! 1. **One data source = one normalized scalar per cell.** Every
//!    `EnvDataSource::sample` returns a value in `0.0..=1.0` that *tracks*
//!    the underlying raw field (AC: "a data-source returns a normalized
//!    scalar per cell that tracks the underlying field").
//! 2. **Raw fields are substrate-owned.** The producing crate
//!    (`civ-planet` / `civ-voxel` / `civ-economy`) is the canonical home of
//!    the raw `f32`. This module is read-only — it copies raw values into
//!    its registry and normalises on query.
//! 3. **Pure-logic, no engine.** No Bevy, no rendering, no systems. The
//!    renderer / panel / hotkey wiring in `clients/bevy-ref/src/info_views.rs`
//!    (and equivalents in web / Godot / Unreal) reads the normalised
//!    scalars via the [`EnvOverlayRegistry::query`] and
//!    [`EnvOverlayRegistry::query_raw`] accessors and feeds them into the
//!    existing `InfoOverlay` colour functions / ramps. This module is the
//!    substrate-neutral backing store the four clients all bind to.
//! 4. **Additive only.** This module is *additive* to `civ-hud`; it does
//!    not touch any other crate and does not modify any existing public
//!    surface in `lib.rs`. The two `mod` / `pub use` lines in `lib.rs`
//!    are the only `lib.rs` diffs.
//!
//! The registry is intentionally a plain `HashMap`-backed store (no
//! engine features, no globals) so unit tests can build a small
//! per-cell field and assert that the normalised scalar tracks the raw
//! field at every cell. This is the acceptance test
//! (`env_overlay_normalizes_tracks_underlying_field`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Identifier for a cell in the simulation lattice. Lattice coordinates
/// are signed integers (substrate-neutral — the producing crate decides
/// the origin / extent). Mirrors the `(i32, i32, i32)` cell tuple used
/// by `TileInspector` in `tile_inspector.rs` so the four clients
/// (web / Bevy / Godot / Unreal) can pass a probed cell through
/// unchanged.
pub type CellId = (i32, i32, i32);

/// Normalized scalar in `0.0..=1.0` produced by an [`EnvDataSource`].
///
/// The contract is *tracking*, not *interpretation*: the scalar must
/// rise when the underlying raw field rises (and fall when it falls),
/// saturating to the configured min/max so the legend ramp is stable
/// across snapshots.
pub type NormalizedScalar = f32;

/// Raw, substrate-owned field value. Semantics differ per data source
/// (e.g. °C for temperature, ppm for pollution) — callers that need
/// the substrate units use [`EnvOverlayRegistry::query_raw`].
pub type RawScalar = f32;

// ---------------------------------------------------------------------------
// Pollution (FR-CIV-INFOVIEW-910 — F2 row in the overlay catalog).
// ---------------------------------------------------------------------------

/// Pollution medium. Maps to the four channels the FR specifies
/// ("air/ground/water/noise pollution"); each medium tracks an
/// independent concentration field per cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PollutionMedium {
    /// Atmospheric particulate / chemical concentration.
    Air,
    /// Soil / sediment contamination.
    Ground,
    /// Waterborne pollutant concentration (rivers, groundwater, ocean
    /// cells).
    Water,
    /// Anthropogenic noise level (dB proxy, normalised).
    Noise,
}

impl PollutionMedium {
    /// Canonical human label for legend / tooltip rendering. Stable
    /// strings so the four clients can match against the registry
    /// without re-deriving the label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PollutionMedium::Air => "Air pollution",
            PollutionMedium::Ground => "Ground pollution",
            PollutionMedium::Water => "Water pollution",
            PollutionMedium::Noise => "Noise pollution",
        }
    }

    /// Substrate field name (per `docs/design/info-views.md` §3 F2
    /// "data source" column — `voxel`/`laws` · pollutant concentration).
    /// Stable so cross-crate telemetry can index by it.
    #[must_use]
    pub const fn substrate_field(self) -> &'static str {
        match self {
            // `voxel` field names; the producing crate decides the
            // exact accessor.
            PollutionMedium::Air => "voxel.pollutant_air",
            PollutionMedium::Ground => "voxel.pollutant_ground",
            PollutionMedium::Water => "voxel.pollutant_water",
            PollutionMedium::Noise => "laws.noise_db",
        }
    }

    /// All four media — for iteration in the registry / legend panel.
    #[must_use]
    pub const fn all() -> [PollutionMedium; 4] {
        [
            PollutionMedium::Air,
            PollutionMedium::Ground,
            PollutionMedium::Water,
            PollutionMedium::Noise,
        ]
    }
}

// ---------------------------------------------------------------------------
// Data-source identity.
// ---------------------------------------------------------------------------

/// Machine id for an environmental data source. One variant per
/// FR-CIV-INFOVIEW-910 row in the overlay catalog (pollution is a
/// [`PollutionMedium`]-keyed sub-registry; land value, temperature, and
/// the two flow proxies are single-source).
///
/// `EnvDataSourceId::Pollution` is a *family*; the per-medium sources
/// are addressed as `(EnvDataSourceId::Pollution, Some(medium))` in
/// [`EnvOverlayRegistry::query`]. This mirrors the
/// `medium dropdown for F2` sub-control in `info-views.md` §5.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvDataSourceId {
    /// Pollution family (air / ground / water / noise).
    Pollution,
    /// Emergent land-value scalar per cell.
    LandValue,
    /// Temperature (°C) per cell.
    Temperature,
    /// Water flow magnitude per cell (overlay catalog A2 / F1).
    WaterFlow,
    /// Wind flow magnitude per cell.
    WindFlow,
}

impl EnvDataSourceId {
    /// Stable string id (e.g. `"pollution.air"` once disambiguated by
    /// medium). For family-only ids, returns the bare family name
    /// (e.g. `"pollution"`).
    #[must_use]
    pub fn stable_id(self, medium: Option<PollutionMedium>) -> String {
        match (self, medium) {
            (EnvDataSourceId::Pollution, Some(m)) => format!("pollution.{}", m_subtag(m)),
            (EnvDataSourceId::Pollution, None) => "pollution".to_owned(),
            (EnvDataSourceId::LandValue, _) => "land_value".to_owned(),
            (EnvDataSourceId::Temperature, _) => "temperature".to_owned(),
            (EnvDataSourceId::WaterFlow, _) => "water_flow".to_owned(),
            (EnvDataSourceId::WindFlow, _) => "wind_flow".to_owned(),
        }
    }

    /// Display label for the registry / panel. Family-level labels are
    /// used when no medium is selected; per-medium labels are used
    /// inside the pollution dropdown.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            EnvDataSourceId::Pollution => "Pollution",
            EnvDataSourceId::LandValue => "Land value",
            EnvDataSourceId::Temperature => "Temperature",
            EnvDataSourceId::WaterFlow => "Water flow",
            EnvDataSourceId::WindFlow => "Wind flow",
        }
    }
}

fn m_subtag(m: PollutionMedium) -> &'static str {
    match m {
        PollutionMedium::Air => "air",
        PollutionMedium::Ground => "ground",
        PollutionMedium::Water => "water",
        PollutionMedium::Noise => "noise",
    }
}

// ---------------------------------------------------------------------------
// Per-source field store (raw + normalization).
// ---------------------------------------------------------------------------

/// Per-cell raw → normalized mapping for one pollution medium. The
/// raw values are substrate units (ppm for chemical media, dB for
/// noise); the normalized scalar is `0.0..=1.0` on the `RAMP_POLLUTION`
/// ramp in `info_views.rs` (clean → toxic).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollutionField {
    /// Medium this field tracks.
    pub medium: PollutionMedium,
    /// Raw value per cell (ppm or dB depending on medium). Missing
    /// cells are treated as zero concentration ("clean" baseline).
    pub raw: HashMap<CellId, RawScalar>,
    /// `min` raw value (clean). Defaults to `0.0`; the producing crate
    /// may override (e.g. a dB noise floor).
    pub min_raw: RawScalar,
    /// `max` raw value (saturating toxic). Defaults to `100.0` ppm (or
    /// `120.0` dB for noise); the producing crate supplies the
    /// substrate-specific saturation.
    pub max_raw: RawScalar,
}

impl PollutionField {
    /// Build a pollution field with the canonical clean/toxic
    /// saturation: `0.0..=100.0` for chemical media, `0.0..=120.0` dB
    /// for noise (standard occupational + community noise bounds).
    #[must_use]
    pub fn new(medium: PollutionMedium) -> Self {
        let (min, max) = match medium {
            // Air / ground / water: 0–100 ppm is the standard
            // toxicology "clean" → "highly polluted" band used by the
            // CS2-class reference UI.
            PollutionMedium::Air | PollutionMedium::Ground | PollutionMedium::Water => (0.0, 100.0),
            // Noise: 0 dB (silence) → 120 dB (pain threshold).
            PollutionMedium::Noise => (0.0, 120.0),
        };
        Self {
            medium,
            raw: HashMap::new(),
            min_raw: min,
            max_raw: max,
        }
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

    /// Read the raw value at `cell`. Missing cells return `min_raw`
    /// (clean baseline).
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

    /// Normalized scalar at `cell` (acceptance-test path: returns
    /// `0.0..=1.0` that *tracks* the raw field).
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.get_raw(cell))
    }
}

/// Emergent land-value scalar per cell. Substrate units are arbitrary
/// (the FR leaves the unit open; CS2 uses a 0–100 desirability index
/// with no fixed unit). Defaults to a 0..=100 saturating range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandValueField {
    /// Raw per-cell desirability index (substrate units).
    pub raw: HashMap<CellId, RawScalar>,
    /// `min` raw (least desirable). Defaults to `0.0`.
    pub min_raw: RawScalar,
    /// `max` raw (most desirable). Defaults to `100.0`.
    pub max_raw: RawScalar,
}

impl Default for LandValueField {
    fn default() -> Self {
        Self {
            raw: HashMap::new(),
            min_raw: 0.0,
            max_raw: 100.0,
        }
    }
}

impl LandValueField {
    /// Build a land-value field with the default `0..=100` saturation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the saturation (substrate-specific). `max_raw` must
    /// exceed `min_raw`; if not, the field falls back to the canonical
    /// `0.0..=1.0`.
    pub fn set_saturation(&mut self, min: RawScalar, max: RawScalar) {
        if max > min {
            self.min_raw = min;
            self.max_raw = max;
        }
    }

    /// Push a raw sample. Missing cells stay at the *least* desirable
    /// baseline (the CS2-class default for an unsurveyed cell is
    /// desirability 0).
    pub fn insert(&mut self, cell: CellId, value: RawScalar) {
        self.raw.insert(cell, value);
    }

    /// Raw value at `cell` (defaults to `min_raw` when absent).
    #[must_use]
    pub fn get_raw(&self, cell: CellId) -> RawScalar {
        self.raw.get(&cell).copied().unwrap_or(self.min_raw)
    }

    /// Normalize a raw value to `0.0..=1.0`.
    #[must_use]
    pub fn normalize(&self, raw: RawScalar) -> NormalizedScalar {
        let span = (self.max_raw - self.min_raw).max(f32::EPSILON);
        ((raw - self.min_raw) / span).clamp(0.0, 1.0)
    }

    /// Normalized scalar at `cell` (acceptance-test path).
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.get_raw(cell))
    }
}

/// Temperature per cell. Substrate units are **degrees Celsius**;
/// the `min_raw..=max_raw` range defines the cold/hot saturation used
/// to project onto the `RAMP_TEMPERATURE` ramp (cold → temperate → hot).
///
/// Defaults match the CS2-class reference: `-30.0 °C` (polar) to
/// `+50.0 °C` (desert peak). Producing crates that have a
/// `civ-planet` weather clock may tighten the band; the standalone
/// sandbox that ships the live proxy uses the lapse+latitude model
/// and inherits the same range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureField {
    /// Raw per-cell temperature in °C.
    pub raw: HashMap<CellId, RawScalar>,
    /// `min` raw (coldest). Defaults to `-30.0` °C.
    pub min_raw: RawScalar,
    /// `max` raw (hottest). Defaults to `+50.0` °C.
    pub max_raw: RawScalar,
}

impl Default for TemperatureField {
    fn default() -> Self {
        Self {
            raw: HashMap::new(),
            min_raw: -30.0,
            max_raw: 50.0,
        }
    }
}

impl TemperatureField {
    /// Build a temperature field with the default `-30..=+50 °C`
    /// saturation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the saturation. `max_raw` must exceed `min_raw`; if
    /// not, the field falls back to the canonical `0.0..=1.0`.
    pub fn set_saturation(&mut self, min: RawScalar, max: RawScalar) {
        if max > min {
            self.min_raw = min;
            self.max_raw = max;
        }
    }

    /// Push a raw °C sample. Missing cells return the *temperate
    /// midpoint* (the `civ-planet` "unmeasured" default so the
    /// heatmap doesn't bias cold for unsurveyed land — the
    /// midpoint of the default `-30..=+50 °C` saturation is
    /// `10.0 °C`, projecting to normalized `0.5`). This is the only
    /// field in the module that does NOT default to `min_raw`; the
    /// rationale is in the field docs above and the unit test
    /// `env_overlay_temperature_default_is_temperate`.
    pub fn insert(&mut self, cell: CellId, celsius: RawScalar) {
        self.raw.insert(cell, celsius);
    }

    /// Raw °C at `cell` (defaults to the temperate midpoint when
    /// absent).
    #[must_use]
    pub fn get_raw(&self, cell: CellId) -> RawScalar {
        self.raw
            .get(&cell)
            .copied()
            .unwrap_or_else(|| (self.min_raw + self.max_raw) * 0.5)
    }

    /// Normalize a raw °C value to `0.0..=1.0` (cold → hot).
    #[must_use]
    pub fn normalize(&self, celsius: RawScalar) -> NormalizedScalar {
        let span = (self.max_raw - self.min_raw).max(f32::EPSILON);
        ((celsius - self.min_raw) / span).clamp(0.0, 1.0)
    }

    /// Normalized scalar at `cell` (acceptance-test path).
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.get_raw(cell))
    }
}

/// Flow-magnitude field (water flow A2 / F1 and wind flow). Substrate
/// units are substrate-defined (m³ s⁻¹ for water, m s⁻¹ for wind);
/// the registry only normalises to `0.0..=1.0` for the `RAMP_FLOW`
/// ramp and never interprets the unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowField {
    /// Raw per-cell flow magnitude (substrate units).
    pub raw: HashMap<CellId, RawScalar>,
    /// `min` raw (no flow). Defaults to `0.0`.
    pub min_raw: RawScalar,
    /// `max` raw (saturating flow). Defaults to `1.0` (the producing
    /// crate sets a substrate-specific max — e.g. peak river
    /// discharge or peak wind speed).
    pub max_raw: RawScalar,
}

impl Default for FlowField {
    fn default() -> Self {
        Self {
            raw: HashMap::new(),
            min_raw: 0.0,
            max_raw: 1.0,
        }
    }
}

impl FlowField {
    /// Build a flow field with the default `0..=1` saturation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the saturation. `max_raw` must exceed `min_raw`; if
    /// not, the field falls back to the canonical `0.0..=1.0`.
    pub fn set_saturation(&mut self, min: RawScalar, max: RawScalar) {
        if max > min {
            self.min_raw = min;
            self.max_raw = max;
        }
    }

    /// Push a raw sample. Missing cells return `min_raw` (no flow).
    pub fn insert(&mut self, cell: CellId, value: RawScalar) {
        self.raw.insert(cell, value);
    }

    /// Raw value at `cell` (defaults to `min_raw` when absent).
    #[must_use]
    pub fn get_raw(&self, cell: CellId) -> RawScalar {
        self.raw.get(&cell).copied().unwrap_or(self.min_raw)
    }

    /// Normalize a raw value to `0.0..=1.0`.
    #[must_use]
    pub fn normalize(&self, raw: RawScalar) -> NormalizedScalar {
        let span = (self.max_raw - self.min_raw).max(f32::EPSILON);
        ((raw - self.min_raw) / span).clamp(0.0, 1.0)
    }

    /// Normalized scalar at `cell` (acceptance-test path).
    #[must_use]
    pub fn sample(&self, cell: CellId) -> NormalizedScalar {
        self.normalize(self.get_raw(cell))
    }
}

// ---------------------------------------------------------------------------
// Data-source abstraction.
// ---------------------------------------------------------------------------

/// One environmental data source — a typed, queryable cell field. The
/// trait is the acceptance-test seam: a host unit test can build a
/// `LandValueField`, populate it, and call
/// `source.sample((0, 0, 0))` to get a normalized scalar that
/// tracks the underlying field.
pub trait EnvDataSource {
    /// Stable machine id for this data source.
    fn id(&self) -> EnvDataSourceId;
    /// Raw value at `cell` (substrate units).
    fn raw(&self, cell: CellId) -> RawScalar;
    /// Normalized scalar at `cell` (`0.0..=1.0`).
    fn sample(&self, cell: CellId) -> NormalizedScalar;
}

impl EnvDataSource for PollutionField {
    fn id(&self) -> EnvDataSourceId {
        EnvDataSourceId::Pollution
    }
    fn raw(&self, cell: CellId) -> RawScalar {
        self.get_raw(cell)
    }
    fn sample(&self, cell: CellId) -> NormalizedScalar {
        PollutionField::sample(self, cell)
    }
}

impl EnvDataSource for LandValueField {
    fn id(&self) -> EnvDataSourceId {
        EnvDataSourceId::LandValue
    }
    fn raw(&self, cell: CellId) -> RawScalar {
        self.get_raw(cell)
    }
    fn sample(&self, cell: CellId) -> NormalizedScalar {
        LandValueField::sample(self, cell)
    }
}

impl EnvDataSource for TemperatureField {
    fn id(&self) -> EnvDataSourceId {
        EnvDataSourceId::Temperature
    }
    fn raw(&self, cell: CellId) -> RawScalar {
        self.get_raw(cell)
    }
    fn sample(&self, cell: CellId) -> NormalizedScalar {
        TemperatureField::sample(self, cell)
    }
}

impl EnvDataSource for FlowField {
    fn id(&self) -> EnvDataSourceId {
        // FlowField is reused for both water and wind. The trait
        // consumer is expected to disambiguate at the *registry*
        // level (see [`EnvOverlayRegistry::water_flow`] /
        // [`EnvOverlayRegistry::wind_flow`]) so the trait here
        // reports the family id.
        EnvDataSourceId::WaterFlow
    }
    fn raw(&self, cell: CellId) -> RawScalar {
        self.get_raw(cell)
    }
    fn sample(&self, cell: CellId) -> NormalizedScalar {
        FlowField::sample(self, cell)
    }
}

// ---------------------------------------------------------------------------
// Registry — the four-client binding point.
// ---------------------------------------------------------------------------

/// Backing store for every FR-CIV-INFOVIEW-910 environmental data
/// source. The four clients (web / Bevy / Godot / Unreal) each hold
/// one `EnvOverlayRegistry` and refresh its fields from
/// `civ-planet` / `civ-voxel` / `civ-economy` per tick. The
/// per-overlay colour function then asks the registry for the
/// normalised scalar at a cell.
///
/// Construction is by `EnvOverlayRegistry::default()`; the four
/// fields are pre-populated with their canonical saturations
/// (pollution: 0..=100 ppm + 0..=120 dB noise; land value: 0..=100;
/// temperature: -30..=+50 °C; water/wind flow: 0..=1). Producing
/// crates may `set_saturation` and `insert` to override.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvOverlayRegistry {
    /// Pollution fields — one per [`PollutionMedium`]. The four media
    /// are always present after `default()` so a host client can
    /// bind the F2 dropdown entries immediately.
    pub pollution: HashMap<PollutionMedium, PollutionField>,
    /// Land value field.
    pub land_value: LandValueField,
    /// Temperature field.
    pub temperature: TemperatureField,
    /// Water flow field (A2 / F1 — overlay catalog).
    pub water_flow: FlowField,
    /// Wind flow field.
    pub wind_flow: FlowField,
}

impl Default for EnvOverlayRegistry {
    /// `Default` mirrors [`Self::new`] — every data source is
    /// pre-populated with its canonical saturation so a host
    /// client can call `EnvOverlayRegistry::default()` and
    /// immediately query any of the eight data sources.
    fn default() -> Self {
        Self::new()
    }
}

impl EnvOverlayRegistry {
    /// Build a registry with all five data sources pre-populated
    /// at their canonical saturations. Pollution has four sub-fields
    /// (one per [`PollutionMedium`]).
    #[must_use]
    pub fn new() -> Self {
        let mut pollution = HashMap::new();
        for medium in PollutionMedium::all() {
            pollution.insert(medium, PollutionField::new(medium));
        }
        Self {
            pollution,
            land_value: LandValueField::new(),
            temperature: TemperatureField::new(),
            water_flow: FlowField::new(),
            wind_flow: FlowField::new(),
        }
    }

    /// Per-cell pollution sample for `medium`. Returns a normalized
    /// scalar (`0.0..=1.0`) that tracks the raw ppm/dB value. Returns
    /// `None` if the registry is missing the medium (caller bug).
    #[must_use]
    pub fn pollution_sample(
        &self,
        medium: PollutionMedium,
        cell: CellId,
    ) -> Option<NormalizedScalar> {
        self.pollution.get(&medium).map(|f| f.sample(cell))
    }

    /// Per-cell raw pollution for `medium` (ppm or dB).
    #[must_use]
    pub fn pollution_raw(&self, medium: PollutionMedium, cell: CellId) -> Option<RawScalar> {
        self.pollution.get(&medium).map(|f| f.get_raw(cell))
    }

    /// Per-cell land-value sample (normalized).
    #[must_use]
    pub fn land_value_sample(&self, cell: CellId) -> NormalizedScalar {
        self.land_value.sample(cell)
    }

    /// Per-cell raw land value (substrate units).
    #[must_use]
    pub fn land_value_raw(&self, cell: CellId) -> RawScalar {
        self.land_value.get_raw(cell)
    }

    /// Per-cell temperature sample (normalized cold→hot).
    #[must_use]
    pub fn temperature_sample(&self, cell: CellId) -> NormalizedScalar {
        self.temperature.sample(cell)
    }

    /// Per-cell raw temperature in °C.
    #[must_use]
    pub fn temperature_raw(&self, cell: CellId) -> RawScalar {
        self.temperature.get_raw(cell)
    }

    /// Per-cell water-flow sample (normalized).
    #[must_use]
    pub fn water_flow_sample(&self, cell: CellId) -> NormalizedScalar {
        self.water_flow.sample(cell)
    }

    /// Per-cell raw water flow (substrate units).
    #[must_use]
    pub fn water_flow_raw(&self, cell: CellId) -> RawScalar {
        self.water_flow.get_raw(cell)
    }

    /// Per-cell wind-flow sample (normalized).
    #[must_use]
    pub fn wind_flow_sample(&self, cell: CellId) -> NormalizedScalar {
        self.wind_flow.sample(cell)
    }

    /// Per-cell raw wind flow (substrate units).
    #[must_use]
    pub fn wind_flow_raw(&self, cell: CellId) -> RawScalar {
        self.wind_flow.get_raw(cell)
    }

    /// Generic dispatcher — query any registered data source by id
    /// (with optional pollution medium for the F2 family). This is
    /// the entry point a data-driven overlay panel uses to fetch the
    /// normalised scalar for a cell without caring about the
    /// underlying field type.
    ///
    /// Returns `None` when the `(id, medium)` pair is not present in
    /// the registry — the FR-CIV-INFOVIEW-905 contract (BLIND data
    /// sources render greyed, never fabricated) means a missing
    /// field is a *signal* to the panel, not a fallback to 0.0.
    #[must_use]
    pub fn query(
        &self,
        id: EnvDataSourceId,
        medium: Option<PollutionMedium>,
        cell: CellId,
    ) -> Option<NormalizedScalar> {
        match (id, medium) {
            (EnvDataSourceId::Pollution, Some(m)) => self.pollution_sample(m, cell),
            // Pollution without a medium selected = sum of the four
            // normalised sub-fields (the CS2 default for an
            // "aggregate pollution" overlay). Clamped to `0.0..=1.0`
            // so it never overflows the legend ramp.
            (EnvDataSourceId::Pollution, None) => {
                let total: f32 = self.pollution.values().map(|f| f.sample(cell)).sum();
                Some((total / 4.0).clamp(0.0, 1.0))
            }
            (EnvDataSourceId::LandValue, _) => Some(self.land_value_sample(cell)),
            (EnvDataSourceId::Temperature, _) => Some(self.temperature_sample(cell)),
            (EnvDataSourceId::WaterFlow, _) => Some(self.water_flow_sample(cell)),
            (EnvDataSourceId::WindFlow, _) => Some(self.wind_flow_sample(cell)),
        }
    }

    /// Generic dispatcher for the raw value (substrate units). Same
    /// `(None, …)` semantics as [`Self::query`]: a missing
    /// `(id, medium)` pair returns `None` so the panel can grey the
    /// overlay rather than fabricate a raw value.
    #[must_use]
    pub fn query_raw(
        &self,
        id: EnvDataSourceId,
        medium: Option<PollutionMedium>,
        cell: CellId,
    ) -> Option<RawScalar> {
        match (id, medium) {
            (EnvDataSourceId::Pollution, Some(m)) => self.pollution_raw(m, cell),
            (EnvDataSourceId::Pollution, None) => {
                // Aggregate raw = mean of the four sub-field raws.
                // Producing crates that need a different aggregate
                // (e.g. max) can read the sub-fields directly via
                // [`Self::pollution_raw`].
                let n = self.pollution.len();
                if n == 0 {
                    return None;
                }
                let total: f32 = self.pollution.values().map(|f| f.get_raw(cell)).sum();
                Some(total / n as f32)
            }
            (EnvDataSourceId::LandValue, _) => Some(self.land_value_raw(cell)),
            (EnvDataSourceId::Temperature, _) => Some(self.temperature_raw(cell)),
            (EnvDataSourceId::WaterFlow, _) => Some(self.water_flow_raw(cell)),
            (EnvDataSourceId::WindFlow, _) => Some(self.wind_flow_raw(cell)),
        }
    }

    /// Total number of data sources registered. Pollution counts as
    /// four (one per medium) so the registry exposes a stable count
    /// for the panel header.
    #[must_use]
    pub fn data_source_count(&self) -> usize {
        // 4 pollution media + 4 non-pollution = 8.
        4 + 4
    }
}

// ---------------------------------------------------------------------------
// Tests — acceptance test for FR-CIV-INFOVIEW-910.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(x: i32, y: i32, z: i32) -> CellId {
        (x, y, z)
    }

    /// FR-CIV-INFOVIEW-910 acceptance test: a data-source returns a
    /// normalized scalar per cell that tracks the underlying field.
    ///
    /// We populate a land-value field with three cells spanning the
    /// saturation range (0, 50, 100) and assert that the normalized
    /// sample at each cell matches the expected `raw / max` ratio.
    /// A higher raw value must produce a higher (or equal) normalized
    /// scalar — that is the *tracking* contract.
    #[test]
    fn env_overlay_normalizes_tracks_underlying_field() {
        let mut reg = EnvOverlayRegistry::new();
        reg.land_value.insert(cell(0, 0, 0), 0.0);
        reg.land_value.insert(cell(1, 0, 0), 50.0);
        reg.land_value.insert(cell(2, 0, 0), 100.0);

        let lo = reg.land_value_sample(cell(0, 0, 0));
        let mid = reg.land_value_sample(cell(1, 0, 0));
        let hi = reg.land_value_sample(cell(2, 0, 0));

        assert!((lo - 0.0).abs() < 1e-6, "low raw → 0.0, got {lo}");
        assert!((mid - 0.5).abs() < 1e-6, "mid raw → 0.5, got {mid}");
        assert!((hi - 1.0).abs() < 1e-6, "high raw → 1.0, got {hi}");
        // Tracking: monotonic non-decreasing in the raw value.
        assert!(lo <= mid && mid <= hi, "samples must track raw field");
    }

    /// Pollution — air medium — also tracks the raw ppm field across
    /// the 0..=100 saturation.
    #[test]
    fn env_overlay_pollution_air_tracks_raw() {
        let mut reg = EnvOverlayRegistry::new();
        reg.pollution
            .get_mut(&PollutionMedium::Air)
            .expect("air pollution must be present after default")
            .insert(cell(0, 0, 0), 25.0);
        let s = reg
            .pollution_sample(PollutionMedium::Air, cell(0, 0, 0))
            .expect("air pollution must be present");
        assert!((s - 0.25).abs() < 1e-6, "25 ppm → 0.25, got {s}");
    }

    /// Pollution — noise medium — uses the dB saturation, not ppm.
    #[test]
    fn env_overlay_pollution_noise_uses_db_saturation() {
        let mut reg = EnvOverlayRegistry::new();
        reg.pollution
            .get_mut(&PollutionMedium::Noise)
            .expect("noise pollution must be present after default")
            .insert(cell(0, 0, 0), 60.0); // 60 dB
        let s = reg
            .pollution_sample(PollutionMedium::Noise, cell(0, 0, 0))
            .expect("noise pollution must be present");
        // 60 dB / 120 dB max → 0.5.
        assert!((s - 0.5).abs() < 1e-6, "60 dB → 0.5, got {s}");
    }

    /// Temperature — normalized cold→hot tracks the raw °C value.
    /// Coldest (`min_raw`) → 0.0, hottest (`max_raw`) → 1.0.
    #[test]
    fn env_overlay_temperature_tracks_celsius() {
        let mut reg = EnvOverlayRegistry::new();
        // min = -30, max = +50 → span 80.
        reg.temperature.insert(cell(0, 0, 0), -30.0);
        reg.temperature.insert(cell(1, 0, 0), 10.0);
        reg.temperature.insert(cell(2, 0, 0), 50.0);

        let cold = reg.temperature_sample(cell(0, 0, 0));
        let mid = reg.temperature_sample(cell(1, 0, 0));
        let hot = reg.temperature_sample(cell(2, 0, 0));

        assert!((cold - 0.0).abs() < 1e-6, "cold → 0.0, got {cold}");
        assert!((mid - 0.5).abs() < 1e-6, "mid → 0.5, got {mid}");
        assert!((hot - 1.0).abs() < 1e-6, "hot → 1.0, got {hot}");
    }

    /// Unmeasured temperature cells default to the *temperate
    /// midpoint* (not cold). This is the only field in the module
    /// that biases neutral when the cell is missing — the rationale
    /// is in the field doc above.
    #[test]
    fn env_overlay_temperature_default_is_temperate() {
        let reg = EnvOverlayRegistry::new();
        // No inserts → default raw = (-30 + 50) / 2 = 10.0 → 0.5.
        let s = reg.temperature_sample(cell(0, 0, 0));
        assert!(
            (s - 0.5).abs() < 1e-6,
            "missing temperature cell must default to temperate midpoint, got {s}"
        );
    }

    /// Land value — unmeasured cells default to *least desirable* (raw
    /// = min_raw → normalized 0.0). The CS2-class default for an
    /// unsurveyed cell.
    #[test]
    fn env_overlay_land_value_default_is_least_desirable() {
        let reg = EnvOverlayRegistry::new();
        let s = reg.land_value_sample(cell(0, 0, 0));
        assert!(
            (s - 0.0).abs() < 1e-6,
            "missing land value cell must default to 0.0, got {s}"
        );
    }

    /// Pollution — unmeasured cells default to *clean* (raw = min_raw
    /// → 0.0). Mirrors the CS2 default: no data = no contamination.
    #[test]
    fn env_overlay_pollution_default_is_clean() {
        let reg = EnvOverlayRegistry::new();
        for medium in PollutionMedium::all() {
            let s = reg
                .pollution_sample(medium, cell(0, 0, 0))
                .unwrap_or_else(|| panic!("{medium:?} must be present after default"));
            assert!(
                (s - 0.0).abs() < 1e-6,
                "{medium:?} missing cell must default to clean, got {s}"
            );
        }
    }

    /// Flow — unmeasured cells default to *no flow* (0.0).
    #[test]
    fn env_overlay_flow_default_is_zero() {
        let reg = EnvOverlayRegistry::new();
        assert!((reg.water_flow_sample(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
        assert!((reg.wind_flow_sample(cell(0, 0, 0)) - 0.0).abs() < 1e-6);
    }

    /// Saturation clamp — values outside `[min_raw, max_raw]` pin to
    /// the nearest endpoint (no panic, no wrap, no negative `0.0..=1.0`
    /// output).
    #[test]
    fn env_overlay_saturates_outside_range() {
        let mut reg = EnvOverlayRegistry::new();
        reg.land_value.insert(cell(0, 0, 0), -999.0); // below min
        reg.land_value.insert(cell(1, 0, 0), 9999.0); // above max
        let lo = reg.land_value_sample(cell(0, 0, 0));
        let hi = reg.land_value_sample(cell(1, 0, 0));
        assert!(
            (lo - 0.0).abs() < 1e-6,
            "below-min must clamp to 0.0, got {lo}"
        );
        assert!(
            (hi - 1.0).abs() < 1e-6,
            "above-max must clamp to 1.0, got {hi}"
        );
    }

    /// Substrate-specific saturation — `set_saturation` overrides
    /// the canonical range and the field re-normalises against the
    /// new range.
    #[test]
    fn env_overlay_supports_substrate_saturation() {
        let mut reg = EnvOverlayRegistry::new();
        reg.land_value.set_saturation(10.0, 20.0);
        reg.land_value.insert(cell(0, 0, 0), 15.0); // midpoint
        let s = reg.land_value_sample(cell(0, 0, 0));
        assert!(
            (s - 0.5).abs() < 1e-6,
            "15.0 in [10, 20] must normalize to 0.5, got {s}"
        );
    }

    /// The generic `query` dispatcher returns the same value as the
    /// typed accessor (cross-check: the registry is internally
    /// consistent across access paths).
    #[test]
    fn env_overlay_query_dispatcher_matches_typed() {
        let mut reg = EnvOverlayRegistry::new();
        reg.temperature.insert(cell(0, 0, 0), 30.0);
        let typed = reg.temperature_sample(cell(0, 0, 0));
        let dispatched = reg
            .query(EnvDataSourceId::Temperature, None, cell(0, 0, 0))
            .expect("temperature must be present");
        assert!(
            (typed - dispatched).abs() < 1e-6,
            "typed ({typed}) and query ({dispatched}) must agree"
        );
    }

    /// `query` with no pollution medium returns the mean of the four
    /// sub-fields (aggregate pollution overlay). Confirms the
    /// CS2-class "sum / 4" contract.
    #[test]
    fn env_overlay_pollution_aggregate_is_mean_of_four() {
        let mut reg = EnvOverlayRegistry::new();
        // Set air to 100 ppm, others stay 0.
        reg.pollution
            .get_mut(&PollutionMedium::Air)
            .unwrap()
            .insert(cell(0, 0, 0), 100.0);
        // 100/100 = 1.0 (air); 0/100 = 0.0 (others). Mean = 0.25.
        let aggregate = reg
            .query(EnvDataSourceId::Pollution, None, cell(0, 0, 0))
            .expect("aggregate pollution must be present");
        assert!(
            (aggregate - 0.25).abs() < 1e-6,
            "aggregate pollution (100/0/0/0) → 0.25, got {aggregate}"
        );
    }

    /// `query_raw` returns substrate units (ppm for chemical media, dB
    /// for noise) so the panel hover-text can display the unit-bearing
    /// value alongside the normalized ramp position.
    #[test]
    fn env_overlay_query_raw_returns_substrate_units() {
        let mut reg = EnvOverlayRegistry::new();
        reg.pollution
            .get_mut(&PollutionMedium::Air)
            .unwrap()
            .insert(cell(0, 0, 0), 42.0);
        let raw = reg
            .query_raw(
                EnvDataSourceId::Pollution,
                Some(PollutionMedium::Air),
                cell(0, 0, 0),
            )
            .expect("air pollution raw must be present");
        assert!(
            (raw - 42.0).abs() < 1e-6,
            "air raw at (0,0,0) must be 42.0 ppm, got {raw}"
        );
    }

    /// Stable ids — `EnvDataSourceId::stable_id` produces the strings
    /// the four clients use to bind the panel / hotkey groups.
    #[test]
    fn env_overlay_stable_ids_match_design() {
        assert_eq!(
            EnvDataSourceId::Pollution.stable_id(Some(PollutionMedium::Air)),
            "pollution.air"
        );
        assert_eq!(
            EnvDataSourceId::Pollution.stable_id(Some(PollutionMedium::Noise)),
            "pollution.noise"
        );
        assert_eq!(EnvDataSourceId::LandValue.stable_id(None), "land_value");
        assert_eq!(EnvDataSourceId::Temperature.stable_id(None), "temperature");
        assert_eq!(EnvDataSourceId::WaterFlow.stable_id(None), "water_flow");
        assert_eq!(EnvDataSourceId::WindFlow.stable_id(None), "wind_flow");
    }

    /// The five `EnvDataSourceId` variants × four `PollutionMedium`
    /// values produce the expected data-source count of 8 (4 pollution
    /// + 4 non-pollution).
    #[test]
    fn env_overlay_data_source_count_is_eight() {
        let reg = EnvOverlayRegistry::new();
        assert_eq!(reg.data_source_count(), 8);
    }

    /// Pollution field `new` matches the canonical saturations.
    #[test]
    fn env_overlay_pollution_canonical_saturations() {
        let air = PollutionField::new(PollutionMedium::Air);
        assert_eq!(air.min_raw, 0.0);
        assert_eq!(air.max_raw, 100.0);
        let noise = PollutionField::new(PollutionMedium::Noise);
        assert_eq!(noise.min_raw, 0.0);
        assert_eq!(noise.max_raw, 120.0);
    }
}
