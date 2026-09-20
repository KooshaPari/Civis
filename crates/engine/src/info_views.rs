//! Info-view overlay types (FR-CIV-INFOVIEW-*).
//!
//! Data-driven overlay registry for CS2-class info-view overlays. Each overlay
//! is an [`InfoOverlay`] value (id, name, group, render_kind, availability,
//! legend). Adding a new overlay = appending one registration + one sampler.
//!
//! Charter alignment: overlays are read-only measurements over emergent data.
//! Categorical overlays derive color from emergent cluster ids only (no
//! authored taxonomy).

use std::collections::BTreeMap;

/// Six CS2-class overlay groups (FR-CIV-INFOVIEW-902).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OverlayGroup {
    /// Terrain / Environment overlays (elevation, water, temperature, etc.)
    Terrain,
    /// Population overlays (density, needs pressure, migration)
    Population,
    /// Economy overlays (wealth, prosperity, markets)
    Economy,
    /// Territory overlays (factions, borders, culture)
    Territory,
    /// Infrastructure overlays (roads, networks)
    Infrastructure,
    /// Hazard overlays (disasters, pollution)
    Hazard,
}

impl OverlayGroup {
    /// Human-readable display name for the UI accordion header.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Terrain => "Terrain",
            Self::Population => "Population",
            Self::Economy => "Economy",
            Self::Territory => "Territory",
            Self::Infrastructure => "Infrastructure",
            Self::Hazard => "Hazards",
        }
    }
}

/// Three render kinds dispatched from the overlay registry (FR-CIV-INFOVIEW-903).
/// New render kinds are additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderKind {
    /// Heatmap over the terrain lattice (default, reuses gizmo grid).
    LatticeRecolor,
    /// Lines/arrows/borders (roads, traffic flow, trade routes).
    Gizmo,
    /// Recolor agents/structures directly (happiness, health, wealth).
    EntityTint,
}

/// Data availability status for an overlay (FR-CIV-INFOVIEW-905).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAvailability {
    /// Data exists today (terrain/sim already computed).
    Live,
    /// Producing crate exists; field needs a thin read accessor.
    Near,
    /// Field is incomplete/absent; overlay specified + gated.
    Blind,
}

/// Legend kind for rendering (FR-CIV-INFOVIEW-901).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendKind {
    /// Continuous color ramp (elevation, temperature, etc.)
    Continuous,
    /// Categorical distinct colors (territory, culture, language).
    Categorical,
}

/// A single legend stop on a continuous ramp (FR-CIV-INFOVIEW-901).
#[derive(Debug, Clone, PartialEq)]
pub struct LegendStop {
    /// Position in [0.0, 1.0] along the ramp.
    pub position: f32,
    /// Human-readable label for this position.
    pub label: String,
    /// RGB color [0.0..1.0] per channel.
    pub color: [f32; 3],
}

/// A data-driven info overlay registration (FR-CIV-INFOVIEW-900, extended).
#[derive(Debug, Clone)]
pub struct InfoOverlay {
    /// Unique overlay identifier (stable across saves).
    pub id: &'static str,
    /// Human-readable name shown in the panel.
    pub name: &'static str,
    /// The CS2-class group this overlay belongs to.
    pub group: OverlayGroup,
    /// How the overlay renders.
    pub render_kind: RenderKind,
    /// Whether the producing data is live, near, or blind.
    pub availability: OverlayAvailability,
    /// Legend kind for the overlay.
    pub legend_kind: LegendKind,
    /// Legend stops for continuous ramps (empty for categorical).
    pub legend_stops: Vec<LegendStop>,
    /// Description text for hover/tooltip.
    pub description: &'static str,
}

/// Hash an emergent cluster id to a stable color (FR-CIV-INFOVIEW-904).
///
/// Categorical overlays MUST use this function instead of any authored taxonomy.
/// The hash ensures that color assignment is deterministic and derived purely
/// from the cluster id, never from a human-authored enum.
#[must_use]
pub fn cluster_color(cluster_id: u64) -> [f32; 3] {
    // Simple deterministic hash -> hue rotation via golden ratio
    let h = cluster_id as f32 * 0.618_034; // golden ratio conjugate
    let hue = (h - h.floor()) * 360.0;
    // HSV to RGB with S=0.7, V=0.9
    let s = 0.7;
    let v = 0.9;
    let c = v * s;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match hue as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// Hotkey bindings for the info-view toggle UX (FR-CIV-INFOVIEW-906).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoViewHotkeys {
    /// Cycle to next overlay.
    pub next: String,
    /// Cycle to previous overlay.
    pub prev: String,
    /// Toggle overlay on/off.
    pub toggle: String,
    /// Direct-select group 1..6.
    pub group_keys: Vec<String>,
    /// Close info-view panel.
    pub close: String,
    /// Toggle follow/camera focus.
    pub focus: String,
}

