//! Formal negotiation transcripts — FR-CIV-DIPLO-005.
//!
//! A negotiation **transcript** is the structured, replayable record of how
//! a dialogue between two parties progressed from opening offer through to
//! resolution. This module adds exactly one slice on top of the existing
//! substrate and concession machinery:
//!
//! * [`TranscriptClause`] — an enum of every kind of structured clause that
//!   may appear in a negotiation log. The variants are typed (party, level,
//!   amount, round) rather than textual; free-text wording lives only in
//!   [`Transcript::flavor_text`].
//! * [`Transcript`] — an ordered `Vec<TranscriptClause>` plus an optional
//!   `flavor_text` garnish layer. The vector is the source of truth; the
//!   flavor text is decoration produced for player-facing display.
//!
//! ## Why "structured clauses, not free text"?
//!
//! Per `FR-CIV-DIPLO-005` ("negotiation transcripts SHALL store STRUCTURED
//! clauses (typed data, not free-text) as the source of truth, separate
//! from any LLM-generated flavor/wording garnish that might decorate the
//! transcript for player display"), the only way to keep diplomacy
//! deterministic and replay-safe is to never let free-text — let alone
//! LLM-authored prose — leak into the state machine or the audit trail.
//! LLM-generated wording is permitted **only** as a transitive rendering
//! layer; even then, `FR-CIV-DIPLO-006` further constrains that the LLM
//! may "only decorate this text, never construct the Transcript itself".
//!
//! That two-layer split is enforced here by making [`TranscriptClause`]
//! serde-roundtrippable and by having [`Transcript::render_flavor_text`]
//! remain the *only* method that produces a `String`. Anything that
//! would consume the transcript for gameplay decisions must read
//! [`Transcript::clauses()`], not [`Transcript::flavor_text`].
//!
//! ## Reuse of existing terms/offer types
//!
//! Variants reuse the existing shared types rather than inventing new
//! ones:
//!
//! * The "party" of an offer/counter/accept/reject/concession is a
//!   [`crate::PolityId`] — the same id used by [`crate::Pair`] and
//!   [`crate::ConcessionStateMachine`].
//! * The concession `level` and `amount` fields are `u32`s matching the
//!   [`crate::concession_rounds::NegotiationState::offer`] /
//!   [`crate::concession_rounds::ConcessionRound::gap`] conventions so a
//!   transcript can be cross-checked against a
//!   [`crate::concession_rounds::ConcessionOutcome`] trace.
//! * The `round` field is a `u32` matching
//!   [`crate::concession_rounds::ConcessionRound::round`].
//!
//! ## LLM seam
//!
//! [`Transcript::render_flavor_text`] is the seam where LLM wording plugs
//! in later (`FR-CIV-DIPLO-006`). For this slice it returns a simple,
//! fully deterministic formatting of the structured clauses — no model
//! call, no RNG, no IO. The string is **decorative only**: callers MUST
//! treat it as opaque for any game-relevant decision, including
//! acceptance testing, scoring, and replay reconciliation. The internal
//! representation of the method is intentionally trivial so a later
//! slice can swap a real LLM client in without altering the transcript
//! type signatures or the surrounding serialization surface.

use serde::{Deserialize, Serialize};

use crate::PolityId;

/// One structured clause in a negotiation transcript.
///
/// Variants are typed; recorded wording lives separately in
/// [`Transcript::flavor_text`]. The variants are the canonical union of
/// events the diplomacy state machine can produce:
///
/// * [`Offer`](Self::Offer) — opening offer (or any re-propose) from one party.
/// * [`Counter`](Self::Counter) — counter-offer from the *other* party that
///   raises the concession level. Captures both the new and prior levels so
///   the magnitude of the move is auditable without referring back to the
///   trace.
/// * [`Accept`](Self::Accept) — final acceptance by one party. `agreed_level`
///   is the level at which the parties agreed (closes the gap to zero).
/// * [`Reject`](Self::Reject) — final rejection by one party. `last_level`
///   records the offer on the table at the time of rejection so the audit
///   trail captures "how close did we get".
/// * [`Concession`](Self::Concession) — an explicit per-round concession
///   delta emitted by the Zeuthen/Rubinstein driver. Lets the transcript
///   record partial convergence that did not (yet) reach the agreed level.
///
/// All variants derive `Serialize`/`Deserialize` so a transcript can be
/// written to disk and replayed byte-for-byte; this is the property that
/// the "source of truth" requirement of `FR-CIV-DIPLO-005` rides on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptClause {
    /// Opening (or re-) offer from one party at a particular concession level.
    Offer {
        /// Party authoring the offer.
        party: PolityId,
        /// Concession level offered. Matches the `offer` field of
        /// [`crate::concession_rounds::NegotiationState`].
        level: u32,
    },
    /// Counter-offer from the other party, raising the concession level
    /// above the prior offer.
    Counter {
        /// Party authoring the counter-offer.
        party: PolityId,
        /// New concession level after the counter.
        level: u32,
        /// Concession level of the prior offer being countered.
        prior_level: u32,
    },
    /// Final acceptance by one party. Records the agreed concession level.
    Accept {
        /// Party accepting.
        party: PolityId,
        /// Concession level at which the parties agreed.
        agreed_level: u32,
    },
    /// Final rejection by one party. Records the offer on the table so the
    /// audit trail shows how close the negotiation was at termination.
    Reject {
        /// Party rejecting.
        party: PolityId,
        /// Last offer on the table at the time of rejection.
        last_level: u32,
    },
    /// A single per-round concession delta made by one party.
    Concession {
        /// 1-based round index in which the concession was made.
        round: u32,
        /// Party conceding.
        party: PolityId,
        /// Amount conceded (in level units). Matches the convention of
        /// [`crate::concession_rounds::ConcessionRound::gap`].
        amount: u32,
    },
}

