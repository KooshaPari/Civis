//! Data-driven overlay registry — pure data/query layer.
//!
//! Implements `FR-CIV-INFOVIEW-901`: the overlay registry SHALL be data-driven
//! (each overlay = id, label, data-source binding, color ramp), so new overlays
//! add without code forks.
//!
//! This module is deliberately **render-free**: no Bevy types, no systems, no
//! plugins, no GPU. It is the substrate-neutral vocabulary that any host client
//! (Bevy / Godot / Unreal / web) can read to drive its own recolor pass. The
//! Bevy reference client already mirrors this shape in
//! `clients/bevy-ref/src/info_views.rs`; this module is the canonical,
//! engine-agnostic home, lifted out of the client so the wire shape is shared.
//!
//! ## Shape
//!
//! ```text
//! OverlayDef {
//!     id:     "elevation",                   // stable machine id (String)
//!     label:  "Elevation",                   // human display name
//!     source: DataSource::TerrainHeight,     // data-source binding
//!     ramp:   ColorRamp { stops: [...] },    // 0.0..=1.0 → sRGB triple
//! }
//! ```
//!
//! ## Acceptance
//!
//! Per the FR:
//! 1. **Registering an overlay makes it retrievable by id** — `register` then
//!    `get(id) -> Option<&OverlayDef>`.
//! 2. **Color-ramp maps a value to a color** — `ColorRamp::map(value)` clamps
//!    out-of-range values and linearly interpolates between adjacent stops.
//!
//! Adding a new overlay is a single `registry.register(def)` call; no
//! recompilation of render code is required.

use serde::{Deserialize, Serialize};

/// Stable machine id for a registered overlay.
///
/// `String` (not `&'static str`) so the struct is `serde`-friendly: serde's
/// `'de` lifetime cannot outlive `'static`, so borrowed-string fields break
/// `Deserialize`. `OverlayId::new` accepts `&'static str` literals so seed
/// definitions stay ergonomic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OverlayId(String);

impl OverlayId {
    /// Construct an `OverlayId` from a `&'static str` literal (the seed pattern).
    #[must_use]
    pub fn new(id: &'static str) -> Self {
        Self(id.to_owned())
    }

    /// Borrow the inner id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for OverlayId {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl From<String> for OverlayId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

// ---------------------------------------------------------------------------
// Data source binding (FR-CIV-INFOVIEW-901 §"data-source binding").
// ---------------------------------------------------------------------------

/// The data-source an overlay reads.
///
/// `FR-CIV-INFOVIEW-901` requires each overlay to carry a *data-source
/// binding* — which measured field this overlay visualizes. New sources are
/// added as new variants, but the registry itself never forks: callers stay
/// data-driven. The `Custom(String)` escape hatch keeps the registry open to
/// downstream crates (e.g. `civ-economy`, `civ-laws`) that ship their own
/// measured fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DataSource {
    /// Terrain surface height (e.g. `civ-planet` measured field).
    TerrainHeight,
    /// Water depth / hydrology (e.g. `civ-planet` measured field).
    WaterDepth,
    /// Surface material type (e.g. `civ-voxel` measured field).
    Material,
    /// Climate proxy — temperature band.
    Temperature,
    /// Agent density from the live simulation (`civ-agents`).
    Population,
    /// Aggregate civilian needs pressure (`civ-agents::Needs`).
    NeedsPressure,
    /// Dominant faction / cluster id per cell (`civ-agents`).
    Cluster,
    /// Wind / airflow (e.g. `civ-planet` measured field).
    Wind,
    /// Pollution — air/ground/water/noise.
    Pollution,
    /// Land value (composite emergent measurement).
    LandValue,
    /// Natural resources — ore / wood / fertile / energy.
    Resources,
    /// Open extension: any measured field not yet enumerated. The inner
    /// string is the measured-field name (e.g. `"civ_economy.flow"`).
    Custom(String),
}

impl DataSource {
    /// Stable, kebab-case identifier for this data source.
    ///
    /// Useful for UI / debug surfaces that want a short label without
    /// destructuring the enum.
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        match self {
            DataSource::TerrainHeight => "terrain-height",
            DataSource::WaterDepth => "water-depth",
            DataSource::Material => "material",
            DataSource::Temperature => "temperature",
            DataSource::Population => "population",
            DataSource::NeedsPressure => "needs-pressure",
            DataSource::Cluster => "cluster",
            DataSource::Wind => "wind",
            DataSource::Pollution => "pollution",
            DataSource::LandValue => "land-value",
            DataSource::Resources => "resources",
            DataSource::Custom(_) => "custom",
        }
    }
}

