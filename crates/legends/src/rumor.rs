use crate::ids::{Epoch, LegendEntityId, LegendEventId};
use crate::model::{EventKind, EventNode, Tag};
use rand::Rng;
use smallvec::SmallVec;
use tracery::Grammar;

#[derive(Debug, Clone, Copy)]
pub struct HistorianMind {
    pub embellishment: f32,
    pub reliability: f32,
    pub id: u64,
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

fn embellish_rumor<R: Rng + ?Sized>(
    historian: &HistorianMind,
    rumor: &mut Rumor,
    candidates: &[LegendEntityId],
    rng: &mut R,
) {
    let drift_sigma = (historian.embellishment * 0.15).max(0.0);
    if drift_sigma > 0.0 {
        let drift = rng.gen_range(-drift_sigma..drift_sigma);
        rumor.claimed_magnitude = clamp01(rumor.claimed_magnitude + drift);
    }

    let swap_prob = (historian.embellishment * (1.0 - historian.reliability) * 0.3).clamp(0.0, 1.0);
    if !candidates.is_empty() && rng.gen::<f32>() < swap_prob {
        let idx = rng.gen_range(0..candidates.len());
        rumor.subject = candidates[idx];
    }

    if rng.gen::<f32>() < (historian.embellishment * 0.4).clamp(0.0, 1.0) {
        let tags = ["heroic", "cursed", "vast", "treacherous"];
        let chosen = tags[rng.gen_range(0..tags.len())];
        rumor.tags.push(chosen.to_string());
    }
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
        };
        let high = HistorianMind {
            embellishment: 0.95,
            reliability: 0.05,
            id: 2,
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
            },
            HistorianMind {
                embellishment: 0.5,
                reliability: 0.8,
                id: 11,
            },
            HistorianMind {
                embellishment: 0.4,
                reliability: 0.4,
                id: 12,
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
        };
        let mut rng = StdRng::seed_from_u64(123);
        let ev = base_event();
        let seen = witness(&historian, &ev, &mut rng);
        assert!(seen.is_some());
    }
}
