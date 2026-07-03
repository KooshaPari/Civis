//! Data/query layer for the overlay legend — `FR-CIV-INFOVIEW-920`.
//!
//! Per `FR-CIV-INFOVIEW-920`: **each overlay exposes a legend (scale min/max
//! + units) updating with live values.** This module is the substrate-neutral
//! data structure that any host client (Bevy / Godot / Unreal / web) reads
//! to render the legend strip beside an info-view.
//!
//! ## Shape
//!
//! ```text
//! OverlayLegend {
//!     overlay_id:   OverlayId,        // ties legend back to OverlayDef
//!     title:        String,           // legend header text
//!     units:        String,           // SI / canonical unit string (e.g. "m")
//!     scale_min:    f32,              // low end of the live value range
//!     scale_max:    f32,              // high end of the live value range
//!     stops:        Vec<LegendTick>,  // tick marks shown on the legend bar
//!     live_value:   Option<f32>,      // optional current sampled value
//! }
//! ```
//!
//! ## Acceptance
//!
//! `FR-CIV-INFOVIEW-920` acceptance test: **"legend reports correct
//! min/max/units for a value range"**. The acceptance test is satisfied by:
//!
//! 1. Constructing a legend with explicit `scale_min`, `scale_max`, `units`.
//! 2. Updating the live range via [`OverlayLegend::update_range`].
//! 3. Querying back via [`OverlayLegend::scale_min`], [`OverlayLegend::scale_max`],
//!    [`OverlayLegend::units`], and confirming the values match.
//!
//! This module is deliberately **render-free**: no Bevy types, no systems, no
//! plugins, no GPU. It is the substrate-neutral vocabulary that any host can
//! read to render its own legend strip. It composes with `overlay_registry`
//! (the legend's `overlay_id` keys into the same `OverlayDef` table) and is
//! designed for additive registration — new overlays add legends by
//! registering a new [`OverlayLegend`] via the [`LegendRegistry`]; no
//! render-code fork.

use serde::{Deserialize, Serialize};

use crate::overlay_registry::{OverlayId, OverlayRegistry};

// ---------------------------------------------------------------------------
// Legend tick — a labelled position on the legend bar.
// ---------------------------------------------------------------------------

/// One labelled tick on the legend bar.
///
/// Ticks are placed at a normalised position `at` (`0.0` = low end,
/// `1.0` = high end) and carry a human-readable label (e.g. `"500 m"`,
/// `"Sea level"`). Hosts render the tick as a vertical mark + label on the
/// legend strip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegendTick {
    /// Normalised position on the legend bar (`0.0..=1.0`).
    pub at: f32,
    /// Human-readable label for this tick.
    pub label: String,
    /// Optional associated value in the overlay's units (e.g. `0.0` for
    /// `"Sea level"` on the elevation overlay). `None` for purely
    /// qualitative labels.
    pub value: Option<f32>,
}

impl LegendTick {
    /// Construct a qualitative tick (no associated value).
    #[must_use]
    pub fn new(at: f32, label: &str) -> Self {
        Self {
            at,
            label: label.to_owned(),
            value: None,
        }
    }

    /// Construct a tick with an associated value in the overlay's units.
    #[must_use]
    pub fn with_value(at: f32, label: &str, value: f32) -> Self {
        Self {
            at,
            label: label.to_owned(),
            value: Some(value),
        }
    }
}

// ---------------------------------------------------------------------------
// Overlay legend — `FR-CIV-INFOVIEW-920` core record.
// ---------------------------------------------------------------------------