// ---------------------------------------------------------------------------
// Color ramp (FR-CIV-INFOVIEW-901 §"color-ramp").
// ---------------------------------------------------------------------------

/// A single colour-ramp stop: a normalised position, a human label, and an
/// sRGB triple (linear-channel values in `0.0..=1.0`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegendStop {
    /// Normalised position on the ramp (`0.0` = low end, `1.0` = high end).
    pub at: f32,
    /// Human label shown beside the swatch (e.g. `"Sea level"`).
    pub label: String,
    /// sRGB colour for this stop (linear channels in `0.0..=1.0`).
    pub rgb: [f32; 3],
}

impl LegendStop {
    /// Construct a `LegendStop` from a `&'static str` literal (the seed
    /// pattern). The label widens to `String` so the value is `serde`-friendly.
    #[must_use]
    pub fn new(at: f32, label: &'static str, rgb: [f32; 3]) -> Self {
        Self {
            at,
            label: label.to_owned(),
            rgb,
        }
    }
}

/// A color-ramp: an ordered, low→high list of [`LegendStop`]s plus a
/// `map(value)` query that clamps out-of-range inputs and linearly
/// interpolates between adjacent stops.
///
/// Stops are kept sorted on `at` so the interpolation sweep is a single
/// linear pass. `map` is pure and has no engine / GPU / IO dependencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorRamp {
    /// Stops in ascending `at` order. A ramp with 0 or 1 stop is legal —
    /// `map` then returns the (only) stop's colour, or a neutral grey when
    /// empty.
    pub stops: Vec<LegendStop>,
}

impl ColorRamp {
    /// Construct a `ColorRamp` from stops, sorting them in ascending `at`
    /// order. Duplicates by `at` are stable; the first one wins on lookups.
    #[must_use]
    pub fn from_stops(mut stops: Vec<LegendStop>) -> Self {
        stops.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
        Self { stops }
    }

    /// Number of stops in the ramp.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stops.len()
    }

    /// `true` when the ramp has zero stops.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// Map a normalised value in `0.0..=1.0` to an sRGB colour.
    ///
    /// Behaviour:
    /// - empty ramp → neutral grey `[0.5, 0.5, 0.5]`;
    /// - single stop → that stop's `rgb` regardless of `value`;
    /// - values below the first stop clamp to the first stop;
    /// - values above the last stop clamp to the last stop;
    /// - values exactly on a stop snap to that stop (no float drift);
    /// - values between two stops are linearly interpolated per channel.
    #[must_use]
    pub fn map(&self, value: f32) -> [f32; 3] {
        let stops = &self.stops;
        if stops.is_empty() {
            return [0.5, 0.5, 0.5];
        }
        if stops.len() == 1 {
            return stops[0].rgb;
        }
        let v = value.clamp(0.0, 1.0);
        if v <= stops[0].at {
            return stops[0].rgb;
        }
        if let Some(last) = stops.last() {
            if v >= last.at {
                return last.rgb;
            }
        }
        // Snap to an exact stop so on-stop values are precise.
        for stop in stops {
            if (v - stop.at).abs() <= f32::EPSILON {
                return stop.rgb;
            }
        }
        for pair in stops.windows(2) {
            let (lo, hi) = (pair[0], pair[1]);
            if v >= lo.at && v <= hi.at {
                let span = (hi.at - lo.at).max(f32::EPSILON);
                let t = (v - lo.at) / span;
                return [
                    lo.rgb[0] + (hi.rgb[0] - lo.rgb[0]) * t,
                    lo.rgb[1] + (hi.rgb[1] - lo.rgb[1]) * t,
                    lo.rgb[2] + (hi.rgb[2] - lo.rgb[2]) * t,
                ];
            }
        }
        stops[stops.len() - 1].rgb
    }
}

// ---------------------------------------------------------------------------
// Overlay definition + registry (FR-CIV-INFOVIEW-901).
// ---------------------------------------------------------------------------

