//! Cultural, religion & language phases extracted from engine.rs (Pass 9).

use crate::culture::advance_faction_ideologies;
use crate::language::{
    borrow_word, ensure_seeded_word, faction_isolation_pressure, person_name_meaning,
    place_name_meaning, seeded_language_state, tick_language_for_lineage,
};
use crate::religion::{
    apply_big_gods_response, substrate_gradients_for, ReligionEvent, ReligiousProfile,
    SubstrateGradients, MAX_MISERY_UNREST,
};
use crate::settlement_helpers::{
    accumulate_profile_diffusion, diplomacy_faction_pairs_from_settlement_contact,
    fabric_tier_signal, faction_language_centroids, faction_religion_signals,
    settlement_actor_hardship_signal, settlement_actors_by_settlement, settlement_contact_pairs,
    settlement_dominant_factions, settlement_kinship_density_signal, settlement_member_counts,
    settlement_religion_spread_edges, settlement_trade_contact_signal,
    SETTLEMENT_CONTACT_RADIUS_FP,
};
use crate::Simulation;
use civ_genetics::sentience::{evaluate_sentience, SentienceEvent};
use civ_genetics::Dna;
use std::collections::{BTreeMap, BTreeSet};

impl Simulation {
    pub(crate) fn phase_belief(&mut self) {
        let tick = self.current_tick;
        self.last_tick_religion_events.clear();
        let settlement_ids: Vec<u32> = self.settlements.keys().copied().collect();
        for sid in settlement_ids {
            let gradients = self.religion_gradients_for_settlement(sid);
            let population = self.settlements.get(&sid).copied().unwrap_or(0);
            let profile = self
                .religious_profiles
                .entry(sid)
                .or_insert_with(|| ReligiousProfile::new(population, tick));
            profile.population = population;
            apply_big_gods_response(profile, &gradients, tick);

            let event = ReligionEvent::tick(
                sid,
                profile.monitoring,
                profile.mythic_coherence,
                profile.uncertainty_reduction,
                tick,
            );
            if event.is_notable() {
                self.last_tick_religion_events.push(event);
            }
        }
        self.spread_religion_between_settlements();
    }

    pub(crate) fn religion_gradients_for_settlement(
        &self,
        settlement_id: u32,
    ) -> SubstrateGradients {
        let base = substrate_gradients_for(settlement_id);
        let population = self.settlements.get(&settlement_id).copied().unwrap_or(0);
        let food = self
            .settlement_food_stocked
            .get(&settlement_id)
            .copied()
            .unwrap_or(0)
            .max(0);
        let food_pressure = if population == 0 {
            0.0
        } else {
            (1.0 - (food as f32 / population.max(1) as f32)).clamp(0.0, 1.0)
        };
        let hardship = settlement_actor_hardship_signal(
            settlement_id,
            &settlement_actors_by_settlement(&self.actor_settlement),
            &self.actor_hardship,
        );
        let kinship = settlement_kinship_density_signal(
            settlement_id,
            &settlement_actors_by_settlement(&self.actor_settlement),
            &self.kinship,
        );
        let unrest = self
            .last_tick_unrest_snapshots
            .get(&settlement_id)
            .map(|snapshot| (snapshot.score.max(0) as f32 / 500.0) * MAX_MISERY_UNREST)
            .unwrap_or_else(|| {
                let gini = (self
                    .unrest_settlement_gini
                    .get(&settlement_id)
                    .copied()
                    .unwrap_or(0.0)
                    .clamp(0.0, 1.0) as f32);
                gini * MAX_MISERY_UNREST
            });
        let cohesion = self
            .last_tick_cohesion
            .get(&settlement_id)
            .map(|snapshot| fabric_tier_signal(snapshot.fabric))
            .unwrap_or(0.5);
        let trade_contact =
            settlement_trade_contact_signal(settlement_id, &self.last_tick_settlement_trade_flows);

        SubstrateGradients {
            grad_T: base.grad_T.max(hardship),
            grad_M: base.grad_M.max(food_pressure),
            grad_B: base.grad_B.max(food_pressure.max(hardship * 0.5)),
            kinship_density: base.kinship_density.min(kinship),
            unrest: base.unrest.max(unrest.clamp(0.0, MAX_MISERY_UNREST)),
            migration_rate: base
                .migration_rate
                .max((1.0_f32 - cohesion).clamp(0.0, 1.0)),
            language_distance: base.language_distance.max((1.0 - trade_contact) * 0.25),
        }
    }