/// The legend strip for a single overlay.
///
/// Every legend carries an `overlay_id` so it keys back into the
/// [`OverlayRegistry`], plus the **scale** (min/max value + units) that maps
/// the legend bar to the overlay's measured field. The scale is mutable via
/// [`update_range`](Self::update_range) and [`update_units`](Self::update_units)
/// so the legend can track live values without rebuilding the record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlayLegend {
    /// Overlay this legend describes. Keys back into `OverlayRegistry::get`.
    pub overlay_id: OverlayId,
    /// Human-readable header shown above the legend strip (e.g. `"Elevation"`).
    pub title: String,
    /// Canonical units string (e.g. `"m"`, `"people/km²"`, `"°C"`). Empty
    /// when the overlay is unitless.
    pub units: String,
    /// Low end of the live value range.
    pub scale_min: f32,
    /// High end of the live value range.
    pub scale_max: f32,
    /// Ticks shown on the legend bar (low → high).
    pub stops: Vec<LegendTick>,
    /// Optional current sampled value (the live cursor that tracks the
    /// value under the user's inspect probe, if any).
    pub live_value: Option<f32>,
}

impl OverlayLegend {
    /// Construct a legend with a scale range and units string. The id
    /// widens via `OverlayId::new`; title and units widen to `String` so the
    /// value is `serde`-friendly. `stops` may be empty — hosts render a
    /// bare gradient bar in that case.
    #[must_use]
    pub fn new(
        overlay_id: &'static str,
        title: &'static str,
        units: &'static str,
        scale_min: f32,
        scale_max: f32,
        stops: Vec<LegendTick>,
    ) -> Self {
        Self {
            overlay_id: OverlayId::new(overlay_id),
            title: title.to_owned(),
            units: units.to_owned(),
            scale_min,
            scale_max,
            stops,
            live_value: None,
        }
    }

    /// Borrow the overlay id this legend describes.
    #[must_use]
    pub fn overlay_id_str(&self) -> &str {
        self.overlay_id.as_str()
    }

    /// Borrow the legend header text.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the units string (empty when unitless).
    #[must_use]
    pub fn units(&self) -> &str {
        &self.units
    }

    /// Low end of the live value range.
    #[must_use]
    pub fn scale_min(&self) -> f32 {
        self.scale_min
    }

    /// High end of the live value range.
    #[must_use]
    pub fn scale_max(&self) -> f32 {
        self.scale_max
    }

    /// Width of the live value range (`scale_max - scale_min`). Always
    /// non-negative: a degenerate / inverted range is collapsed to `0.0`.
    #[must_use]
    pub fn range_span(&self) -> f32 {
        (self.scale_max - self.scale_min).max(0.0)
    }

    /// Normalise a value in the overlay's units to `0.0..=1.0` against the
    /// current scale. Out-of-range values are **clamped**. Returns `0.0`
    /// when the range is degenerate (`scale_min == scale_max`).
    #[must_use]
    pub fn normalised(&self, value: f32) -> f32 {
        let span = self.range_span();
        if span <= f32::EPSILON {
            return 0.0;
        }
        ((value - self.scale_min) / span).clamp(0.0, 1.0)
    }

    /// Update the live value range. Hosts call this as new samples stream
    /// in (e.g. running min/max across the visible cells). If `min > max`
    /// after the update, the values are silently swapped so the legend
    /// stays consistent (`scale_min <= scale_max` invariant).
    pub fn update_range(&mut self, min: f32, max: f32) {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        self.scale_min = lo;
        self.scale_max = hi;
    }

    /// Update the canonical units string (e.g. when the overlay switches
    /// between metric / imperial or when a downstream crate redefines the
    /// measured field's unit). Pass `""` for unitless overlays.
    pub fn update_units(&mut self, units: &str) {
        self.units = units.to_owned();
    }

    /// Set the live cursor value (e.g. the value at the user's inspect
    /// probe). `None` clears the cursor.
    pub fn set_live_value(&mut self, value: Option<f32>) {
        self.live_value = value;
    }

    /// Number of ticks on the legend bar.
    #[must_use]
    pub fn tick_count(&self) -> usize {
        self.stops.len()
    }

    /// `true` when the legend has zero ticks (hosts may render a bare
    /// gradient bar in this case).
    #[must_use]
    pub fn ticks_is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// Iterate the ticks in registration order.
    pub fn iter_ticks(&self) -> std::slice::Iter<'_, LegendTick> {
        self.stops.iter()
    }

    /// `true` when `scale_min <= scale_max` — the legend's range invariant
    /// holds. Always true after construction or [`update_range`](Self::update_range).
    #[must_use]
    pub fn range_is_valid(&self) -> bool {
        self.scale_min <= self.scale_max
    }
}

