#![deny(unsafe_code)]

use civ_agents::{spawn_civilian_at, ActorVisualKind, Alignment};
use civ_voxel::material::{AIR, STONE};
use civ_voxel::{MaterialId, WorldCoord};
use serde::{Deserialize, Serialize};

use crate::disasters::DisasterKind;
use crate::engine::Simulation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolRequest {
    Terraform(TerraformRequest),
    Material(MaterialRequest),
    Life(LifeRequest),
    Disaster(DisasterRequest),
    Inspect(InspectRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerraformRequest {
    pub op: TerraformOp,
    pub center: WorldCoord,
    pub delta: i32,
    pub target_height: i32,
    pub radius: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TerraformOp {
    Raise,
    Lower,
    Level,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialRequest {
    pub center: WorldCoord,
    pub material: MaterialId,
    pub radius: i32,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifeRequest {
    pub spawn: SpawnOrganism,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SpawnOrganism {
    pub civilian_id: u64,
    pub alignment: Alignment,
    pub x: f32,
    pub y: f32,
    pub visual: ActorVisualKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DisasterRequest {
    pub op: DisasterOp,
    pub center: WorldCoord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DisasterOp {
    Meteor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InspectRequest {
    pub op: InspectOp,
    pub coord: WorldCoord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InspectOp {
    Probe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GodToolReceipt {
    Terraform {
        op: TerraformOp,
        cells_written: u32,
        center: WorldCoord,
    },
    Material {
        cells_written: u32,
        material: MaterialId,
        center: WorldCoord,
    },
    Spawn {
        entity: hecs::Entity,
        civilian_id: u64,
        coord: WorldCoord,
    },
    Disaster {
        kind: DisasterKind,
        fired: bool,
        center: WorldCoord,
    },
    Inspect {
        op: InspectOp,
        material: MaterialId,
        nearest_agent: Option<hecs::Entity>,
        coord: WorldCoord,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolError {
    InvalidDimension { field: &'static str, value: i32 },
    OutOfBounds { axis: &'static str, value: f32 },
}

impl std::fmt::Display for GodToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GodToolError::InvalidDimension { field, value } => {
                write!(f, "invalid {field}: {value} (must be > 0)")
            }
            GodToolError::OutOfBounds { axis, value } => {
                write!(f, "out-of-bounds {axis}: {value} (must be in [0, 1])")
            }
        }
    }
}

impl std::error::Error for GodToolError {}

impl Simulation {
    pub fn apply_god_tool(&mut self, req: GodToolRequest) -> Result<GodToolReceipt, GodToolError> {
        match req {
            GodToolRequest::Terraform(t) => self.apply_terraform(t),
            GodToolRequest::Material(m) => self.apply_material(m),
            GodToolRequest::Life(l) => self.apply_life(l),
            GodToolRequest::Disaster(d) => self.apply_disaster(d),
            GodToolRequest::Inspect(i) => self.apply_inspect(i),
        }
    }

    fn apply_terraform(&mut self, t: TerraformRequest) -> Result<GodToolReceipt, GodToolError> {
        if t.radius < 0 {
            return Err(GodToolError::InvalidDimension { field: "radius", value: t.radius });
        }
        match t.op {
            TerraformOp::Raise => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension { field: "delta", value: t.delta });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Raise, cells_written: self.raise_footprint(t.center, t.radius, t.delta), center: t.center })
            }
            TerraformOp::Lower => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension { field: "delta", value: t.delta });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Lower, cells_written: self.lower_footprint(t.center, t.radius, t.delta), center: t.center })
            }
            TerraformOp::Level => {
                if t.target_height < 0 {
                    return Err(GodToolError::InvalidDimension { field: "target_height", value: t.target_height });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Level, cells_written: self.level_footprint(t.center, t.radius, t.target_height), center: t.center })
            }
        }
    }

    fn raise_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: center.y + i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, STONE);
                    written += 1;
                }
            }
        }
        written
    }

    fn lower_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        let top_y = self.top_voxel_y(center);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: top_y + i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, AIR);
                    written += 1;
                }
            }
        }
        written
    }

    fn level_footprint(&mut self, center: WorldCoord, radius: i32, target_height: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..target_height {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, STONE);
                    written += 1;
                }
            }
        }
        written
    }

    fn top_voxel_y(&self, center: WorldCoord) -> i64 {
        let scale = civ_voxel::FIXED_SCALE as i64;
        let mut y = center.y;
        for _ in 0..64 {
            let next = WorldCoord { x: center.x, y: y + scale, z: center.z };
            if self.voxel().read(next) == AIR {
                return y;
            }
            y += scale;
        }
        y
    }

    fn apply_material(&mut self, m: MaterialRequest) -> Result<GodToolReceipt, GodToolError> {
        if m.radius < 0 {
            return Err(GodToolError::InvalidDimension { field: "radius", value: m.radius });
        }
        if m.depth <= 0 {
            return Err(GodToolError::InvalidDimension { field: "depth", value: m.depth });
        }
        let written = self.material_replace_footprint(&m);
        Ok(GodToolReceipt::Material { cells_written: written, material: m.material, center: m.center })
    }

    fn material_replace_footprint(&mut self, m: &MaterialRequest) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -m.radius..=m.radius {
            for dz in -m.radius..=m.radius {
                for n in 0..m.depth {
                    self.push_voxel_write(WorldCoord { x: m.center.x + i64::from(dx) * scale, y: m.center.y + i64::from(n) * scale, z: m.center.z + i64::from(dz) * scale }, m.material);
                    written += 1;
                }
            }
        }
        written
    }

    fn apply_life(&mut self, l: LifeRequest) -> Result<GodToolReceipt, GodToolError> {
        let s = l.spawn;
        if !(0.0..=1.0).contains(&s.x) {
            return Err(GodToolError::OutOfBounds { axis: "x", value: s.x });
        }
        if !(0.0..=1.0).contains(&s.y) {
            return Err(GodToolError::OutOfBounds { axis: "y", value: s.y });
        }
        let mut rng = self.rng_mut().clone();
        let entity = spawn_civilian_at(
            &mut self.world,
            s.civilian_id,
            s.alignment,
            s.x,
            s.y,
            s.visual,
            &mut rng,
        );
        *self.rng_mut() = rng;
        Ok(GodToolReceipt::Spawn { entity, civilian_id: s.civilian_id, coord: WorldCoord { x: (s.x.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64, y: 0, z: (s.y.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64 } })
    }

    fn apply_disaster(&mut self, d: DisasterRequest) -> Result<GodToolReceipt, GodToolError> {
        let kind = match d.op {
            DisasterOp::Meteor => DisasterKind::Meteor,
        };
        let fired = self.invoke_divine_disaster(kind, d.center, 0);
        Ok(GodToolReceipt::Disaster { kind, fired, center: d.center })
    }

    fn apply_inspect(&mut self, i: InspectRequest) -> Result<GodToolReceipt, GodToolError> {
        let material = self.voxel().read(i.coord);
        Ok(GodToolReceipt::Inspect { op: i.op, material, nearest_agent: self.nearest_agent(i.coord), coord: i.coord })
    }

    fn nearest_agent(&self, coord: WorldCoord) -> Option<hecs::Entity> {
        use civ_agents::Position3d;
        let scale = civ_voxel::FIXED_SCALE as i64;
        let range = 32 * scale;
        let mut best: Option<(hecs::Entity, i128)> = None;
        for (entity, pos) in self.world.query::<&Position3d>().iter() {
            let dx = (pos.coord.x - coord.x) as i128;
            let dy = (pos.coord.y - coord.y) as i128;
            let dz = (pos.coord.z - coord.z) as i128;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > (range as i128) * (range as i128) {
                continue;
            }
            match best {
                Some((_, d)) if d <= dist_sq => {}
                _ => best = Some((entity, dist_sq)),
            }
        }
        best.map(|(e, _)| e)
    }
}