impl Default for InfoViewHotkeys {
    fn default() -> Self {
        Self {
            next: "Tab".into(),
            prev: "Shift+Tab".into(),
            toggle: "Backtick".into(),
            group_keys: vec![
                "1".into(), "2".into(), "3".into(),
                "4".into(), "5".into(), "6".into(),
            ],
            close: "Esc".into(),
            focus: "F".into(),
        }
    }
}

/// A complete overlay registry (FR-CIV-INFOVIEW-902, -930).
#[derive(Debug, Clone, Default)]
pub struct OverlayRegistry {
    overlays: Vec<InfoOverlay>,
}

impl OverlayRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an overlay (last registration for same id wins).
    pub fn register(&mut self, overlay: InfoOverlay) {
        self.overlays.retain(|o| o.id != overlay.id);
        self.overlays.push(overlay);
    }

    /// All registered overlays, ordered by registration.
    #[must_use]
    pub fn overlays(&self) -> &[InfoOverlay] {
        &self.overlays
    }

    /// Overlays filtered by group.
    #[must_use]
    pub fn overlays_in_group(&self, group: OverlayGroup) -> Vec<&InfoOverlay> {
        self.overlays.iter().filter(|o| o.group == group).collect()
    }

    /// Count overlays per group.
    #[must_use]
    pub fn group_counts(&self) -> BTreeMap<OverlayGroup, usize> {
        let mut counts = BTreeMap::new();
        for o in &self.overlays {
            *counts.entry(o.group).or_insert(0) += 1;
        }
        counts
    }

    /// Total number of registered overlays.
    #[must_use]
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }

    /// Find an overlay by id.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&InfoOverlay> {
        self.overlays.iter().find(|o| o.id == id)
    }
}

/// Priority-12 overlay definitions (FR-CIV-INFOVIEW-911..921).
///
/// These are the highest-value additions ranked by (legibility payoff x
/// data-availability) / effort. The first (elevation) is FR-CIV-INFOVIEW-910.
pub fn register_priority_12(registry: &mut OverlayRegistry) {
    // A2 - Water / Hydrology
    registry.register(InfoOverlay {
        id: "info_water",
        name: "Water / Hydrology",
        group: OverlayGroup::Terrain,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Dry".into(), color: [0.9, 0.8, 0.6] },
            LegendStop { position: 0.5, label: "Shallow".into(), color: [0.3, 0.6, 0.9] },
            LegendStop { position: 1.0, label: "Deep".into(), color: [0.1, 0.2, 0.7] },
        ],
        description: "Water depth and hydrology",
    });

    // A6 - Material / Surface
    registry.register(InfoOverlay {
        id: "info_material",
        name: "Material / Surface",
        group: OverlayGroup::Terrain,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Categorical,
        legend_stops: vec![],
        description: "Surface material composition",
    });

    // B1 - Population Density
    registry.register(InfoOverlay {
        id: "info_pop_density",
        name: "Population Density",
        group: OverlayGroup::Population,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Empty".into(), color: [0.1, 0.1, 0.1] },
            LegendStop { position: 1.0, label: "Dense".into(), color: [0.9, 0.2, 0.2] },
        ],
        description: "Agent density per lattice cell",
    });

    // B2 - Needs Pressure
    registry.register(InfoOverlay {
        id: "info_needs_pressure",
        name: "Needs Pressure",
        group: OverlayGroup::Population,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Satisfied".into(), color: [0.2, 0.8, 0.2] },
            LegendStop { position: 1.0, label: "Strained".into(), color: [0.9, 0.1, 0.1] },
        ],
        description: "Emergent unmet needs pressure",
    });

    // D1 - Territory
    registry.register(InfoOverlay {
        id: "info_territory",
        name: "Territory",
        group: OverlayGroup::Territory,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Categorical,
        legend_stops: vec![],
        description: "Emergent faction territory from cluster ids",
    });

    // A3 - Temperature
    registry.register(InfoOverlay {
        id: "info_temperature",
        name: "Temperature",
        group: OverlayGroup::Terrain,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Frozen".into(), color: [0.6, 0.8, 1.0] },
            LegendStop { position: 0.5, label: "Temperate".into(), color: [0.4, 0.8, 0.3] },
            LegendStop { position: 1.0, label: "Scorching".into(), color: [1.0, 0.3, 0.0] },
        ],
        description: "Surface temperature proxy",
    });

    // A8 - Resource Deposits
    registry.register(InfoOverlay {
        id: "info_resources",
        name: "Resource Deposits",
        group: OverlayGroup::Terrain,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Near,
        legend_kind: LegendKind::Categorical,
        legend_stops: vec![],
        description: "Strategic resource locations",
    });

    // E1 - Roads / Network
    registry.register(InfoOverlay {
        id: "info_roads",
        name: "Roads / Network",
        group: OverlayGroup::Infrastructure,
        render_kind: RenderKind::Gizmo,
        availability: OverlayAvailability::Near,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Light".into(), color: [0.6, 0.6, 0.6] },
            LegendStop { position: 1.0, label: "Heavy".into(), color: [0.2, 0.2, 0.2] },
        ],
        description: "Traffic graph overlay",
    });

    // C4 - Wealth / Prosperity
    registry.register(InfoOverlay {
        id: "info_wealth",
        name: "Wealth / Prosperity",
        group: OverlayGroup::Economy,
        render_kind: RenderKind::EntityTint,
        availability: OverlayAvailability::Near,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Poor".into(), color: [0.4, 0.2, 0.1] },
            LegendStop { position: 1.0, label: "Prosperous".into(), color: [1.0, 0.84, 0.0] },
        ],
        description: "Per-entity wealth tint",
    });

    // B6 - Migration Flow
    registry.register(InfoOverlay {
        id: "info_migration",
        name: "Migration Flow",
        group: OverlayGroup::Population,
        render_kind: RenderKind::Gizmo,
        availability: OverlayAvailability::Near,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Static".into(), color: [0.5, 0.5, 0.5] },
            LegendStop { position: 1.0, label: "High Flow".into(), color: [0.1, 0.5, 0.9] },
        ],
        description: "Agent migration arrows between settlements",
    });
}

