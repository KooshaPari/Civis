//! PolicyMod guest API (CIV-0700 §5).
//!
//! Defines the [`PolicyMod`] trait, read-only [`PolicyContext`] views, and
//! [`PolicyAction`] values emitted into Phase 3a. Host-side enforcement maps
//! action discriminants to `civlab::action_emit` type constants (see
//! `civ-mod-host::policy_action`).

/// `action_emit` type: set tax rate (CIV-0700 §5.3 / mod-host `ACTION_SET_TAX_RATE`).
pub const ACTION_SET_TAX_RATE: u32 = 1;
/// `action_emit` type: set policy parameter (`ACTION_SET_POLICY_PARAM`).
pub const ACTION_SET_POLICY_PARAM: u32 = 2;
/// `action_emit` type: set subsidy rate (reserved; host enforces separately).
pub const ACTION_SET_SUBSIDY_RATE: u32 = 3;
/// `action_emit` type: transfer funds between actors.
pub const ACTION_TRANSFER_FUNDS: u32 = 4;
/// `action_emit` type: trigger a simulation event.
pub const ACTION_TRIGGER_EVENT: u32 = 5;

/// World-state domain tags for read-only context slices (mirrors mod-host [`WorldDomain`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WorldDomain {
    /// Economy ledger and production state.
    Economy = 0,
    /// Climate and atmospheric state.
    Climate = 1,
    /// Military units and conflict state.
    Military = 2,
    /// Diplomacy and treaties.
    Diplomacy = 3,
    /// Citizen demographics and welfare.
    Citizens = 4,
}

impl WorldDomain {
    /// Parse a domain tag from the guest ABI (`i32`).
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Economy),
            1 => Some(Self::Climate),
            2 => Some(Self::Military),
            3 => Some(Self::Diplomacy),
            4 => Some(Self::Citizens),
            _ => None,
        }
    }
}

/// Minimal economy snapshot visible to policy mods (MVP stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EconomySnapshot {
    /// Nation treasury balance (fixed-point milli-units).
    pub treasury_millijoules: i64,
}

/// Minimal climate snapshot visible to policy mods (MVP stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClimateSnapshot {
    /// CO₂ concentration in milli-ppm.
    pub co2_ppm_milliunits: i64,
}

/// Minimal military snapshot visible to policy mods (MVP stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MilitarySnapshot {
    /// Aggregate unit count for the bound nation.
    pub unit_count: i32,
}

/// Minimal diplomacy snapshot visible to policy mods (MVP stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiplomacySnapshot {
    /// Count of active treaties involving the bound nation.
    pub treaty_count: i32,
}

/// Minimal citizen welfare snapshot visible to policy mods (MVP stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CitizensSnapshot {
    /// Population headcount (fixed-point not required for MVP).
    pub population: i64,
}

/// Read-only view of world state provided to [`PolicyMod::on_tick`].
///
/// Domain snapshots are present only when the mod declared the corresponding
/// read permission in its manifest; otherwise the field is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolicyContext {
    /// Current simulation tick (monotonically increasing from 0).
    pub tick: u64,
    /// Economy slice when `read_economy = true`.
    pub economy: Option<EconomySnapshot>,
    /// Climate slice when `read_climate = true`.
    pub climate: Option<ClimateSnapshot>,
    /// Military slice when `read_military = true`.
    pub military: Option<MilitarySnapshot>,
    /// Diplomacy slice when `read_diplomacy = true`.
    pub diplomacy: Option<DiplomacySnapshot>,
    /// Citizens slice when `read_citizens = true`.
    pub citizens: Option<CitizensSnapshot>,
}

/// Actions a [`PolicyMod`] can emit into Phase 3a.
///
/// MVP surface: tax rate and policy-parameter updates. Additional variants from
/// CIV-0700 §5.3 will be added without breaking existing discriminants.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PolicyAction {
    /// Set the headline tax rate (permille: 0..=1000 maps to 0%..=100%).
    SetTaxRate {
        /// Tax rate in permille (host validates/clamps).
        rate_permille: i64,
    },
    /// Set a named policy parameter on the bound nation.
    SetPolicyParam {
        /// Stable hash of the parameter key (host resolves to manifest key table).
        key_hash: u64,
        /// Parameter value (fixed-point scale matches engine policy params).
        value: i64,
    },
}

