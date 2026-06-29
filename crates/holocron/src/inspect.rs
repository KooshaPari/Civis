//! Inspect-target resolution — FR-CIV-INSPECT-900.
//!
//! The data + query backing layer for the click-to-inspect world flow. Given a
//! `WorldPick` (a position the player clicked, optionally with a renderer hint),
//! [`InspectRegistry::resolve`] returns the most specific entity at that
//! position as a `Summary`, or `None` for empty space.
//!
//! # Scope (per FR-CIV-INSPECT-900)
//!
//! FR-CIV-INSPECT-900 says: "Clicking any world element (voxel, agent,
//! settlement cluster, structure, vehicle) SHALL open a context inspector for
//! that entity." Acceptance: a pick at an agent's location resolves to that
//! agent's summary; empty space resolves to none.
//!
//! This module is the **pure-logic data/query layer** that the Bevy picker
//! (or any other client) feeds picks into. It does not touch Bevy, the
//! renderer, the substrate, or any other crate. It only owns:
//!
//! - Small, dependency-free data structs ([`WorldPos`], [`WorldPick`],
//!   [`EntityId`], [`EntityKind`], [`Summary`], the per-kind summary structs).
//! - In-memory indexes ([`AgentIndex`], [`SettlementIndex`], [`StructureIndex`],
//!   [`VehicleIndex`], [`VoxelIndex`]) that map `EntityId → WorldPos` and back.
//! - A registry ([`InspectRegistry`]) that bundles the indexes and a
//!   [`resolve`](InspectRegistry::resolve) query.
//!
//! # Resolution priority
//!
//! When multiple kinds occupy the same world position (an agent standing in
//! a settlement on a voxel), resolution is **foreground-first**:
//!
//! 1. `Agent`     (units — smallest, topmost)
//! 2. `Vehicle`   (mobile structures)
//! 3. `Structure` (buildings)
//! 4. `Settlement` (cluster — large footprint)
//! 5. `Voxel`     (always present if not empty)
//!
//! `Empty` (returning `None`) only happens when no voxel is registered at
//! the pick's position either — i.e. the pick is outside the world bounds.
//!
//! # Additive contract
//!
//! This module is **additive** with respect to the rest of the crate. It
//! only adds types and queries; it never mutates existing logic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 3-axis integer world position (voxel coordinates).
///
/// The integer type matches the rest of the world (voxel, physics-substrate)
/// and gives the registry a deterministic, hashable key. Rendering layers
/// can project from this into world-space floats when they need to.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct WorldPos {
    /// X axis (voxel column).
    pub x: i32,
    /// Y axis (voxel column).
    pub y: i32,
    /// Z axis — vertical (voxel column).
    pub z: i32,
}

impl WorldPos {
    /// Construct a `WorldPos`. Provided as `const` so test data and
    /// `static` indexes can build picks without a runtime helper.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Stable identifier for an inspectable entity.
///
/// Each kind owns its own id space (an agent id and a structure id can share
/// the same `u64` value without collision because they are tagged by the
/// owning index). The registry does not assume global uniqueness across
/// kinds — only within a kind.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Construct an `EntityId` from a raw `u64`.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw underlying value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Coarse classification of the thing under the cursor.
///
/// Resolution priority (highest → lowest): `Agent`, `Vehicle`, `Structure`,
/// `Settlement`, `Voxel`. `Empty` is never returned by the registry — see
/// [`InspectRegistry::resolve`] for the `Option` contract.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// A living unit (citizen, animal, deity avatar, ...).
    Agent,
    /// A mobile, agent-piloted construct (cart, ship, caravan).
    Vehicle,
    /// A stationary building (house, temple, workshop, ...).
    Structure,
    /// A settlement cluster (overlap set — NOT a `faction:u32`).
    Settlement,
    /// A single voxel cell (material, temperature, phase, ...).
    Voxel,
}

