//! Sequential god-power synergy / penalty computation (FR-CIV-POWER-SYNERGY).
//!
//! When the player invokes two or more god-powers in sequence (or in
//! close temporal proximity as the engine tracks), the resulting
//! "momentum" of substrate writes may be **synergistic** (compatible
//! sequencing; the second power compounds the first's effect and yields
//! a bonus multiplier > 1.0) or **conflicting** (incompatible sequencing;
//! the writes fight each other and the second power under-delivers for
//! a penalty multiplier < 1.0).
//!
//! Pure logic. No substrate writes. The engine multiplies each
//! power's effect magnitude by [`SynergyOutcome::multiplier`] before
//! dispatching the god-tool to its substrate handler.
//!
//! ## Design
//!
//! Two powers are **compatible** when they share a substrate target
//! mask bit AND/OR live in the same `PowerTab`. Two powers are
//! **incompatible** when one is a `Mutating` disaster and the other is
//! a `Mutating` `Life`-affecting verb in the same window (disasters
//! wipe out freshly-spawned life).
//!
//! The bonus/penalty is computed as a left-fold over the sequence:
//! each step's [`SynergyEdge`] is classified and the running
//! multiplier is updated. Equal-or-equal tabs and matching masks
//! add a `+0.10` synergy bump; opposing categories (e.g.
//! `Disaster` after `Life.SpawnOrganism`) deduct `-0.15`. The
//! multiplier is clamped to the `[MIN_MULT, MAX_MULT]` range so a
//! single bad pair can't zero out a god-power.
//!
//! The classification table lives in [`SynergyEdge::classify`].

use crate::{PowerCategory, PowerDef, PowerTargetMask};

/// Minimum allowed synergy multiplier. Stops a god-power from being
/// fully nullified by a single incompatible pairing in the recent
/// window.
pub const MIN_MULT: f32 = 0.25;

/// Maximum allowed synergy multiplier. Caps compounding so a
/// god-power can't become unreasonably potent through stacking
/// compatible pairs.
pub const MAX_MULT: f32 = 2.50;

/// Per-edge nudge applied when two consecutive powers are
/// **synergistic** (compatible).
pub const SYNERGY_BUMP: f32 = 0.10;

/// Per-edge nudge applied when two consecutive powers are
/// **incompatible**.
pub const PENALTY_NUDGE: f32 = 0.15;

/// Outcome of scoring a sequence of god-powers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynergyOutcome {
    /// Aggregate bonus/penalty multiplier. `> 1.0` means the
    /// sequence was net-synergistic; `< 1.0` means net-conflicting.
    pub multiplier: f32,
    /// Number of compatibility edges that contributed a bonus.
    pub compatible_edges: u32,
    /// Number of incompatibility edges that contributed a penalty.
    pub incompatible_edges: u32,
    /// Total edges in the fold (== `sequence.len().saturating_sub(1)`).
    pub edges: u32,
}

/// Classification of a single consecutive pair in a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynergyEdge {
    /// The two powers compose well; the next multiplier nudges up.
    Compatible,
    /// The two powers fight each other; the next multiplier nudges
    /// down.
    Incompatible,
    /// No interaction (e.g. one of the pair is `Universal` /
    /// read-only / out of mask overlap).
    Neutral,
}

