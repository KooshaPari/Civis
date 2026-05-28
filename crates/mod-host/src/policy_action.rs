//! Map PolicyMod action discriminants to `civlab::action_emit` type constants (CIV-0700 §5.3).

use crate::capability::{
    ACTION_SET_POLICY_PARAM, ACTION_SET_SUBSIDY_RATE, ACTION_SET_TAX_RATE, ACTION_TRANSFER_FUNDS,
    ACTION_TRIGGER_EVENT,
};

/// Known policy action discriminants understood by the MVP host bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PolicyActionKind {
    /// Set tax rate (`ACTION_SET_TAX_RATE`).
    SetTaxRate = ACTION_SET_TAX_RATE,
    /// Set policy parameter (`ACTION_SET_POLICY_PARAM`).
    SetPolicyParam = ACTION_SET_POLICY_PARAM,
    /// Set subsidy rate (`ACTION_SET_SUBSIDY_RATE`).
    SetSubsidyRate = ACTION_SET_SUBSIDY_RATE,
    /// Transfer funds (`ACTION_TRANSFER_FUNDS`).
    TransferFunds = ACTION_TRANSFER_FUNDS,
    /// Trigger event (`ACTION_TRIGGER_EVENT`).
    TriggerEvent = ACTION_TRIGGER_EVENT,
}

impl PolicyActionKind {
    /// Parse a guest ABI action type into a known kind.
    #[must_use]
    pub fn from_emit_type(action_type: u32) -> Option<Self> {
        match action_type {
            ACTION_SET_TAX_RATE => Some(Self::SetTaxRate),
            ACTION_SET_POLICY_PARAM => Some(Self::SetPolicyParam),
            ACTION_SET_SUBSIDY_RATE => Some(Self::SetSubsidyRate),
            ACTION_TRANSFER_FUNDS => Some(Self::TransferFunds),
            ACTION_TRIGGER_EVENT => Some(Self::TriggerEvent),
            _ => None,
        }
    }

    /// Return the `action_emit` type constant for this kind.
    #[must_use]
    pub fn to_emit_type(self) -> u32 {
        self as u32
    }
}

/// Map a policy action discriminant to the host `action_emit` type constant.
///
/// Returns `None` when the discriminant is not a registered policy action.
#[must_use]
pub fn policy_action_to_emit_type(action_type: u32) -> Option<u32> {
    PolicyActionKind::from_emit_type(action_type).map(PolicyActionKind::to_emit_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_action_constants_match_civlab_sdk() {
        // Must stay in sync with civlab-sdk/src/policy.rs (CIV-0700 §5.3).
        assert_eq!(ACTION_SET_TAX_RATE, 1);
        assert_eq!(ACTION_SET_POLICY_PARAM, 2);
        assert_eq!(ACTION_SET_SUBSIDY_RATE, 3);
        assert_eq!(ACTION_TRANSFER_FUNDS, 4);
        assert_eq!(ACTION_TRIGGER_EVENT, 5);
    }

    #[test]
    fn policy_action_kind_round_trips_emit_type() {
        for kind in [
            PolicyActionKind::SetTaxRate,
            PolicyActionKind::SetPolicyParam,
            PolicyActionKind::SetSubsidyRate,
            PolicyActionKind::TransferFunds,
            PolicyActionKind::TriggerEvent,
        ] {
            let emit = kind.to_emit_type();
            assert_eq!(PolicyActionKind::from_emit_type(emit), Some(kind));
            assert_eq!(policy_action_to_emit_type(emit), Some(emit));
        }
    }

    #[test]
    fn unknown_policy_action_discriminant_returns_none() {
        assert_eq!(policy_action_to_emit_type(0), None);
        assert_eq!(policy_action_to_emit_type(99), None);
    }
}
