//! Emergent phoneme drift, lexicon growth, and naming (FR-CIV-LANG-*).
//!
//! No authored language-name tables — words are coined from drifted phoneme
//! inventories keyed by semantic kind + entity id.

use std::collections::BTreeMap;

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::culture::TraitVector;

/// Distinctive-feature dimension count (IPA-inspired, not locale-specific).
pub const PHONEME_FEATURES: usize = 6;

/// Default inventory size per cluster dialect.
pub const DEFAULT_INVENTORY_SIZE: usize = 8;

/// Per-phoneme feature vector in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Phoneme {
    pub features: [f32; PHONEME_FEATURES],
}

/// Drifted phoneme inventory for a population cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhonemeInventory {
    pub phonemes: Vec<Phoneme>,
    pub tick: u64,
}

impl PhonemeInventory {
    /// Deterministic inventory from a scalar seed (FR-CIV-LANG-002).
    pub fn seed_from(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut phonemes = Vec::with_capacity(DEFAULT_INVENTORY_SIZE);
        for _ in 0..DEFAULT_INVENTORY_SIZE {
            let mut features = [0.0f32; PHONEME_FEATURES];
            for feature in &mut features {
                *feature = rng.gen::<f32>();
            }
            phonemes.push(Phoneme { features });
        }
        Self { phonemes, tick: 0 }
    }

    /// Derive inventory seed from the culture trait vector.
    pub fn from_trait_seed(seed: TraitVector) -> Self {
        let mut h = seed[0].to_bits() as u64;
        h ^= (seed[1].to_bits() as u64).wrapping_mul(0x9E37_79B9);
        h ^= (seed[2].to_bits() as u64).wrapping_mul(0x85EB_CA6B);
        h ^= (seed[3].to_bits() as u64).wrapping_mul(0xC2B2_AE35);
        Self::seed_from(h)
    }
}

/// Semantic category for coined lexemes (not a language-name enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LexemeKind {
    Settlement,
    Faction,
    Event,
}

/// A coined word referencing phoneme inventory indices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lexeme {
    pub syllables: Vec<u8>,
    pub kind: LexemeKind,
    pub entity_id: u64,
}

/// Per-cluster evolved lexicon (grows as settlements/factions/events appear).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EvolvedLexicon {
    entries: BTreeMap<(LexemeKind, u64), Lexeme>,
}