    pub(crate) fn spread_religion_between_settlements(&mut self) {
        if self.religious_profiles.len() < 2 {
            return;
        }

        let mut edges = settlement_religion_spread_edges(&self.last_tick_settlement_trade_flows);
        let member_counts: BTreeMap<u64, u32> = self
            .settlements
            .iter()
            .map(|(&settlement_id, &population)| (u64::from(settlement_id), population))
            .collect();
        for (left, right) in
            settlement_contact_pairs(&self.world, &member_counts, SETTLEMENT_CONTACT_RADIUS_FP)
        {
            edges.insert((left as u32, right as u32), 0.35);
        }

        let before = self.religious_profiles.clone();
        let mut deltas: BTreeMap<u32, (f32, f32, f32)> = BTreeMap::new();
        for ((a, b), strength) in edges {
            let a32 = u32::try_from(a).unwrap_or(0);
            let b32 = u32::try_from(b).unwrap_or(0);
            let (Some(pa), Some(pb)) = (before.get(&a32), before.get(&b32)) else {
                continue;
            };
            let cohesion_a = self
                .last_tick_cohesion
                .get(&a32)
                .map(|s| fabric_tier_signal(s.fabric))
                .unwrap_or(0.5);
            let cohesion_b = self
                .last_tick_cohesion
                .get(&b32)
                .map(|s| fabric_tier_signal(s.fabric))
                .unwrap_or(0.5);
            let spread = (0.015 * strength * ((cohesion_a + cohesion_b) * 0.5)).clamp(0.0, 0.02);
            accumulate_profile_diffusion(pa, pb, spread, &mut deltas, a32, b32);
        }

        let mut after = before;
        for (settlement_id, (dm, dc, du)) in deltas {
            let Some(profile) = after.get_mut(&settlement_id) else {
                continue;
            };
            profile.monitoring = (profile.monitoring + dm).clamp(0.0, 1.0);
            profile.mythic_coherence = (profile.mythic_coherence + dc).clamp(0.0, 1.0);
            profile.uncertainty_reduction = (profile.uncertainty_reduction + du).clamp(0.0, 1.0);
        }
        self.religious_profiles = after;
    }

    /// Culture phase (FR-CIV-CULTURE) — advance per-faction ideology/trait
    /// drift from fresh settlement culture, contact, religion, era, and climate
    /// signals.
    pub(crate) fn phase_culture(&mut self) {
        let cluster_member_counts = settlement_member_counts(&self.world);
        let dominant = settlement_dominant_factions(&self.world, &cluster_member_counts);
        if dominant.is_empty() || self.cluster_cultures.is_empty() {
            return;
        }

        let contacts = settlement_contact_pairs(
            &self.world,
            &cluster_member_counts,
            SETTLEMENT_CONTACT_RADIUS_FP,
        );
        let religion_by_faction =
            faction_religion_signals(&self.religious_profiles, &dominant, &cluster_member_counts);
        let faction_ages = self.era_progression.faction_ages.clone();
        let prior = self.faction_ideologies.clone();

        self.faction_ideologies = advance_faction_ideologies(
            self.state.tick,
            &self.cluster_cultures,
            &dominant,
            &cluster_member_counts,
            &contacts,
            &self.climate,
            &religion_by_faction,
            &faction_ages,
            &prior,
            &mut self.rng,
        );

        // --- Legend significance accumulator wiring (#962) ---
        // Legend significance wiring TODO: wire when civ_legends API stabilizes
        // See crates/legends/src/significance.rs for SignificanceAccumulator API
        // The drift_magnitude calculation above is ready; just needs
        // civ_legends::significance::AccumulatorConfig and record_event() call.
    }

