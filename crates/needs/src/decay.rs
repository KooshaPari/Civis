//! FR-CIV-NEEDS-DECAY — additive needs-decay model.
//!
//! This module is intentionally narrow and **additive**: it does not modify the
//! existing [`crate::Needs`] / [`crate::DecayRates`] / health pipeline. It
//! models a small, focused agent-needs loop that gameplay / AI layers can
//! layer on top when they need a minimal "hunger / rest / social" driver
//! without pulling in the full FR-CIV-LIFE sickness / death machinery.
//!
//! # Semantics
//!
//! Each [`NeedLevel`] is a *pressure* (deprivation) value in `[0, 1]` where
//! `1.0` means "fully deprived / critical" and `0.0` means "fully sated".
//! Every simulation tick the configured [`RiseRates`] are added to the three
//! levels (hunger / rest / social). When the agent spends a resource to
//! satisfy a need, [`apply_resource`] subtracts an amount and clamps at `0.0`.
//!
//! All arithmetic is `f32` and free of any RNG / wall-clock input, matching
//! the ADR-008 determinism invariants used by the rest of the crate.
//!
//! Traceability: `FR-CIV-NEEDS-DECAY` (needs rise over ticks, drop on resource).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// The three agent needs tracked by this decay model.
///
/// Unlike [`crate::NeedKind`] (which enumerates six survival channels plus
/// mirrors health), this is the minimal agent-facing triplet that gameplay
/// code most commonly references: hunger, rest, social.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeedLevel {
    /// Caloric deprivation pressure (`0.0` = fed, `1.0` = starving).
    pub hunger: f32,
    /// Sleep deprivation pressure (`0.0` = rested, `1.0` = exhausted).
    pub rest: f32,
    /// Social-contact deprivation pressure (`0.0` = connected, `1.0` = isolated).
    pub social: f32,
}

impl Default for NeedLevel {
    fn default() -> Self {
        Self::sated()
    }
}

impl NeedLevel {
    /// A fully-sated agent: every need pressure at zero.
    #[must_use]
    pub const fn sated() -> Self {
        Self {
            hunger: 0.0,
            rest: 0.0,
            social: 0.0,
        }
    }

    /// `true` when any need pressure has reached `critical` (inclusive).
    #[must_use]
    pub fn any_critical(&self, critical: f32) -> bool {
        self.hunger >= critical || self.rest >= critical || self.social >= critical
    }

    /// The single most-pressured need and its level.
    ///
    /// Ties are broken by field order (`hunger`, then `rest`, then `social`)
    /// so the selection is deterministic and stable across replays.
    #[must_use]
    pub fn most_pressing(&self) -> (NeedChannel, f32) {
        // Stable tie-break: prefer the lowest-ordered channel.
        if self.hunger >= self.rest && self.hunger >= self.social {
            (NeedChannel::Hunger, self.hunger)
        } else if self.rest >= self.social {
            (NeedChannel::Rest, self.rest)
        } else {
            (NeedChannel::Social, self.social)
        }
    }
}

/// Identifies one of the three needs in [`NeedLevel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedChannel {
    /// Caloric need.
    Hunger,
    /// Rest need.
    Rest,
    /// Social need.
    Social,
}

impl NeedChannel {
    /// Canonical iteration order (matches [`NeedLevel`] field order).
    pub const ALL: [NeedChannel; 3] = [NeedChannel::Hunger, NeedChannel::Rest, NeedChannel::Social];

    /// Read the pressure for this channel from a [`NeedLevel`].
    #[must_use]
    pub fn get(self, n: &NeedLevel) -> f32 {
        match self {
            NeedChannel::Hunger => n.hunger,
            NeedChannel::Rest => n.rest,
            NeedChannel::Social => n.social,
        }
    }

    /// Write the pressure for this channel on a [`NeedLevel`], clamping to `[0, 1]`.
    pub fn set(self, n: &mut NeedLevel, v: f32) {
        let clamped = v.clamp(0.0, 1.0);
        match self {
            NeedChannel::Hunger => n.hunger = clamped,
            NeedChannel::Rest => n.rest = clamped,
            NeedChannel::Social => n.social = clamped,
        }
    }
}

/// Per-tick rates at which each need's pressure rises.
///
/// Values are interpreted as **additive pressure per tick** — `0.0` disables
/// that channel. Nothing in this module enforces a sum cap, but realistic
/// configurations usually keep each rate in roughly `0.001..=0.05`. Rates are
/// deterministic constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiseRates {
    /// Hunger pressure added per tick.
    pub hunger: f32,
    /// Rest pressure added per tick.
    pub rest: f32,
    /// Social pressure added per tick.
    pub social: f32,
}

