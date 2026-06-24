use civ_agents::{Civilian, LodTier, Position3d};

/// Birth event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BirthEvent {
    /// Simulation tick.
    pub tick: u64,
    /// New civilian.
    pub civilian: Civilian,
    /// Spawn position.
    pub position: Position3d,
}

/// Death event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathEvent {
    /// Simulation tick.
    pub tick: u64,
    /// Dead civilian.
    pub civilian: Civilian,
    /// Current LOD tier at death.
    pub lod: LodTier,
}

/// Tech-change event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechEvent {
    /// Simulation tick.
    pub tick: u64,
    /// Stable tech id or era tag.
    pub tech_id: String,
    /// Civilian affected by the tech change.
    pub civilian: Civilian,
}

/// Unified simulation event for mod hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationEvent {
    /// Birth.
    Birth(BirthEvent),
    /// Death.
    Death(DeathEvent),
    /// Technology adoption or unlock.
    Tech(TechEvent),
}

/// Trait for event listeners.
pub trait SimulationEventHook {
    /// Handle a birth event.
    fn on_birth(&mut self, event: &BirthEvent);

    /// Handle a death event.
    fn on_death(&mut self, event: &DeathEvent);

    /// Handle a tech event.
    fn on_tech(&mut self, event: &TechEvent);
}