    /// Language phase (FR-CIV-LANG-001 / FR-LANGUAGE-001) — per-faction language
    /// emerges from current cluster culture vectors (`cluster_cultures`) and drifts
    /// under isolation pressure.
    ///
    /// Emergence flow is:
    /// - seed / refresh each active faction's `LanguageState` from dominant
    ///   settlement culture language centroids;
    /// - apply isolation-aware drift (isolated groups drift faster, producing
    ///   greater divergence);
    /// - borrow naming words for contact-connected factions so contact zones
    ///   reduce divergence.
    pub(crate) fn phase_language(&mut self) {
        let cluster_member_counts = settlement_member_counts(&self.world);
        let dominant = settlement_dominant_factions(&self.world, &cluster_member_counts);
        let centroids =
            faction_language_centroids(self.cluster_cultures(), &dominant, &cluster_member_counts);
        let contacts = settlement_contact_pairs(
            &self.world,
            &cluster_member_counts,
            SETTLEMENT_CONTACT_RADIUS_FP,
        );
        let faction_pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);

        let mut active_factions: BTreeSet<u32> = self.faction_languages.keys().copied().collect();
        active_factions.extend(centroids.keys().copied());

        for faction_id in active_factions {
            let centroid_seed = centroids.get(&faction_id).copied();
            let lang = self
                .faction_languages
                .entry(faction_id)
                .or_insert_with(|| seeded_language_state(centroid_seed.unwrap_or([0.5; 4])));
            lang.drift_rate = 0.05;
            lang.split_threshold = 0.35;
            if let Some(signature) = centroid_seed {
                ensure_seeded_word(lang, place_name_meaning(faction_id, 0), signature);
                ensure_seeded_word(lang, person_name_meaning(faction_id, 0), signature);
            }
            let isolation = centroid_seed
                .map(|_| {
                    faction_isolation_pressure(
                        faction_id,
                        &dominant,
                        &cluster_member_counts,
                        &contacts,
                    )
                })
                .unwrap_or(1.0);
            tick_language_for_lineage(lang, isolation, u64::from(faction_id));
        }

        for (left, right) in faction_pairs {
            if left == right {
                continue;
            }

            let left_seed = centroids.get(&left).copied().unwrap_or([0.5; 4]);
            let right_seed = centroids.get(&right).copied().unwrap_or([0.5; 4]);

            let mut left_lang = self
                .faction_languages
                .get(&left)
                .cloned()
                .unwrap_or_else(|| seeded_language_state(left_seed));
            let mut right_lang = self
                .faction_languages
                .get(&right)
                .cloned()
                .unwrap_or_else(|| seeded_language_state(right_seed));

            let cross_left_place = place_name_meaning(left, right);
            let cross_right_place = place_name_meaning(right, left);
            let cross_left_person = person_name_meaning(left, right);
            let cross_right_person = person_name_meaning(right, left);

            ensure_seeded_word(&mut left_lang, cross_left_place, left_seed);
            ensure_seeded_word(&mut left_lang, cross_left_person, left_seed);
            ensure_seeded_word(&mut right_lang, cross_right_place, right_seed);
            ensure_seeded_word(&mut right_lang, cross_right_person, right_seed);

            borrow_word(&mut left_lang, &right_lang, cross_left_place);
            borrow_word(&mut left_lang, &right_lang, cross_left_person);
            borrow_word(&mut right_lang, &left_lang, cross_right_place);
            borrow_word(&mut right_lang, &left_lang, cross_right_person);

            let _ = self.faction_languages.insert(left, left_lang);
            let _ = self.faction_languages.insert(right, right_lang);
        }

        let first_faction = self.faction_languages.values().next();
        self.language_state = first_faction.cloned().unwrap_or_default();
    }

    /// Sentience phase (FR-CIV-GENETICS / FR-CIV-LEGENDS) — evaluates every
    /// DNA-bearing agent against [`Self::sentience_threshold`]. Crossings
    /// produce `SentienceEvent` records that downstream emergence coupling
    /// (cohesion pulse, awakening→cohesion nudge, etc.) consumes.
    ///
    /// Runs AFTER [`Self::phase_emergence`] and [`Self::phase_language`] so
    /// the dependent couplings observe the post-emergence agent state; runs
    /// BEFORE [`Self::phase_diffusion`] so diffusion does not re-mutate the
    /// just-evaluated agent set this tick. The set of crossings is captured
    /// on `self.last_tick_sentience_events` for downstream phase + tests.
    pub(crate) fn phase_sentience(&mut self) {
        let mut events = Vec::new();
        for (_, dna) in self.world.query::<&Dna>().iter() {
            events.push(evaluate_sentience(
                None,
                dna,
                &self.sentience_profile,
                self.sentience_threshold,
            ));
        }
        self.last_tick_sentience_events = events;
    }
}