impl SynergyEdge {
    /// Classify the relationship between two consecutive
    /// [`PowerDef`]s in the player's invocation sequence.
    ///
    /// Rules (in order):
    /// 1. Either power is `Universal` → [`SynergyEdge::Neutral`].
    /// 2. Different shared `PowerTargetMask` bits (they don't touch
    ///    the same substrate field) → [`SynergyEdge::Neutral`].
    /// 3. Same `PowerTab` AND both `Mutating` → [`SynergyEdge::Compatible`].
    /// 4. `Life` followed by any `Disaster` (both `Mutating`) →
    ///    [`SynergyEdge::Incompatible`].
    /// 5. Any `Disaster` followed by `Life.Bless|Curse|Heal|Extinct`
    ///    (the disaster just wrecked what the actor-effect was
    ///    about to touch) → [`SynergyEdge::Incompatible`].
    /// 6. Everything else → [`SynergyEdge::Compatible`] when
    ///    `same_tab == true`, else [`SynergyEdge::Neutral`].
    #[must_use]
    pub fn classify(prev: &PowerDef, next: &PowerDef) -> Self {
        // Rule 1: universal / read-only powers don't participate.
        if prev.category == PowerCategory::Universal
            || next.category == PowerCategory::Universal
        {
            return Self::Neutral;
        }

        // Rule 2: if the two powers don't share any substrate
        // target mask bit there is no substrate-level conflict
        // and no synergy to be gained.
        let prev_mask = target_mask_for(prev);
        let next_mask = target_mask_for(next);
        let shared = prev_mask.0 & next_mask.0;
        if shared == 0 {
            return Self::Neutral;
        }

        // Rule 4 & 5: disaster vs life conflicts.
        let prev_disaster = matches!(prev.tab, crate::PowerTab::Disaster);
        let next_disaster = matches!(next.tab, crate::PowerTab::Disaster);
        let prev_life = matches!(prev.tab, crate::PowerTab::Life);
        let next_life = matches!(next.tab, crate::PowerTab::Life);
        let prev_actor_effect = prev.request == crate::PowerRequestKind::ActorEffect;
        let next_actor_effect = next.request == crate::PowerRequestKind::ActorEffect;

        if prev_life && next_disaster {
            return Self::Incompatible;
        }
        if prev_disaster && next_life && next_actor_effect {
            return Self::Incompatible;
        }

        // Rule 3: same-tab mutating ops compose well
        // (e.g. two TERRAIN edits stack into a smoother sculpt).
        if prev.tab == next.tab
            && prev.category == PowerCategory::Mutating
            && next.category == PowerCategory::Mutating
        {
            return Self::Compatible;
        }

        // Cross-tab but overlapping mask bits at least share a
        // substrate target; treat as neutral unless the previous
        // rule (disaster/life) fired.
        if prev.tab == next.tab {
            return Self::Compatible;
        }

        Self::Neutral
    }
}

/// Score a sequence of [`PowerDef`]s and return the aggregate
/// synergy/penalty multiplier.
///
/// The sequence is folded left-to-right; an empty or single-element
/// sequence returns a neutral `multiplier = 1.0` (no edges means no
/// compounding). The multiplier is clamped to
/// [`MIN_MULT`]..=[`MAX_MULT`].
///
/// Reads-only and universal entries do not contribute edges; they are
/// skipped when looking at the previous/next pair so a quick
/// "Inspect" peek between two `terrain.raise` invocations does not
/// break a synergy streak.
#[must_use]
pub fn synergy_multiplier(sequence: &[&PowerDef]) -> SynergyOutcome {
    // Collect only the entries that can interact (skip
    // Universal/read-only — they're never substrate writes).
    let interacting: Vec<&PowerDef> = sequence
        .iter()
        .copied()
        .filter(|p| p.category != PowerCategory::Universal)
        .collect();

    if interacting.len() < 2 {
        return SynergyOutcome {
            multiplier: 1.0,
            compatible_edges: 0,
            incompatible_edges: 0,
            edges: 0,
        };
    }

    let mut multiplier = 1.0_f32;
    let mut compatible = 0_u32;
    let mut incompatible = 0_u32;

    for window in interacting.windows(2) {
        let prev = window[0];
        let next = window[1];
        match SynergyEdge::classify(prev, next) {
            SynergyEdge::Compatible => {
                multiplier += SYNERGY_BUMP;
                compatible += 1;
            }
            SynergyEdge::Incompatible => {
                multiplier -= PENALTY_NUDGE;
                incompatible += 1;
            }
            SynergyEdge::Neutral => {}
        }
    }

    multiplier = multiplier.clamp(MIN_MULT, MAX_MULT);

    SynergyOutcome {
        multiplier,
        compatible_edges: compatible,
        incompatible_edges: incompatible,
        edges: compatible + incompatible,
    }
}

