//! FR-CIV-SOCIAL-001-INSTITUTIONS — institution policy / membership record.
//!
//! Per the spec at `agileplus-specs/civ-021-recovered-requirements/spec.md` and
//! the `PLAN.md:147-148` line, a civic institution carries:
//! - a set of active policies (string keyed)
//! - a roster of member citizen ids
//! - a treasury / budget (basis points of joule surplus)
//! - an approval rating in `[0, 1]`
//!
//! This complements the existing `Institution { kind, level }` record (which is
//! a population-gated L1/L2 unlock marker) without changing its shape.

use serde::{Deserialize, Serialize};

/// Civic institution record carrying policies, membership, budget, approval.
///
/// Use [`InstitutionPolicy::new`] to construct a fresh empty record and
/// [`add_member`](Self::add_member), [`remove_member`](Self::remove_member),
/// [`update_policy`](Self::update_policy) to mutate it. All numeric fields are
/// clamped to their declared ranges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionPolicy {
    /// Citizen ids currently on the roster. Order is not significant.
    pub members: Vec<u32>,
    /// Policy name → policy value (free-form string). The engine treats
    /// policy values as opaque tokens and only this struct owns them.
    pub policies: Vec<(String, String)>,
    /// Budget in basis points of joule surplus; positive = surplus, negative = deficit.
    pub budget_bp: i64,
    /// Public approval rating in `[0, 1]` (1.0 = unanimous, 0.0 = total dissent).
    pub approval_rating_fp: i32,
}

/// Approval-rating fixed-point scale (divide by this to recover `[0, 1]`).
pub const APPROVAL_FP_SCALE: i32 = 1_000;

impl InstitutionPolicy {
    /// Construct an empty institution record (no members, no policies, zero
    /// budget, neutral 0.5 approval rating).
    #[must_use]
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
            policies: Vec::new(),
            budget_bp: 0,
            approval_rating_fp: APPROVAL_FP_SCALE / 2,
        }
    }

    /// Add `citizen_id` to the membership roster. Returns `true` if the citizen
    /// was newly added (idempotent: re-adding an existing member is a no-op).
    pub fn add_member(&mut self, citizen_id: u32) -> bool {
        if self.members.contains(&citizen_id) {
            return false;
        }
        self.members.push(citizen_id);
        true
    }

    /// Remove `citizen_id` from the roster. Returns `true` if the citizen was
    /// present and removed; `false` if they were not on the roster.
    pub fn remove_member(&mut self, citizen_id: u32) -> bool {
        if let Some(pos) = self.members.iter().position(|&id| id == citizen_id) {
            self.members.swap_remove(pos);
            return true;
        }
        false
    }

    /// Current number of members on the roster.
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Set or replace a policy entry. Returns the previous value (if any).
    pub fn update_policy(&mut self, name: &str, value: &str) -> Option<String> {
        if let Some(entry) = self.policies.iter_mut().find(|(k, _)| k == name) {
            let prev = std::mem::replace(&mut entry.1, value.to_string());
            return Some(prev);
        }
        self.policies.push((name.to_string(), value.to_string()));
        None
    }

    /// Look up a policy value by name.
    #[must_use]
    pub fn policy(&self, name: &str) -> Option<&str> {
        self.policies
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Update the approval rating. `value_fp` is in fixed-point
    /// (`[0, APPROVAL_FP_SCALE]`); out-of-range inputs are clamped.
    pub fn set_approval_fp(&mut self, value_fp: i32) {
        self.approval_rating_fp = value_fp.clamp(0, APPROVAL_FP_SCALE);
    }

    /// Approval rating as a `f32` in `[0, 1]`.
    #[must_use]
    pub fn approval_rating(&self) -> f32 {
        self.approval_rating_fp as f32 / APPROVAL_FP_SCALE as f32
    }

    /// Adjust the budget by `delta_bp` (basis points of joule surplus).
    pub fn adjust_budget(&mut self, delta_bp: i64) {
        self.budget_bp = self.budget_bp.saturating_add(delta_bp);
    }
}

impl Default for InstitutionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_institution_is_empty_and_neutral() {
        let inst = InstitutionPolicy::new();
        assert_eq!(inst.member_count(), 0);
        assert!(inst.policies.is_empty());
        assert_eq!(inst.budget_bp, 0);
        assert!((inst.approval_rating() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn add_member_is_idempotent() {
        let mut inst = InstitutionPolicy::new();
        assert!(inst.add_member(7));
        assert!(!inst.add_member(7));
        assert_eq!(inst.member_count(), 1);
    }

    #[test]
    fn remove_member_returns_true_when_present() {
        let mut inst = InstitutionPolicy::new();
        inst.add_member(3);
        assert!(inst.remove_member(3));
        assert!(!inst.remove_member(3));
        assert_eq!(inst.member_count(), 0);
    }

    #[test]
    fn update_policy_returns_previous_value() {
        let mut inst = InstitutionPolicy::new();
        assert!(inst.update_policy("tax_rate", "0.10").is_none());
        assert_eq!(inst.update_policy("tax_rate", "0.15"), Some("0.10".into()));
        assert_eq!(inst.policy("tax_rate"), Some("0.15"));
    }

    #[test]
    fn approval_rating_clamps_and_converts() {
        let mut inst = InstitutionPolicy::new();
        inst.set_approval_fp(-100);
        assert_eq!(inst.approval_rating_fp, 0);
        inst.set_approval_fp(APPROVAL_FP_SCALE + 500);
        assert_eq!(inst.approval_rating_fp, APPROVAL_FP_SCALE);
        assert!((inst.approval_rating() - 1.0).abs() < 1e-6);
    }

    // FR-CIV-SOCIAL-001-INSTITUTIONS — the institution record owns a member
    // roster, string-keyed policies, a basis-point treasury, and a clamped
    // approval rating.
    #[test]
    fn fr_civ_social_001_institutions_policy_membership_budget_lifecycle() {
        let mut inst = InstitutionPolicy::new();
        assert_eq!(inst.member_count(), 0);
        assert!(inst.add_member(10));
        assert!(inst.add_member(20));
        assert!(!inst.add_member(10), "roster add is idempotent");
        assert_eq!(inst.member_count(), 2);
        assert!(inst.remove_member(10));
        assert!(!inst.remove_member(10), "removing a non-member reports false");
        assert_eq!(inst.members, vec![20]);

        // Policies are opaque string tokens owned by this record.
        assert_eq!(inst.update_policy("tax_rate", "0.10"), None);
        assert_eq!(
            inst.update_policy("tax_rate", "0.20"),
            Some("0.10".to_string())
        );
        assert_eq!(inst.policy("tax_rate"), Some("0.20"));
        assert_eq!(inst.policy("missing"), None);

        // Treasury moves in basis points with saturating arithmetic.
        inst.adjust_budget(1_500);
        assert_eq!(inst.budget_bp, 1_500);
        inst.adjust_budget(-10_000);
        assert_eq!(inst.budget_bp, -8_500);

        // Approval is fixed-point and clamped into [0, APPROVAL_FP_SCALE].
        inst.set_approval_fp(750);
        assert!((inst.approval_rating() - 0.75).abs() < 1e-6);
        inst.set_approval_fp(APPROVAL_FP_SCALE + 10);
        assert_eq!(inst.approval_rating_fp, APPROVAL_FP_SCALE);
        inst.set_approval_fp(-10);
        assert_eq!(inst.approval_rating_fp, 0);
    }
}