/// A pick event from the world — what the renderer hands us when the player
/// clicks (or hover-targets) a point.
///
/// `hint` is an optional pre-classification from the picking layer (e.g.
/// `bevy_picking` already knows whether it hit a sprite, a mesh, or empty
/// air). The registry treats it as a tie-breaker: if a `hint` is present,
/// that kind is preferred *among equal-priority candidates at the same
/// position*. It never *creates* a hit at an empty position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldPick {
    /// The voxel-space coordinate of the pick.
    pub pos: WorldPos,
    /// Optional pre-classification from the picking layer.
    pub hint: Option<EntityKind>,
}

impl WorldPick {
    /// Construct a pick at `pos` with no hint.
    pub const fn at(pos: WorldPos) -> Self {
        Self { pos, hint: None }
    }

    /// Construct a pick at `pos` with a kind hint.
    pub const fn at_with_hint(pos: WorldPos, hint: EntityKind) -> Self {
        Self { pos, hint: Some(hint) }
    }
}

/// Minimal identity summary for an [`EntityKind::Agent`].
///
/// Mirrors the FR-CIV-INSPECT-901 fields as **labels only** — this layer does
/// not read from `civ-species` / `civ-agents`. A renderer that needs the real
/// values wires its own subscriber; this struct is the *display* shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSummary {
    pub id: EntityId,
    pub display_name: String,
    pub pos: WorldPos,
    pub age_years: Option<u32>,
    pub current_activity: Option<String>,
}

impl AgentSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        match &self.current_activity {
            Some(act) => format!("{} — {} (age {})", self.display_name, act, self.age_years.unwrap_or(0)),
            None => format!("{} (age {})", self.display_name, self.age_years.unwrap_or(0)),
        }
    }
}

/// Minimal summary for an [`EntityKind::Settlement`] (cluster).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementSummary {
    pub id: EntityId,
    pub display_name: String,
    pub centroid: WorldPos,
    pub population: u32,
}

impl SettlementSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        format!("{} — pop. {}", self.display_name, self.population)
    }
}

/// Minimal summary for an [`EntityKind::Structure`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureSummary {
    pub id: EntityId,
    pub display_name: String,
    pub pos: WorldPos,
    pub kind: String,
}

impl StructureSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        format!("{} ({})", self.display_name, self.kind)
    }
}

/// Minimal summary for an [`EntityKind::Vehicle`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleSummary {
    pub id: EntityId,
    pub display_name: String,
    pub pos: WorldPos,
    pub kind: String,
}

impl VehicleSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        format!("{} ({})", self.display_name, self.kind)
    }
}

/// Minimal summary for an [`EntityKind::Voxel`] (material cell).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoxelSummary {
    pub pos: WorldPos,
    pub material: String,
    pub temperature: i32,
    pub pressure: i32,
    pub phase: String,
}

impl VoxelSummary {
    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        format!("{} @ {}K, {}hPa ({})", self.material, self.temperature, self.pressure, self.phase)
    }
}

/// Resolved inspect target — the kind-tagged summary the UI will render.
///
/// `Summary` is an enum (not a single struct with `Option`s) because the
/// FR-CIV-INSPECT-9xx family specifies that each entity kind has its own
/// field set (agent inspector ≠ settlement inspector ≠ voxel inspector).
/// Encoding the variants as an enum keeps the type system honest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Summary {
    Agent(AgentSummary),
    Vehicle(VehicleSummary),
    Structure(StructureSummary),
    Settlement(SettlementSummary),
    Voxel(VoxelSummary),
}

impl Summary {
    /// The kind of entity this summary describes.
    pub fn kind(&self) -> EntityKind {
        match self {
            Self::Agent(_) => EntityKind::Agent,
            Self::Vehicle(_) => EntityKind::Vehicle,
            Self::Structure(_) => EntityKind::Structure,
            Self::Settlement(_) => EntityKind::Settlement,
            Self::Voxel(_) => EntityKind::Voxel,
        }
    }

