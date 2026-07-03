pub mod cooldown;
pub mod registry;
pub mod synergy;

pub use cooldown::PowerCooldown;
pub use registry::{
    PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRegistry, PowerRequestKind,
    PowerTab, PowerTargetMask, FORBIDDEN_TARGET_FIELDS,
};
pub use synergy::{synergy_multiplier, SynergyEdge, SynergyOutcome, MAX_MULT, MIN_MULT};

/// Default (empty) set of god-powers. Phase 5 mod-registration will
/// populate this with the full catalog.
#[must_use]
pub fn default_powers() -> &'static [PowerDef] {
    &[]
}
