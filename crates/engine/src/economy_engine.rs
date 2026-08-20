//! Economy engine subsystem for the simulation engine.
//!
//! This module contains helpers and types related to economy state management,
//! resource allocation, and market mechanics. The actual `phase_economy` method
//! remains in `engine.rs` as it requires `&mut Simulation` access.

use crate::engine::{Fixed, ResourceType, WorldState, SCALE};
use civ_economy::EconomyState;

/// Derive an [`EconomyState`] from the current [`WorldState`].
///
/// Syncs the energy budget and tick counter so the economy phase starts from
/// a consistent snapshot.
pub(crate) fn economy_state_from_world(world: &WorldState) -> EconomyState {
    let energy_budget_joules = i64::from(world.energy_budget_joules.to_bits()) / SCALE;
    let mut state = EconomyState::with_energy_budget(energy_budget_joules);
    state.tick = world.tick;
    state
}

/// Compute a baseline market price from supply/demand balance.
///
/// Returns a price in milliunits centered on `1_000` with a ±250 band
/// driven by the demand surplus or deficit.
pub(crate) fn market_price_from_balance(supply: i64, demand: i64) -> i64 {
    let supply = supply.max(0);
    let demand = demand.max(0);
    let balance = demand.saturating_sub(supply);
    1_000i64.saturating_add(balance.clamp(-250, 250))
}
