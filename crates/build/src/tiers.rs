//! FR-CIV-BUILD-001 / 002 / 003 — Building tiers, production chains, and
//! construction sites.
//!
//! Pinned by `agileplus-specs/civ-004-building-tiers/spec.md`:
//!
//! * FR-CIV-BUILD-001 — `BuildingTier` enum with tier-dependent slot counts,
//!   joule cost, and construction time.
//! * FR-CIV-BUILD-002 — Production chain halts when any required input is
//!   zero; halt logged as `ProductionEvent`.
//! * FR-CIV-BUILD-003 — Scenario YAML overrides apply without recompile;
//!   invalid overrides return a descriptive `BuildingSpecOverrideError`.

use civ_economy::{EconomyState, Good as ProductionGood};
use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};

use crate::BuildingGraph;

/// Era-driven tier classification for a building. Each tier scales slot
/// counts, joule cost, and construction time (`FR-CIV-BUILD-001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingTier {
    /// Hand-crafted, low-throughput (1 input + 1 output slot).
    Primitive,
    /// Skilled-trade era workshops (2 + 2 slots).
    Artisan,
    /// Mechanized industrial production (3 + 3 slots).
    Industrial,
    /// Advanced research / fabrication (4 + 4 slots).
    Advanced,
}

impl BuildingTier {
    /// Number of input slots a building at this tier may consume per tick.
    #[must_use]
    pub const fn input_slots(self) -> u8 {
        match self {
            BuildingTier::Primitive => 1,
            BuildingTier::Artisan => 2,
            BuildingTier::Industrial => 3,
            BuildingTier::Advanced => 4,
        }
    }

    /// Number of output slots a building at this tier may produce per tick.
    #[must_use]
    pub const fn output_slots(self) -> u8 {
        self.input_slots()
    }

    /// Base construction cost in joules for a building at this tier.
    #[must_use]
    pub const fn base_joules(self) -> i64 {
        match self {
            BuildingTier::Primitive => 100,
            BuildingTier::Artisan => 250,
            BuildingTier::Industrial => 500,
            BuildingTier::Advanced => 1_000,
        }
    }

    /// Ticks required to complete construction at this tier.
    #[must_use]
    pub const fn construction_ticks(self) -> u32 {
        match self {
            BuildingTier::Primitive => 5,
            BuildingTier::Artisan => 10,
            BuildingTier::Industrial => 20,
            BuildingTier::Advanced => 40,
        }
    }
}

/// Canonical production chain recipes for a building
/// (`FR-CIV-BUILD-002`). Each chain pins its input and output good
/// signatures so that scenario YAML, production logic, and tests share a
/// single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductionChain {
    /// Agricultural production — consumes no inputs, produces food.
    Farm,
    /// Skilled workshop — consumes wood, produces tools.
    Workshop,
    /// Industrial factory — consumes wood + metal, produces energy + tools.
    Factory,
}

impl ProductionChain {
    /// Goods consumed per tick when this chain is active.
    #[must_use]
    pub const fn inputs(self) -> &'static [ProductionGood] {
        match self {
            ProductionChain::Farm => &[],
            ProductionChain::Workshop => &[ProductionGood::Wood],
            ProductionChain::Factory => &[ProductionGood::Wood, ProductionGood::Metal],
        }
    }

    /// Goods produced per tick when this chain is active.
    #[must_use]
    pub const fn outputs(self) -> &'static [ProductionGood] {
        match self {
            ProductionChain::Farm => &[ProductionGood::Food],
            ProductionChain::Workshop => &[ProductionGood::Tools],
            ProductionChain::Factory => &[ProductionGood::Energy, ProductionGood::Tools],
        }
    }
}

/// Complete specification for a building: tier, chain, and tunable rate
/// (`FR-CIV-BUILD-001` + `FR-CIV-BUILD-003`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSpec {
    /// Tier classification (scales slots, joules, construction time).
    pub tier: BuildingTier,
    /// Production chain recipe (drives input/output goods).
    pub chain: ProductionChain,
    /// Output multiplier applied per tick. Must be ≥ 1.
    pub production_rate: u32,
}

