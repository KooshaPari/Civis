//! Institution faction-split semantics for **FR-CIV-FACTION-SPLIT**.
//!
//! A civic institution is rarely monolithic — internally it comprises
//! multiple constituencies (clergy vs. laity, garrison officers vs.
//! rank-and-file, reformer vs. traditionalist factions, ...). When
//! internal disagreement rises past a cohesion threshold, the institution
//! can no longer hold together as a single body and splits into rival
//! factions: the original **parent** institution continues, and a new
//! **splinter** faction is spawned from the dissenting members.
//!
//! The split is modeled here as a pure-logic function over an
//! [`InstitutionCohesion`] reading; the owning simulation is expected to
//! invoke [`maybe_split_faction`] once per `(settlement_id, kind)` per
//! tick and to track already-emitted splits itself if one-shot event
//! semantics are required (mirroring the pattern used by
//! `civ-institutions::legitimacy`).

use serde::{Deserialize, Serialize};

/// Default cohesion threshold for a newly-created institution.
///
/// Cohesion is normalized in `[0.0, 1.0]`. A value of `0.5` corresponds
/// to "the institution is more cohesive than not"; values **below** the
/// threshold indicate that internal disagreement has overcome cohesion
/// and the institution is at risk of splitting.
pub const DEFAULT_COHESION_THRESHOLD: f32 = 0.5;

/// Minimum allowed cohesion value.
pub const MIN_COHESION: f32 = 0.0;

/// Maximum allowed cohesion value.
pub const MAX_COHESION: f32 = 1.0;

/// Mutable cohesion state for a civic institution.
///
/// `value` is the current normalized cohesion reading (higher = more
/// cohesive). `disagreement` is a separate signal that the institution
/// internally reports; when `disagreement` exceeds `1.0 - cohesion`,
/// the institution is considered to have lost its internal consensus
/// and [`maybe_split_faction`] will return a [`FactionSplitEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InstitutionCohesion {
    /// Current normalized cohesion scalar in `[MIN_COHESION, MAX_COHESION]`.
    pub value: f32,
    /// Threshold below which internal disagreement is considered to
    /// exceed cohesion. The split fires when
    /// `disagreement > 1.0 - cohesion`.
    pub cohesion_threshold: f32,
}

impl Default for InstitutionCohesion {
    fn default() -> Self {
        Self {
            value: DEFAULT_COHESION_THRESHOLD,
            cohesion_threshold: DEFAULT_COHESION_THRESHOLD,
        }
    }
}

impl InstitutionCohesion {
    /// Creates a cohesion reading with a clamped normalized value.
    pub fn new(value: f32) -> Self {
        Self {
            value: clamp_cohesion(value),
            cohesion_threshold: DEFAULT_COHESION_THRESHOLD,
        }
    }

    /// Returns true when the institution has lost internal consensus
    /// for the supplied `disagreement` reading. This is the same
    /// predicate used by [`maybe_split_faction`].
    pub fn should_split(self, disagreement: f32) -> bool {
        disagreement > MAX_COHESION - self.value
    }
}

/// A faction that exists within an institution.
///
/// Factions are lightweight named identities. They share the parent
/// institution's settlement and kind but carry an ideological
/// `dissatisfaction` scalar that drives their behavior (e.g. resource
/// allocation, hostile actions, defection).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Faction {
    /// Stable, human-readable identifier for this faction (e.g.
    /// `"temple-reformists"`, `"garrison-veterans"`).
    pub id: String,
    /// Human-readable display name (e.g. `"Reformists"`,
    /// `"Veteran Officers"`).
    pub name: String,
    /// Normalized dissatisfaction with the parent institution in
    /// `[0.0, 1.0]`. A value of `1.0` means the faction is fully
    /// alienated from the parent.
    pub dissatisfaction: f32,
}

impl Faction {
    /// Creates a new faction with the given id, display name, and
    /// dissatisfaction level (clamped to `[0.0, 1.0]`).
    pub fn new(id: impl Into<String>, name: impl Into<String>, dissatisfaction: f32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dissatisfaction: clamp_unit(dissatisfaction),
        }
    }
}

/// Event emitted by [`maybe_split_faction`] when internal disagreement
/// has overcome cohesion and a new splinter faction has been spawned.
///
/// The owning simulation is expected to broadcast this on its event
/// bus / replay log exactly once per `(settlement_id, kind)` per split
/// if it wants one-shot semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionSplitEvent {
    /// Settlement in which the split occurred.
    pub settlement_id: u32,
    /// Kind of the parent institution.
    pub kind: super::InstitutionKind,
    /// Level of the parent institution at the moment of the split.
    pub level: u8,
    /// The newly-spawned splinter faction.
    pub splinter: Faction,
    /// Cohesion reading at the moment of the split.
    pub cohesion_at_split: f32,
    /// Disagreement reading that triggered the split.
    pub disagreement_at_split: f32,
}

