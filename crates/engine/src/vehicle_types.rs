//! Vehicle archetype catalog and capability gates (FR-CIV-VEHICLE-*).
//!
//! Data model for emergent vehicle systems described in
//! `docs/design/vehicles-logistics.md`. Vehicles are tools that
//! civilizations *produce when tech + resource thresholds are met*.
//! The `VehicleKind` enum is a catalog of physically-distinct movement
//! archetypes, not a per-civ roster.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// ── Medium (§2 of vehicles-logistics.md) ──────────────────────────────────────

/// The physical medium a vehicle traverses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Medium {
    /// Land (walkable terrain — trail, road, highway).
    Land,
    /// Water (navigable rivers, lakes, ocean).
    Water,
    /// Rail (tramway, mainline).
    Rail,
    /// Air (great-circle between airfields).
    Air,
}

// ── Vehicle kind catalog (§2 table) ──────────────────────────────────────────

/// Catalog of physically-distinct vehicle archetypes.
/// Each is a data row; whether a civ builds one depends on
/// accumulated tech traits + local material stock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleKind {
    /// Pack animal (off-road, 2-unit capacity).
    PackAnimal,
    /// Hand cart (land, trail+).
    Cart,
    /// Draft wagon (land, road+).
    Wagon,
    /// Riverboat / raft (water).
    Riverboat,
    /// Sailing ship (water, wind-coupled).
    SailingShip,
    /// Coach / stagecoach (land, fast passenger).
    Coach,
    /// Steam locomotive (rail, bulk).
    SteamLocomotive,
    /// Steamship (water, bulk).
    Steamship,
    /// Truck (land, fuel-consuming).
    Truck,
    /// Cargo rail (diesel/electric).
    CargoRail,
    /// Container ship (water, port node).
    ContainerShip,
    /// Aircraft (air, high speed, low capacity).
    Aircraft,
    /// Near-future (maglev / drone / autonomous).
    NearFuture,
}

// ── Vehicle archetype (§2 schema) ────────────────────────────────────────────

/// A data row in the vehicle archetype catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleArchetype {
    /// Which kind of vehicle this is.
    pub kind: VehicleKind,
    /// Physical medium.
    pub medium: Medium,
    /// Emergent tech traits the locale must have accumulated.
    pub requires_traits: BTreeSet<String>,
    /// Materials required from the locale's stock.
    pub requires_materials: BTreeSet<String>,
    /// Build cost in joules, charged against locale energy budget.
    pub build_cost: i64,
    /// Goods-units capacity per trip.
    pub capacity: u32,
    /// Multiplier on top of lane speed.
    pub base_speed_mult: f32,
    /// UI/legends hint; NOT the capability gate.
    pub era_hint: u16,
}

// ── Capability gate (§2, FR-CIV-VEHICLE-001) ─────────────────────────────────

/// Result of evaluating whether an archetype is buildable in a locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityGate {
    /// All three gates pass; the archetype is buildable.
    Pass,
    /// One or more gates fail. The set names what is missing.
    Fail { missing: Vec<String> },
}

/// Check whether a locale can build a vehicle of the given archetype.
///
/// All three gates must pass: `traits ⊇ requires_traits`,
/// `materials ⊇ requires_materials`, and `medium` available.
pub fn check_build_capability(
    archetype: &VehicleArchetype,
    locale_traits: &BTreeSet<String>,
    locale_materials: &BTreeSet<String>,
    medium_available: bool,
) -> CapabilityGate {
    let mut missing = Vec::new();

    for trait_name in &archetype.requires_traits {
        if !locale_traits.contains(trait_name) {
            missing.push(format!("trait:{}", trait_name));
        }
    }
    for mat in &archetype.requires_materials {
        if !locale_materials.contains(mat) {
            missing.push(format!("material:{}", mat));
        }
    }
    if !medium_available {
        missing.push(format!("medium:{:?}", archetype.medium).to_lowercase());
    }

    if missing.is_empty() {
        CapabilityGate::Pass
    } else {
        CapabilityGate::Fail { missing }
    }
}