/// Build the full 31-overlay catalog (FR-CIV-INFOVIEW-930).
///
/// Registers the Priority-12 overlays plus BLIND/Near placeholders for the
/// remaining overlays specified in the design doc (§3).
pub fn build_full_catalog() -> OverlayRegistry {
    let mut registry = OverlayRegistry::new();
    register_priority_12(&mut registry);

    // A1 - Elevation (the #1 priority, FR-CIV-INFOVIEW-910)
    registry.register(InfoOverlay {
        id: "info_elevation",
        name: "Elevation",
        group: OverlayGroup::Terrain,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Live,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![
            LegendStop { position: 0.0, label: "Deep".into(), color: [0.1, 0.1, 0.3] },
            LegendStop { position: 0.3, label: "Coast".into(), color: [0.3, 0.7, 0.9] },
            LegendStop { position: 0.5, label: "Lowland".into(), color: [0.3, 0.7, 0.3] },
            LegendStop { position: 0.7, label: "Highland".into(), color: [0.6, 0.5, 0.3] },
            LegendStop { position: 1.0, label: "Peak".into(), color: [0.9, 0.9, 0.9] },
        ],
        description: "Height above sea level",
    });

    // F1 - Disasters / Hazards (BLIND placeholder)
    registry.register(InfoOverlay {
        id: "info_disasters",
        name: "Disasters / Hazards",
        group: OverlayGroup::Hazard,
        render_kind: RenderKind::LatticeRecolor,
        availability: OverlayAvailability::Near,
        legend_kind: LegendKind::Continuous,
        legend_stops: vec![],
        description: "Active disaster/hazard zones",
    });

    // BLIND overlays (registered but greyed in panel)
    let blind_overlays = [
        ("info_culture", "Culture", OverlayGroup::Territory),
        ("info_language", "Language", OverlayGroup::Territory),
        ("info_ideology", "Ideology", OverlayGroup::Territory),
        ("info_market_type", "Market Type", OverlayGroup::Economy),
        ("info_pollution", "Pollution", OverlayGroup::Hazard),
        ("info_health", "Health", OverlayGroup::Population),
        ("info_happiness", "Happiness", OverlayGroup::Population),
        ("info_psychology", "Psychology", OverlayGroup::Population),
        ("info_trade_routes", "Trade Routes", OverlayGroup::Infrastructure),
        ("info_construction", "Construction", OverlayGroup::Infrastructure),
        ("info_energy_grid", "Energy Grid", OverlayGroup::Infrastructure),
        ("info_food_supply", "Food Supply", OverlayGroup::Economy),
        ("info_water_quality", "Water Quality", OverlayGroup::Terrain),
        ("info_soil_fertility", "Soil Fertility", OverlayGroup::Terrain),
        ("info_wildlife", "Wildlife", OverlayGroup::Terrain),
        ("info_forest_cover", "Forest Cover", OverlayGroup::Terrain),
        ("info_air_quality", "Air Quality", OverlayGroup::Hazard),
        ("info_noise_pollution", "Noise Pollution", OverlayGroup::Hazard),
        ("info_fire_risk", "Fire Risk", OverlayGroup::Hazard),
    ];

    for (id, name, group) in blind_overlays {
        registry.register(InfoOverlay {
            id,
            name,
            group,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Blind,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Data not yet surfaced",
        });
    }

    registry
}