impl EvolvedLexicon {
    /// Coin a lexeme if absent; returns stable reference.
    pub fn coin(
        &mut self,
        rng: &mut impl Rng,
        inventory: &PhonemeInventory,
        kind: LexemeKind,
        entity_id: u64,
    ) -> &Lexeme {
        let key = (kind, entity_id);
        if !self.entries.contains_key(&key) {
            let inv_len = inventory.phonemes.len().max(1);
            let syllable_count = 2 + (rng.gen::<u32>() % 2) as usize;
            let syllables: Vec<u8> = (0..syllable_count)
                .map(|_| (rng.gen::<u32>() as usize % inv_len) as u8)
                .collect();
            self.entries.insert(
                key,
                Lexeme {
                    syllables,
                    kind,
                    entity_id,
                },
            );
        }
        self.entries.get(&key).expect("coined lexeme")
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn get(&self, kind: LexemeKind, entity_id: u64) -> Option<&Lexeme> {
        self.entries.get(&(kind, entity_id))
    }
}

/// L2 distance between two phoneme inventories in `[0, 1]`.
#[must_use]
pub fn phoneme_inventory_distance(a: &PhonemeInventory, b: &PhonemeInventory) -> f32 {
    let len = a.phonemes.len().min(b.phonemes.len());
    if len == 0 {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for i in 0..len {
        sum += phoneme_distance(&a.phonemes[i], &b.phonemes[i]);
    }
    (sum / len as f32).min(1.0)
}

/// Drift all phonemes; returns L2 movement (capped by `max_drift_permille`).
pub fn drift_phonemes(
    rng: &mut impl Rng,
    inventory: &mut PhonemeInventory,
    rate: f32,
    max_drift_permille: u32,
) -> f32 {
    let rate = rate.clamp(0.0, 1.0);
    let cap = max_drift_permille as f32 / 1000.0;
    let before: Vec<[f32; PHONEME_FEATURES]> =
        inventory.phonemes.iter().map(|p| p.features).collect();

    for phoneme in &mut inventory.phonemes {
        for feature in &mut phoneme.features {
            let delta = (rng.gen::<f32>() - 0.5) * 2.0 * rate;
            *feature = (*feature + delta).clamp(0.0, 1.0);
        }
    }

    let mut l2 = 0.0f32;
    for (orig, phoneme) in before.iter().zip(&inventory.phonemes) {
        for i in 0..PHONEME_FEATURES {
            let d = orig[i] - phoneme.features[i];
            l2 += d * d;
        }
    }
    l2 = l2.sqrt();

    if l2 > cap && l2 > f32::EPSILON {
        let scale = cap / l2;
        for (phoneme, orig) in inventory.phonemes.iter_mut().zip(before.iter()) {
            for i in 0..PHONEME_FEATURES {
                phoneme.features[i] = orig[i] + (phoneme.features[i] - orig[i]) * scale;
            }
        }
        l2 = cap;
    }

    inventory.tick = inventory.tick.wrapping_add(1);
    l2
}

/// Render a coined lexeme as a display name from the evolved inventory.
#[must_use]
pub fn render_lexeme(lexeme: &Lexeme, inventory: &PhonemeInventory) -> String {
    const VOWELS: &[u8] = b"aeiou";
    const CONSONANTS: &[u8] = b"ktpsmnrlvz";
    let inv_len = inventory.phonemes.len().max(1);
    let mut out = String::new();
    for &idx in &lexeme.syllables {
        let p = &inventory.phonemes[idx as usize % inv_len];
        let c_idx = (p.features[0] * (CONSONANTS.len() - 1) as f32).round() as usize;
        let v_idx = (p.features[1] * (VOWELS.len() - 1) as f32).round() as usize;
        out.push(CONSONANTS[c_idx.min(CONSONANTS.len() - 1)] as char);
        out.push(VOWELS[v_idx.min(VOWELS.len() - 1)] as char);
    }
    if out.is_empty() {
        return out;
    }
    out.make_ascii_lowercase();
    let mut chars = out.chars();
    let first = chars.next().unwrap_or('?');
    first.to_uppercase().collect::<String>() + chars.as_str()
}

/// Resolve a display name from lexicon + inventory, if coined.
#[must_use]
pub fn name_from_lexicon(
    lexicon: &EvolvedLexicon,
    inventory: &PhonemeInventory,
    kind: LexemeKind,
    entity_id: u64,
) -> Option<String> {
    lexicon
        .get(kind, entity_id)
        .map(|lexeme| render_lexeme(lexeme, inventory))
}

/// Deterministic phoneme drift accumulated per `inventory_seed` and `tick` (FR-CIV-LANG-002).
///
/// Unlike `drift_phonemes` which takes an external RNG, this function derives its own RNG
/// from `inventory_seed ^ tick` so two calls with identical arguments are bitwise-equal.
/// Returns the drifted inventory without mutating the original.
#[must_use]
pub fn tick_seeded_phoneme_drift(
    base: &PhonemeInventory,
    inventory_seed: u64,
    tick: u64,
    rate: f32,
    max_drift_permille: u32,
) -> PhonemeInventory {
    let rng_seed = inventory_seed.wrapping_add(tick.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut rng = ChaCha8Rng::seed_from_u64(rng_seed);
    let mut out = base.clone();
    drift_phonemes(&mut rng, &mut out, rate, max_drift_permille);
    out.tick = tick;
    out
}

/// Coin a lexeme from a post-drift evolved inventory (FR-CIV-LANG-002 + FR-CIV-LANG-009).
///
/// Entity ids are deterministic under a fixed seed because `EvolvedLexicon::coin` is
/// idempotent: calling it twice for the same `(kind, entity_id)` returns the existing entry.
pub fn coin_evolved<'a>(
    lexicon: &'a mut EvolvedLexicon,
    inventory: &PhonemeInventory,
    kind: LexemeKind,
    entity_id: u64,
    entity_seed: u64,
) -> &'a Lexeme {
    let key = (kind, entity_id);
    if !lexicon.entries.contains_key(&key) {
        let mut rng = ChaCha8Rng::seed_from_u64(entity_seed ^ entity_id);
        lexicon.coin(&mut rng, inventory, kind, entity_id);
    }
    lexicon.entries.get(&key).expect("coined evolved lexeme")
}

fn phoneme_distance(a: &Phoneme, b: &Phoneme) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..PHONEME_FEATURES {
        let d = a.features[i] - b.features[i];
        sum += d * d;
    }
    (sum / PHONEME_FEATURES as f32).sqrt().min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn phoneme_drift_deterministic_per_seed() {
        let mut a = PhonemeInventory::seed_from(99);
        let mut b = PhonemeInventory::seed_from(99);
        let mut rng_a = rng(1);
        let mut rng_b = rng(1);
        for _ in 0..40 {
            drift_phonemes(&mut rng_a, &mut a, 0.05, 80);
            drift_phonemes(&mut rng_b, &mut b, 0.05, 80);
        }
        assert_eq!(a, b, "same seed must yield identical phoneme vectors at tick N");
    }