/// Map a `PowerDef` to its substrate target mask. Read-only and
/// universal verbs share no substrate bits and so never contribute a
/// synergy edge (see [`SynergyEdge::classify`]).
fn target_mask_for(p: &PowerDef) -> PowerTargetMask {
    use crate::PowerRequestKind as R;
    match p.request {
        R::MaterialEdit | R::TerraformEdit => PowerTargetMask::VOXEL,
        R::ActorSpawn | R::ActorEffect => PowerTargetMask::AGENT | PowerTargetMask::SETTLEMENT,
        R::Disaster => PowerTargetMask::VOXEL | PowerTargetMask::AGENT | PowerTargetMask::FIELD,
        R::Law => PowerTargetMask::SETTLEMENT | PowerTargetMask::FIELD,
        R::Time => PowerTargetMask::TIME,
        R::NoOp => PowerTargetMask(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRequestKind, PowerTab};

    /// Build a minimal `PowerDef` for tests.
    fn def(id: &'static str, tab: PowerTab, category: PowerCategory, request: PowerRequestKind) -> PowerDef {
        PowerDef {
            id: PowerId::new_const(id),
            label: id,
            tab,
            category,
            request,
            availability: PowerAvailability::Live,
            coupling_note: "",
        }
    }

    /// AC-SYN-1 (FR-CIV-POWER-SYNERGY): a sequence of compatible
    /// god-powers (same tab, same substrate mask, both mutating)
    /// yields a bonus multiplier > 1.0.
    #[test]
    fn compatible_sequence_yields_bonus() {
        let raise = def("terrain.raise", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);
        let smooth = def("terrain.smooth", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);
        let level = def("terrain.level", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);

        let outcome = synergy_multiplier(&[&raise, &smooth, &level]);

        assert_eq!(outcome.compatible_edges, 2, "two compatible pairs in a 3-verb sequence");
        assert_eq!(outcome.incompatible_edges, 0);
        assert!(
            outcome.multiplier > 1.0,
            "compatible sequence must produce a bonus multiplier, got {}",
            outcome.multiplier
        );
        assert!(outcome.multiplier <= MAX_MULT);
    }

    /// AC-SYN-2 (FR-CIV-POWER-SYNERGY): a sequence of incompatible
    /// god-powers (life followed by disaster on the same substrate)
    /// yields a penalty multiplier < 1.0.
    #[test]
    fn incompatible_sequence_yields_penalty() {
        let spawn = def("life.spawn_organism", PowerTab::Life, PowerCategory::Mutating,
            PowerRequestKind::ActorSpawn);
        let meteor = def("disaster.meteor", PowerTab::Disaster, PowerCategory::Mutating,
            PowerRequestKind::Disaster);

        let outcome = synergy_multiplier(&[&spawn, &meteor]);

        assert_eq!(outcome.incompatible_edges, 1);
        assert!(
            outcome.multiplier < 1.0,
            "life -> disaster sequence must produce a penalty multiplier, got {}",
            outcome.multiplier
        );
        assert!(outcome.multiplier >= MIN_MULT);
    }

    /// Empty / single-element sequences must be neutral (`1.0`).
    /// The pure-logic function never invents edges that aren't
    /// there.
    #[test]
    fn empty_or_singleton_sequence_is_neutral() {
        let raise = def("terrain.raise", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);

        let empty = synergy_multiplier(&[]);
        assert_eq!(empty.multiplier, 1.0);
        assert_eq!(empty.edges, 0);

        let single = synergy_multiplier(&[&raise]);
        assert_eq!(single.multiplier, 1.0);
        assert_eq!(single.edges, 0);
    }

    /// The multiplier is always clamped to `[MIN_MULT, MAX_MULT]`
    /// even with extreme stacking.
    #[test]
    fn multiplier_is_clamped() {
        let raise = def("terrain.raise", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);
        // 50 same-tab pairs would push the sum well past MAX_MULT
        // without the clamp.
        let seq: Vec<&PowerDef> = std::iter::repeat(&raise).take(51).collect();
        let outcome = synergy_multiplier(&seq);
        assert!(
            outcome.multiplier <= MAX_MULT,
            "multiplier must be clamped to MAX_MULT, got {}",
            outcome.multiplier
        );
    }

    /// Universal entries (camera/time/inspect) don't contribute
    /// edges and don't break a synergy streak. Engine callers can
    /// pass the full god-tool log including ancillary verbs and
    /// still receive a deterministic outcome.
    #[test]
    fn universal_entries_are_skipped() {
        let raise = def("terrain.raise", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);
        let smooth = def("terrain.smooth", PowerTab::Terrain, PowerCategory::Mutating,
            PowerRequestKind::TerraformEdit);
        let probe = def("inspect.probe", PowerTab::Inspect, PowerCategory::ReadOnly,
            PowerRequestKind::NoOp);
        let pause = def("time.pause", PowerTab::Time, PowerCategory::Universal,
            PowerRequestKind::Time);

        let outcome = synergy_multiplier(&[&raise, &probe, &pause, &smooth]);

        assert_eq!(outcome.edges, 1);
        assert!(
            outcome.multiplier > 1.0,
            "raise -> smooth with skipped universals should still be a single compatible edge"
        );
    }
}