/// Evaluates whether the institution identified by `(settlement_id,
/// kind, level)` should split into a splinter faction given its current
/// `cohesion` and the supplied `disagreement` reading.
///
/// - When `cohesion.should_split(disagreement)` is `true`, a new
///   [`FactionSplitEvent`] is returned containing a freshly-created
///   [`Faction`] whose `dissatisfaction` reflects how badly the
///   splinter disagrees with the parent. The splinter's `id` and
///   `name` are derived from the institution kind in a stable, readable
///   way.
/// - When the institution is still cohesive, `None` is returned.
///
/// The function is pure: it does not mutate `cohesion`, does not
/// maintain a registry of factions, and does not enforce one-shot
/// semantics. The caller is responsible for those concerns.
pub fn maybe_split_faction(
    settlement_id: u32,
    kind: super::InstitutionKind,
    level: u8,
    cohesion: InstitutionCohesion,
    disagreement: f32,
) -> Option<FactionSplitEvent> {
    if !cohesion.should_split(disagreement) {
        return None;
    }

    let splinter = Faction::new(
        splinter_id(kind),
        splinter_name(kind),
        splinter_dissatisfaction(disagreement),
    );

    Some(FactionSplitEvent {
        settlement_id,
        kind,
        level,
        splinter,
        cohesion_at_split: cohesion.value,
        disagreement_at_split: disagreement,
    })
}

/// Returns the canonical splinter faction id for a given institution
/// kind. Stable across runs so that replay / save files remain
/// diff-friendly.
pub fn splinter_id(kind: super::InstitutionKind) -> String {
    match kind {
        super::InstitutionKind::Temple => "temple-splinter".to_string(),
        super::InstitutionKind::Garrison => "garrison-splinter".to_string(),
    }
}

/// Returns the canonical splinter faction display name for a given
/// institution kind.
pub fn splinter_name(kind: super::InstitutionKind) -> String {
    match kind {
        super::InstitutionKind::Temple => "Temple Splinter".to_string(),
        super::InstitutionKind::Garrison => "Garrison Splinter".to_string(),
    }
}

fn splinter_dissatisfaction(disagreement: f32) -> f32 {
    // The further past the cohesion ceiling the disagreement reading
    // is, the more alienated the splinter is. We clamp to [0, 1] so
    // downstream consumers can treat dissatisfaction as a normalized
    // probability / weight without further sanitization.
    clamp_unit(disagreement)
}

fn clamp_cohesion(value: f32) -> f32 {
    value.clamp(MIN_COHESION, MAX_COHESION)
}

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rising_disagreement_past_threshold_spawns_splinter_faction() {
        // FR-CIV-FACTION-SPLIT acceptance test:
        // rising internal disagreement past threshold spawns a splinter faction.
        let mut cohesion = InstitutionCohesion::default();

        // Below-threshold disagreement: no split.
        let below = maybe_split_faction(
            1,
            super::super::InstitutionKind::Temple,
            2,
            cohesion,
            0.10,
        );
        assert!(below.is_none(), "low disagreement must not split");

        // At-threshold disagreement: still no split (predicate is strict).
        let at = maybe_split_faction(
            1,
            super::super::InstitutionKind::Temple,
            2,
            cohesion,
            0.50,
        );
        assert!(at.is_none(), "threshold disagreement must not split");

        // Now disagreement rises past the threshold.
        cohesion.value = 0.40; // drops cohesion so 0.65 disagreement clears the bar.
        let above = maybe_split_faction(
            1,
            super::super::InstitutionKind::Temple,
            2,
            cohesion,
            0.65,
        );

        let event = above.expect("rising disagreement past threshold must spawn a splinter");
        assert_eq!(event.settlement_id, 1);
        assert_eq!(event.kind, super::super::InstitutionKind::Temple);
        assert_eq!(event.level, 2);
        assert_eq!(event.splinter.id, "temple-splinter");
        assert_eq!(event.splinter.name, "Temple Splinter");
        assert!(event.splinter.dissatisfaction > 0.0);
        assert!(event.splinter.dissatisfaction <= 1.0);
        assert_eq!(event.cohesion_at_split, cohesion.value);
        assert_eq!(event.disagreement_at_split, 0.65);
    }

    #[test]
    fn garrison_split_uses_garrison_splinter_identity() {
        let cohesion = InstitutionCohesion {
            value: 0.20,
            ..InstitutionCohesion::default()
        };
        let event = maybe_split_faction(
            7,
            super::super::InstitutionKind::Garrison,
            1,
            cohesion,
            0.95,
        )
        .expect("very high disagreement must split garrison");

        assert_eq!(event.kind, super::super::InstitutionKind::Garrison);
        assert_eq!(event.splinter.id, "garrison-splinter");
        assert_eq!(event.splinter.name, "Garrison Splinter");
    }

    #[test]
    fn should_split_predicate_is_strict() {
        let cohesion = InstitutionCohesion::default(); // value == 0.5, threshold == 0.5
        assert!(!cohesion.should_split(0.0));
        assert!(!cohesion.should_split(0.5));
        assert!(cohesion.should_split(0.5001));
        assert!(cohesion.should_split(1.0));
    }

    #[test]
    fn faction_dissatisfaction_is_clamped() {
        let f = Faction::new("x", "X", 5.0);
        assert_eq!(f.dissatisfaction, 1.0);
        let f = Faction::new("x", "X", -2.0);
        assert_eq!(f.dissatisfaction, 0.0);
    }
}