// ── Speed model (§4) ─────────────────────────────────────────────────────────

/// Calculate effective speed on a lane for a vehicle with given load.
///
/// `effective_speed = base_walk_speed × lane_speed × vehicle_speed
///                    × medium_coupling × load_factor × congestion`
pub fn effective_speed(
    base_walk_speed: f32,
    lane_speed_mult: f32,
    vehicle_speed_mult: f32,
    medium_coupling: f32,
    load_ratio: f32,
    congestion: f32,
) -> f32 {
    let load_factor = if load_ratio <= 0.0 {
        1.0
    } else if load_ratio >= 1.0 {
        0.5 // fully laden = half speed minimum
    } else {
        1.0 - (load_ratio * 0.5) // linear interpolation (0,1] -> (0.5, 1.0]
    };
    let congestion = congestion.clamp(0.01, 1.0);

    base_walk_speed * lane_speed_mult * vehicle_speed_mult * medium_coupling * load_factor * congestion
}

// ── Lane class (§3.1) ────────────────────────────────────────────────────────

/// Lane class including media extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneClass {
    /// Off-road trail (land).
    Trail,
    /// Paved road (land).
    Road,
    /// Highway (land, fast).
    Highway,
    /// Water lane (navigable waterway).
    Water,
    /// Rail lane (tramway / mainline).
    Rail,
    /// Air lane (great-circle between airfields).
    Air,
}

impl LaneClass {
    /// Whether this lane class admits the given medium.
    /// Land covers Trail/Road/Highway.
    pub fn admits_medium(self, medium: Medium) -> bool {
        match medium {
            Medium::Land => matches!(self, Self::Trail | Self::Road | Self::Highway),
            Medium::Water => self == Self::Water,
            Medium::Rail => self == Self::Rail,
            Medium::Air => self == Self::Air,
        }
    }

    /// Speed multiplier for this lane class.
    pub fn speed_mult(self) -> f32 {
        match self {
            Self::Trail => 0.8,
            Self::Road => 1.0,
            Self::Highway => 1.5,
            Self::Water => 1.2,
            Self::Rail => 2.0,
            Self::Air => 3.0,
        }
    }
}

// ── Infra provenance (§2, FR-CIV-VEHICLE-004) ─────────────────────────────────

/// How a vehicle instance was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfraProvenance {
    /// Built by the civilization locale.
    CivBuilt,
    /// Placed by the user in sandbox mode.
    UserPlaced,
}

// ── Catalog (§2, FR-CIV-VEHICLE-003) ─────────────────────────────────────────