    /// Stable primary identifier (for log lines, "follow this entity", ...).
    pub fn id(&self) -> Option<EntityId> {
        match self {
            Self::Agent(s) => Some(s.id),
            Self::Vehicle(s) => Some(s.id),
            Self::Structure(s) => Some(s.id),
            Self::Settlement(s) => Some(s.id),
            Self::Voxel(_) => None,
        }
    }

    /// The world position of the resolved entity.
    pub fn pos(&self) -> WorldPos {
        match self {
            Self::Agent(s) => s.pos,
            Self::Vehicle(s) => s.pos,
            Self::Structure(s) => s.pos,
            Self::Settlement(s) => s.centroid,
            Self::Voxel(s) => s.pos,
        }
    }

    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        match self {
            Self::Agent(s) => s.one_line(),
            Self::Vehicle(s) => s.one_line(),
            Self::Structure(s) => s.one_line(),
            Self::Settlement(s) => s.one_line(),
            Self::Voxel(s) => s.one_line(),
        }
    }
}

// ---------------------------------------------------------------------------
// Indexes — one per kind. Each is `BTreeMap<EntityId, WorldPos>` so the
// registry can do O(log n) lookup by id and O(n) reverse lookup by position.
// We keep them separate (not a single heterogeneous map) because the
// per-kind `Summary` payloads are type-distinct.
// ---------------------------------------------------------------------------

/// In-memory index of agents by id and position.
///
/// Backed by two `BTreeMap`s: a primary `id → pos` for `get(id)` and a
/// secondary `pos → id` for `at(pos)`. Both are kept in sync by every
/// mutating method.
#[derive(Debug, Clone, Default)]
pub struct AgentIndex {
    by_id: BTreeMap<EntityId, WorldPos>,
    by_pos: BTreeMap<WorldPos, EntityId>,
}

impl AgentIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or move) an agent. Returns the previous position if any.
    pub fn insert(&mut self, id: EntityId, pos: WorldPos) -> Option<WorldPos> {
        let prev = self.by_id.insert(id, pos);
        if let Some(prev_pos) = prev {
            if prev_pos != pos {
                // Only remove the stale by_pos entry if it still points to us
                // (another agent could have since claimed the same slot).
                if self.by_pos.get(&prev_pos).copied() == Some(id) {
                    self.by_pos.remove(&prev_pos);
                }
            }
        }
        self.by_pos.insert(pos, id);
        prev
    }

    /// Look up an agent's position by id.
    pub fn get(&self, id: EntityId) -> Option<WorldPos> {
        self.by_id.get(&id).copied()
    }

    /// Look up the agent (if any) at a given position.
    pub fn at(&self, pos: WorldPos) -> Option<EntityId> {
        self.by_pos.get(&pos).copied()
    }

    /// Number of registered agents.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// True if no agents are registered.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate all `(id, pos)` pairs in id order.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, WorldPos)> + '_ {
        self.by_id.iter().map(|(k, v)| (*k, *v))
    }
}

/// In-memory index of settlements by id (cluster centroid).
#[derive(Debug, Clone, Default)]
pub struct SettlementIndex {
    by_id: BTreeMap<EntityId, WorldPos>,
    by_pos: BTreeMap<WorldPos, EntityId>,
}

impl SettlementIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or move) a settlement centroid. Returns the previous position if any.
    pub fn insert(&mut self, id: EntityId, pos: WorldPos) -> Option<WorldPos> {
        let prev = self.by_id.insert(id, pos);
        if let Some(prev_pos) = prev {
            if prev_pos != pos && self.by_pos.get(&prev_pos).copied() == Some(id) {
                self.by_pos.remove(&prev_pos);
            }
        }
        self.by_pos.insert(pos, id);
        prev
    }

    /// Look up a settlement's centroid by id.
    pub fn get(&self, id: EntityId) -> Option<WorldPos> {
        self.by_id.get(&id).copied()
    }

    /// Look up the settlement (if any) whose centroid equals `pos`.
    pub fn at(&self, pos: WorldPos) -> Option<EntityId> {
        self.by_pos.get(&pos).copied()
    }

    /// Number of registered settlements.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// True if no settlements are registered.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// In-memory index of structures (buildings) by id and position.