    #[test]
    fn phoneme_drift_rate_capped() {
        let mut inv = PhonemeInventory::seed_from(7);
        let mut rng = rng(3);
        let l2 = drift_phonemes(&mut rng, &mut inv, 1.0, 10);
        assert!(l2 <= 0.011, "per-tick L2 drift must respect max_drift_permille");
    }

    #[test]
    fn lexicon_grows_for_settlement_faction_event() {
        let inv = PhonemeInventory::seed_from(42);
        let mut lex = EvolvedLexicon::default();
        let mut rng = rng(5);
        lex.coin(&mut rng, &inv, LexemeKind::Settlement, 100);
        lex.coin(&mut rng, &inv, LexemeKind::Faction, 1);
        lex.coin(&mut rng, &inv, LexemeKind::Event, 500);
        assert_eq!(lex.len(), 3);
        let name = name_from_lexicon(&lex, &inv, LexemeKind::Settlement, 100).unwrap();
        assert!(!name.is_empty());
        assert!(name.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn coined_names_stable_under_fixed_seed() {
        let inv = PhonemeInventory::seed_from(11);
        let mut lex_a = EvolvedLexicon::default();
        let mut lex_b = EvolvedLexicon::default();
        let mut rng_a = rng(77);
        let mut rng_b = rng(77);
        lex_a.coin(&mut rng_a, &inv, LexemeKind::Faction, 2);
        lex_b.coin(&mut rng_b, &inv, LexemeKind::Faction, 2);
        let na = name_from_lexicon(&lex_a, &inv, LexemeKind::Faction, 2).unwrap();
        let nb = name_from_lexicon(&lex_b, &inv, LexemeKind::Faction, 2).unwrap();
        assert_eq!(na, nb);
    }

    #[test]
    fn diverged_inventories_increase_distance() {
        let mut a = PhonemeInventory::seed_from(1);
        let mut b = PhonemeInventory::seed_from(2);
        let mut rng = rng(9);
        for _ in 0..30 {
            drift_phonemes(&mut rng, &mut a, 0.08, 80);
            drift_phonemes(&mut rng, &mut b, 0.08, 80);
        }
        assert!(
            phoneme_inventory_distance(&a, &b) > 0.05,
            "isolated drift must diverge inventories"
        );
    }

    // --- Sub-feature 1: deterministic phoneme inventory drift per seed+tick ---

    #[test]
    fn tick_seeded_phoneme_drift_bitwise_equal_across_runs() {
        let base = PhonemeInventory::seed_from(42);
        let a = tick_seeded_phoneme_drift(&base, 42, 100, 0.05, 50);
        let b = tick_seeded_phoneme_drift(&base, 42, 100, 0.05, 50);
        assert_eq!(a, b, "same seed+tick must yield bitwise-equal phoneme vectors");
    }

    #[test]
    fn tick_seeded_phoneme_drift_differs_across_ticks() {
        let base = PhonemeInventory::seed_from(7);
        let t10 = tick_seeded_phoneme_drift(&base, 7, 10, 0.1, 100);
        let t20 = tick_seeded_phoneme_drift(&base, 7, 20, 0.1, 100);
        assert_ne!(t10.phonemes, t20.phonemes, "different ticks must produce different drift");
    }

    #[test]
    fn tick_seeded_phoneme_drift_accumulates_over_ticks() {
        let base = PhonemeInventory::seed_from(13);
        // Drift 50 ticks forward; distance should be > 0
        let t0_dist = phoneme_inventory_distance(&base, &base);
        let t50 = tick_seeded_phoneme_drift(&base, 13, 50, 0.08, 80);
        let dist_after = phoneme_inventory_distance(&base, &t50);
        assert!(t0_dist == 0.0, "distance from self must be zero");
        assert!(dist_after > 0.0, "drift must accumulate and move phonemes");
    }

    // --- Sub-feature 2: lexicon growth with evolved phonemes ---

    #[test]
    fn lexicon_grows_with_evolved_phoneme_set() {
        let base = PhonemeInventory::seed_from(55);
        // Evolve the inventory forward 30 ticks
        let evolved = tick_seeded_phoneme_drift(&base, 55, 30, 0.07, 70);
        let mut lex = EvolvedLexicon::default();
        coin_evolved(&mut lex, &evolved, LexemeKind::Settlement, 1, 1001);
        coin_evolved(&mut lex, &evolved, LexemeKind::Faction, 2, 2002);
        coin_evolved(&mut lex, &evolved, LexemeKind::Event, 3, 3003);
        assert_eq!(lex.len(), 3, "lexicon must grow to 3 entries after coining 3 concepts");
        let name = name_from_lexicon(&lex, &evolved, LexemeKind::Settlement, 1).unwrap();
        assert!(!name.is_empty(), "coined name must be non-empty");
        assert!(name.chars().next().unwrap().is_uppercase(), "name must start uppercase");
    }

    #[test]
    fn lexicon_coin_evolved_idempotent_for_same_entity() {
        let inv = PhonemeInventory::seed_from(88);
        let mut lex = EvolvedLexicon::default();
        coin_evolved(&mut lex, &inv, LexemeKind::Faction, 10, 42);
        coin_evolved(&mut lex, &inv, LexemeKind::Faction, 10, 42);
        assert_eq!(lex.len(), 1, "coining same entity twice must not duplicate");
    }

    #[test]
    fn lexicon_names_stable_after_evolved_inventory_drift() {
        let base = PhonemeInventory::seed_from(33);
        let evolved = tick_seeded_phoneme_drift(&base, 33, 15, 0.06, 60);
        let mut lex_a = EvolvedLexicon::default();
        let mut lex_b = EvolvedLexicon::default();
        // Same entity_seed → same name
        coin_evolved(&mut lex_a, &evolved, LexemeKind::Event, 500, 9999);
        coin_evolved(&mut lex_b, &evolved, LexemeKind::Event, 500, 9999);
        let na = name_from_lexicon(&lex_a, &evolved, LexemeKind::Event, 500).unwrap();
        let nb = name_from_lexicon(&lex_b, &evolved, LexemeKind::Event, 500).unwrap();
        assert_eq!(na, nb, "same evolved inventory + same entity_seed must yield same name");
    }
}
