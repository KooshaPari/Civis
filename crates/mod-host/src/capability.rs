//! Runtime capability enforcement for WASM mods (CIV-0700 §4.3 / §5–8 MVP).

use crate::ModPermissions;

/// Host error code: domain or action type not in mod capability set.
pub const ERR_PERMISSION_DENIED: i32 = -2;

/// Policy action: set tax rate for a good category.
pub const ACTION_SET_TAX_RATE: u32 = 1;
/// Policy action: set a named policy parameter.
pub const ACTION_SET_POLICY_PARAM: u32 = 2;
/// Policy action: set subsidy rate.
pub const ACTION_SET_SUBSIDY_RATE: u32 = 3;
/// Policy action: transfer funds between actors.
pub const ACTION_TRANSFER_FUNDS: u32 = 4;
/// Policy action: trigger a world event.
pub const ACTION_TRIGGER_EVENT: u32 = 5;

/// World-state domain tags for `world_read` capability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Runtime mod lifecycle status (CIV-0700 §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModStatus {
    /// Normal operation.
    #[default]
    Active,
    /// Too many permission violations in one tick; callbacks skipped until next tick.
    Suspended,
    /// Requires operator reset after repeated suspensions.
    Faulted,
    /// Approaching CPU budget limits.
    Degraded,
    /// Unload in progress.
    Unloading,
}

/// Compiled manifest permissions used for runtime enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModCapabilitySet {
    read_economy: bool,
    read_climate: bool,
    read_military: bool,
    read_diplomacy: bool,
    read_citizens: bool,
    write_policy: bool,
    write_economy: bool,
    write_events: bool,
    write_scenario: bool,
    transfer_funds: bool,
}

impl ModCapabilitySet {
    /// Build a capability set from manifest `[permissions]`.
    #[must_use]
    pub fn from_permissions(perms: &ModPermissions) -> Self {
        Self {
            read_economy: perms.read_economy,
            read_climate: perms.read_climate,
            read_military: perms.read_military,
            read_diplomacy: perms.read_diplomacy,
            read_citizens: perms.read_citizens,
            write_policy: perms.write_policy,
            write_economy: perms.write_economy,
            write_events: perms.write_events,
            write_scenario: perms.write_scenario,
            transfer_funds: perms.transfer_funds,
        }
    }

    /// Permissive set for host-side tests that do not model manifest permissions.
    #[must_use]
    pub fn allow_all() -> Self {
        Self {
            read_economy: true,
            read_climate: true,
            read_military: true,
            read_diplomacy: true,
            read_citizens: true,
            write_policy: true,
            write_economy: true,
            write_events: true,
            write_scenario: true,
            transfer_funds: true,
        }
    }

    /// Whether the mod may read the given world domain.
    #[must_use]
    pub fn can_read_domain(&self, domain: WorldDomain) -> bool {
        match domain {
            WorldDomain::Economy => self.read_economy,
            WorldDomain::Climate => self.read_climate,
            WorldDomain::Military => self.read_military,
            WorldDomain::Diplomacy => self.read_diplomacy,
            WorldDomain::Citizens => self.read_citizens,
        }
    }

    /// Whether the mod may emit the given action type discriminant.
    #[must_use]
    pub fn can_emit_action(&self, action_type: u32) -> bool {
        match action_type {
            ACTION_SET_TAX_RATE | ACTION_SET_POLICY_PARAM | ACTION_SET_SUBSIDY_RATE => {
                self.write_policy
            }
            ACTION_TRANSFER_FUNDS => self.transfer_funds && self.write_policy,
            ACTION_TRIGGER_EVENT => self.write_events,
            _ => self.write_economy || self.write_scenario,
        }
    }
}

/// Per-tick violation tracking passed into guest invocation.
#[derive(Debug, Clone, Default)]
pub struct ModEnforcementCtx {
    /// Violations accumulated during the current simulation tick.
    pub violations: u32,
    /// Set when violations reach the suspension threshold within a tick.
    pub suspended: bool,
    /// Host import that caused the most recent denial (`world_read`, `action_emit`, …).
    pub last_call: Option<String>,
    /// World domain tag when the denial was from `world_read`.
    pub last_domain: Option<WorldDomain>,
}

impl ModEnforcementCtx {
    /// Record a permission denial; may set [`ModEnforcementCtx::suspended`].
    pub fn record_denial(&mut self, call: &str, domain: Option<WorldDomain>) {
        self.violations = self.violations.saturating_add(1);
        self.last_call = Some(call.to_owned());
        self.last_domain = domain;
        if self.violations >= 5 {
            self.suspended = true;
        }
    }
}

/// Format `mod.permission_violation.v1` as JSON for the replay bus.
#[must_use]
pub fn format_mod_permission_violation_json(
    mod_id: &str,
    tick: u64,
    call: &str,
    domain: Option<WorldDomain>,
) -> String {
    serde_json::json!({
        "event": "mod.permission_violation.v1",
        "mod_id": mod_id,
        "tick": tick,
        "call": call,
        "domain": domain.map(|d| format!("{d:?}")),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_set_from_manifest_permissions() {
        let perms = ModPermissions {
            read_economy: true,
            read_military: true,
            write_policy: true,
            ..ModPermissions::default()
        };
        let caps = ModCapabilitySet::from_permissions(&perms);
        assert!(caps.can_read_domain(WorldDomain::Economy));
        assert!(caps.can_read_domain(WorldDomain::Military));
        assert!(!caps.can_read_domain(WorldDomain::Climate));
        assert!(caps.can_emit_action(ACTION_SET_TAX_RATE));
        assert!(caps.can_emit_action(ACTION_SET_POLICY_PARAM));
        assert!(!caps.can_emit_action(ACTION_TRIGGER_EVENT));
    }

    #[test]
    fn record_denial_tracks_call_and_domain() {
        let mut ctx = ModEnforcementCtx::default();
        ctx.record_denial("world_read", Some(WorldDomain::Military));
        assert_eq!(ctx.violations, 1);
        assert_eq!(ctx.last_call.as_deref(), Some("world_read"));
        assert_eq!(ctx.last_domain, Some(WorldDomain::Military));
    }

    #[test]
    fn permission_violation_json_includes_event_and_call() {
        let json = format_mod_permission_violation_json("demo-mod", 42, "action_emit", None);
        assert!(json.contains("mod.permission_violation.v1"));
        assert!(json.contains("demo-mod"));
        assert!(json.contains("action_emit"));
    }
}
