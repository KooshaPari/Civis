//! God-tool UI extensions — FR-CIV-UI-001, FR-CIV-UI-002, FR-CIV-UI-003.
//!
//! Per `docs/guides/voxel-emergent-vision-and-migration.md` §4.3 (P-VM-6),
//! the god-tool UI surface is split into three FRs:
//!
//! - **FR-CIV-UI-001** — Material brush tool: paint / erase / heat / cool
//!   with a configurable radius. Writes go through a CA command queue
//!   (deterministic; same input sequence ⇒ same world state).
//! - **FR-CIV-UI-002** — Condition overlay: toggleable heatmap overlays
//!   for temperature, material density, life density, and cluster
//!   membership rendered as transparent screen-space passes.
//! - **FR-CIV-UI-003** — Emergence notification: HUD toasts fired when a
//!   speciation, cluster formation, or proto-life event arrives.
//!   Dismissible; linked to the event log.
//!
//! This module is the HUD-side pure data layer for that surface. The
//! existing `godtool_brush` and `notifications` modules own the kernel
//! math and the FIFO queue respectively; this module adds the
//! god-tool-specific vocabulary (brush kinds, overlay modes, event
//! kinds) that hosts compose with the kernel / queue.
//!
//! ## Design contract
//!
//! 1. **Pure data, no engine.** All structs are `serde`-serialisable.
//! 2. **Additive only.** Does not modify the existing `civ_hud` surface.
//! 3. **Deterministic.** All enums are stable and ordered; iteration
//!    over [`BrushKind::ALL`] / [`ConditionOverlayMode::ALL`] /
//!    [`EmergenceNotificationKind::ALL`] is declaration-order, suitable
//!    for tests and host UI cycling.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// The verb a material brush applies — FR-CIV-UI-001.
///
/// Per the spec, the player can paint a material, erase a material, add
/// heat, or remove heat — every brush stamp must go through the CA
/// command queue (deterministic). Hosts choose a kind and the
/// `BrushKernelParams` (radius / strength / falloff / shape) from the
/// existing [`crate::godtool_brush`] kernel; the kernel multiplies the
/// kind-specific payload into the world write.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushKind {
    /// Paint a material into the affected cells. Deterministic: same
    /// payload sequence ⇒ identical world state.
    #[default]
    Paint,
    /// Erase a material from the affected cells (set to empty).
    Erase,
    /// Add heat (raise temperature). Compatible with the CA's heat
    /// transfer rules.
    Heat,
    /// Remove heat (cool temperature).
    Cool,
}

impl BrushKind {
    /// All brush kinds, declaration order. Used by host UIs that want
    /// to cycle through every kind in their tool palette.
    pub const ALL: [BrushKind; 4] = [
        BrushKind::Paint,
        BrushKind::Erase,
        BrushKind::Heat,
        BrushKind::Cool,
    ];

    /// Wire-stable identifier used in settings files and HUD config
    /// payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BrushKind::Paint => "paint",
            BrushKind::Erase => "erase",
            BrushKind::Heat => "heat",
            BrushKind::Cool => "cool",
        }
    }

    /// Inverse of [`Self::as_str`]. Returns `None` for unknown wire ids.
    #[must_use]
    pub fn parse_wire(s: &str) -> Option<Self> {
        match s {
            "paint" => Some(Self::Paint),
            "erase" => Some(Self::Erase),
            "heat" => Some(Self::Heat),
            "cool" => Some(Self::Cool),
            _ => None,
        }
    }
}

/// One entry in the material palette — FR-CIV-UI-001.
///
/// Hosts enumerate every available material as a [`MaterialPalette`]
/// and let the player pick one before stamping a [`BrushKind::Paint`]
/// brush. The `id` is stable across runs (matches the RON material
/// table id in `crates/voxel/src/material.rs`); the `label` is the
/// human-readable name shown in the palette UI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialEntry {
    /// Stable material id (e.g. `"stone"`, `"water"`, `"sand"`).
    pub id: String,
    /// Human-readable label shown in the palette UI.
    pub label: String,
    /// RGB sample for the palette swatch (matches the default palette's
    /// faction-color scheme). Optional — some materials are colorless.
    pub swatch: Option<(u8, u8, u8)>,
}

impl MaterialEntry {
    /// Construct an entry with no swatch.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            swatch: None,
        }
    }

    /// Construct an entry with an 8-bit RGB swatch.
    #[must_use]
    pub fn with_swatch(
        id: impl Into<String>,
        label: impl Into<String>,
        r: u8,
        g: u8,
        b: u8,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            swatch: Some((r, g, b)),
        }
    }
}

/// The full material palette available to the god-tool — FR-CIV-UI-001.
///
/// Hosts populate this once at startup (from the substrate's material
/// RON), then read [`Self::entry`] / [`Self::ids`] on every palette
/// render. Writes through the brush go through the CA command queue
/// (deterministic) — this struct is read-only on the hot path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialPalette {
    entries: Vec<MaterialEntry>,
}

