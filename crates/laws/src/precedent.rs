//! Legal precedent — FR-CIV-LEGAL-PRECEDENT.
//!
//! Past rulings bias the outcome of later similar cases. The mechanism is a
//! pure-logic *precedent book*:
//!
//! - Each `Ruling` is a binding outcome for a `Case` (set of fact tokens).
//! - `PrecedentBook::record` appends a ruling.
//! - `PrecedentBook::biased_outcome` aggregates the influence of every prior
//!   ruling whose case shares at least one fact token with the query case and
//!   shifts the base outcome accordingly. Similarity is Jaccard over the
//!   multiset union of fact tokens; influence scales with similarity and with
//!   how recent the precedent is relative to the current tick.
//!
//! Determinism: all inputs are value types, all outputs are value types.
//! Adding a precedent never mutates existing rulings.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// Canonical namespaced ID for this functional requirement.
pub const FR_ID: &str = "FR-CIV-LEGAL-PRECEDENT";

/// Discrete outcomes a ruling may bind to.
///
/// The type is closed on purpose: precedent logic only needs to know which
/// direction a ruling pushes a future case, not the full legal taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Ruling in favour of the plaintiff / accused party.
    ForPlaintiff,
    /// Ruling in favour of the defendant / state.
    ForDefendant,
    /// Dismissed for procedural reasons (no merit decision).
    Dismissed,
}

impl Outcome {
    /// Directional sign used when aggregating bias contributions.
    /// `ForPlaintiff = +1`, `ForDefendant = -1`, `Dismissed = 0`.
    #[must_use]
    pub fn sign(self) -> i32 {
        match self {
            Outcome::ForPlaintiff => 1,
            Outcome::ForDefendant => -1,
            Outcome::Dismissed => 0,
        }
    }

    /// Invert a definitive ruling. `Dismissed` is invariant.
    #[must_use]
    pub fn opposite(self) -> Self {
        match self {
            Outcome::ForPlaintiff => Outcome::ForDefendant,
            Outcome::ForDefendant => Outcome::ForPlaintiff,
            Outcome::Dismissed => Outcome::Dismissed,
        }
    }
}

/// A legal case as a multiset of fact tokens (statutes, evidence tags, …).
///
/// `facts` are kept in a `BTreeSet` so similarity is order-independent and
/// fully deterministic across runs / platforms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Case {
    /// Stable id (e.g. docket number). Not used in similarity — purely a label.
    pub id: String,
    /// Set of fact-token strings shared with other cases.
    pub facts: BTreeSet<String>,
}

impl Case {
    /// Construct a case from an id and any iterable of fact tokens.
    pub fn new<I, S>(id: impl Into<String>, facts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: id.into(),
            facts: facts.into_iter().map(Into::into).collect(),
        }
    }
}

/// A single recorded ruling. `tick` orders precedents chronologically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruling {
    /// Tick at which the ruling was issued.
    pub tick: u64,
    /// Case the ruling was issued for.
    pub case: Case,
    /// Outcome bound by the ruling.
    pub outcome: Outcome,
    /// Free-form rationale (judge, statute cited, …). Not used in scoring.
    pub rationale: String,
}

/// One precedent entry: a ruling tagged with its remaining influence over
/// later cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Precedent {
    /// Stable id assigned when the precedent was recorded.
    pub id: u64,
    /// The original ruling.
    pub ruling: Ruling,
    /// 0..=255 — multiplied into the bias contribution so a court can mark
    /// *binding* vs *persuasive* precedents. Default 100.
    pub weight: u8,
}

impl Precedent {
    /// Effective weight as a float multiplier.
    #[must_use]
    pub fn weight_factor(&self) -> f32 {
        f32::from(self.weight) / 100.0
    }
}

/// Aggregate bias of a single precedent against a candidate case.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaseBias {
    /// The originating precedent id.
    pub precedent_id: u64,
    /// Jaccard similarity in `0.0..=1.0`.
    pub similarity: f32,
    /// Signed contribution: `sign(outcome) * similarity * weight * recency`.
    pub contribution: f32,
}

