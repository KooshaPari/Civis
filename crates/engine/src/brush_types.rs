//! Unified brush-cluster tool types (FR-CIV-BRUSH-*).
//!
//! Data model for the cluster -> mode -> action brush system described in
//! `docs/design/brush-tool-system.md`. These types drive the Brush Control
//! Panel and the substrate dispatch through `godtools::GodToolRequest`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// ── Shared brush parameters (§1 of brush-tool-system.md) ──────────────────────

/// Shared brush parameters used by every cluster's Brush Control Panel.
/// This is the single source of brush geometry, replacing the earlier
/// duplicated `terraform_brush::BrushSettings` / `SelectedMaterial` split.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrushSettings {
    /// Brush radius in voxels. Must be > 0.
    pub radius: u8,
    /// Brush strength (0..=100). Meaning per cluster:
    /// - Terraform: delta-y cap
    /// - Material: deposit thickness
    /// - Life: effect magnitude
    pub strength: u8,
    /// Falloff curve applied from center to edge.
    pub falloff: BrushFalloff,
    /// Brush footprint shape.
    pub shape: BrushShape,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            radius: 4,
            strength: 50,
            falloff: BrushFalloff::Linear,
            shape: BrushShape::Disc,
        }
    }
}

/// Falloff curve from brush center to edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushFalloff {
    /// Uniform weight across the footprint (no falloff).
    Uniform,
    /// Linear falloff from center (1.0) to edge (0.0).
    Linear,
    /// Gaussian falloff (smooth, bell-shaped).
    Gaussian,
}

/// Brush footprint shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushShape {
    /// Circular / disc footprint.
    Disc,
    /// Square footprint.
    Square,
    /// Diamond (rotated square) footprint.
    Diamond,
}

// ── Cluster types (§1.1) ─────────────────────────────────────────────────────

/// Top-level toolbar slot. Selecting a cluster activates it and
/// repopulates the Brush Control Panel with its action modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushCluster {
    /// Pick / inspect sandbox entities.
    Select,
    /// Material palette — write material into the voxel substrate.
    Material,
    /// Terrain sculpting — raise, lower, level, smooth, slope, etc.
    Terraform,
    /// Life — spawn organisms, herds; bless/curse/heal.
    Life,
    /// Structure — place buildings (House, Farm, Workshop, etc.).
    Structure,
    /// Infrastructure — road, trail, highway, bridge, canal.
    Infrastructure,
    /// Disaster — meteor, flood, quake, storm, wildfire, plague.
    Disaster,
    /// Diplomacy — alliance, war, trade relations.
    Diplomacy,
    /// Policy — tax, edict, religion adjustments.
    Policy,
}

