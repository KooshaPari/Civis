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
    /// Relative conductivity (0-255) for thermal exchange.
    pub heat_conduct: u8,
    /// Temperature where solid becomes liquid.
    pub melting_point: i16,
    /// Temperature where liquid becomes gas.
    pub boiling_point: i16,
    /// Temperature where liquid may refreeze.
    pub freeze_point: i16,
    /// Latent heat exchange used to stall phase fronts.
    pub latent_heat: u16,
    /// Porosity on 0-255 scale.
    pub porosity: u8,
    /// Water saturation capacity for porous cells (0-255).
    pub field_capacity: u8,
    /// Relative temperature used for phase transitions and initial conditions.
    pub temperature: i16,
    /// Relative flammability on a 0-100 scale.
    pub flammability: u8,
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
/// Material id `12`.
pub const SALT_WATER: MaterialId = MaterialId(12);
/// Material id `13`.
pub const OIL: MaterialId = MaterialId(13);
/// Material id `14`.
pub const ACID: MaterialId = MaterialId(14);
/// Material id `15`.
pub const BLOOD: MaterialId = MaterialId(15);
/// Material id `16`.
pub const MUD: MaterialId = MaterialId(16);
/// Material id `17`.
pub const MOLTEN_METAL: MaterialId = MaterialId(17);
/// Material id `18`.
pub const CLAY: MaterialId = MaterialId(18);
/// Material id `19`.
pub const ASH: MaterialId = MaterialId(19);
/// Material id `20`.
pub const SNOW: MaterialId = MaterialId(20);
/// Material id `21`.
pub const GUNPOWDER: MaterialId = MaterialId(21);
/// Material id `22`.
pub const SALT: MaterialId = MaterialId(22);
/// Material id `23`.
pub const GRANITE: MaterialId = MaterialId(23);
/// Material id `24`.
pub const WOOD: MaterialId = MaterialId(24);
/// Material id `25`.
pub const COAL: MaterialId = MaterialId(25);
/// Material id `26`.
pub const GLASS: MaterialId = MaterialId(26);
/// Material id `27`.
pub const CRYSTAL: MaterialId = MaterialId(27);
/// Material id `28`.
pub const BRICK: MaterialId = MaterialId(28);
/// Material id `29`.
pub const BONE: MaterialId = MaterialId(29);
/// Material id `30`.
pub const SMOKE: MaterialId = MaterialId(30);
/// Material id `31`.
pub const METHANE: MaterialId = MaterialId(31);
/// Material id `32`.
pub const TOXIC_GAS: MaterialId = MaterialId(32);
/// Material id `33`.
pub const CO2: MaterialId = MaterialId(33);
/// Material id `34`.
pub const FIRE: MaterialId = MaterialId(34);
/// Material id `35`.
pub const EMBER: MaterialId = MaterialId(35);
/// Material id `36`.
pub const PLASMA: MaterialId = MaterialId(36);
/// Material id `37`.
pub const SPARK: MaterialId = MaterialId(37);
/// Material id `38`.
pub const PLANT: MaterialId = MaterialId(38);
/// Material id `39`.
pub const MOSS: MaterialId = MaterialId(39);
/// Material id `40`.
pub const MOLD: MaterialId = MaterialId(40);