impl MaterialPalette {
    /// Construct an empty palette.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entry. Order is preserved (palette UI iterates in
    /// insertion order; tests rely on declaration order).
    pub fn push(&mut self, entry: MaterialEntry) {
        self.entries.push(entry);
    }

    /// Number of entries in the palette.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the palette has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries, in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[MaterialEntry] {
        &self.entries
    }

    /// All material ids, in insertion order.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.id.as_str()).collect()
    }

    /// Look up an entry by material id. Returns `None` for unknown ids.
    #[must_use]
    pub fn entry(&self, id: &str) -> Option<&MaterialEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

/// The condition overlay modes — FR-CIV-UI-002.
///
/// Per the spec, the player can toggle heatmap overlays for
/// temperature, material density, life density, and cluster
/// membership. Every variant renders as a transparent screen-space
/// pass; multiple modes may compose (e.g. "Temperature + Cluster").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOverlayMode {
    /// No overlay rendered. Default.
    #[default]
    Off,
    /// Heatmap of per-cell temperature.
    Temperature,
    /// Heatmap of per-cell material density.
    MaterialDensity,
    /// Heatmap of per-cell life density (agent count).
    LifeDensity,
    /// Overlay tinting each cell by its `SocialCluster` membership.
    ClusterMembership,
}

impl ConditionOverlayMode {
    /// All modes, declaration order. `Off` is index 0 (the default).
    pub const ALL: [ConditionOverlayMode; 5] = [
        ConditionOverlayMode::Off,
        ConditionOverlayMode::Temperature,
        ConditionOverlayMode::MaterialDensity,
        ConditionOverlayMode::LifeDensity,
        ConditionOverlayMode::ClusterMembership,
    ];

    /// Wire-stable identifier used in HUD config payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ConditionOverlayMode::Off => "off",
            ConditionOverlayMode::Temperature => "temperature",
            ConditionOverlayMode::MaterialDensity => "material_density",
            ConditionOverlayMode::LifeDensity => "life_density",
            ConditionOverlayMode::ClusterMembership => "cluster_membership",
        }
    }

    /// Inverse of [`Self::as_str`]. Returns `None` for unknown wire ids.
    #[must_use]
    pub fn parse_wire(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Self::Off),
            "temperature" => Some(Self::Temperature),
            "material_density" => Some(Self::MaterialDensity),
            "life_density" => Some(Self::LifeDensity),
            "cluster_membership" => Some(Self::ClusterMembership),
            _ => None,
        }
    }

    /// True when the mode is the "no overlay" sentinel. Used by the
    /// host UI to short-circuit the overlay render pass entirely.
    #[must_use]
    pub const fn is_off(self) -> bool {
        matches!(self, ConditionOverlayMode::Off)
    }

    /// Number of active modes (everything except [`Off`]). Useful for
    /// tests asserting the spec's "≥4 active modes" contract.
    #[must_use]
    pub const fn active_count() -> usize {
        Self::ALL.len() - 1
    }
}

/// The kind of emergence event that fires an HUD toast — FR-CIV-UI-003.
///
/// Per the spec, three kinds of events must produce HUD notifications:
///
/// - **Speciation** — a new `SpeciesRecord` is added to the
///   `SpeciesRegistry`.
/// - **Cluster formation** — a new `SocialCluster` is created (any
///   kind: kinship / cultural / territorial).
/// - **Proto-life** — a `proto_life` event fires from the abiogenesis
///   threshold system.
///
/// Hosts wire each variant to its notification sink; the
/// [`crate::notifications::NotificationQueue`] is the FIFO recipient.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmergenceNotificationKind {
    /// Speciation — a new `SpeciesRecord` was created. Default is the
    /// most common (kinship-driven speciation events).
    #[default]
    Speciation,
    /// Cluster formation — a new `SocialCluster` was created (any kind).
    ClusterFormation,
    /// Proto-life — a `proto_life` event fired from abiogenesis.
    ProtoLife,
}

impl EmergenceNotificationKind {
    /// All kinds, declaration order.
    pub const ALL: [EmergenceNotificationKind; 3] = [
        EmergenceNotificationKind::Speciation,
        EmergenceNotificationKind::ClusterFormation,
        EmergenceNotificationKind::ProtoLife,
    ];

    /// Default label per kind — used by the host to seed a
    /// [`crate::notifications::Notification`] when an event fires.
    #[must_use]
    pub const fn default_label(self) -> &'static str {
        match self {
            EmergenceNotificationKind::Speciation => "speciation",
            EmergenceNotificationKind::ClusterFormation => "cluster.formed",
            EmergenceNotificationKind::ProtoLife => "proto.life",
        }
    }
}