/// One registered overlay: identity, label, data-source binding, color ramp.
///
/// The shape is the literal FR contract: `{id, label, data-source binding,
/// color-ramp}`. New overlays are *new `OverlayDef` values* — no render-code
/// fork.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayDef {
    /// Stable machine id (e.g. `"elevation"`). Used by callers to look the
    /// overlay up after registration.
    pub id: OverlayId,
    /// Human display name shown on the toggle button / legend header.
    pub label: String,
    /// Data-source binding (which measured field this overlay visualizes).
    pub source: DataSource,
    /// Color-ramp — `0.0..=1.0` value → sRGB colour.
    pub ramp: ColorRamp,
}

impl OverlayDef {
    /// Construct an `OverlayDef` from `&'static str` literals (the seed
    /// pattern). The id widens via `OverlayId::new`; the label widens to
    /// `String` so the value is `serde`-friendly.
    #[must_use]
    pub fn new(
        id: &'static str,
        label: &'static str,
        source: DataSource,
        ramp: ColorRamp,
    ) -> Self {
        Self {
            id: OverlayId::new(id),
            label: label.to_owned(),
            source,
            ramp,
        }
    }
}

/// Data-driven overlay registry.
///
/// `FR-CIV-INFOVIEW-901` calls for overlays to be addable *without code
/// changes*; this registry is the data structure that makes that possible.
/// Hosts construct a `OverlayRegistry`, call [`register`](Self::register)
/// for each overlay they want exposed, and query the result by id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OverlayRegistry {
    overlays: Vec<OverlayDef>,
}

impl OverlayRegistry {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a registry pre-populated with the given overlays, in order.
    /// Duplicates by id are not deduplicated; the first match wins on
    /// [`get`](Self::get).
    #[must_use]
    pub fn with_overlays(overlays: Vec<OverlayDef>) -> Self {
        Self { overlays }
    }

    /// Number of registered overlays.
    #[must_use]
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// `true` when no overlays are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }

    /// Register an overlay. Returns `true` on insert, `false` when an
    /// overlay with the same id was already present (no-op insert, no
    /// overwrite — callers that want to replace should
    /// [`upsert`](Self::upsert) or `remove` first).
    pub fn register(&mut self, overlay: OverlayDef) -> bool {
        if self.overlays.iter().any(|o| o.id == overlay.id) {
            return false;
        }
        self.overlays.push(overlay);
        true
    }

    /// Replace the overlay with the same id (inserting if absent). Returns
    /// `true` if an existing overlay was replaced, `false` if it was newly
    /// inserted.
    pub fn upsert(&mut self, overlay: OverlayDef) -> bool {
        if let Some(slot) = self.overlays.iter_mut().find(|o| o.id == overlay.id) {
            *slot = overlay;
            true
        } else {
            self.overlays.push(overlay);
            false
        }
    }

    /// Remove the overlay with the given id. Returns `true` if one was
    /// removed.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(i) = self.overlays.iter().position(|o| o.id.as_str() == id) {
            self.overlays.remove(i);
            true
        } else {
            false
        }
    }

    /// Borrow the overlay with the given id, if registered.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&OverlayDef> {
        self.overlays.iter().find(|o| o.id.as_str() == id)
    }

    /// Borrow the overlay at the given index, if in range.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&OverlayDef> {
        self.overlays.get(index)
    }

    /// Iterate all registered overlays in registration order.
    pub fn iter(&self) -> std::slice::Iter<'_, OverlayDef> {
        self.overlays.iter()
    }

    /// Borrow the list of registered ids in registration order.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.overlays.iter().map(|o| o.id.as_str()).collect()
    }

    /// `true` if an overlay with the given id is registered.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }
}

// ---------------------------------------------------------------------------
// Seed definitions — the overlays that ship with the default registry.
// These are the same overlays `clients/bevy-ref/src/info_views.rs` ships,
// lifted into the engine-neutral vocabulary. Hosts are free to register
// additional overlays on top.
// ---------------------------------------------------------------------------

/// `elevation` overlay — terrain height above sea level.
#[must_use]
pub fn elevation_overlay() -> OverlayDef {
    OverlayDef::new(
        "elevation",
        "Elevation",
        DataSource::TerrainHeight,
        ColorRamp::from_stops(vec![
            LegendStop::new(0.0, "Deep", [0.10, 0.20, 0.55]),
            LegendStop::new(0.18, "Coast", [0.86, 0.78, 0.52]),
            LegendStop::new(0.48, "Lowland", [0.28, 0.58, 0.24]),
            LegendStop::new(0.70, "Highland", [0.50, 0.45, 0.40]),
            LegendStop::new(1.0, "Peak", [0.97, 0.97, 0.97]),
        ]),
    )
}