impl BuildingSpec {
    /// Builds a minimal spec with the default production rate of 1 per tick.
    #[must_use]
    pub fn minimal(tier: BuildingTier, chain: ProductionChain) -> Self {
        Self {
            tier,
            chain,
            production_rate: 1,
        }
    }

    /// Returns the spec's [`BuildingTier`].
    #[must_use]
    pub const fn tier(&self) -> BuildingTier {
        self.tier
    }

    /// Returns the spec's [`ProductionChain`].
    #[must_use]
    pub const fn chain(&self) -> ProductionChain {
        self.chain
    }

    /// Returns the spec's per-tick output multiplier.
    #[must_use]
    pub const fn production_rate(&self) -> u32 {
        self.production_rate
    }

    /// Number of input slots provided by this spec (= tier input slots).
    #[must_use]
    pub const fn input_slots(&self) -> u8 {
        self.tier.input_slots()
    }

    /// Number of output slots provided by this spec (= tier output slots).
    #[must_use]
    pub const fn output_slots(&self) -> u8 {
        self.tier.output_slots()
    }

    /// Base joule cost derived from the spec's tier.
    #[must_use]
    pub const fn base_joules(&self) -> i64 {
        self.tier.base_joules()
    }

    /// Applies a [`BuildingSpecOverride`] (typically loaded from a scenario
    /// YAML) on top of this spec (`FR-CIV-BUILD-003`). The override is
    /// validated and rejected with a descriptive error if it would produce
    /// an inconsistent state.
    pub fn apply_override(
        self,
        override_fields: BuildingSpecOverride,
    ) -> Result<Self, BuildingSpecOverrideError> {
        if override_fields.production_rate == 0 {
            return Err(BuildingSpecOverrideError::InvalidValue {
                field: "production_rate".to_owned(),
                value: i64::from(override_fields.production_rate),
            });
        }

        Ok(Self {
            production_rate: override_fields.production_rate,
            ..self
        })
    }
}

/// Override fields that scenario YAML may apply on top of a minimal spec
/// (`FR-CIV-BUILD-003`). Only fields exposed here are tunable at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSpecOverride {
    /// Replacement per-tick output multiplier.
    pub production_rate: u32,
}

/// Error returned when a [`BuildingSpecOverride`] is rejected. The variant
/// always includes the offending field name and rejected value so callers
/// can surface a useful diagnostic (`FR-CIV-BUILD-003`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildingSpecOverrideError {
    /// A field received a value outside the allowed range.
    InvalidValue {
        /// Field path (e.g. `"production_rate"`).
        field: String,
        /// Rejected value.
        value: i64,
    },
}

impl std::fmt::Display for BuildingSpecOverrideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue { field, value } => write!(
                f,
                "invalid override value for field `{field}`: {value} (must be ≥ 1)"
            ),
        }
    }
}

impl std::error::Error for BuildingSpecOverrideError {}

/// Snapshot of a completed building recorded in [`BuildingGraph`]. The full
/// spec is captured so that `phase_economy` can derive per-tick outputs
/// without re-reading the build site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedBuilding {
    /// Stable building id.
    pub id: BuildingId,
    /// Spec the building was constructed with.
    pub spec: BuildingSpec,
    /// World-space origin of the completed building.
    pub origin: WorldCoord,
}

impl CompletedBuilding {
    /// Returns the tier of the completed building.
    #[must_use]
    pub const fn tier(&self) -> BuildingTier {
        self.spec.tier()
    }

    /// Returns the chain of the completed building.
    #[must_use]
    pub const fn chain(&self) -> ProductionChain {
        self.spec.chain()
    }
}

/// Per-tick event recorded by [`BuildSite::produce_and_collect`]
/// (`FR-CIV-BUILD-002` non-functional: production events logged with entity
/// id, good type, quantity, tick number).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionEvent {
    /// The site successfully produced outputs this tick.
    Produced {
        /// Stable building id.
        building_id: BuildingId,
        /// Produced good.
        good: ProductionGood,
        /// Quantity produced.
        quantity: i64,
        /// Simulation tick.
        tick: u64,
    },
    /// The site could not run because a required input was at zero.
    Halted {
        /// Stable building id.
        building_id: BuildingId,
        /// Missing input good.
        missing: ProductionGood,
        /// Simulation tick when the halt was detected.
        tick: u64,
    },
}