#[derive(Debug, Clone, Default)]
pub struct StructureIndex {
    by_id: BTreeMap<EntityId, WorldPos>,
    by_pos: BTreeMap<WorldPos, EntityId>,
}

impl StructureIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or move) a structure. Returns the previous position if any.
    pub fn insert(&mut self, id: EntityId, pos: WorldPos) -> Option<WorldPos> {
        let prev = self.by_id.insert(id, pos);
        if let Some(prev_pos) = prev {
            if prev_pos != pos && self.by_pos.get(&prev_pos).copied() == Some(id) {
                self.by_pos.remove(&prev_pos);
            }
        }
        self.by_pos.insert(pos, id);
        prev
    }

    /// Look up a structure's position by id.
    pub fn get(&self, id: EntityId) -> Option<WorldPos> {
        self.by_id.get(&id).copied()
    }

    /// Look up the structure (if any) at a given position.
    pub fn at(&self, pos: WorldPos) -> Option<EntityId> {
        self.by_pos.get(&pos).copied()
    }

    /// Number of registered structures.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// True if no structures are registered.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// In-memory index of vehicles by id and position.
#[derive(Debug, Clone, Default)]
pub struct VehicleIndex {
    by_id: BTreeMap<EntityId, WorldPos>,
    by_pos: BTreeMap<WorldPos, EntityId>,
}

impl VehicleIndex {
    /// Construct an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or move) a vehicle. Returns the previous position if any.
    pub fn insert(&mut self, id: EntityId, pos: WorldPos) -> Option<WorldPos> {
        let prev = self.by_id.insert(id, pos);
        if let Some(prev_pos) = prev {
            if prev_pos != pos && self.by_pos.get(&prev_pos).copied() == Some(id) {
                self.by_pos.remove(&prev_pos);
            }
        }
        self.by_pos.insert(pos, id);
        prev
    }

    /// Look up a vehicle's position by id.
    pub fn get(&self, id: EntityId) -> Option<WorldPos> {
        self.by_id.get(&id).copied()
    }

    /// Look up the vehicle (if any) at a given position.
    pub fn at(&self, pos: WorldPos) -> Option<EntityId> {
        self.by_pos.get(&pos).copied()
    }

    /// Number of registered vehicles.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// True if no vehicles are registered.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// In-memory index of voxel cells. Each cell is identified by its
/// `WorldPos` and holds the raw material/temperature/pressure/phase
/// measurements the inspector should display.
///
/// Voxels are keyed by position only (there is no `EntityId` for a cell
/// — they are dense world data, not discrete entities).
#[derive(Debug, Clone, Default)]
pub struct VoxelIndex {
    cells: BTreeMap<WorldPos, VoxelSummary>,
}

impl VoxelIndex {
    /// Construct an empty voxel index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a voxel cell. Returns the previous cell at that position, if any.
    pub fn insert(&mut self, cell: VoxelSummary) -> Option<VoxelSummary> {
        self.cells.insert(cell.pos, cell)
    }

    /// Look up a voxel cell by position.
    pub fn get(&self, pos: WorldPos) -> Option<&VoxelSummary> {
        self.cells.get(&pos)
    }

    /// Number of voxel cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// True if no voxel cells are registered.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

// ---------------------------------------------------------------------------
// The registry — bundles every index and owns the resolve() query.
// ---------------------------------------------------------------------------

/// Bundles all inspect indexes and exposes the [`resolve`](Self::resolve) query.
///
/// Construction is plain: callers fill the indexes with whatever world data
/// they have, then call `resolve(pick)` to get a `Summary` back. The
/// registry does **not** own the world — it is a derived, read-only view
/// that the substrate (or a test) can rebuild on demand.
///
/// The registry is `Clone` because the indexes are `Clone`; for a
/// long-lived process the typical pattern is to keep one canonical
/// registry behind a `RwLock` and clone it for inspection.
#[derive(Debug, Clone, Default)]
pub struct InspectRegistry {
    agents: AgentIndex,
    settlements: SettlementIndex,
    structures: StructureIndex,
    vehicles: VehicleIndex,
    voxels: VoxelIndex,
}

impl InspectRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    // --- mutable accessors (one per index) ---