/// Counts of how many emergence notifications have fired, by kind —
/// FR-CIV-UI-003.
///
/// Hosts increment the appropriate counter when an event arrives;
/// tests assert the count rises deterministically across a fixture
/// scenario run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmergenceNotificationCounters {
    /// Speciation counter.
    pub speciation: u64,
    /// Cluster formation counter.
    pub cluster_formation: u64,
    /// Proto-life counter.
    pub proto_life: u64,
}

impl EmergenceNotificationCounters {
    /// All counters start at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter for `kind`. Returns the new value.
    pub fn bump(&mut self, kind: EmergenceNotificationKind) -> u64 {
        let slot = match kind {
            EmergenceNotificationKind::Speciation => &mut self.speciation,
            EmergenceNotificationKind::ClusterFormation => &mut self.cluster_formation,
            EmergenceNotificationKind::ProtoLife => &mut self.proto_life,
        };
        *slot = slot.saturating_add(1);
        *slot
    }

    /// Total notifications fired across all kinds.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.speciation + self.cluster_formation + self.proto_life
    }

    /// Read the counter for `kind` without mutating.
    #[must_use]
    pub fn get(&self, kind: EmergenceNotificationKind) -> u64 {
        match kind {
            EmergenceNotificationKind::Speciation => self.speciation,
            EmergenceNotificationKind::ClusterFormation => self.cluster_formation,
            EmergenceNotificationKind::ProtoLife => self.proto_life,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-CIV-UI-001 -----------------------------------------------------------------------

    /// FR-CIV-UI-001: brush kinds are stable, distinct, and the wire
    /// round-trip is lossless.
    #[test]
    fn brush_kind_round_trip_and_order() {
        for kind in BrushKind::ALL {
            let s = kind.as_str();
            assert_eq!(BrushKind::parse_wire(s), Some(kind));
        }
        // Unknown wire ids are rejected.
        assert!(BrushKind::parse_wire("unknown").is_none());
    }

    /// FR-CIV-UI-001: a populated palette iterates in insertion order
    /// and lookups are O(n) but correct.
    #[test]
    fn material_palette_insertion_order_and_lookup() {
        let mut pal = MaterialPalette::new();
        pal.push(MaterialEntry::with_swatch("stone", "Stone", 128, 128, 128));
        pal.push(MaterialEntry::with_swatch("water", "Water", 30, 80, 200));
        pal.push(MaterialEntry::new("sand", "Sand"));

        let ids = pal.ids();
        assert_eq!(ids, vec!["stone", "water", "sand"]);
        assert_eq!(pal.len(), 3);
        assert!(pal.entry("water").is_some());
        assert!(pal.entry("missing").is_none());
    }

    // FR-CIV-UI-002 -----------------------------------------------------------------------

    /// FR-CIV-UI-002: condition overlay modes cover the spec's four
    /// required axes (temperature / material / life / cluster) plus the
    /// `Off` sentinel.
    #[test]
    fn condition_overlay_modes_cover_spec_axes() {
        let modes: std::collections::HashSet<&'static str> = ConditionOverlayMode::ALL
            .iter()
            .map(|m| m.as_str())
            .collect();
        assert!(modes.contains("off"));
        assert!(modes.contains("temperature"));
        assert!(modes.contains("material_density"));
        assert!(modes.contains("life_density"));
        assert!(modes.contains("cluster_membership"));
        // The spec requires ≥4 active modes (everything except Off).
        assert!(ConditionOverlayMode::active_count() >= 4);
    }

    /// `Off` is the default and short-circuits the overlay render pass.
    #[test]
    fn condition_overlay_off_is_default_and_sentinel() {
        assert_eq!(ConditionOverlayMode::default(), ConditionOverlayMode::Off);
        assert!(ConditionOverlayMode::Off.is_off());
        assert!(!ConditionOverlayMode::Temperature.is_off());
    }

    // FR-CIV-UI-003 -----------------------------------------------------------------------

    /// FR-CIV-UI-003: the three required event kinds are exposed and
    /// counters track each independently.
    #[test]
    fn emergence_notification_kinds_and_counters() {
        assert_eq!(EmergenceNotificationKind::ALL.len(), 3);

        let mut c = EmergenceNotificationCounters::new();
        assert_eq!(c.total(), 0);
        assert_eq!(c.bump(EmergenceNotificationKind::Speciation), 1);
        assert_eq!(c.bump(EmergenceNotificationKind::Speciation), 2);
        assert_eq!(c.bump(EmergenceNotificationKind::ClusterFormation), 1);
        assert_eq!(c.bump(EmergenceNotificationKind::ProtoLife), 1);
        assert_eq!(c.total(), 4);
        assert_eq!(c.get(EmergenceNotificationKind::Speciation), 2);
        assert_eq!(c.get(EmergenceNotificationKind::ClusterFormation), 1);
        assert_eq!(c.get(EmergenceNotificationKind::ProtoLife), 1);
    }
}