/// Formal register negotiation transcript.
///
/// The ordered [`clauses`](Self::clauses) vector is the source of truth;
/// the [`flavor_text`](Self::flavor_text) is a *decorative* layer rendered
/// for player display only and MUST NOT influence any gameplay decision
/// (`FR-CIV-DIPLO-006`).
///
/// `Transcript` is intentionally not coupled to any specific driver — it
/// is the *output* medium for the concession machinery and any future
/// scripted-negotiation logic, and is itself a pure value (no I/O, no
/// hidden state). Identical clause sequences produce identical
/// `render_flavor_text` outputs across runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Transcript {
    /// Ordered structured clauses. The Vec's order is the order in which
    /// the clauses were recorded.
    clauses: Vec<TranscriptClause>,
    /// Decorative LLM-authored flavor text. Optional: a transcript may
    /// have no flavor yet (the LLM has not run) and still be a valid
    /// source of truth.
    flavor_text: Option<String>,
}

impl Transcript {
    /// Construct a transcript from a clause vector and optional flavor
    /// text.
    pub fn new(clauses: Vec<TranscriptClause>, flavor_text: Option<String>) -> Self {
        Self { clauses, flavor_text }
    }

    /// Construct an empty transcript.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Append one structured clause.
    pub fn push(&mut self, clause: TranscriptClause) {
        self.clauses.push(clause);
    }

    /// Borrow the structured clauses in record order.
    pub fn clauses(&self) -> &[TranscriptClause] {
        &self.clauses
    }

    /// Borrow the flavor text (if any). Returns `Some(&text)` when an
    /// LLM-generated garnish has been attached, `None` otherwise.
    pub fn flavor_text(&self) -> Option<&str> {
        self.flavor_text.as_deref()
    }

    /// Attach or replace the flavor text. Use this when an LLM garnish
    /// becomes available after the structured transcript was finalized.
    /// The structured clauses are untouched.
    pub fn set_flavor_text<S: Into<String>>(&mut self, text: S) {
        self.flavor_text = Some(text.into());
    }

    /// Render the structured clauses as a deterministic, human-readable
    /// string.
    ///
    /// This is the **LLM seam** for `FR-CIV-DIPLO-005`. A later slice may
    /// replace this body with a call into a real LLM client that takes
    /// the structured clauses and decorates them with prose — `FR-CIV-DIPLO-006`
    /// constrains that the LLM may only decorate, never construct the
    /// [`Transcript`] itself.
    ///
    /// Current behavior: format the clauses deterministically (no LLM,
    /// no RNG). Calling this method twice on the same transcript
    /// produces identical bytes, which the unit test
    /// `fr_civ_diplo_005_render_flavor_text_is_deterministic` asserts.
    pub fn render_flavor_text(&self) -> String {
        // Build a stable, deterministic rendering. We do NOT include the
        // existing `flavor_text` field in the output: that would couple
        // re-rendering to itself (idempotency violation) and is a smell
        // closer to "stacked decorations" than to "source of truth".
        let mut out = String::new();
        out.push_str("transcript:\n");
        if self.clauses.is_empty() {
            out.push_str("  (no clauses)\n");
            return out;
        }
        for (idx, clause) in self.clauses.iter().enumerate() {
            out.push_str(&format!("  [{}] {}\n", idx, format_clause(clause)));
        }
        out
    }
}