impl Default for RiseRates {
    fn default() -> Self {
        Self {
            hunger: 0.01,
            rest: 0.008,
            social: 0.005,
        }
    }
}

/// Advance every need's pressure by its configured per-tick rate, clamping to `[0, 1]`.
///
/// This is the *rise* half of the decay model: pressure grows monotonically
/// tick over tick as long as the agent is not consuming resources.
pub fn tick_rise(needs: &mut NeedLevel, rates: &RiseRates) {
    needs.hunger = (needs.hunger + rates.hunger).clamp(0.0, 1.0);
    needs.rest = (needs.rest + rates.rest).clamp(0.0, 1.0);
    needs.social = (needs.social + rates.social).clamp(0.0, 1.0);
}

/// Apply a resource of `amount` to a [`NeedChannel`], dropping its pressure.
///
/// The amount is subtracted from the channel's current pressure and the
/// result is clamped to `[0, 1]`. The new level is returned so callers can
/// decide whether further consumption is worthwhile.
///
/// Negative `amount` values are clamped to `0.0` (you cannot make a need
/// more deprived by consuming a resource).
#[must_use]
pub fn apply_resource(needs: &mut NeedLevel, channel: NeedChannel, amount: f32) -> f32 {
    let amount = amount.max(0.0);
    let current = channel.get(needs);
    let next = (current - amount).clamp(0.0, 1.0);
    channel.set(needs, next);
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-NEEDS-DECAY — a need rises over ticks and drops when a resource satisfies it.
    ///
    /// The test uses an explicit, deterministic `RiseRates` triple so the
    /// rise after `RISE_TICKS` ticks is exactly known, then asserts that a
    /// single `apply_resource` call drops the hunger level below the
    /// pre-satisfaction value.
    #[test]
    fn need_rises_over_ticks_then_drops_on_resource() {
        const RISE_TICKS: u32 = 10;
        let rates = RiseRates {
            hunger: 0.05,
            rest: 0.02,
            social: 0.01,
        };
        let mut needs = NeedLevel::sated();

        // After one tick, every channel must have strictly increased.
        tick_rise(&mut needs, &rates);
        let after_one = needs;
        assert!(after_one.hunger > 0.0);
        assert!(after_one.rest > 0.0);
        assert!(after_one.social > 0.0);

        // Continue ticking for `RISE_TICKS - 1` more (we already ticked once).
        for _ in 1..RISE_TICKS {
            tick_rise(&mut needs, &rates);
        }
        let pre_satisfaction = needs.hunger;

        // Sanity: pressure rose monotonically over the run.
        assert!(
            (pre_satisfaction - 0.5).abs() < 1e-6,
            "hunger should be 0.5 after 10 ticks at rate 0.05 (got {pre_satisfaction})"
        );
        assert!(pre_satisfaction > after_one.hunger);

        // Apply a resource large enough to drop hunger substantially.
        let resource_amount = 0.4;
        let post = apply_resource(&mut needs, NeedChannel::Hunger, resource_amount);
        let expected = (pre_satisfaction - resource_amount).clamp(0.0, 1.0);

        assert!(
            post < pre_satisfaction,
            "resource must drop the pressure below the pre-satisfaction level"
        );
        assert!(
            (needs.hunger - post).abs() < 1e-6,
            "apply_resource must mutate the level in place"
        );
        assert!(
            (post - expected).abs() < 1e-6,
            "resource amount must be subtracted from the current level (got {post}, expected {expected})"
        );
        assert!(
            post >= 0.0 && post <= 1.0,
            "post-satisfaction pressure must remain in [0, 1]"
        );
    }

    /// FR-CIV-NEEDS-DECAY — `apply_resource` cannot push a pressure below zero.
    #[test]
    fn apply_resource_clamps_at_zero() {
        let mut needs = NeedLevel::sated();
        tick_rise(&mut needs, &RiseRates::default());
        // Over-satisfy: should clamp at 0.0, not wrap.
        let post = apply_resource(&mut needs, NeedChannel::Hunger, 5.0);
        assert_eq!(post, 0.0);
        assert_eq!(needs.hunger, 0.0);
    }

    /// FR-CIV-NEEDS-DECAY — `apply_resource` ignores negative amounts.
    #[test]
    fn apply_resource_ignores_negative_amount() {
        let mut needs = NeedLevel::sated();
        tick_rise(&mut needs, &RiseRates::default());
        let before = needs.hunger;
        let post = apply_resource(&mut needs, NeedChannel::Hunger, -0.5);
        assert_eq!(post, before);
        assert_eq!(needs.hunger, before);
    }
}
