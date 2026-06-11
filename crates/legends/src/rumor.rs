use crate::ids::{Epoch, LegendEntityId, LegendEventId, NameRef};
use crate::model::{EventKind, EventNode, Tag};
use rand::{Rng, SeedableRng};
use smallvec::SmallVec;
use tracery::Grammar;

/// Five-factor OCEAN psyche snapshot (FR-CIV-LEGENDS-003) carried on the
/// historian's mind. The values are normalized to `0.0..=1.0`; `legends` does
/// not depend on the `needs` crate so the type lives here as a data bag. A
/// producer that wants OCEAN-gated embellishment fills the bag from the
/// sim-side `Psyche` component; the engine itself only consumes the floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ocean {
    pub openness: f32,
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
}

impl Default for Ocean {
    fn default() -> Self {
        // Flat "average mind" so the existing call sites that don't fill OCEAN
        // continue to behave the way the unit tests in this module assert.
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

impl Ocean {
    /// OCEAN gate on embellishment (FR-CIV-LEGENDS-003). High openness +
    /// low conscientiousness ⇒ more drift, more actor-swap. Symmetric clamp
    /// to `[0.0, 1.0]` so a malformed bag can't poison the rumor mill.
    pub fn embellishment_gate(self) -> f32 {
        let raw = 0.5 * self.openness + 0.4 * (1.0 - self.conscientiousness)
            - 0.2 * self.agreeableness
            + 0.1 * self.neuroticism;
        raw.clamp(0.0, 1.0)
    }

    /// OCEAN gate on swap probability (FR-CIV-LEGENDS-003). Extraversion +
    /// low agreeableness ⇒ more actor-swap. Symmetric clamp.
    pub fn swap_gate(self) -> f32 {
        let raw = 0.5 * self.extraversion + 0.3 * (1.0 - self.agreeableness)
            - 0.4 * self.conscientiousness;
        raw.clamp(0.0, 1.0)
    }
}

/// Cultural register the prose surface emits (FR-CIV-LEGENDS-007).
/// `Narrative` is the default for the historian's retelling; `Formal` is
/// reserved for treaty / law / chronicle text (see `crates/diplomacy`); the
/// rumor mill never crosses registers — the choice is explicit at the call
/// site, never inferred from the prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    /// Storytelling register — embellishment + tracery + prose cache.
    Narrative,
    /// Treaty / law / chronicle register — no embellishment, no tracery.
    Formal,
    /// Sacred register — deity sphere tags surface, no actor-swap.
    Sacred,
}

impl Default for Register {
    fn default() -> Self {
        Register::Narrative
    }
}

/// Resolves a `NameRef` to a display name (FR-CIV-LEGENDS-008). The legends
/// engine does NOT own a name store — the `ai-rnd` crate is the source of
/// truth and exposes its own resolver. The trait lets callers wire that
/// resolver in without making `civ-legends` depend on `ai-rnd`. A no-op
/// fallback (`|n| format!("#{}", n.0)`) is used when no resolver is wired.
pub trait NameResolver {
    fn resolve(&self, name: NameRef) -> String;
}

/// Default resolver: render the id as `entity:N` so the engine never hard-codes
/// English. Callers that want real names hand in their own resolver.
///
/// The `entity:` prefix is intentional: tracery treats `#` as a tag opener, so
/// the default fallback MUST avoid `#` (otherwise the rule text would be
/// parsed as a tag reference and flatten would fail with a `MissingKeyError`).
/// The chosen prefix is unambiguous, still obviously a synthetic id, and
/// downstream UI can grep for it to mark "no name resolved" rows.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultNameResolver;

impl NameResolver for DefaultNameResolver {
    fn resolve(&self, name: NameRef) -> String {
        format!("entity:{}", name.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HistorianMind {
    /// Authoring bias. `0.0` = verbatim, `1.0` = mythmaker.
    pub embellishment: f32,
    /// Trustworthiness of this teller. `1.0` = saw it themselves.
    pub reliability: f32,
    /// Stable id; helps traces + rumor-chain audits.
    pub id: u64,
    /// OCEAN traits (FR-CIV-LEGENDS-003). Gates embellishment + actor-swap
    /// probabilities per retelling hop.
    pub ocean: Ocean,
    /// Cultural register the historian writes in (FR-CIV-LEGENDS-007).
    pub register: Register,
    /// Deity sphere the historian serves (FR-CIV-LEGENDS-004). When `Some`,
    /// the prose surface appends a cult/taboo/art reference tag. `None`
    /// means the historian is secular; no sphere is named in the prose.
    pub deity_sphere: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct Rumor {
    pub event_id: LegendEventId,
    pub origin_epoch: Epoch,
    pub hop: u16,
    pub subject: LegendEntityId,
    pub claimed_kind: EventKind,
    pub claimed_magnitude: f32,
    pub tags: SmallVec<[Tag; 4]>,
    pub text: String,
    pub chain: SmallVec<[u64; 4]>,
    /// Bladeink-style salience score (FR-CIV-LEGENDS-004). `0.0..=1.0`,
    /// rank-ordered: higher = the prose should be surfaced first. Set by
    /// `embellish_rumor`; consumed by the literature/narrator UI through
    /// `register_render` so the prose surface never has to re-derive it.
    pub salience: f32,
}

pub fn witness<R: Rng + ?Sized>(
    historian: &HistorianMind,
    ev: &EventNode,
    rng: &mut R,
) -> Option<Rumor> {
    let witness_prob = (historian.reliability + ev.magnitude).clamp(0.0, 1.0);
    if rng.gen::<f32>() >= witness_prob {
        return None;
    }

    let subject = ev
        .participants
        .first()
        .copied()
        .unwrap_or(LegendEntityId(0));
    let mut rumor = Rumor {
        event_id: ev.id,
        origin_epoch: ev.epoch,
        hop: 0,
        subject,
        claimed_kind: ev.kind.clone(),
        claimed_magnitude: ev.magnitude,
        tags: SmallVec::new(),
        text: String::new(),
        chain: SmallVec::new(),
        salience: 0.0,
    };
    rumor.chain.push(historian.id);
    embellish_rumor(historian, &mut rumor, &ev.participants, rng);
    rumor.text = render_with_rng(&rumor, rng);
    Some(rumor)
}

pub fn retell<R: Rng + ?Sized>(
    historian: &HistorianMind,
    prior: &Rumor,
    candidates: &[LegendEntityId],
    rng: &mut R,
) -> Rumor {
    let mut rumor = Rumor {
        hop: prior.hop.saturating_add(1),
        salience: prior.salience,
        ..prior.clone()
    };
    rumor.chain.push(historian.id);
    embellish_rumor(historian, &mut rumor, candidates, rng);
    rumor.text = render_with_rng(&rumor, rng);
    rumor
}

pub fn render(rumor: &Rumor) -> String {
    format!(
        "Epoch {} says {} {} (magnitude {:.2})",
        rumor.origin_epoch.0,
        rumor.subject.0,
        rumor.claimed_kind.label(),
        rumor.claimed_magnitude,
    )
}

/// Render a rumor in an explicit register (FR-CIV-LEGENDS-007 + 008).
///
/// - `Register::Narrative` — embellished prose via tracery; the resolved
///   subject name is substituted in directly. The historian's `deity_sphere`
///   is *not* appended in narrative register (the sphere tag is reserved for
///   sacred text, FR-CIV-LEGENDS-004).
/// - `Register::Formal` — no embellishment, no tracery, no actor-swap, no
///   tags. The output is a single-line factual record, suitable for treaty
///   text and law chronicles (see `crates/diplomacy`). The rumor mill's
///   embellishment is *bypassed* here because formal text is not the rumor
///   mill's job.
/// - `Register::Sacred` — narrative-style prose with the resolved subject
///   name and a sphere tag (when the historian carries one). The factual
///   layer is already locked (no swap) by `embellish_rumor`.
///
/// The `NameResolver` is the bridge to the language-drift name store
/// (FR-CIV-LEGENDS-008); the engine itself never hard-codes English labels,
/// so the function takes the resolver by reference and threads the resolved
/// name into the tracery template.
pub fn register_render<R: NameResolver + ?Sized>(
    rumor: &Rumor,
    register: Register,
    historian: &HistorianMind,
    resolver: &R,
) -> String {
    let subject_name = resolver.resolve(NameRef(rumor.subject.0));
    match register {
        Register::Formal => format!(
            "[FORMAL] epoch={} subject={} kind={} magnitude={:.3} salience={:.3}",
            rumor.origin_epoch.0,
            subject_name,
            rumor.claimed_kind.label(),
            rumor.claimed_magnitude,
            rumor.salience,
        ),
        Register::Narrative | Register::Sacred => {
            // Build a tracery template that uses the resolved subject name
            // (FR-CIV-LEGENDS-008: never hardcode the integer id into the
            // prose surface). The seed mixes the event id with the register
            // so the sacred branch and the narrative branch produce distinct
            // outputs for the same rumor.
            let seed = match register {
                Register::Narrative => rumor.origin_epoch.0.wrapping_add(rumor.event_id.0),
                Register::Sacred => {
                    rumor.origin_epoch.0.wrapping_add(rumor.event_id.0 ^ 0xC0FFEE)
                }
                Register::Formal => 0, // unreachable
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut prose = render_with_resolved_subject(rumor, &subject_name, &mut rng);
            if register == Register::Sacred {
                append_deity_tag(&mut prose, historian.deity_sphere);
            }
            prose
        }
    }
}

fn render_with_resolved_subject<R: Rng + ?Sized>(
    rumor: &Rumor,
    subject: &str,
    rng: &mut R,
) -> String {
    let kind = rumor.claimed_kind.label();
    let adjectives = collect_adjectives(rumor);
    let grammar_json = format!(
        r#"{{
  "origin": ["{origin} tells of {subject}, who #verb# in a #adj# #kind# deed."],
  "verb": ["witnessed", "recorded", "recounted", "hailed", "lamented", "remembered"],
  "subject": ["{subject}"],
  "kind": ["{kind}"],
  "adj": [{adjectives}]
}}"#,
        origin = rumor.origin_epoch.0,
        subject = subject,
        kind = kind,
    );
    let mut grammar = match Grammar::from_json(grammar_json.as_str()) {
        Ok(g) => g,
        Err(_) => return format!("{} tells of {}", rumor.origin_epoch.0, subject),
    };
    grammar.set_default_rule("origin");
    let mut output = match grammar.flatten(rng) {
        Ok(s) => s,
        Err(_) => format!("{} tells of {}", rumor.origin_epoch.0, subject),
    };
    output.push_str(&format!(" [hop {}]", rumor.hop));
    output
}

fn append_deity_tag(prose: &mut String, sphere: Option<&'static str>) {
    if let Some(s) = sphere {
        prose.push_str(&format!(" [sphere:{s}]"));
    }
}

fn embellish_rumor<R: Rng + ?Sized>(
    historian: &HistorianMind,
    rumor: &mut Rumor,
    candidates: &[LegendEntityId],
    rng: &mut R,
) {
    // Sacred register never swaps actors (the priest who names the saint
    // does not lie) and never tags — only the prose surface differs. This is
    // the FR-CIV-LEGENDS-004 deity-sphere guard rail: the *prose* may be
    // sacred, but the *facts* the historian asserts stay intact.
    let sacred_lock = historian.register == Register::Sacred;
    let allow_swap = !sacred_lock;
    let allow_tags = !sacred_lock;

    // FR-CIV-LEGENDS-003: OCEAN gates the per-hop mutation.
    // `embellishment` stays the master dial; OCEAN is a *bias* on it, so a
    // historian with `embellishment=0` still doesn't drift (their prose
    // matches a verbatim witness), and a high-OCEAN low-embellishment
    // historian stays close to source.
    let ocean_emb = historian.ocean.embellishment_gate();
    let ocean_swap = historian.ocean.swap_gate();
    let effective_emb = (historian.embellishment * 0.7 + ocean_emb * 0.3).clamp(0.0, 1.0);

    let drift_sigma = (effective_emb * 0.15).max(0.0);
    if drift_sigma > 0.0 {
        let drift = rng.gen_range(-drift_sigma..drift_sigma);
        rumor.claimed_magnitude = clamp01(rumor.claimed_magnitude + drift);
    }

    // OCEAN-gated swap probability (FR-CIV-LEGENDS-003). The base
    // `embellishment * (1 - reliability)` term is the design-of-record from
    // before OCEAN existed; we keep it and bias it through the OCEAN
    // `swap_gate`. The combined probability is clamped to [0, 1] so a
    // pathological historian (embellishment=1, reliability=0, extravert=1,
    // disagreeable=1) cannot produce a `> 1.0` swap rate.
    let base_swap = (effective_emb * (1.0 - historian.reliability) * 0.3).clamp(0.0, 1.0);
    let swap_prob = (base_swap * 0.7 + ocean_swap * 0.3).clamp(0.0, 1.0);
    if allow_swap && !candidates.is_empty() && rng.gen::<f32>() < swap_prob {
        let idx = rng.gen_range(0..candidates.len());
        rumor.subject = candidates[idx];
    }

    if allow_tags && rng.gen::<f32>() < (effective_emb * 0.4).clamp(0.0, 1.0) {
        let tags = ["heroic", "cursed", "vast", "treacherous"];
        let chosen = tags[rng.gen_range(0..tags.len())];
        rumor.tags.push(chosen.to_string());
    }

    // Bladeink salience (FR-CIV-LEGENDS-004). The score mixes the event's
    // raw magnitude, the historian's reliability (1.0 = saw it themselves),
    // and the hop count (later retellings lose salience as the chain ages).
    // Bounded to [0, 1] so callers can rank-order across registers.
    let raw = rumor.claimed_magnitude * 0.5
        + historian.reliability * 0.4
        + (1.0 / (1.0 + rumor.hop as f32)) * 0.1;
    rumor.salience = clamp01(raw);
}

fn render_with_rng<R: Rng + ?Sized>(rumor: &Rumor, rng: &mut R) -> String {
    let subject = rumor.subject.0.to_string();
    let kind = rumor.claimed_kind.label();
    let adjectives = collect_adjectives(rumor);
    let grammar_json = format!(
        r#"{{
  "origin": ["{origin} tells of {subject}, who #verb# in a #adj# #kind# deed."],
  "verb": ["witnessed", "recorded", "recounted", "hailed", "lamented", "remembered"],
  "subject": ["{subject}"],
  "kind": ["{kind}"],
  "adj": [{adjectives}]
}}"#,
        origin = rumor.origin_epoch.0,
        subject = subject,
        kind = kind,
    );
    let mut grammar = match Grammar::from_json(grammar_json.as_str()) {
        Ok(g) => g,
        Err(_) => return format!("{} tells of {}", rumor.origin_epoch.0, subject),
    };
    grammar.set_default_rule("origin");
    let mut output = match grammar.flatten(rng) {
        Ok(s) => s,
        Err(_) => format!("{} tells of {}", rumor.origin_epoch.0, subject),
    };
    output.push_str(&format!(" [hop {}]", rumor.hop));
    output
}

fn collect_adjectives(rumor: &Rumor) -> String {
    let mut adjectives = vec!["heroic", "cursed", "vast", "treacherous"];
    for tag in rumor.tags.iter() {
        if !adjectives.iter().any(|a| *a == tag.as_str()) {
            adjectives.push(tag.as_str());
        }
    }
    adjectives.truncate(6);
    adjectives
        .into_iter()
        .map(|word| format!("\"{}\"", word))
        .collect::<Vec<_>>()
        .join(",")
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

const RUMOR_LIMIT: usize = 512;

#[derive(Debug, Default)]
pub struct RumorMill {
    pub rumors: Vec<Rumor>,
}

impl RumorMill {
    pub fn step<R: Rng + ?Sized>(
        &mut self,
        historians: &[HistorianMind],
        events: &[EventNode],
        candidates: &[LegendEntityId],
        rng: &mut R,
    ) {
        for ev in events {
            if historians.is_empty() {
                break;
            }
            let historian = &historians[rng.gen_range(0..historians.len())];
            if let Some(rumor) = witness(historian, ev, rng) {
                self.rumors.push(rumor);
            }
        }

        if historians.is_empty() {
            self.cap_bounded();
            return;
        }

        let spread_budget = (self.rumors.len().min(4)).max(1);
        let total_rumors = self.rumors.len();
        for _ in 0..spread_budget {
            if self.rumors.is_empty() || total_rumors == 0 {
                break;
            }

            let historian = &historians[rng.gen_range(0..historians.len())];
            let rumor_idx = rng.gen_range(0..total_rumors);
            let prior = self.rumors[rumor_idx].clone();
            let retold = retell(historian, &prior, candidates, rng);
            self.rumors.push(retold);
        }

        self.cap_bounded();
    }

    fn cap_bounded(&mut self) {
        if self.rumors.len() <= RUMOR_LIMIT {
            return;
        }

        self.rumors
            .sort_by(|a, b| a.origin_epoch.0.cmp(&b.origin_epoch.0));
        let keep = self.rumors.len() - RUMOR_LIMIT;
        self.rumors.drain(0..keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn base_rumor() -> Rumor {
        Rumor {
            event_id: LegendEventId(1),
            origin_epoch: Epoch(10),
            hop: 0,
            subject: LegendEntityId(12),
            claimed_kind: EventKind::Battle,
            claimed_magnitude: 0.5,
            tags: SmallVec::new(),
            text: String::from("origin"),
            chain: SmallVec::new(),
            salience: 0.0,
        }
    }

    fn base_event() -> EventNode {
        EventNode {
            id: LegendEventId(2),
            epoch: Epoch(11),
            region: None,
            kind: EventKind::Battle,
            magnitude: 0.6,
            participants: SmallVec::from_iter([LegendEntityId(7), LegendEntityId(8)]),
            summary_key: [0; 32],
            source_crate: crate::ids::SourceCrate::Agents,
            provenance: crate::ids::Provenance::Lived,
            raw_ref: None,
        }
    }

    fn event_stream(events: usize) -> Vec<EventNode> {
        (0..events)
            .map(|idx| EventNode {
                id: LegendEventId(idx as u64),
                epoch: Epoch(idx as u64),
                region: None,
                kind: EventKind::WarDeclared,
                magnitude: 0.7,
                participants: SmallVec::from_iter([LegendEntityId(idx as u64 + 1)]),
                summary_key: [0; 32],
                source_crate: crate::ids::SourceCrate::Agents,
                provenance: crate::ids::Provenance::Lived,
                raw_ref: None,
            })
            .collect()
    }

    #[test]
    fn retell_produces_mutating_chain_and_text_variation() {
        let historian = HistorianMind {
            embellishment: 0.95,
            reliability: 0.1,
            id: 42,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let mut rng = StdRng::seed_from_u64(99);
        let mut rumor = base_rumor();
        let candidates = [LegendEntityId(13), LegendEntityId(14)];

        for _ in 0..10 {
            rumor = retell(&historian, &rumor, &candidates, &mut rng);
        }

        assert_ne!(rumor.claimed_magnitude, base_rumor().claimed_magnitude);
        assert_ne!(rumor.text, base_rumor().text);
        assert_eq!(rumor.chain.len(), 10);
    }

    #[test]
    fn higher_embellishment_drifts_more_over_hops() {
        let low = HistorianMind {
            embellishment: 0.1,
            reliability: 0.95,
            id: 1,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let high = HistorianMind {
            embellishment: 0.95,
            reliability: 0.05,
            id: 2,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let candidates = [LegendEntityId(101), LegendEntityId(102)];
        let mut low_rng = StdRng::seed_from_u64(1_234);
        let mut high_rng = StdRng::seed_from_u64(1_234);

        let origin = 0.7_f32;
        let mut low_rumor = Rumor {
            claimed_magnitude: origin,
            ..base_rumor()
        };
        let mut high_rumor = Rumor {
            claimed_magnitude: origin,
            ..base_rumor()
        };
        let mut low_total = 0.0_f32;
        let mut high_total = 0.0_f32;
        let hops = 40usize;

        for _ in 0..hops {
            low_rumor = retell(&low, &low_rumor, &candidates, &mut low_rng);
            high_rumor = retell(&high, &high_rumor, &candidates, &mut high_rng);
            low_total += (low_rumor.claimed_magnitude - origin).abs();
            high_total += (high_rumor.claimed_magnitude - origin).abs();
        }

        let low_mean = low_total / hops as f32;
        let high_mean = high_total / hops as f32;
        assert!(low_mean < high_mean);
    }

    #[test]
    fn rumor_mill_spreads_existing_rumors() {
        let mut rng = StdRng::seed_from_u64(7);
        let historians = vec![
            HistorianMind {
                embellishment: 0.7,
                reliability: 0.6,
                id: 10,
                ocean: Ocean::default(),
                register: Register::Narrative,
                deity_sphere: None,
            },
            HistorianMind {
                embellishment: 0.5,
                reliability: 0.8,
                id: 11,
                ocean: Ocean::default(),
                register: Register::Narrative,
                deity_sphere: None,
            },
            HistorianMind {
                embellishment: 0.4,
                reliability: 0.4,
                id: 12,
                ocean: Ocean::default(),
                register: Register::Narrative,
                deity_sphere: None,
            },
        ];
        let candidates = vec![LegendEntityId(1), LegendEntityId(2), LegendEntityId(3)];
        let mut mill = RumorMill::default();

        for idx in 0..20 {
            let events = vec![EventNode {
                id: LegendEventId(idx as u64),
                epoch: Epoch(idx as u64),
                region: None,
                kind: EventKind::Migration,
                magnitude: 0.9,
                participants: SmallVec::from_iter([LegendEntityId(1), LegendEntityId(2)]),
                summary_key: [0; 32],
                source_crate: crate::ids::SourceCrate::Agents,
                provenance: crate::ids::Provenance::Lived,
                raw_ref: None,
            }];

            mill.step(&historians, &events, &candidates, &mut rng);
        }

        assert!(mill.rumors.iter().any(|r| r.hop > 0));
        assert!(!mill.rumors.is_empty());
    }

    #[test]
    fn witness_respects_probability_toward_source_truth() {
        let historian = HistorianMind {
            embellishment: 0.5,
            reliability: 0.9,
            id: 99,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let mut rng = StdRng::seed_from_u64(123);
        let ev = base_event();
        let seen = witness(&historian, &ev, &mut rng);
        assert!(seen.is_some());
    }

    // --- FR-CIV-LEGENDS-003: OCEAN-gated retell mutation ------------------
    // Each retelling hop SHALL mutate rumors (actor swap, amplification,
    // teller psyche/culture tags) with gates from OCEAN traits.
    #[test]
    fn fr_legends_003_ocean_gates_swap_and_amplify() {
        // Open + extravert + disagreeable -> high swap rate and high amp.
        let wild = HistorianMind {
            embellishment: 0.0,
            reliability: 1.0,
            id: 7,
            ocean: Ocean {
                openness: 1.0,
                conscientiousness: 0.0,
                extraversion: 1.0,
                agreeableness: 0.0,
                neuroticism: 1.0,
            },
            register: Register::Narrative,
            deity_sphere: None,
        };
        // Conscientious + agreeable + stable -> gate nearly closed.
        let tame = HistorianMind {
            embellishment: 0.0,
            reliability: 1.0,
            id: 8,
            ocean: Ocean {
                openness: 0.0,
                conscientiousness: 1.0,
                extraversion: 0.0,
                agreeableness: 1.0,
                neuroticism: 0.0,
            },
            register: Register::Narrative,
            deity_sphere: None,
        };
        let candidates = [
            LegendEntityId(101),
            LegendEntityId(102),
            LegendEntityId(103),
            LegendEntityId(104),
        ];

        // With embellishment = 0 and reliability = 1.0 the base swap is 0.0
        // (effective_emb = 0, base_swap = 0). The OCEAN swap gate only
        // raises the rate, so any swap observed MUST be driven by the OCEAN
        // gates, not by embellishment. This is the OCEAN-gate contract: a
        // verbatim historian with a wild OCEAN swaps more often than the
        // same historian with a tame OCEAN.
        let mut rng = StdRng::seed_from_u64(5);
        let mut wild_swaps = 0;
        let mut tame_swaps = 0;
        for _ in 0..400 {
            let mut r = base_rumor();
            r = retell(&wild, &r, &candidates, &mut rng);
            if r.subject != LegendEntityId(12) {
                wild_swaps += 1;
            }
            let mut r2 = base_rumor();
            r2 = retell(&tame, &r2, &candidates, &mut rng);
            if r2.subject != LegendEntityId(12) {
                tame_swaps += 1;
            }
        }
        assert!(
            wild_swaps > tame_swaps,
            "OCEAN swap gate must let a wild mind swap more than a tame mind: wild={} tame={}",
            wild_swaps,
            tame_swaps
        );

        // Amplification: a wild OCEAN with no embellishment must still drift
        // the claimed magnitude, because OCEAN amplifies the drift sigma.
        let mut wild_rumor = base_rumor();
        let mut tame_rumor = base_rumor();
        let mut wild_rng = StdRng::seed_from_u64(11);
        let mut tame_rng = StdRng::seed_from_u64(11);
        for _ in 0..40 {
            wild_rumor = retell(&wild, &wild_rumor, &candidates, &mut wild_rng);
            tame_rumor = retell(&tame, &tame_rumor, &candidates, &mut tame_rng);
        }
        let wild_drift = (wild_rumor.claimed_magnitude - 0.5).abs();
        let tame_drift = (tame_rumor.claimed_magnitude - 0.5).abs();
        assert!(
            wild_drift > tame_drift,
            "OCEAN amplifies magnitude drift: wild_drift={} tame_drift={}",
            wild_drift,
            tame_drift
        );
    }

    // --- FR-CIV-LEGENDS-004: prose surface + bladeink salience + deity sphere
    // Prose surface SHALL wrap tracery templates with bladeink salience;
    // deity spheres SHALL tag cults/taboos/art references.
    #[test]
    fn fr_legends_004_bladeink_salience_and_deity_sphere_in_prose() {
        // Salience is computed in embellish_rumor (called from retell/witness).
        // The score is finite and in [0, 1].
        let historian = HistorianMind {
            embellishment: 0.4,
            reliability: 0.8,
            id: 1,
            ocean: Ocean::default(),
            register: Register::Sacred,
            deity_sphere: Some("war"),
        };
        let mut rng = StdRng::seed_from_u64(42);
        let rumor = retell(
            &historian,
            &base_rumor(),
            &[LegendEntityId(99)],
            &mut rng,
        );
        assert!(rumor.salience.is_finite());
        assert!((0.0..=1.0).contains(&rumor.salience));

        // Render in Sacred register with a deity sphere must surface the
        // sphere tag in the prose.
        let resolver = DefaultNameResolver;
        let prose = register_render(&rumor, Register::Sacred, &historian, &resolver);
        assert!(
            prose.contains("[sphere:war]"),
            "Sacred register must surface the deity sphere tag: {}",
            prose
        );

        // Narrative register must NOT surface the sphere tag (sphere is
        // reserved for sacred text; narrative is the historian's retelling).
        let prose_n = register_render(&rumor, Register::Narrative, &historian, &resolver);
        assert!(
            !prose_n.contains("[sphere:"),
            "Narrative register must not surface the deity sphere tag: {}",
            prose_n
        );
    }

    // --- FR-CIV-LEGENDS-007: cultural register separation ------------------
    // Cultural register output SHALL feed literature/historian UI; formal
    // register SHALL remain separate from treaty text. The engine SHALL
    // expose at least three registers (Narrative, Formal, Sacred); formal
    // SHALL be a structured factual record (no tracery, no embellishment),
    // narrative/sacred SHALL be tracery-driven prose.
    #[test]
    fn fr_legends_007_register_separation_is_deterministic() {
        let historian = HistorianMind {
            embellishment: 0.5,
            reliability: 0.7,
            id: 1,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let resolver = DefaultNameResolver;
        let rumor = base_rumor();

        // Formal register: structured factual record, no tracery verbs.
        let formal = register_render(&rumor, Register::Formal, &historian, &resolver);
        assert!(formal.starts_with("[FORMAL]"), "formal prose: {}", formal);
        // Formal MUST NOT mix in tracery verbs reserved for the narrative surface.
        for verb in ["witnessed", "recorded", "recounted", "hailed", "lamented", "remembered"] {
            assert!(!formal.contains(verb), "formal prose leaked tracery verb '{}': {}", verb, formal);
        }
        // Formal MUST NOT carry the treaty hand-off marker; treaty text is a
        // distinct downstream consumer of formal prose, not the prose itself.
        assert!(!formal.contains("[treaty-text]"));

        // Narrative register: tracery-driven prose, distinct from formal.
        let narrative = register_render(&rumor, Register::Narrative, &historian, &resolver);
        assert!(!narrative.starts_with("[FORMAL]"));
        assert!(!narrative.contains("[FORMAL]"));

        // Sacred register: distinct from narrative and formal.
        let sacred = register_render(&rumor, Register::Sacred, &historian, &resolver);
        assert!(!sacred.starts_with("[FORMAL]"));
        assert_ne!(narrative, sacred, "narrative and sacred must diverge for the same rumor+seed family");
    }

    // --- FR-CIV-LEGENDS-008: NameRef from language drift --------------------
    // Names in legend nodes SHALL reference NameRef from language drift, not
    // hardcoded English strings in the engine. The render surface SHALL accept
    // a NameResolver and use it to substitute the subject token.
    #[test]
    fn fr_legends_008_render_uses_nameref_not_hardcoded_english() {
        // Custom resolver that maps LegendEntityId(12) -> "Ash-Veined Tsar".
        struct Resolver;
        impl NameResolver for Resolver {
            fn resolve(&self, name: NameRef) -> String {
                if name.0 == 12 {
                    "Ash-Veined Tsar".to_string()
                } else {
                    format!("#{}", name.0)
                }
            }
        }

        let historian = HistorianMind {
            embellishment: 0.0,
            reliability: 1.0,
            id: 1,
            ocean: Ocean::default(),
            register: Register::Narrative,
            deity_sphere: None,
        };
        let resolver = Resolver;
        let prose = register_render(&base_rumor(), Register::Narrative, &historian, &resolver);
        assert!(
            prose.contains("Ash-Veined Tsar"),
            "resolver-provided NameRef must be in prose: {}",
            prose
        );

        // Default resolver produces `entity:N`; with it the prose MUST NOT
        // contain the resolved English label and MUST surface the bracketed
        // id. (`#N` is reserved by tracery as a tag opener; we use a colon
        // prefix to keep the rule text parseable while still being
        // recognisably a synthetic id.)
        let default_resolver = DefaultNameResolver;
        let prose_default = register_render(&base_rumor(), Register::Narrative, &historian, &default_resolver);
        assert!(!prose_default.contains("Ash-Veined Tsar"));
        assert!(prose_default.contains("entity:12"));
    }
}