/// The full vehicle archetype catalog. Additive/forward-only: new
/// archetypes slot in by adding a row + traits.
pub fn default_archetype_catalog() -> Vec<VehicleArchetype> {
    vec![
        VehicleArchetype {
            kind: VehicleKind::PackAnimal,
            medium: Medium::Land,
            requires_traits: BTreeSet::from(["animal-domestication".into()]),
            requires_materials: BTreeSet::from(["livestock".into()]),
            build_cost: 100,
            capacity: 2,
            base_speed_mult: 1.15,
            era_hint: 0,
        },
        VehicleArchetype {
            kind: VehicleKind::Cart,
            medium: Medium::Land,
            requires_traits: BTreeSet::from(["wheel".into()]),
            requires_materials: BTreeSet::from(["wood".into()]),
            build_cost: 200,
            capacity: 4,
            base_speed_mult: 1.3,
            era_hint: 1,
        },
        VehicleArchetype {
            kind: VehicleKind::Wagon,
            medium: Medium::Land,
            requires_traits: BTreeSet::from(["wheel".into(), "harness".into()]),
            requires_materials: BTreeSet::from(["wood".into(), "livestock".into()]),
            build_cost: 500,
            capacity: 12,
            base_speed_mult: 1.7,
            era_hint: 2,
        },
        VehicleArchetype {
            kind: VehicleKind::Riverboat,
            medium: Medium::Water,
            requires_traits: BTreeSet::from(["boatbuilding".into()]),
            requires_materials: BTreeSet::from(["wood".into()]),
            build_cost: 400,
            capacity: 20,
            base_speed_mult: 1.5,
            era_hint: 2,
        },
        VehicleArchetype {
            kind: VehicleKind::SailingShip,
            medium: Medium::Water,
            requires_traits: BTreeSet::from(["sail".into(), "rope".into()]),
            requires_materials: BTreeSet::from(["wood".into()]),
            build_cost: 1000,
            capacity: 60,
            base_speed_mult: 2.2,
            era_hint: 4,
        },
        VehicleArchetype {
            kind: VehicleKind::Coach,
            medium: Medium::Land,
            requires_traits: BTreeSet::from(["suspension".into()]),
            requires_materials: BTreeSet::from(["wood".into(), "iron".into(), "livestock".into()]),
            build_cost: 800,
            capacity: 8,
            base_speed_mult: 2.4,
            era_hint: 4,
        },
        VehicleArchetype {
            kind: VehicleKind::SteamLocomotive,
            medium: Medium::Rail,
            requires_traits: BTreeSet::from(["steam".into(), "iron-rail".into()]),
            requires_materials: BTreeSet::from(["iron".into()]),
            build_cost: 5000,
            capacity: 240,
            base_speed_mult: 4.0,
            era_hint: 5,
        },
        VehicleArchetype {
            kind: VehicleKind::Steamship,
            medium: Medium::Water,
            requires_traits: BTreeSet::from(["steam".into(), "iron-hull".into()]),
            requires_materials: BTreeSet::from(["iron".into()]),
            build_cost: 4000,
            capacity: 300,
            base_speed_mult: 3.0,
            era_hint: 5,
        },
        VehicleArchetype {
            kind: VehicleKind::Truck,
            medium: Medium::Land,
            requires_traits: BTreeSet::from(["internal-combustion".into()]),
            requires_materials: BTreeSet::from(["refined-fuel".into(), "steel".into()]),
            build_cost: 3000,
            capacity: 30,
            base_speed_mult: 5.0,
            era_hint: 6,
        },
        VehicleArchetype {
            kind: VehicleKind::CargoRail,
            medium: Medium::Rail,
            requires_traits: BTreeSet::from(["diesel".into()]),
            requires_materials: BTreeSet::from(["steel".into()]),
            build_cost: 8000,
            capacity: 600,
            base_speed_mult: 6.0,
            era_hint: 7,
        },
        VehicleArchetype {
            kind: VehicleKind::ContainerShip,
            medium: Medium::Water,
            requires_traits: BTreeSet::from(["containerization".into()]),
            requires_materials: BTreeSet::from(["steel".into()]),
            build_cost: 12000,
            capacity: 2000,
            base_speed_mult: 4.0,
            era_hint: 7,
        },
        VehicleArchetype {
            kind: VehicleKind::Aircraft,
            medium: Medium::Air,
            requires_traits: BTreeSet::from(["aviation".into()]),
            requires_materials: BTreeSet::from(["alloy".into(), "refined-fuel".into()]),
            build_cost: 15000,
            capacity: 40,
            base_speed_mult: 12.0,
            era_hint: 8,
        },
        VehicleArchetype {
            kind: VehicleKind::NearFuture,
            medium: Medium::Rail,
            requires_traits: BTreeSet::from(["superconductor".into()]),
            requires_materials: BTreeSet::from(["advanced-alloy".into()]),
            build_cost: 20000,
            capacity: 100,
            base_speed_mult: 10.0,
            era_hint: 9,
        },
    ]
}

/// Find an archetype by kind in the catalog.
pub fn find_archetype(catalog: &[VehicleArchetype], kind: VehicleKind) -> Option<&VehicleArchetype> {
    catalog.iter().find(|a| a.kind == kind)
}