/// Standard material registry used by the CA and worldgen layers.
pub const STANDARD_MATERIALS: [MaterialDef; 41] = [
    MaterialDef {
        id: AIR,
        name: "Air",
        phase: Phase::Gas,
        density: 0,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 4,
        melting_point: -273,
        boiling_point: 32000,
        freeze_point: -273,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
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
        heat_conduct: 90,
        melting_point: 0,
        boiling_point: 100,
        freeze_point: 0,
        latent_heat: 2_257,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
        color: [40, 96, 168, 220],
    },
    MaterialDef {
        id: LAVA,
        name: "Lava",
        phase: Phase::Liquid,
        density: 3_100,
        flow_rate: 3,
        viscosity: 96,
        angle_of_repose: None,
        heat_conduct: 15,
        melting_point: 1_100,
        boiling_point: 3_000,
        freeze_point: 700,
        latent_heat: 400,
        porosity: 0,
        field_capacity: 0,
        temperature: 1_200,
        flammability: 0,
        color: [230, 92, 18, 255],
    },
    MaterialDef {
        id: SAND,
        name: "Sand",
        phase: Phase::Powder,
        density: 1_600,
        flow_rate: 5,
        viscosity: 2,
        angle_of_repose: Some(34),
        heat_conduct: 45,
        melting_point: 1_710,
        boiling_point: 2_600,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 60,
        field_capacity: 32,
        temperature: 20,
        flammability: 0,
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
        heat_conduct: 50,
        melting_point: 1_500,
        boiling_point: 2_600,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 80,
        field_capacity: 80,
        temperature: 18,
        flammability: 0,
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
        heat_conduct: 55,
        melting_point: 1_600,
        boiling_point: 2_600,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 20,
        field_capacity: 16,
        temperature: 18,
        flammability: 0,
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
        heat_conduct: 80,
        melting_point: 1_450,
        boiling_point: 2_900,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        color: [104, 106, 110, 255],
    },
    MaterialDef {
        id: PACKED_DIRT,
        name: "PackedDirt",
        phase: Phase::Solid,
        density: 1_700,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 60,
        melting_point: 1_500,
        boiling_point: 2_600,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 10,
        field_capacity: 20,
        temperature: 18,
        flammability: 0,
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
        heat_conduct: 25,
        melting_point: 0,
        boiling_point: 120,
        freeze_point: -20,
        latent_heat: 334,
        porosity: 0,
        field_capacity: 0,
        temperature: -5,
        flammability: 0,
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
        heat_conduct: 15,
        melting_point: 100,
        boiling_point: 32000,
        freeze_point: 0,
        latent_heat: 2_257,
        porosity: 0,
        field_capacity: 0,
        temperature: 95,
        flammability: 0,
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
        heat_conduct: 50,
        melting_point: 1_530,
        boiling_point: 3_500,
        freeze_point: 20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
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
        heat_conduct: 20,
        melting_point: 3_500,
        boiling_point: 9_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        // Exposed bedrock cliff faces dominate the visible hull. At [32,32,36]
        // they crushed to near-black even under a noon sun + ambient fill,
        // reading as an unlit slab. Lift to a legible dark slate-grey so the
        // cliffs show form/shading instead of a black silhouette.
        color: [86, 88, 96, 255],
    },
    MaterialDef {
        id: SALT_WATER,
        name: "SaltWater",
        phase: Phase::Liquid,
        density: 1_030,
        flow_rate: 10,
        viscosity: 3,
        angle_of_repose: None,
        heat_conduct: 88,
        melting_point: -21,
        boiling_point: 100,
        freeze_point: -21,
        latent_heat: 2_400,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
        color: [44, 100, 184, 255],
    },
    MaterialDef {
        id: OIL,
        name: "Oil",
        phase: Phase::Liquid,
        density: 850,
        flow_rate: 8,
        viscosity: 12,
        angle_of_repose: None,
        heat_conduct: 30,
        melting_point: -60,
        boiling_point: 300,
        freeze_point: -60,
        latent_heat: 400,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 82,
        color: [84, 64, 24, 255],
    },
    MaterialDef {
        id: ACID,
        name: "Acid",
        phase: Phase::Liquid,
        density: 1_080,
        flow_rate: 9,
        viscosity: 4,
        angle_of_repose: None,
        heat_conduct: 44,
        melting_point: -20,
        boiling_point: 120,
        freeze_point: -20,
        latent_heat: 280,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
        color: [94, 204, 56, 255],
    },
    MaterialDef {
        id: BLOOD,
        name: "Blood",
        phase: Phase::Liquid,
        density: 1_060,
        flow_rate: 9,
        viscosity: 5,
        angle_of_repose: None,
        heat_conduct: 45,
        melting_point: 34,
        boiling_point: 110,
        freeze_point: -20,
        latent_heat: 320,
        porosity: 0,
        field_capacity: 0,
        temperature: 37,
        flammability: 0,
        color: [140, 14, 20, 255],
    },
    MaterialDef {
        id: MUD,
        name: "Mud",
        phase: Phase::Liquid,
        density: 1_500,
        flow_rate: 4,
        viscosity: 18,
        angle_of_repose: None,
        heat_conduct: 50,
        melting_point: 100,
        boiling_point: 200,
        freeze_point: -10,
        latent_heat: 500,
        porosity: 180,
        field_capacity: 120,
        temperature: 18,
        flammability: 0,
        color: [88, 68, 44, 255],
    },
    MaterialDef {
        id: MOLTEN_METAL,
        name: "MoltenMetal",
        phase: Phase::Liquid,
        density: 6_900,
        flow_rate: 4,
        viscosity: 52,
        angle_of_repose: None,
        heat_conduct: 60,
        melting_point: 1_450,
        boiling_point: 3_200,
        freeze_point: 1_200,
        latent_heat: 800,
        porosity: 0,
        field_capacity: 0,
        temperature: 1_450,
        flammability: 0,
        color: [196, 164, 84, 255],
    },
    MaterialDef {
        id: CLAY,
        name: "Clay",
        phase: Phase::Powder,
        density: 1_650,
        flow_rate: 3,
        viscosity: 6,
        angle_of_repose: Some(36),
        heat_conduct: 55,
        melting_point: 1_650,
        boiling_point: 2_500,
        freeze_point: -10,
        latent_heat: 0,
        porosity: 100,
        field_capacity: 100,
        temperature: 18,
        flammability: 0,
        color: [154, 112, 84, 255],
    },
    MaterialDef {
        id: ASH,
        name: "Ash",
        phase: Phase::Powder,
        density: 420,
        flow_rate: 8,
        viscosity: 1,
        angle_of_repose: Some(24),
        heat_conduct: 35,
        melting_point: 700,
        boiling_point: 2_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 20,
        field_capacity: 10,
        temperature: 30,
        flammability: 0,
        color: [144, 140, 136, 255],
    },
    MaterialDef {
        id: SNOW,
        name: "Snow",
        phase: Phase::Powder,
        density: 320,
        flow_rate: 7,
        viscosity: 1,
        angle_of_repose: Some(22),
        heat_conduct: 15,
        melting_point: 0,
        boiling_point: -50,
        freeze_point: -10,
        latent_heat: 334,
        porosity: 220,
        field_capacity: 220,
        temperature: -8,
        flammability: 0,
        color: [242, 248, 255, 255],
    },
    MaterialDef {
        id: GUNPOWDER,
        name: "Gunpowder",
        phase: Phase::Powder,
        density: 1_650,
        flow_rate: 5,
        viscosity: 2,
        angle_of_repose: Some(31),
        heat_conduct: 40,
        melting_point: 200,
        boiling_point: 2_800,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 20,
        field_capacity: 0,
        temperature: 20,
        flammability: 100,
        color: [58, 54, 50, 255],
    },
    MaterialDef {
        id: SALT,
        name: "Salt",
        phase: Phase::Powder,
        density: 2_170,
        flow_rate: 4,
        viscosity: 1,
        angle_of_repose: Some(29),
        heat_conduct: 45,
        melting_point: 801,
        boiling_point: 1_413,
        freeze_point: 0,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
        color: [232, 232, 224, 255],
    },
    MaterialDef {
        id: GRANITE,
        name: "Granite",
        phase: Phase::Solid,
        density: 2_700,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 80,
        melting_point: 1_215,
        boiling_point: 3_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        color: [116, 104, 108, 255],
    },
    MaterialDef {
        id: WOOD,
        name: "Wood",
        phase: Phase::Solid,
        density: 650,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 30,
        melting_point: 300,
        boiling_point: 400,
        freeze_point: -10,
        latent_heat: 0,
        porosity: 100,
        field_capacity: 40,
        temperature: 18,
        flammability: 92,
        color: [140, 100, 52, 255],
    },
    MaterialDef {
        id: COAL,
        name: "Coal",
        phase: Phase::Solid,
        density: 1_350,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 40,
        melting_point: 380,
        boiling_point: 400,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 30,
        field_capacity: 0,
        temperature: 18,
        flammability: 78,
        color: [32, 32, 36, 255],
    },
    MaterialDef {
        id: GLASS,
        name: "Glass",
        phase: Phase::Solid,
        density: 2_500,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 20,
        melting_point: 1_430,
        boiling_point: 2_700,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        color: [180, 220, 232, 190],
    },
    MaterialDef {
        id: CRYSTAL,
        name: "Crystal",
        phase: Phase::Solid,
        density: 2_800,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 30,
        melting_point: 1_600,
        boiling_point: 2_700,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        color: [146, 200, 255, 210],
    },
    MaterialDef {
        id: BRICK,
        name: "Brick",
        phase: Phase::Solid,
        density: 1_900,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 35,
        melting_point: 1_300,
        boiling_point: 2_100,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 18,
        flammability: 0,
        color: [162, 84, 64, 255],
    },
    MaterialDef {
        id: BONE,
        name: "Bone",
        phase: Phase::Solid,
        density: 1_850,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 40,
        melting_point: 1_300,
        boiling_point: 2_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 10,
        field_capacity: 4,
        temperature: 18,
        flammability: 28,
        color: [226, 220, 198, 255],
    },
    MaterialDef {
        id: SMOKE,
        name: "Smoke",
        phase: Phase::Gas,
        density: 2,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 12,
        melting_point: 80,
        boiling_point: 32000,
        freeze_point: -273,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 90,
        flammability: 0,
        color: [92, 92, 92, 150],
    },
    MaterialDef {
        id: METHANE,
        name: "Methane",
        phase: Phase::Gas,
        density: 1,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 10,
        melting_point: -182,
        boiling_point: -162,
        freeze_point: -183,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 90,
        color: [184, 224, 184, 85],
    },
    MaterialDef {
        id: TOXIC_GAS,
        name: "ToxicGas",
        phase: Phase::Gas,
        density: 2,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 10,
        melting_point: -195,
        boiling_point: -150,
        freeze_point: -300,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 20,
        flammability: 0,
        color: [116, 196, 56, 120],
    },
    MaterialDef {
        id: CO2,
        name: "CO2",
        phase: Phase::Gas,
        density: 2,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 8,
        melting_point: -56,
        boiling_point: -78,
        freeze_point: -194,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 10,
        flammability: 0,
        color: [216, 224, 236, 120],
    },
    MaterialDef {
        id: FIRE,
        name: "Fire",
        phase: Phase::Gas,
        density: 1,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 60,
        melting_point: 1_100,
        boiling_point: 3_000,
        freeze_point: -200,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 1_100,
        flammability: 0,
        color: [255, 146, 32, 220],
    },
    MaterialDef {
        id: EMBER,
        name: "Ember",
        phase: Phase::Solid,
        density: 1_100,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 70,
        melting_point: 800,
        boiling_point: 3_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 600,
        flammability: 0,
        color: [180, 86, 24, 255],
    },
    MaterialDef {
        id: PLASMA,
        name: "Plasma",
        phase: Phase::Gas,
        density: 1,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 1,
        melting_point: 4_000,
        boiling_point: 10_000,
        freeze_point: -400,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 5_000,
        flammability: 0,
        color: [255, 70, 180, 220],
    },
    MaterialDef {
        id: SPARK,
        name: "Spark",
        phase: Phase::Gas,
        density: 1,
        flow_rate: 12,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 40,
        melting_point: 700,
        boiling_point: 3_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 0,
        field_capacity: 0,
        temperature: 900,
        flammability: 0,
        color: [255, 240, 160, 220],
    },
    MaterialDef {
        id: PLANT,
        name: "Plant",
        phase: Phase::Solid,
        density: 900,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 60,
        melting_point: 200,
        boiling_point: 1_000,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 120,
        field_capacity: 80,
        temperature: 18,
        flammability: 75,
        color: [64, 156, 72, 255],
    },
    MaterialDef {
        id: MOSS,
        name: "Moss",
        phase: Phase::Solid,
        density: 520,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 55,
        melting_point: 120,
        boiling_point: 900,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 150,
        field_capacity: 120,
        temperature: 16,
        flammability: 35,
        color: [72, 128, 64, 255],
    },
    MaterialDef {
        id: MOLD,
        name: "Mold",
        phase: Phase::Solid,
        density: 500,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        heat_conduct: 55,
        melting_point: 80,
        boiling_point: 900,
        freeze_point: -20,
        latent_heat: 0,
        porosity: 180,
        field_capacity: 130,
        temperature: 18,
        flammability: 10,
        color: [92, 124, 76, 255],
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
        assert_eq!(registry.get(FIRE).unwrap().temperature, 1_100);
        assert_eq!(registry.get(OIL).unwrap().flammability, 82);
    }

    #[test]
    fn registry_is_data_driven_and_lookup_by_name_works() {
        let registry = MaterialRegistry::standard();
        let steam = registry.by_name("Steam").expect("steam");
        assert!(steam.is_gas());
        assert_eq!(steam.id, STEAM);
        assert_eq!(registry.materials().len(), 41);
        assert!(registry.by_name("LAVA").is_none());
    }
}