impl BrushCluster {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select / Inspect",
            Self::Material => "Material",
            Self::Terraform => "Terraform",
            Self::Life => "Life",
            Self::Structure => "Structure",
            Self::Infrastructure => "Infrastructure",
            Self::Disaster => "Disaster",
            Self::Diplomacy => "Diplomacy",
            Self::Policy => "Policy",
        }
    }

    /// Number of action modes in this cluster's catalog.
    pub fn mode_count(self) -> usize {
        self.modes().len()
    }

    /// All action modes available in this cluster.
    pub fn modes(self) -> &'static [ActionMode] {
        match self {
            Self::Select => &[
                ActionMode { kind: ActionKind::Select, label: "Select", icon: "select", group: ModeGroup::None },
                ActionMode { kind: ActionKind::Inspect, label: "Inspect", icon: "inspect", group: ModeGroup::None },
            ],
            Self::Material => &[
                ActionMode { kind: ActionKind::MaterialReplace, label: "Replace", icon: "replace", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::MaterialAdditiveDrop, label: "Drop", icon: "drop", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::MaterialErase, label: "Erase", icon: "erase", group: ModeGroup::Remove },
                ActionMode { kind: ActionKind::MaterialSurfacePaint, label: "Surface", icon: "surface", group: ModeGroup::Surface },
            ],
            Self::Terraform => &[
                ActionMode { kind: ActionKind::TerraformRaise, label: "Raise", icon: "raise", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformLower, label: "Lower", icon: "lower", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformLevel, label: "Level", icon: "level", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformSmooth, label: "Smooth", icon: "smooth", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformSlope, label: "Slope", icon: "slope", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformFlatten, label: "Flatten", icon: "flatten", group: ModeGroup::Precise },
                ActionMode { kind: ActionKind::TerraformAddLand, label: "Add Land", icon: "add_land", group: ModeGroup::God },
                ActionMode { kind: ActionKind::TerraformDigOcean, label: "Dig Ocean", icon: "dig_ocean", group: ModeGroup::God },
                ActionMode { kind: ActionKind::TerraformRaiseMountain, label: "Mountain", icon: "mountain", group: ModeGroup::God },
                ActionMode { kind: ActionKind::TerraformDropBiome, label: "Drop Biome", icon: "biome", group: ModeGroup::God },
            ],
            Self::Life => &[
                ActionMode { kind: ActionKind::LifeSpawnOrganism, label: "Organism", icon: "organism", group: ModeGroup::Spawn },
                ActionMode { kind: ActionKind::LifeSpawnHerd, label: "Herd", icon: "herd", group: ModeGroup::Spawn },
                ActionMode { kind: ActionKind::LifeBless, label: "Bless", icon: "bless", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::LifeCurse, label: "Curse", icon: "curse", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::LifeHeal, label: "Heal", icon: "heal", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::LifeExtinct, label: "Extinct", icon: "extinct", group: ModeGroup::Effect },
            ],
            Self::Structure => &[
                ActionMode { kind: ActionKind::StructureHouse, label: "House", icon: "house", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureFarm, label: "Farm", icon: "farm", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureWorkshop, label: "Workshop", icon: "workshop", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureMarket, label: "Market", icon: "market", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureWall, label: "Wall", icon: "wall", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureTower, label: "Tower", icon: "tower", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::StructureMonument, label: "Monument", icon: "monument", group: ModeGroup::Placement },
            ],
            Self::Infrastructure => &[
                ActionMode { kind: ActionKind::InfraRoad, label: "Road", icon: "road", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::InfraTrail, label: "Trail", icon: "trail", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::InfraHighway, label: "Highway", icon: "highway", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::InfraBridge, label: "Bridge", icon: "bridge", group: ModeGroup::Placement },
                ActionMode { kind: ActionKind::InfraCanal, label: "Canal", icon: "canal", group: ModeGroup::Placement },
            ],
            Self::Disaster => &[
                ActionMode { kind: ActionKind::DisasterMeteor, label: "Meteor", icon: "meteor", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::DisasterFlood, label: "Flood", icon: "flood", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::DisasterQuake, label: "Quake", icon: "quake", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::DisasterStorm, label: "Storm", icon: "storm", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::DisasterWildfire, label: "Wildfire", icon: "wildfire", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::DisasterPlague, label: "Plague", icon: "plague", group: ModeGroup::Effect },
            ],
            Self::Diplomacy => &[
                ActionMode { kind: ActionKind::DiploAlliance, label: "Alliance", icon: "alliance", group: ModeGroup::Relation },
                ActionMode { kind: ActionKind::DiploWar, label: "War", icon: "war", group: ModeGroup::Relation },
                ActionMode { kind: ActionKind::DiploTrade, label: "Trade", icon: "trade", group: ModeGroup::Relation },
            ],
            Self::Policy => &[
                ActionMode { kind: ActionKind::PolicyTax, label: "Tax", icon: "tax", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::PolicyEdict, label: "Edict", icon: "edict", group: ModeGroup::Effect },
                ActionMode { kind: ActionKind::PolicyReligion, label: "Religion", icon: "religion", group: ModeGroup::Effect },
            ],
        }
    }

    /// All nine clusters in toolbar order.
    pub fn all() -> &'static [BrushCluster] {
        &[
            Self::Select,
            Self::Material,
            Self::Terraform,
            Self::Life,
            Self::Structure,
            Self::Infrastructure,
            Self::Disaster,
            Self::Diplomacy,
            Self::Policy,
        ]
    }
}

// ── Action mode types (§1.2) ─────────────────────────────────────────────────

/// A single verb within a cluster's action group, shown as a segmented
/// blade group in the Brush Control Panel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionMode {
    /// The world-effect verb.
    pub kind: ActionKind,
    /// Human-readable label.
    pub label: &'static str,
    /// Icon key for the toolbar.
    pub icon: &'static str,
    /// Sub-row grouping in the panel.
    pub group: ModeGroup,
}

