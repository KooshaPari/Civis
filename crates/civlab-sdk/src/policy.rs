//! PolicyMod guest API (CIV-0700 §5).

/// `action_emit` type: set tax rate (CIV-0700 §5.3 / mod-host `ACTION_SET_TAX_RATE`).
pub const ACTION_SET_TAX_RATE: u32 = 1;
/// `action_emit` type: set policy parameter (`ACTION_SET_POLICY_PARAM`).
pub const ACTION_SET_POLICY_PARAM: u32 = 2;
/// `action_emit` type: set subsidy rate.
pub const ACTION_SET_SUBSIDY_RATE: u32 = 3;
/// `action_emit` type: transfer funds between actors.
pub const ACTION_TRANSFER_FUNDS: u32 = 4;
/// `action_emit` type: trigger a simulation event.
pub const ACTION_TRIGGER_EVENT: u32 = 5;

/// World-state domain tags for read-only context slices.
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
    /// Population headcount.
    pub population: i64,
}

/// Read-only view of world state provided to [`PolicyMod::on_tick`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolicyContext {
    /// Current simulation tick.
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PolicyAction {
    /// Set the headline tax rate (permille).
    SetTaxRate {
        /// Tax rate in permille.
        rate_permille: i64,
    },
    /// Set a named policy parameter on the bound nation.
    SetPolicyParam {
        /// Stable hash of the parameter key.
        key_hash: u64,
        /// Parameter value.
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
    /// Stable hash of the event type string.
    pub event_type_hash: u64,
    /// Opaque payload length in bytes.
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
pub trait PolicyMod: Send + 'static {
    /// Called once per tick during Phase 3a (Policy Application).
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction>;

    /// Called when a matching [`SimEvent`] arrives.
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
        assert_eq!(ACTION_SET_TAX_RATE, 1);
        assert_eq!(ACTION_SET_POLICY_PARAM, 2);
        assert_eq!(ACTION_SET_SUBSIDY_RATE, 3);
        assert_eq!(ACTION_TRANSFER_FUNDS, 4);
        assert_eq!(ACTION_TRIGGER_EVENT, 5);
    }

    #[test]
    fn policy_action_action_type_mapping() {
        assert_eq!(
            PolicyAction::SetTaxRate {
                rate_permille: 100
            }
            .action_type(),
            ACTION_SET_TAX_RATE
        );
    }

    #[test]
    fn noop_policy_mod_trait_object() {
        let mut policy = NoopPolicy;
        let ctx = PolicyContext::default();
        assert!(policy.on_tick(&ctx).is_empty());
        assert_eq!(policy.metadata().id, "noop");
    }
}