    /// Mutable handle to the agent index (insert/move agents).
    pub fn agents_mut(&mut self) -> &mut AgentIndex {
        &mut self.agents
    }

    /// Mutable handle to the settlement index.
    pub fn settlements_mut(&mut self) -> &mut SettlementIndex {
        &mut self.settlements
    }

    /// Mutable handle to the structure index.
    pub fn structures_mut(&mut self) -> &mut StructureIndex {
        &mut self.structures
    }

    /// Mutable handle to the vehicle index.
    pub fn vehicles_mut(&mut self) -> &mut VehicleIndex {
        &mut self.vehicles
    }

    /// Mutable handle to the voxel index.
    pub fn voxels_mut(&mut self) -> &mut VoxelIndex {
        &mut self.voxels
    }

    // --- read-only accessors ---

    /// Borrow the agent index.
    pub fn agents(&self) -> &AgentIndex {
        &self.agents
    }

    /// Borrow the settlement index.
    pub fn settlements(&self) -> &SettlementIndex {
        &self.settlements
    }

    /// Borrow the structure index.
    pub fn structures(&self) -> &StructureIndex {
        &self.structures
    }

    /// Borrow the vehicle index.
    pub fn vehicles(&self) -> &VehicleIndex {
        &self.vehicles
    }

    /// Borrow the voxel index.
    pub fn voxels(&self) -> &VoxelIndex {
        &self.voxels
    }

    // --- core query ---