/// Render one [`TranscriptClause`] as a compact stable string. Private
/// helper for [`Transcript::render_flavor_text`]; deliberately not
/// public because callers should consume [`Transcript::clauses`]
/// directly for any structured inspection.
fn format_clause(clause: &TranscriptClause) -> String {
    match clause {
        TranscriptClause::Offer { party, level } => {
            format!("offer party={} level={}", party.0, level)
        }
        TranscriptClause::Counter {
            party,
            level,
            prior_level,
        } => {
            format!(
                "counter party={} level={} prior_level={}",
                party.0, level, prior_level
            )
        }
        TranscriptClause::Accept { party, agreed_level } => {
            format!("accept party={} agreed_level={}", party.0, agreed_level)
        }
        TranscriptClause::Reject { party, last_level } => {
            format!("reject party={} last_level={}", party.0, last_level)
        }
        TranscriptClause::Concession {
            round,
            party,
            amount,
        } => {
            format!(
                "concession round={} party={} amount={}",
                round, party.0, amount
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    /// FR-CIV-DIPLO-005: build a small (offer, counter, accept) transcript
    /// and assert:
    ///
    /// 1. The structured clauses round-trip via serde (the source of truth
    ///    is wire-stable across processes and replays).
    /// 2. `render_flavor_text` produces non-empty output and is fully
    ///    deterministic — calling it twice on the same transcript yields
    ///    byte-identical strings (the LLM seam is currently a pure
    ///    deterministic rendering; later slices may swap in a real LLM
    ///    call behind this method without changing the transcript data
    ///    flow).
    #[test]
    fn fr_civ_diplo_005_transcript_round_trips_and_renders_deterministically() {
        let parties = (p(1), p(2));

        // Build the transcript via `push` so we exercise the same code
        // path callers will use. Order: opening offer, counter, accept.
        let mut transcript = Transcript::empty();
        transcript.push(TranscriptClause::Offer {
            party: parties.0,
            level: 10,
        });
        transcript.push(TranscriptClause::Counter {
            party: parties.1,
            level: 14,
            prior_level: 10,
        });
        transcript.push(TranscriptClause::Accept {
            party: parties.0,
            agreed_level: 14,
        });

        // 1. Clause sequence round-trips via serde. The serialized form
        //    must contain each variant's tag plus its fields, and the
        //    deserialized value must equal the original.
        let wire = serde_json::to_string(&transcript).expect("serialize transcript");
        assert!(wire.contains("\"Offer\""), "wire form encodes Offer variant");
        assert!(
            wire.contains("\"Counter\""),
            "wire form encodes Counter variant"
        );
        assert!(
            wire.contains("\"Accept\""),
            "wire form encodes Accept variant"
        );

        let decoded: Transcript =
            serde_json::from_str(&wire).expect("deserialize transcript");
        assert_eq!(
            decoded.clauses(),
            transcript.clauses(),
            "transcript clauses round-trip via serde"
        );

        // Equality on the whole `Transcript` also holds — the
        // plain serde-derived `PartialEq` covers the optional
        // flavor text too.
        assert_eq!(decoded, transcript);

        // 2. `render_flavor_text` is non-empty and deterministic across
        //    two calls on the same transcript.
        let first = transcript.render_flavor_text();
        let second = transcript.render_flavor_text();
        assert!(!first.is_empty(), "render_flavor_text must be non-empty");
        assert_eq!(
            first, second,
            "render_flavor_text must be deterministic across calls"
        );

        // The flavor text is also reproducible across a serde round-trip:
        // a sanitized transcript (no LLM garnish) renders the same bytes
        // as the freshly-built one.
        let third = decoded.render_flavor_text();
        assert_eq!(
            first, third,
            "render_flavor_text must be stable across serde round-trip"
        );

        // Sanity: the rendered string is the LLM seam; it must mention
        // each clause variant by name so a future LLM decorator can see
        // what it is dressing up. This pins the placeholder rendering —
        // if a future slice swaps in a real LLM, this assertion will be
        // updated to match the new format. The assertion itself is the
        // hook for that evolution.
        assert!(first.contains("offer"), "rendered text references Offer");
        assert!(first.contains("counter"), "rendered text references Counter");
        assert!(first.contains("accept"), "rendered text references Accept");
    }

    /// FR-CIV-DIPLO-005: a transcript built from a rejection path also
    /// round-trips, and the deterministic renderer surfaces the rejection
    /// variant. Guards against accidentally dropping the `Reject` branch
    /// whenever the renderer is refactored.
    #[test]
    fn fr_civ_diplo_005_rejection_path_round_trips() {
        let mut t = Transcript::empty();
        t.push(TranscriptClause::Offer {
            party: p(1),
            level: 20,
        });
        t.push(TranscriptClause::Concession {
            round: 1,
            party: p(2),
            amount: 5,
        });
        t.push(TranscriptClause::Reject {
            party: p(2),
            last_level: 25,
        });

        let wire = serde_json::to_string(&t).expect("serialize");
        let decoded: Transcript = serde_json::from_str(&wire).expect("deserialize");
        assert_eq!(decoded, t);

        let rendered = t.render_flavor_text();
        assert!(rendered.contains("reject"));
        assert!(rendered.contains("concession"));
    }
}