/// Result of querying the precedent book for a candidate case.
#[derive(Debug, Clone, PartialEq)]
pub struct BiasVerdict {
    /// Sum of all `CaseBias::contribution` values. Positive ⇒ plaintiff
    /// leaning, negative ⇒ defendant leaning.
    pub net_bias: f32,
    /// Per-precedent breakdown, sorted by `|contribution|` descending.
    pub influences: Vec<CaseBias>,
}

impl BiasVerdict {
    /// Recommended outcome after applying bias to a `base_score`.
    #[must_use]
    pub fn shifted_outcome(&self, base_score: f32) -> Outcome {
        let final_score = base_score + self.net_bias;
        if final_score > 0.0 {
            Outcome::ForPlaintiff
        } else if final_score < 0.0 {
            Outcome::ForDefendant
        } else {
            Outcome::Dismissed
        }
    }
}

/// In-memory book of precedents. Append-only.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrecedentBook {
    entries: Vec<Precedent>,
    next_id: u64,
}

impl PrecedentBook {
    /// Construct an empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of precedents currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the book has no precedents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record a new precedent. Returns the assigned `precedent id`.
    ///
    /// `weight` is clamped to `0..=255`; callers passing a typical `100` get
    /// the default influence magnitude.
    pub fn record(
        &mut self,
        tick: u64,
        case: Case,
        outcome: Outcome,
        rationale: impl Into<String>,
        weight: u8,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push(Precedent {
            id,
            ruling: Ruling {
                tick,
                case,
                outcome,
                rationale: rationale.into(),
            },
            weight,
        });
        id
    }

    /// All precedents that share at least one fact with `case`, ordered most
    /// influential first.
    #[must_use]
    pub fn relevant_precedents(&self, case: &Case) -> Vec<&Precedent> {
        if case.facts.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<&Precedent> = self
            .entries
            .iter()
            .filter(|p| p.ruling.case.facts.iter().any(|f| case.facts.contains(f)))
            .collect();
        // Sort by absolute Jaccard similarity desc, then by id asc for ties.
        hits.sort_by(|a, b| {
            let sa = jaccard(&a.ruling.case.facts, &case.facts);
            let sb = jaccard(&b.ruling.case.facts, &case.facts);
            sb.partial_cmp(&sa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.id.cmp(&b.id))
        });
        hits
    }

    /// Aggregate precedent bias against `case` at evaluation `now_tick`.
    ///
    /// * `now_tick` is the tick of the candidate case; older precedents are
    ///   discounted by `recency = 1 / (1 + age_ticks * decay_per_tick)`.
    /// * `decay_per_tick` defaults to `0.0` (no decay) when unspecified.
    pub fn bias(&self, case: &Case, now_tick: u64) -> BiasVerdict {
        self.bias_with_decay(case, now_tick, 0.0)
    }