/// `water` overlay — hydrology (cells at/below the sim water level).
#[must_use]
pub fn water_overlay() -> OverlayDef {
    OverlayDef::new(
        "water",
        "Water",
        DataSource::WaterDepth,
        ColorRamp::from_stops(vec![
            LegendStop::new(0.0, "Dry", [0.80, 0.72, 0.50]),
            LegendStop::new(1.0, "Submerged", [0.10, 0.35, 0.75]),
        ]),
    )
}

/// `population` overlay — agent density from the live simulation.
#[must_use]
pub fn population_overlay() -> OverlayDef {
    OverlayDef::new(
        "population",
        "Population",
        DataSource::Population,
        ColorRamp::from_stops(vec![
            LegendStop::new(0.0, "Empty", [0.15, 0.15, 0.20]),
            LegendStop::new(0.5, "Settled", [0.95, 0.80, 0.20]),
            LegendStop::new(1.0, "Crowded", [0.90, 0.10, 0.10]),
        ]),
    )
}

/// The default registry: the seed overlays in shipping order. New overlays
/// are `registry.register(...)` on top — no render-code fork.
#[must_use]
pub fn default_overlay_registry() -> OverlayRegistry {
    OverlayRegistry::with_overlays(vec![
        elevation_overlay(),
        water_overlay(),
        population_overlay(),
    ])
}