/// In-flight construction site. Tracks progress against its spec's
/// construction-tick budget, then transitions to "complete" and can produce
/// goods each tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildSite {
    /// Stable building id.
    id: BuildingId,
    /// Spec driving construction and production.
    spec: BuildingSpec,
    /// World-space origin of the site.
    origin: WorldCoord,
    /// Ticks of construction completed so far.
    progress: u32,
    /// Whether the site has finished construction.
    complete: bool,
}

impl BuildSite {
    /// Creates a new construction site in an unfinished state.
    #[must_use]
    pub const fn new(id: BuildingId, spec: BuildingSpec, origin: WorldCoord) -> Self {
        Self {
            id,
            spec,
            origin,
            progress: 0,
            complete: false,
        }
    }

    /// Returns the site id.
    #[must_use]
    pub const fn id(&self) -> BuildingId {
        self.id
    }

    /// Returns the spec driving this site.
    #[must_use]
    pub const fn spec(&self) -> &BuildingSpec {
        &self.spec
    }

    /// Returns the world-space origin.
    #[must_use]
    pub const fn origin(&self) -> WorldCoord {
        self.origin
    }

    /// Returns the number of ticks of construction completed.
    #[must_use]
    pub const fn progress(&self) -> u32 {
        self.progress
    }

    /// Returns whether the site has completed construction.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// Forces the site into a completed state. Used by tests that exercise
    /// production behavior without paying the full construction cost.
    pub fn complete(&mut self) {
        self.complete = true;
        self.progress = self.spec.tier().construction_ticks();
    }

    /// Advances construction by one tick. Returns `Some(CompletedBuilding)`
    /// on the tick that completes the site, otherwise `None`.
    pub fn tick(&mut self) -> Option<CompletedBuilding> {
        if self.complete {
            return None;
        }

        self.progress = self.progress.saturating_add(1);
        if self.progress >= self.spec.tier().construction_ticks() {
            self.complete = true;
            return Some(CompletedBuilding {
                id: self.id,
                spec: self.spec.clone(),
                origin: self.origin,
            });
        }

        None
    }

    /// Runs the production chain for one tick. No-op when the site has not
    /// completed construction. When the chain halts (an input is at zero)
    /// the call still consumes nothing; callers that need the
    /// [`ProductionEvent`] log should use [`BuildSite::produce_and_collect`].
    pub fn produce(&mut self, economy: &mut EconomyState) {
        let _ = self.produce_and_collect(economy, economy.tick);
    }

    /// Runs the production chain for one tick, returning the
    /// [`ProductionEvent`]s emitted. Each produced good is one event; a
    /// halt is one event. Empty when the site is incomplete.
    pub fn produce_and_collect(
        &mut self,
        economy: &mut EconomyState,
        tick: u64,
    ) -> Vec<ProductionEvent> {
        if !self.complete {
            return Vec::new();
        }

        let mut events = Vec::new();
        let stocks = economy.stocks_mut();

        for &input in self.spec.chain.inputs() {
            if stocks.get(input) == 0 {
                events.push(ProductionEvent::Halted {
                    building_id: self.id,
                    missing: input,
                    tick,
                });
                return events;
            }
        }

        let rate = i64::from(self.spec.production_rate);
        for &output in self.spec.chain.outputs() {
            stocks.add(output, rate);
            events.push(ProductionEvent::Produced {
                building_id: self.id,
                good: output,
                quantity: rate,
                tick,
            });
        }

        events
    }
}

/// Extend [`BuildingGraph`] with completed-site tracking.
impl BuildingGraph {
    /// Records a completed site in the graph. Idempotent: re-recording the
    /// same building id replaces the prior entry.
    pub fn record_completed(&mut self, site: &BuildSite) {
        self.completed.insert(
            site.id(),
            CompletedBuilding {
                id: site.id(),
                spec: site.spec().clone(),
                origin: site.origin(),
            },
        );
    }

    /// Looks up a completed building by id.
    #[must_use]
    pub fn completed(&self, id: BuildingId) -> Option<&CompletedBuilding> {
        self.completed.get(&id)
    }

    /// Returns the number of completed buildings currently recorded.
    #[must_use]
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }
}