    /// Same as [`Self::bias`] but with an explicit `decay_per_tick`.
    pub fn bias_with_decay(&self, case: &Case, now_tick: u64, decay_per_tick: f32) -> BiasVerdict {
        let mut influences = Vec::with_capacity(self.entries.len());
        let mut net: f32 = 0.0;
        for prec in &self.entries {
            let sim = jaccard(&prec.ruling.case.facts, &case.facts);
            if sim <= 0.0 {
                continue;
            }
            let age = now_tick.saturating_sub(prec.ruling.tick) as f32;
            let recency = 1.0 / (1.0 + age * decay_per_tick.max(0.0));
            let contribution =
                prec.ruling.outcome.sign() as f32 * sim * prec.weight_factor() * recency;
            net += contribution;
            influences.push(CaseBias {
                precedent_id: prec.id,
                similarity: sim,
                contribution,
            });
        }
        influences.sort_by(|a, b| {
            b.contribution
                .abs()
                .partial_cmp(&a.contribution.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        BiasVerdict {
            net_bias: net,
            influences,
        }
    }
}

/// Jaccard similarity over two fact sets: `|A ∩ B| / |A ∪ B|`.
///
/// Returns `0.0` when both sides are empty so an empty candidate case does not
/// spuriously match any precedent.
#[must_use]
pub fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-LEGAL-PRECEDENT — acceptance test: a past ruling must shift
    /// the outcome of a later, similar case. With base score `+0.4` the
    /// unbiased verdict is `ForPlaintiff`. After recording a near-identical
    /// `ForDefendant` precedent, the bias must flip the verdict.
    #[test]
    fn prior_ruling_shifts_later_similar_case_outcome() {
        let mut book = PrecedentBook::new();
        let prior = Case::new(
            "case-prior-001",
            ["assault", "public", "minor_injury", "no_weapon"],
        );
        // First ruling: defendant wins.
        book.record(10, prior, Outcome::ForDefendant, "judge ada", 100);

        let candidate = Case::new(
            "case-cand-002",
            ["public", "assault", "minor_injury", "witness_present"],
        );
        let base_score = 0.4_f32; // bare facts favour plaintiff
        let unbiased = if base_score > 0.0 {
            Outcome::ForPlaintiff
        } else {
            Outcome::ForDefendant
        };
        assert_eq!(unbiased, Outcome::ForPlaintiff);

        let verdict = book.bias(&candidate, /* now_tick = */ 11);
        let shifted = verdict.shifted_outcome(base_score);

        assert_ne!(
            shifted, unbiased,
            "precedent should shift the outcome (net_bias = {})",
            verdict.net_bias
        );
        assert_eq!(shifted, Outcome::ForDefendant);
        assert_eq!(verdict.influences.len(), 1);
        assert!(verdict.influences[0].similarity > 0.0);
        assert!(verdict.net_bias < 0.0);
    }

    /// FR-CIV-LEGAL-PRECEDENT — empty precedent book exerts no influence.
    #[test]
    fn empty_book_yields_no_bias() {
        let book = PrecedentBook::new();
        let candidate = Case::new("case-x", ["a", "b", "c"]);
        let verdict = book.bias(&candidate, 100);
        assert_eq!(verdict.net_bias, 0.0);
        assert!(verdict.influences.is_empty());
        assert_eq!(verdict.clone().shifted_outcome(0.1), Outcome::ForPlaintiff);
        assert_eq!(verdict.shifted_outcome(-0.1), Outcome::ForDefendant);
        assert_eq!(verdict.shifted_outcome(0.0), Outcome::Dismissed);
    }

    /// FR-CIV-LEGAL-PRECEDENT — precedents with disjoint facts do not
    /// influence the candidate.
    #[test]
    fn disjoint_precedent_is_ignored() {
        let mut book = PrecedentBook::new();
        book.record(
            1,
            Case::new("c1", ["tax_evasion", "shell_company"]),
            Outcome::ForPlaintiff,
            "r1",
            100,
        );
        let candidate = Case::new("c2", ["assault", "public"]);
        let verdict = book.bias(&candidate, 2);
        assert_eq!(verdict.net_bias, 0.0);
        assert!(verdict.influences.is_empty());
    }

    /// FR-CIV-LEGAL-PRECEDENT — multiple precedents aggregate by Jaccard
    /// similarity × weight × outcome sign. Two `ForDefendant` precedents at
    /// different weights should produce a strictly larger (in magnitude)
    /// negative bias than one alone.
    #[test]
    fn multiple_precedents_aggregate_by_weight_and_similarity() {
        let mut book = PrecedentBook::new();
        book.record(
            10,
            Case::new("p1", ["theft", "market"]),
            Outcome::ForDefendant,
            "first",
            100,
        );
        let single_bias = book.bias(&Case::new("c", ["theft", "market", "witness"]), 11);
        assert_eq!(single_bias.influences.len(), 1);
        assert!(single_bias.net_bias < 0.0);

        book.record(
            11,
            Case::new("p2", ["theft", "market", "witness"]),
            Outcome::ForDefendant,
            "second",
            200,
        );
        let both_bias = book.bias(&Case::new("c", ["theft", "market", "witness"]), 11);
        assert_eq!(both_bias.influences.len(), 2);
        assert!(
            both_bias.net_bias < single_bias.net_bias,
            "two precedents ({}) must produce a stronger bias than one ({})",
            both_bias.net_bias,
            single_bias.net_bias
        );
    }

    /// FR-CIV-LEGAL-PRECEDENT — opposing precedents partially cancel.
    #[test]
    fn opposing_precedents_partially_cancel() {
        let mut book = PrecedentBook::new();
        book.record(
            5,
            Case::new("for-plaintiff", ["contract", "breach"]),
            Outcome::ForPlaintiff,
            "p",
            100,
        );
        book.record(
            5,
            Case::new("for-defendant", ["contract", "breach"]),
            Outcome::ForDefendant,
            "d",
            100,
        );
        let verdict = book.bias(&Case::new("c", ["contract", "breach"]), 6);
        assert_eq!(verdict.influences.len(), 2);
        assert!(
            verdict.net_bias.abs() < 0.001,
            "equal-and-opposite precedents must cancel out (~0.0), got {}",
            verdict.net_bias
        );
    }

    /// FR-CIV-LEGAL-PRECEDENT — `record` returns monotonically increasing
    /// ids starting at `0`.
    #[test]
    fn record_assigns_stable_ids() {
        let mut book = PrecedentBook::new();
        let id0 = book.record(0, Case::new("a", ["x"]), Outcome::Dismissed, "", 50);
        let id1 = book.record(1, Case::new("b", ["x"]), Outcome::ForPlaintiff, "", 50);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(book.len(), 2);
    }

    /// FR-CIV-LEGAL-PRECEDENT — recency decay reduces influence of older
    /// precedents. With `decay_per_tick = 0.1`, an old precedent contributes
    /// strictly less than a fresh one with identical similarity and weight.
    #[test]
    fn recency_decay_dampens_old_precedents() {
        let mut book = PrecedentBook::new();
        book.record(
            0,
            Case::new("old", ["theft", "market"]),
            Outcome::ForDefendant,
            "old",
            100,
        );
        let candidate = Case::new("c", ["theft", "market"]);
        let fresh = book.bias_with_decay(&candidate, 0, 0.0);
        let aged = book.bias_with_decay(&candidate, 100, 0.1);
        assert!(fresh.net_bias < 0.0);
        assert!(aged.net_bias > fresh.net_bias);
        assert!(aged.net_bias.abs() < fresh.net_bias.abs());
    }

    /// FR-CIV-LEGAL-PRECEDENT — Jaccard similarity on fact sets.
    #[test]
    fn jaccard_matches_canonical_definition() {
        let a: BTreeSet<String> = ["theft", "market", "witness"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let b: BTreeSet<String> = ["theft", "market"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        // |A ∩ B| = 2, |A ∪ B| = 3 → 2/3
        let s = jaccard(&a, &b);
        assert!((s - (2.0 / 3.0)).abs() < 1e-6);
    }

    /// FR-CIV-LEGAL-PRECEDENT — `Outcome::opposite` and `sign` are
    /// consistent.
    #[test]
    fn outcome_sign_and_opposite_are_consistent() {
        assert_eq!(Outcome::ForPlaintiff.opposite(), Outcome::ForDefendant);
        assert_eq!(Outcome::ForDefendant.opposite(), Outcome::ForPlaintiff);
        assert_eq!(Outcome::Dismissed.opposite(), Outcome::Dismissed);
        assert_eq!(Outcome::ForPlaintiff.sign(), 1);
        assert_eq!(Outcome::ForDefendant.sign(), -1);
        assert_eq!(Outcome::Dismissed.sign(), 0);
    }
}