    /// Resolve a world pick to a [`Summary`], or `None` for empty space.
    ///
    /// Resolution order (foreground → background):
    ///
    /// 1. `Agent` at `pick.pos`
    /// 2. `Vehicle` at `pick.pos`
    /// 3. `Structure` at `pick.pos`
    /// 4. `Settlement` whose centroid equals `pick.pos`
    /// 5. `Voxel` at `pick.pos`
    ///
    /// If `pick.hint` is set, that kind is preferred *among the candidates
    /// that exist at `pick.pos`*. The hint never manufactures a hit; if no
    /// entity of any kind is at `pick.pos`, the registry returns `None`.
    pub fn resolve(&self, pick: WorldPick) -> Option<Summary> {
        // Build the list of candidates that actually exist at the pick pos.
        let mut candidates: Vec<(EntityKind, Summary)> = Vec::with_capacity(5);

        if let Some(id) = self.agents.at(pick.pos) {
            if let Some(pos) = self.agents.get(id) {
                candidates.push((
                    EntityKind::Agent,
                    Summary::Agent(AgentSummary {
                        id,
                        display_name: format!("agent:{}", id.raw()),
                        pos,
                        age_years: None,
                        current_activity: None,
                    }),
                ));
            }
        }
        if let Some(id) = self.vehicles.at(pick.pos) {
            if let Some(pos) = self.vehicles.get(id) {
                candidates.push((
                    EntityKind::Vehicle,
                    Summary::Vehicle(VehicleSummary {
                        id,
                        display_name: format!("vehicle:{}", id.raw()),
                        pos,
                        kind: "vehicle".into(),
                    }),
                ));
            }
        }
        if let Some(id) = self.structures.at(pick.pos) {
            if let Some(pos) = self.structures.get(id) {
                candidates.push((
                    EntityKind::Structure,
                    Summary::Structure(StructureSummary {
                        id,
                        display_name: format!("structure:{}", id.raw()),
                        pos,
                        kind: "structure".into(),
                    }),
                ));
            }
        }
        if let Some(id) = self.settlements.at(pick.pos) {
            if let Some(pos) = self.settlements.get(id) {
                candidates.push((
                    EntityKind::Settlement,
                    Summary::Settlement(SettlementSummary {
                        id,
                        display_name: format!("settlement:{}", id.raw()),
                        centroid: pos,
                        population: 0,
                    }),
                ));
            }
        }
        if let Some(v) = self.voxels.get(pick.pos) {
            candidates.push((EntityKind::Voxel, Summary::Voxel(v.clone())));
        }

        if candidates.is_empty() {
            return None;
        }

        // Pick the winner: hint wins on ties, otherwise foreground-first.
        if let Some(hint) = pick.hint {
            if let Some((_, s)) = candidates.iter().find(|(k, _)| *k == hint) {
                return Some(s.clone());
            }
            // Hint didn't match any candidate — fall through to foreground
            // priority. (The hint is advisory, not a hard filter: a player
            // who clicks a wall and the picker says "agent" should still
            // see the wall.)
        }

        // Foreground priority: Agent > Vehicle > Structure > Settlement > Voxel.
        // We rely on the `EntityKind` `Ord` derived from declaration order.
        candidates.into_iter().min_by_key(|(k, _)| *k).map(|(_, s)| s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populated_registry() -> InspectRegistry {
        let mut r = InspectRegistry::new();

        // Two agents: Aldric at (3,4,5), Bryn at (10,0,0).
        r.agents_mut()
            .insert(EntityId::new(101), WorldPos::new(3, 4, 5));
        r.agents_mut()
            .insert(EntityId::new(102), WorldPos::new(10, 0, 0));

        // A structure at the same voxel as Aldric.
        r.structures_mut()
            .insert(EntityId::new(201), WorldPos::new(3, 4, 5));

        // A settlement whose centroid is (3,4,5) (same as Aldric).
        r.settlements_mut()
            .insert(EntityId::new(301), WorldPos::new(3, 4, 5));

        // A vehicle at (50,50,0) and a voxel there.
        r.vehicles_mut()
            .insert(EntityId::new(401), WorldPos::new(50, 50, 0));
        r.voxels_mut().insert(VoxelSummary {
            pos: WorldPos::new(50, 50, 0),
            material: "dirt".into(),
            temperature: 293,
            pressure: 1013,
            phase: "solid".into(),
        });

        // A standalone voxel at (0,0,0).
        r.voxels_mut().insert(VoxelSummary {
            pos: WorldPos::new(0, 0, 0),
            material: "stone".into(),
            temperature: 280,
            pressure: 1013,
            phase: "solid".into(),
        });

        r
    }

    #[test]
    fn world_pos_construction_and_equality() {
        let a = WorldPos::new(1, 2, 3);
        let b = WorldPos::new(1, 2, 3);
        let c = WorldPos::new(3, 2, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.x, 1);
        assert_eq!(a.y, 2);
        assert_eq!(a.z, 3);
    }

    #[test]
    fn entity_id_round_trip() {
        let id = EntityId::new(0xDEAD_BEEF);
        assert_eq!(id.raw(), 0xDEAD_BEEF);
    }

    #[test]
    fn pick_at_no_hint() {
        let pick = WorldPick::at(WorldPos::new(0, 0, 0));
        assert!(pick.hint.is_none());
        assert_eq!(pick.pos, WorldPos::new(0, 0, 0));
    }

    #[test]
    fn pick_with_hint() {
        let pick = WorldPick::at_with_hint(WorldPos::new(0, 0, 0), EntityKind::Agent);
        assert_eq!(pick.hint, Some(EntityKind::Agent));
    }

    #[test]
    fn agent_index_insert_and_lookup() {
        let mut idx = AgentIndex::new();
        assert!(idx.is_empty());
        assert!(idx.insert(EntityId::new(1), WorldPos::new(1, 2, 3)).is_none());
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.get(EntityId::new(1)), Some(WorldPos::new(1, 2, 3)));
        assert_eq!(idx.at(WorldPos::new(1, 2, 3)), Some(EntityId::new(1)));
        assert_eq!(idx.at(WorldPos::new(0, 0, 0)), None);
    }

    #[test]
    fn agent_index_move_updates_both_maps() {
        let mut idx = AgentIndex::new();
        idx.insert(EntityId::new(7), WorldPos::new(1, 1, 1));
        let prev = idx.insert(EntityId::new(7), WorldPos::new(2, 2, 2));
        assert_eq!(prev, Some(WorldPos::new(1, 1, 1)));
        assert_eq!(idx.get(EntityId::new(7)), Some(WorldPos::new(2, 2, 2)));
        // The stale (1,1,1) entry must not still resolve to agent 7.
        assert_eq!(idx.at(WorldPos::new(1, 1, 1)), None);
        assert_eq!(idx.at(WorldPos::new(2, 2, 2)), Some(EntityId::new(7)));
    }

    #[test]
    fn voxel_index_insert_and_get() {
        let mut v = VoxelIndex::new();
        assert!(v.is_empty());
        let cell = VoxelSummary {
            pos: WorldPos::new(0, 0, 0),
            material: "water".into(),
            temperature: 273,
            pressure: 1013,
            phase: "liquid".into(),
        };
        assert!(v.insert(cell.clone()).is_none());
        assert_eq!(v.get(WorldPos::new(0, 0, 0)), Some(&cell));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn summary_kind_matches_variant() {
        let a = Summary::Agent(AgentSummary {
            id: EntityId::new(1),
            display_name: "A".into(),
            pos: WorldPos::new(0, 0, 0),
            age_years: None,
            current_activity: None,
        });
        let s = Summary::Settlement(SettlementSummary {
            id: EntityId::new(2),
            display_name: "S".into(),
            centroid: WorldPos::new(5, 5, 5),
            population: 12,
        });
        let v = Summary::Voxel(VoxelSummary {
            pos: WorldPos::new(0, 0, 0),
            material: "sand".into(),
            temperature: 300,
            pressure: 1000,
            phase: "solid".into(),
        });
        assert_eq!(a.kind(), EntityKind::Agent);
        assert_eq!(s.kind(), EntityKind::Settlement);
        assert_eq!(v.kind(), EntityKind::Voxel);
        assert_eq!(a.id(), Some(EntityId::new(1)));
        assert_eq!(v.id(), None); // voxels have no entity id
        assert_eq!(a.pos(), WorldPos::new(0, 0, 0));
        assert_eq!(s.pos(), WorldPos::new(5, 5, 5));
    }

    // ---- FR-CIV-INSPECT-900 acceptance tests ----

    /// Acceptance: "a pick at an agent's location resolves to that agent's
    /// summary".
    #[test]
    fn acceptance_pick_at_agent_resolves_to_agent() {
        let r = populated_registry();
        let pick = WorldPick::at(WorldPos::new(3, 4, 5));
        let s = r.resolve(pick).expect("must resolve at an agent's position");
        assert_eq!(s.kind(), EntityKind::Agent, "agent must beat structure + settlement at the same pos");
        match s {
            Summary::Agent(a) => {
                assert_eq!(a.id, EntityId::new(101));
                assert_eq!(a.pos, WorldPos::new(3, 4, 5));
            }
            _ => unreachable!(),
        }
    }

    /// Acceptance: "empty space resolves to none".
    #[test]
    fn acceptance_empty_space_resolves_to_none() {
        let r = populated_registry();
        // (999, 999, 999) has no entity of any kind.
        let pick = WorldPick::at(WorldPos::new(999, 999, 999));
        assert!(r.resolve(pick).is_none());
    }

    /// Empty registry, empty pick → `None`.
    #[test]
    fn empty_registry_resolves_to_none() {
        let r = InspectRegistry::new();
        assert!(r.resolve(WorldPick::at(WorldPos::new(0, 0, 0))).is_none());
    }

    /// Voxel-only pick resolves to a `VoxelSummary`.
    #[test]
    fn voxel_only_pick_resolves_to_voxel() {
        let r = populated_registry();
        let s = r.resolve(WorldPick::at(WorldPos::new(0, 0, 0))).expect("voxel present");
        assert_eq!(s.kind(), EntityKind::Voxel);
    }

    /// Agent beats structure when both sit at the same position.
    #[test]
    fn agent_beats_structure_at_same_pos() {
        let r = populated_registry();
        let s = r
            .resolve(WorldPick::at(WorldPos::new(3, 4, 5)))
            .expect("must resolve");
        assert_eq!(s.kind(), EntityKind::Agent);
    }

    /// When no agent is present, a structure wins over a settlement at the
    /// same position.
    #[test]
    fn structure_beats_settlement_at_same_pos() {
        let mut r = InspectRegistry::new();
        r.structures_mut()
            .insert(EntityId::new(201), WorldPos::new(3, 4, 5));
        r.settlements_mut()
            .insert(EntityId::new(301), WorldPos::new(3, 4, 5));
        let s = r
            .resolve(WorldPick::at(WorldPos::new(3, 4, 5)))
            .expect("must resolve");
        assert_eq!(s.kind(), EntityKind::Structure);
    }

    /// The picker's `hint` is honored when it names a kind that exists at
    /// the position. Here we have both an agent and a structure at the
    /// same voxel; a `Structure` hint flips the winner.
    #[test]
    fn hint_overrides_foreground_priority() {
        let r = populated_registry();
        let s = r
            .resolve(WorldPick::at_with_hint(
                WorldPos::new(3, 4, 5),
                EntityKind::Structure,
            ))
            .expect("must resolve");
        assert_eq!(s.kind(), EntityKind::Structure);
    }

    /// If the hint names a kind that is *not* present, the registry falls
    /// back to foreground priority (the hint is advisory, not a filter).
    #[test]
    fn hint_for_missing_kind_falls_back() {
        let r = populated_registry();
        // No `Vehicle` at (3,4,5), but an agent, structure, and settlement are there.
        let s = r
            .resolve(WorldPick::at_with_hint(
                WorldPos::new(3, 4, 5),
                EntityKind::Vehicle,
            ))
            .expect("must still resolve");
        assert_eq!(s.kind(), EntityKind::Agent);
    }

    /// The vehicle index works end-to-end.
    #[test]
    fn vehicle_pick_resolves() {
        let r = populated_registry();
        let s = r
            .resolve(WorldPick::at(WorldPos::new(50, 50, 0)))
            .expect("must resolve");
        assert_eq!(s.kind(), EntityKind::Vehicle);
    }

    /// `one_line` is non-empty for every summary variant.
    #[test]
    fn one_line_is_nonempty_for_every_variant() {
        let r = populated_registry();
        let variants = [
            WorldPos::new(3, 4, 5),    // agent
            WorldPos::new(50, 50, 0),  // vehicle
            WorldPos::new(0, 0, 0),    // voxel
        ];
        for pos in variants {
            let s = r.resolve(WorldPick::at(pos)).expect("must resolve");
            assert!(!s.one_line().is_empty(), "{:?} produced empty one_line", s.kind());
        }
    }

    /// Resolution is deterministic — calling it twice yields identical results.
    #[test]
    fn resolve_is_deterministic() {
        let r = populated_registry();
        let pick = WorldPick::at(WorldPos::new(3, 4, 5));
        let a = r.resolve(pick).expect("must resolve");
        let b = r.resolve(pick).expect("must resolve");
        assert_eq!(a, b);
    }

    /// Agent, settlement, structure, vehicle, voxel indexes are all
    /// independently observable.
    #[test]
    fn registry_exposes_all_indexes() {
        let r = populated_registry();
        assert_eq!(r.agents().len(), 2);
        assert_eq!(r.settlements().len(), 1);
        assert_eq!(r.structures().len(), 1);
        assert_eq!(r.vehicles().len(), 1);
        assert_eq!(r.voxels().len(), 2);
    }
}