// ---------------------------------------------------------------------------
// Tests — `FR-CIV-INFOVIEW-901` acceptance.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `FR-CIV-INFOVIEW-901` §AC #1 — registering an overlay makes it
    /// retrievable by id.
    #[test]
    fn registering_overlay_makes_it_retrievable_by_id() {
        let mut registry = OverlayRegistry::new();
        let elev = elevation_overlay();
        let id_str = elev.id.as_str().to_owned();
        assert!(registry.register(elev));
        assert!(registry.contains(&id_str));
        let retrieved = registry.get(&id_str);
        assert!(retrieved.is_some(), "overlay not retrievable by id");
        assert_eq!(retrieved.unwrap().id.as_str(), "elevation");
    }

    /// `FR-CIV-INFOVIEW-901` §AC #1 — duplicate id is a no-op insert.
    #[test]
    fn duplicate_registration_is_rejected() {
        let mut registry = OverlayRegistry::new();
        assert!(registry.register(elevation_overlay()));
        assert!(!registry.register(elevation_overlay()));
        assert_eq!(registry.len(), 1);
    }

    /// `FR-CIV-INFOVIEW-901` §AC #1 — `upsert` replaces an existing entry
    /// and reports replacement.
    #[test]
    fn upsert_replaces_existing() {
        let mut registry = OverlayRegistry::new();
        assert!(!registry.upsert(elevation_overlay()));
        assert!(registry.upsert(water_overlay())); // replaces elevation
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("water"));
        assert!(!registry.contains("elevation"));
    }

    /// `FR-CIV-INFOVIEW-901` §AC #1 — `remove` drops by id.
    #[test]
    fn remove_drops_by_id() {
        let mut registry = OverlayRegistry::new();
        registry.register(elevation_overlay());
        registry.register(water_overlay());
        assert!(registry.remove("elevation"));
        assert!(!registry.contains("elevation"));
        assert!(!registry.remove("elevation"));
        assert_eq!(registry.len(), 1);
    }

    /// `FR-CIV-INFOVIEW-901` §AC #1 — `with_overlays` pre-populates and
    /// `ids()` reports registration order.
    #[test]
    fn ids_reflect_registration_order() {
        let registry = OverlayRegistry::with_overlays(vec![
            water_overlay(),
            population_overlay(),
            elevation_overlay(),
        ]);
        assert_eq!(
            registry.ids(),
            vec!["water", "population", "elevation"]
        );
    }

    /// `FR-CIV-INFOVIEW-901` §AC #2 — color-ramp maps a value to a color:
    /// exact stops are precise, in-between stops interpolate, out-of-range
    /// values clamp.
    #[test]
    fn color_ramp_maps_value_to_color() {
        let ramp = elevation_overlay().ramp;
        // Below the first stop → clamps to first.
        let lo = ramp.map(-1.0);
        assert_eq!(lo, ramp.stops[0].rgb);
        // Above the last stop → clamps to last.
        let hi = ramp.map(2.0);
        assert_eq!(hi, ramp.stops[ramp.stops.len() - 1].rgb);
        // Between two stops → linearly interpolated.
        let mid = ramp.map(0.5);
        let lowland = ramp.stops[2].rgb;
        let highland = ramp.stops[3].rgb;
        assert!(
            mid[0] > lowland[0] && mid[0] < highland[0],
            "mid should blend between Lowland and Highland: got {mid:?}"
        );
    }

    /// `FR-CIV-INFOVIEW-901` §AC #2 — empty ramp returns neutral grey.
    #[test]
    fn empty_ramp_returns_neutral_grey() {
        let ramp = ColorRamp::from_stops(vec![]);
        assert_eq!(ramp.map(0.0), [0.5, 0.5, 0.5]);
        assert_eq!(ramp.map(1.0), [0.5, 0.5, 0.5]);
    }

    /// `FR-CIV-INFOVIEW-901` §AC #2 — single-stop ramp is constant.
    #[test]
    fn single_stop_ramp_is_constant() {
        let ramp = ColorRamp::from_stops(vec![LegendStop::new(0.5, "Mid", [0.1, 0.2, 0.3])]);
        assert_eq!(ramp.map(0.0), [0.1, 0.2, 0.3]);
        assert_eq!(ramp.map(0.5), [0.1, 0.2, 0.3]);
        assert_eq!(ramp.map(1.0), [0.1, 0.2, 0.3]);
    }

    /// `FR-CIV-INFOVIEW-901` §AC #2 — `from_stops` sorts ascending by `at`.
    #[test]
    fn from_stops_sorts_ascending() {
        let ramp = ColorRamp::from_stops(vec![
            LegendStop::new(0.7, "High", [1.0, 1.0, 1.0]),
            LegendStop::new(0.0, "Low", [0.0, 0.0, 0.0]),
            LegendStop::new(0.4, "Mid", [0.5, 0.5, 0.5]),
        ]);
        assert_eq!(ramp.stops[0].at, 0.0);
        assert_eq!(ramp.stops[1].at, 0.4);
        assert_eq!(ramp.stops[2].at, 0.7);
    }

    /// `FR-CIV-INFOVIEW-901` — new overlays addable without code changes.
    /// Building a fresh registry with a custom overlay round-trips through
    /// the public API.
    #[test]
    fn custom_overlay_registers_without_fork() {
        let mut registry = OverlayRegistry::new();
        let def = OverlayDef::new(
            "soil-fertility",
            "Soil Fertility",
            DataSource::Custom("civ_economy.fertility".to_owned()),
            ColorRamp::from_stops(vec![
                LegendStop::new(0.0, "Barren", [0.4, 0.3, 0.2]),
                LegendStop::new(0.5, "Loam", [0.3, 0.6, 0.2]),
                LegendStop::new(1.0, "Rich", [0.1, 0.4, 0.1]),
            ]),
        );
        assert!(registry.register(def));
        let got = registry
            .get("soil-fertility")
            .expect("registered but not retrievable");
        assert_eq!(got.label, "Soil Fertility");
        assert_eq!(
            got.source,
            DataSource::Custom("civ_economy.fertility".to_owned())
        );
        // Color-ramp maps a value to a color for the new overlay too.
        let mid = got.ramp.map(0.5);
        assert_eq!(mid, [0.3, 0.6, 0.2]);
    }

    /// `FR-CIV-INFOVIEW-901` — the default registry ships the seed
    /// overlays; iteration order matches registration order.
    #[test]
    fn default_registry_seeds_known_overlays() {
        let registry = default_overlay_registry();
        for id in ["elevation", "water", "population"] {
            assert!(registry.contains(id), "default registry missing {id}");
        }
        assert_eq!(
            registry.ids(),
            vec!["elevation", "water", "population"]
        );
    }

    /// `FR-CIV-INFOVIEW-901` — `DataSource::tag` is stable.
    #[test]
    fn data_source_tag_is_stable() {
        assert_eq!(DataSource::TerrainHeight.tag(), "terrain-height");
        assert_eq!(DataSource::Population.tag(), "population");
        assert_eq!(DataSource::Custom("anything".to_owned()).tag(), "custom");
    }

    /// `FR-CIV-INFOVIEW-901` — `OverlayDef` is `serde`-round-trippable.
    #[test]
    fn overlay_def_round_trips_via_serde_json() {
        let def = elevation_overlay();
        let json = serde_json::to_string(&def).expect("serialize");
        let back: OverlayDef = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(def, back);
    }
}