/// Sub-row grouping for action modes within a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeGroup {
    /// No grouping (Select/Inspect).
    None,
    /// Placement operations (material stamp, building, road).
    Placement,
    /// Precise terrain ops (raise, lower, level, smooth, slope, flatten).
    Precise,
    /// God-mode terrain ops (add land, dig ocean, mountain, biome).
    God,
    /// Remove operations (erase).
    Remove,
    /// Surface-only operations (surface paint).
    Surface,
    /// Spawn operations (organism, herd).
    Spawn,
    /// Effect operations (bless, curse, heal, disasters).
    Effect,
    /// Relation operations (diplomacy).
    Relation,
}

// ── Action kinds (§1.3) ──────────────────────────────────────────────────────

/// Union of every verb. Each `ActionKind` knows which applier consumes it
/// and how `BrushSettings` params translate into the emitted request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    // Select cluster
    Select,
    Inspect,
    // Material cluster
    MaterialReplace,
    MaterialAdditiveDrop,
    MaterialErase,
    MaterialSurfacePaint,
    // Terraform cluster
    TerraformRaise,
    TerraformLower,
    TerraformLevel,
    TerraformSmooth,
    TerraformSlope,
    TerraformFlatten,
    TerraformAddLand,
    TerraformDigOcean,
    TerraformRaiseMountain,
    TerraformDropBiome,
    // Life cluster
    LifeSpawnOrganism,
    LifeSpawnHerd,
    LifeBless,
    LifeCurse,
    LifeHeal,
    LifeExtinct,
    // Structure cluster
    StructureHouse,
    StructureFarm,
    StructureWorkshop,
    StructureMarket,
    StructureWall,
    StructureTower,
    StructureMonument,
    // Infrastructure cluster
    InfraRoad,
    InfraTrail,
    InfraHighway,
    InfraBridge,
    InfraCanal,
    // Disaster cluster
    DisasterMeteor,
    DisasterFlood,
    DisasterQuake,
    DisasterStorm,
    DisasterWildfire,
    DisasterPlague,
    // Diplomacy cluster
    DiploAlliance,
    DiploWar,
    DiploTrade,
    // Policy cluster
    PolicyTax,
    PolicyEdict,
    PolicyReligion,
}

impl ActionKind {
    /// Whether this action requires substrate mutation (vs read-only inspect).
    pub fn is_mutating(self) -> bool {
        !matches!(self, Self::Select | Self::Inspect)
    }

    /// The cluster this action belongs to.
    pub fn cluster(self) -> BrushCluster {
        match self {
            Self::Select | Self::Inspect => BrushCluster::Select,
            Self::MaterialReplace | Self::MaterialAdditiveDrop
            | Self::MaterialErase | Self::MaterialSurfacePaint => BrushCluster::Material,
            Self::TerraformRaise | Self::TerraformLower | Self::TerraformLevel
            | Self::TerraformSmooth | Self::TerraformSlope | Self::TerraformFlatten
            | Self::TerraformAddLand | Self::TerraformDigOcean
            | Self::TerraformRaiseMountain | Self::TerraformDropBiome => BrushCluster::Terraform,
            Self::LifeSpawnOrganism | Self::LifeSpawnHerd | Self::LifeBless
            | Self::LifeCurse | Self::LifeHeal | Self::LifeExtinct => BrushCluster::Life,
            Self::StructureHouse | Self::StructureFarm | Self::StructureWorkshop
            | Self::StructureMarket | Self::StructureWall | Self::StructureTower
            | Self::StructureMonument => BrushCluster::Structure,
            Self::InfraRoad | Self::InfraTrail | Self::InfraHighway
            | Self::InfraBridge | Self::InfraCanal => BrushCluster::Infrastructure,
            Self::DisasterMeteor | Self::DisasterFlood | Self::DisasterQuake
            | Self::DisasterStorm | Self::DisasterWildfire | Self::DisasterPlague => BrushCluster::Disaster,
            Self::DiploAlliance | Self::DiploWar | Self::DiploTrade => BrushCluster::Diplomacy,
            Self::PolicyTax | Self::PolicyEdict | Self::PolicyReligion => BrushCluster::Policy,
        }
    }
}

/// The total number of distinct action kinds across all clusters.
pub const TOTAL_ACTION_KINDS: usize = 46;

/// Collect all unique action kinds across every cluster.
pub fn all_action_kinds() -> Vec<ActionKind> {
    let mut kinds = Vec::new();
    for cluster in BrushCluster::all() {
        for mode in cluster.modes() {
            if !kinds.contains(&mode.kind) {
                kinds.push(mode.kind);
            }
        }
    }
    kinds
}