impl PolicyAction {
    /// `civlab::action_emit` type discriminant for this action.
    #[must_use]
    pub fn action_type(&self) -> u32 {
        match self {
            Self::SetTaxRate { .. } => ACTION_SET_TAX_RATE,
            Self::SetPolicyParam { .. } => ACTION_SET_POLICY_PARAM,
        }
    }
}

/// Simulation event delivered to [`PolicyMod::on_event`] (MVP stub).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimEvent {
    /// Simulation tick when the event was raised.
    pub tick: u64,
    /// Stable hash of the event type string (`climate.co2_spike`, etc.).
    pub event_type_hash: u64,
    /// Opaque payload length in bytes (payload bytes live in host memory).
    pub payload_len: u32,
}

/// Static metadata returned by [`PolicyMod::metadata`] during init.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModMetadata {
    /// Mod identifier; must match manifest `mod.id`.
    pub id: String,
    /// Display name for UI and logging.
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Event type hashes this mod wants via [`PolicyMod::on_event`].
    pub subscribed_event_hashes: Vec<u64>,
}

/// Primary trait for policy mods (CIV-0700 §5.1).
///
/// Implementors define custom policy algorithms that inject [`PolicyAction`]
/// values into the engine's Phase 3a pipeline. All methods must be
/// deterministic — do not use wall-clock time or platform RNG.
pub trait PolicyMod: Send + 'static {
    /// Called once per tick during Phase 3a (Policy Application).
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction>;

    /// Called when a [`SimEvent`] matching [`ModMetadata::subscribed_event_hashes`] arrives.
    fn on_event(&mut self, event: &SimEvent) -> Vec<PolicyAction>;

    /// Returns static metadata stored by the host at load time.
    fn metadata(&self) -> ModMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopPolicy;

    impl PolicyMod for NoopPolicy {
        fn on_tick(&mut self, _ctx: &PolicyContext) -> Vec<PolicyAction> {
            Vec::new()
        }

        fn on_event(&mut self, _event: &SimEvent) -> Vec<PolicyAction> {
            Vec::new()
        }

        fn metadata(&self) -> ModMetadata {
            ModMetadata {
                id: "noop".to_owned(),
                name: "No-op".to_owned(),
                version: "0.0.0".to_owned(),
                subscribed_event_hashes: Vec::new(),
            }
        }
    }

    #[test]
    fn policy_action_type_constants_match_mod_host() {
        // Keep in sync with civ-mod-host/src/capability.rs (CIV-0700 §5.3).
        assert_eq!(ACTION_SET_TAX_RATE, 1);
        assert_eq!(ACTION_SET_POLICY_PARAM, 2);
        assert_eq!(ACTION_SET_SUBSIDY_RATE, 3);
        assert_eq!(ACTION_TRANSFER_FUNDS, 4);
        assert_eq!(ACTION_TRIGGER_EVENT, 5);
    }

    #[test]
    fn policy_action_action_type_mapping() {
        assert_eq!(
            PolicyAction::SetTaxRate { rate_permille: 100 }.action_type(),
            ACTION_SET_TAX_RATE
        );
        assert_eq!(
            PolicyAction::SetPolicyParam {
                key_hash: 42,
                value: 7
            }
            .action_type(),
            ACTION_SET_POLICY_PARAM
        );
    }

    #[test]
    fn world_domain_from_i32_round_trips() {
        assert_eq!(WorldDomain::from_i32(0), Some(WorldDomain::Economy));
        assert_eq!(WorldDomain::from_i32(4), Some(WorldDomain::Citizens));
        assert_eq!(WorldDomain::from_i32(99), None);
    }

    #[test]
    fn noop_policy_mod_trait_object() {
        let mut policy = NoopPolicy;
        let ctx = PolicyContext::default();
        assert!(policy.on_tick(&ctx).is_empty());
        assert_eq!(policy.metadata().id, "noop");
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod example_policy_stub {
        use super::super::{
            ClimateSnapshot, ModMetadata, PolicyAction, PolicyContext, PolicyMod, SimEvent,
            ACTION_SET_TAX_RATE,
        };

        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../mods/example-policy/policy_stub.rs"
        ));

        #[test]
        fn example_policy_stub_compiles_and_runs() {
            let mut policy = ExampleCarbonTaxPolicy::default();
            let ctx = PolicyContext {
                tick: 1,
                climate: Some(ClimateSnapshot {
                    co2_ppm_milliunits: 500_000,
                }),
                ..PolicyContext::default()
            };
            let actions = policy.on_tick(&ctx);
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].action_type(), ACTION_SET_TAX_RATE);
        }
    }
}
