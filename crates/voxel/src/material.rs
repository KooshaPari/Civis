//! Material taxonomy and registry used by the voxel CA/worldgen layers.
//!
//! The registry is intentionally engine-agnostic: callers get stable ids, phase
//! classification, density, mobility parameters, and render colors. Rendering
//! code can map the color or the material id to engine-specific assets later.

use serde::{Deserialize, Serialize};

use crate::MaterialId;

/// Broad material phase used by the cellular automata step.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Phase {
    /// Occupies empty cells and rises through heavier gases.
    Gas,
    /// Occupies volume, flows downward, and spreads laterally.
    Liquid,
    /// Occupies volume, falls, and angle-of-repose controls slope stability.
    Powder,
    /// Occupies volume and does not move without external disruption.
    Solid,
    /// Conventionally represents no material at all.
    Empty,
}

/// Static description of a single material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialDef {
    /// Stable identifier used in world storage.
    pub id: MaterialId,
    /// Human-readable name.
    pub name: &'static str,
    /// Broad phase for physics and CA logic.
    pub phase: Phase,
    /// Relative density used to decide sinking / buoyancy.
    pub density: u16,
    /// How quickly the material can spread or flow; higher is faster.
    pub flow_rate: u8,
    /// Effective viscosity; higher is thicker / slower to move.
    pub viscosity: u16,
    /// Angle of repose in degrees for powder materials.
    pub angle_of_repose: Option<u8>,
    /// RGBA render hint for engine adapters.
    pub color: [u8; 4],
}

impl MaterialDef {
    /// Returns `true` when this material is a powder.
    #[must_use]
    pub const fn is_powder(self) -> bool {
        matches!(self.phase, Phase::Powder)
    }

    /// Returns `true` when this material is a liquid.
    #[must_use]
    pub const fn is_liquid(self) -> bool {
        matches!(self.phase, Phase::Liquid)
    }

    /// Returns `true` when this material is a gas.
    #[must_use]
    pub const fn is_gas(self) -> bool {
        matches!(self.phase, Phase::Gas)
    }
}

/// Data-driven material registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialRegistry {
    materials: &'static [MaterialDef],
}

impl MaterialRegistry {
    /// Construct a registry over a fixed slice of material definitions.
    pub const fn new(materials: &'static [MaterialDef]) -> Self {
        Self { materials }
    }

    /// Returns the standard Civis voxel materials.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            materials: &STANDARD_MATERIALS,
        }
    }

    /// Returns the full ordered material list.
    #[must_use]
    pub const fn materials(self) -> &'static [MaterialDef] {
        self.materials
    }

    /// Look up a material by id.
    #[must_use]
    pub fn get(self, id: MaterialId) -> Option<&'static MaterialDef> {
        self.materials.get(id.0 as usize)
    }

    /// Look up a material by name.
    #[must_use]
    pub fn by_name(self, name: &str) -> Option<&'static MaterialDef> {
        self.materials.iter().find(|material| material.name == name)
    }
}

/// Material id `0`.
pub const AIR: MaterialId = MaterialId(0);
/// Material id `1`.
pub const WATER: MaterialId = MaterialId(1);
/// Material id `2`.
pub const LAVA: MaterialId = MaterialId(2);
/// Material id `3`.
pub const SAND: MaterialId = MaterialId(3);
/// Material id `4`.
pub const DIRT: MaterialId = MaterialId(4);
/// Material id `5`.
pub const GRAVEL: MaterialId = MaterialId(5);
/// Material id `6`.
pub const STONE: MaterialId = MaterialId(6);
/// Material id `7`.
pub const PACKED_DIRT: MaterialId = MaterialId(7);
/// Material id `8`.
pub const ICE: MaterialId = MaterialId(8);
/// Material id `9`.
pub const STEAM: MaterialId = MaterialId(9);
/// Material id `10`.
pub const ORE: MaterialId = MaterialId(10);
/// Material id `11`.
pub const BEDROCK: MaterialId = MaterialId(11);

/// Standard material registry used by the CA and worldgen layers.
pub const STANDARD_MATERIALS: [MaterialDef; 12] = [
    MaterialDef {
        id: AIR,
        name: "Air",
        phase: Phase::Gas,
        density: 0,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        color: [0, 0, 0, 0],
    },
    MaterialDef {
        id: WATER,
        name: "Water",
        phase: Phase::Liquid,
        density: 1_000,
        flow_rate: 10,
        viscosity: 3,
        angle_of_repose: None,
        color: [54, 112, 204, 255],
    },
    MaterialDef {
        id: LAVA,
        name: "Lava",
        phase: Phase::Liquid,
        density: 3_100,
        flow_rate: 3,
        viscosity: 96,
        angle_of_repose: None,
        color: [245, 111, 24, 255],
    },
    MaterialDef {
        id: SAND,
        name: "Sand",
        phase: Phase::Powder,
        density: 1_600,
        flow_rate: 5,
        viscosity: 2,
        angle_of_repose: Some(34),
        color: [212, 194, 120, 255],
    },
    MaterialDef {
        id: DIRT,
        name: "Dirt",
        phase: Phase::Powder,
        density: 1_450,
        flow_rate: 4,
        viscosity: 3,
        angle_of_repose: Some(38),
        color: [112, 80, 48, 255],
    },
    MaterialDef {
        id: GRAVEL,
        name: "Gravel",
        phase: Phase::Powder,
        density: 1_800,
        flow_rate: 3,
        viscosity: 4,
        angle_of_repose: Some(28),
        color: [128, 126, 122, 255],
    },
    MaterialDef {
        id: STONE,
        name: "Stone",
        phase: Phase::Solid,
        density: 2_600,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [108, 112, 116, 255],
    },
    MaterialDef {
        id: PACKED_DIRT,
        name: "PackedDirt",
        phase: Phase::Solid,
        density: 1_700,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [86, 64, 40, 255],
    },
    MaterialDef {
        id: ICE,
        name: "Ice",
        phase: Phase::Solid,
        density: 920,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [176, 224, 255, 255],
    },
    MaterialDef {
        id: STEAM,
        name: "Steam",
        phase: Phase::Gas,
        density: 1,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        color: [224, 240, 255, 180],
    },
    MaterialDef {
        id: ORE,
        name: "Ore",
        phase: Phase::Solid,
        density: 3_200,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [176, 136, 72, 255],
    },
    MaterialDef {
        id: BEDROCK,
        name: "Bedrock",
        phase: Phase::Solid,
        density: 9_999,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [32, 32, 36, 255],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_registry_classifies_phases() {
        let registry = MaterialRegistry::standard();
        assert_eq!(registry.get(AIR).unwrap().phase, Phase::Gas);
        assert_eq!(registry.get(WATER).unwrap().phase, Phase::Liquid);
        assert_eq!(registry.get(SAND).unwrap().phase, Phase::Powder);
        assert_eq!(registry.get(STONE).unwrap().phase, Phase::Solid);
        assert_eq!(registry.get(BEDROCK).unwrap().density, 9_999);
    }

    #[test]
    fn registry_is_data_driven_and_lookup_by_name_works() {
        let registry = MaterialRegistry::standard();
        let steam = registry.by_name("Steam").expect("steam");
        assert!(steam.is_gas());
        assert_eq!(steam.id, STEAM);
        assert_eq!(registry.materials().len(), 12);
    }
}