// ---------------------------------------------------------------------------
// Legend registry — data-driven lookup of legends by overlay id.
// ---------------------------------------------------------------------------

/// Data-driven registry of overlay legends.
///
/// Mirrors [`OverlayRegistry`]: hosts construct a `LegendRegistry`, register
/// one [`OverlayLegend`] per overlay they want a legend for, and query the
/// result by overlay id. The two registries are intentionally decoupled —
/// a host may register an overlay without a legend (legend hidden) or a
/// legend without a backing `OverlayDef` (legend shown standalone).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LegendRegistry {
    legends: Vec<OverlayLegend>,
}

impl LegendRegistry {
    /// Construct an empty legend registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a registry pre-populated with the given legends, in
    /// registration order. Duplicates by `overlay_id` are not
    /// deduplicated; the first match wins on [`get`](Self::get).
    #[must_use]
    pub fn with_legends(legends: Vec<OverlayLegend>) -> Self {
        Self { legends }
    }

    /// Number of registered legends.
    #[must_use]
    pub fn len(&self) -> usize {
        self.legends.len()
    }

    /// `true` when no legends are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.legends.is_empty()
    }

    /// Register a legend. Returns `true` on insert, `false` when a legend
    /// for the same `overlay_id` was already present (no-op insert, no
    /// overwrite — callers that want to replace should
    /// [`upsert`](Self::upsert) or `remove_for` first).
    pub fn register(&mut self, legend: OverlayLegend) -> bool {
        if self
            .legends
            .iter()
            .any(|l| l.overlay_id == legend.overlay_id)
        {
            return false;
        }
        self.legends.push(legend);
        true
    }

    /// Replace the legend with the same `overlay_id` (inserting if absent).
    /// Returns `true` if an existing legend was replaced, `false` if it was
    /// newly inserted.
    pub fn upsert(&mut self, legend: OverlayLegend) -> bool {
        if let Some(slot) = self
            .legends
            .iter_mut()
            .find(|l| l.overlay_id == legend.overlay_id)
        {
            *slot = legend;
            true
        } else {
            self.legends.push(legend);
            false
        }
    }

    /// Remove the legend for the given overlay id. Returns `true` if one
    /// was removed.
    pub fn remove_for(&mut self, overlay_id: &str) -> bool {
        if let Some(i) = self
            .legends
            .iter()
            .position(|l| l.overlay_id.as_str() == overlay_id)
        {
            self.legends.remove(i);
            true
        } else {
            false
        }
    }

    /// Borrow the legend for the given overlay id, if registered.
    #[must_use]
    pub fn get(&self, overlay_id: &str) -> Option<&OverlayLegend> {
        self.legends
            .iter()
            .find(|l| l.overlay_id.as_str() == overlay_id)
    }

    /// Mutably borrow the legend for the given overlay id, if registered.
    /// Hosts use this to push live range / units updates into the registry
    /// without round-tripping through `remove_for` + `register`.
    #[must_use]
    pub fn get_mut(&mut self, overlay_id: &str) -> Option<&mut OverlayLegend> {
        self.legends
            .iter_mut()
            .find(|l| l.overlay_id.as_str() == overlay_id)
    }

    /// Push a live range update into the legend for the given overlay id.
    /// Returns `true` if a legend was found and updated.
    pub fn push_range_update(&mut self, overlay_id: &str, min: f32, max: f32) -> bool {
        if let Some(legend) = self.get_mut(overlay_id) {
            legend.update_range(min, max);
            true
        } else {
            false
        }
    }

    /// Iterate all registered legends in registration order.
    pub fn iter(&self) -> std::slice::Iter<'_, OverlayLegend> {
        self.legends.iter()
    }

    /// Borrow the list of registered overlay ids in registration order.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.legends.iter().map(|l| l.overlay_id.as_str()).collect()
    }

    /// `true` if a legend for the given overlay id is registered.
    #[must_use]
    pub fn contains(&self, overlay_id: &str) -> bool {
        self.get(overlay_id).is_some()
    }

    /// Filter the legend registry against an [`OverlayRegistry`], keeping
    /// only legends whose overlay id is present in the overlay registry.
    /// Useful for hosts that want to drive legend visibility from the same
    /// toggle list as the overlays themselves.
    #[must_use]
    pub fn prune_to_overlays(&self, overlays: &OverlayRegistry) -> Vec<&OverlayLegend> {
        self.legends
            .iter()
            .filter(|l| overlays.contains(l.overlay_id.as_str()))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Seed definitions — the legends that ship with the default registry,
// paired with the default `OverlayRegistry` seeds.
// ---------------------------------------------------------------------------

/// `elevation` legend — terrain height in metres above sea level.
#[must_use]
pub fn elevation_legend() -> OverlayLegend {
    OverlayLegend::new(
        "elevation",
        "Elevation",
        "m",
        -50.0,
        3000.0,
        vec![
            LegendTick::with_value(0.0, "Sea level", 0.0),
            LegendTick::with_value(0.25, "500 m", 500.0),
            LegendTick::with_value(0.5, "1000 m", 1000.0),
            LegendTick::with_value(0.75, "2000 m", 2000.0),
            LegendTick::with_value(1.0, "3000 m", 3000.0),
        ],
    )
}

/// `water` legend — water depth in metres (0 = surface, +N = submerged).
#[must_use]
pub fn water_legend() -> OverlayLegend {
    OverlayLegend::new(
        "water",
        "Water",
        "m",
        0.0,
        100.0,
        vec![
            LegendTick::with_value(0.0, "Surface", 0.0),
            LegendTick::with_value(0.5, "50 m", 50.0),
            LegendTick::with_value(1.0, "100 m", 100.0),
        ],
    )
}

/// `population` legend — agent density in people per km².
#[must_use]
pub fn population_legend() -> OverlayLegend {
    OverlayLegend::new(
        "population",
        "Population",
        "people/km²",
        0.0,
        5000.0,
        vec![
            LegendTick::new(0.0, "Empty"),
            LegendTick::with_value(0.25, "Suburban", 1250.0),
            LegendTick::with_value(0.5, "Urban", 2500.0),
            LegendTick::with_value(0.75, "Dense", 3750.0),
            LegendTick::with_value(1.0, "Crowded", 5000.0),
        ],
    )
}

/// The default legend registry: one legend per seed overlay in shipping
/// order. Hosts add more legends via `registry.register(...)` on top.
#[must_use]
pub fn default_legend_registry() -> LegendRegistry {
    LegendRegistry::with_legends(vec![
        elevation_legend(),
        water_legend(),
        population_legend(),
    ])
}

// ---------------------------------------------------------------------------
// Tests — `FR-CIV-INFOVIEW-920` acceptance.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `FR-CIV-INFOVIEW-920` acceptance test — **legend reports correct
    /// min/max/units for a value range**. Construct a legend, update the
    /// range via the public API, and confirm the getters report the same
    /// values back. Units are checked explicitly to lock the FR contract.
    #[test]
    fn legend_reports_correct_min_max_units_for_value_range() {
        let legend = OverlayLegend::new(
            "elevation",
            "Elevation",
            "m",
            -100.0,
            2500.0,
            vec![LegendTick::with_value(0.0, "Sea level", 0.0)],
        );

        // Initial range is exactly what the constructor was given.
        assert_eq!(legend.scale_min(), -100.0);
        assert_eq!(legend.scale_max(), 2500.0);
        assert_eq!(legend.units(), "m");

        // Push a live update and confirm the legend tracks it.
        legend_clone_apply(&legend, |l| {
            l.update_range(-200.0, 3000.0);
        });

        // update_range mutates in place — use a fresh legend to assert.
        let mut live = OverlayLegend::new("elevation", "Elevation", "m", 0.0, 1.0, vec![]);
        live.update_range(-200.0, 3000.0);
        assert_eq!(live.scale_min(), -200.0);
        assert_eq!(live.scale_max(), 3000.0);
        assert_eq!(live.units(), "m");
    }

    /// Helper-free rewrite of the live-update assertion (kept as a separate
    /// test so the failure messages are tight and don't depend on the
    /// `legend_clone_apply` helper). `FR-CIV-INFOVIEW-920`.
    #[test]
    fn legend_update_range_tracks_live_values() {
        let mut legend = OverlayLegend::new("water", "Water", "m", 0.0, 100.0, vec![]);
        legend.update_range(-10.0, 250.0);
        assert_eq!(legend.scale_min(), -10.0);
        assert_eq!(legend.scale_max(), 250.0);
        assert_eq!(legend.units(), "m");
        // Inverted inputs are silently swapped so the invariant holds.
        legend.update_range(500.0, 50.0);
        assert_eq!(legend.scale_min(), 50.0);
        assert_eq!(legend.scale_max(), 500.0);
        assert!(legend.range_is_valid());
    }

    /// `FR-CIV-INFOVIEW-920` — updating units on a live legend reflects on
    /// the public getter.
    #[test]
    fn legend_update_units_reflects_on_getter() {
        let mut legend =
            OverlayLegend::new("population", "Population", "people/km²", 0.0, 1.0, vec![]);
        assert_eq!(legend.units(), "people/km²");
        legend.update_units("ppl/km^2");
        assert_eq!(legend.units(), "ppl/km^2");
        legend.update_units("");
        assert_eq!(legend.units(), "");
    }

    /// `FR-CIV-INFOVIEW-920` — `normalised` clamps and handles a degenerate
    /// range without dividing by zero.
    #[test]
    fn legend_normalised_clamps_and_handles_degenerate_range() {
        let legend = OverlayLegend::new("elevation", "Elevation", "m", 0.0, 1000.0, vec![]);
        assert_eq!(legend.normalised(0.0), 0.0);
        assert_eq!(legend.normalised(1000.0), 1.0);
        assert_eq!(legend.normalised(500.0), 0.5);
        // Out-of-range clamps.
        assert_eq!(legend.normalised(-100.0), 0.0);
        assert_eq!(legend.normalised(5000.0), 1.0);
        // Degenerate range → 0.0 (no panic, no NaN).
        let flat = OverlayLegend::new("flat", "Flat", "", 7.0, 7.0, vec![]);
        assert_eq!(flat.normalised(7.0), 0.0);
        assert_eq!(flat.normalised(42.0), 0.0);
    }

    /// `FR-CIV-INFOVIEW-920` — `LegendRegistry` mirrors `OverlayRegistry`:
    /// register, get-by-id, upsert, remove-for, contains.
    #[test]
    fn legend_registry_mirrors_overlay_registry() {
        let mut legends = LegendRegistry::new();
        assert!(legends.register(elevation_legend()));
        assert!(!legends.register(elevation_legend())); // duplicate id is no-op
        assert!(legends.register(water_legend()));
        assert_eq!(legends.len(), 2);

        assert!(legends.contains("elevation"));
        let elev = legends.get("elevation").expect("registered but missing");
        assert_eq!(elev.units(), "m");
        assert_eq!(elev.scale_min(), -50.0);
        assert_eq!(elev.scale_max(), 3000.0);

        // upsert replaces existing entry and reports replacement.
        let mut replacement = elevation_legend();
        replacement.update_range(-100.0, 4000.0);
        assert!(legends.upsert(replacement));
        let back = legends.get("elevation").expect("post-upsert");
        assert_eq!(back.scale_min(), -100.0);
        assert_eq!(back.scale_max(), 4000.0);

        // remove_for drops by id.
        assert!(legends.remove_for("elevation"));
        assert!(!legends.contains("elevation"));
        assert!(!legends.remove_for("elevation"));
    }

    /// `FR-CIV-INFOVIEW-920` — `push_range_update` writes the new range
    /// into the registered legend without forcing the host to do a
    /// `get_mut` + range update + push back dance.
    #[test]
    fn push_range_update_propagates_to_registered_legend() {
        let mut legends = LegendRegistry::with_legends(vec![population_legend()]);
        assert!(legends.push_range_update("population", 0.0, 10000.0));
        let pop = legends.get("population").expect("registered");
        assert_eq!(pop.scale_min(), 0.0);
        assert_eq!(pop.scale_max(), 10000.0);
        assert!(!legends.push_range_update("ghost", 0.0, 1.0));
    }

    /// `FR-CIV-INFOVIEW-920` — `prune_to_overlays` keeps only legends whose
    /// overlay id is present in the supplied `OverlayRegistry`.
    #[test]
    fn prune_to_overlays_keeps_only_overlapping_ids() {
        let legends = default_legend_registry();
        let overlays = crate::overlay_registry::default_overlay_registry();
        let pruned = legends.prune_to_overlays(&overlays);
        // Default seeds pair 1:1 — every legend matches an overlay.
        assert_eq!(pruned.len(), legends.len());
        for legend in &pruned {
            assert!(overlays.contains(legend.overlay_id.as_str()));
        }

        // A registry with no overlays prunes everything.
        let empty = OverlayRegistry::new();
        assert!(legends.prune_to_overlays(&empty).is_empty());
    }

    /// `FR-CIV-INFOVIEW-920` — the default legend registry seeds the same
    /// three overlays as the default overlay registry, in matching order.
    #[test]
    fn default_legend_registry_matches_default_overlay_registry() {
        let legends = default_legend_registry();
        let overlays = crate::overlay_registry::default_overlay_registry();
        assert_eq!(legends.ids(), overlays.ids());
        for id in ["elevation", "water", "population"] {
            assert!(legends.contains(id), "default legends missing {id}");
            let legend = legends.get(id).expect("registered");
            let overlay = overlays.get(id).expect("registered");
            // The legend's `overlay_id` keys into the same `OverlayDef`.
            assert_eq!(legend.overlay_id, overlay.id);
        }
    }

    /// `FR-CIV-INFOVIEW-920` — `OverlayLegend` is `serde`-round-trippable so
    /// host clients can sync legends across the wire without bespoke glue.
    #[test]
    fn overlay_legend_round_trips_via_serde_json() {
        let legend = elevation_legend();
        let json = serde_json::to_string(&legend).expect("serialize");
        let back: OverlayLegend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(legend, back);
    }

    /// `FR-CIV-INFOVIEW-920` — `LegendTick` constructors expose both the
    /// qualitative and quantitative flavours.
    #[test]
    fn legend_tick_constructors() {
        let qualitative = LegendTick::new(0.5, "Mid");
        assert_eq!(qualitative.at, 0.5);
        assert_eq!(qualitative.label, "Mid");
        assert!(qualitative.value.is_none());

        let quantitative = LegendTick::with_value(0.5, "Mid", 250.0);
        assert_eq!(quantitative.value, Some(250.0));
    }

    /// `FR-CIV-INFOVIEW-920` — `range_span` collapses inverted / degenerate
    /// ranges to a non-negative value so callers never see a negative span.
    #[test]
    fn range_span_is_non_negative() {
        let ok = OverlayLegend::new("ok", "OK", "", 0.0, 100.0, vec![]);
        assert_eq!(ok.range_span(), 100.0);
        let flat = OverlayLegend::new("flat", "Flat", "", 50.0, 50.0, vec![]);
        assert_eq!(flat.range_span(), 0.0);
        let mut inverted = OverlayLegend::new("inv", "Inv", "", 100.0, 0.0, vec![]);
        inverted.update_range(100.0, 0.0);
        assert_eq!(inverted.range_span(), 100.0);
        assert!(inverted.range_is_valid());
    }

    /// Helper used by the acceptance test to apply a closure to a clone —
    /// keeps the assertion block free of `mut` shadowing noise without
    /// changing the test semantics.
    fn legend_clone_apply<F: FnOnce(&mut OverlayLegend)>(legend: &OverlayLegend, f: F) {
        let mut owned = legend.clone();
        f(&mut owned);
    }
}
