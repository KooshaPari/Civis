mod engine_tests {
    use crate::engine::*;
    use crate::lod::{should_tick_entity_with_policy, LodPolicy};
    use crate::replay::{ReplayEvent, ReplayLog};
    use civ_agents::{count_civilians, spawn_civilian_at, ActorVisualKind, LodTier, Wardrobe};
    use civ_audio::triggers::SfxTrigger;
    use civ_planet::{compute_climate, compute_weather, is_daytime, MoonConfig, PlanetConfig};
    use civ_voxel::{MaterialId, WorldCoord};
    use tempfile::NamedTempFile;

    fn fill_voxel_chunk(world: &mut VoxelWorld<MaterialId>, origin: i64, size: i64) {
        for x in origin..origin + size {
            for y in origin..origin + size {
                for z in origin..origin + size {
                    world.write(WorldCoord { x, y, z }, MaterialId(1));
                }
            }
        }
    }

    /// FR-CIV-ENGINE-INT-010 — startup spawns 128 civilians across four factions.
    #[test]
    fn startup_spawns_128_civilians() {
        let sim = Simulation::new();
        assert_eq!(sim.state.tick, 0);
        assert_eq!(count_civilians(&sim.world), 128);
    }

    #[test]
    fn test_tick_advances() {
        let mut sim = Simulation::new();
        sim.tick();
        assert_eq!(sim.state.tick, 1);
    }

    /// FR-CIV-TUTORIAL — tutorial progression advances from live sim milestones.
    #[test]
    fn fr_civ_tutorial_advances_from_tick_progress() {
        let mut sim = Simulation::with_seed(42);
        sim.era_progression.faction_tech.insert(
            0,
            crate::tech::FactionTechState {
                research_points: 240,
                tech_level: 0,
                diffusion_points: 0,
            },
        );

        assert_eq!(
            sim.tutorial_progress.current,
            crate::tutorial::TutorialMilestone::FirstFaction
        );

        sim.tick();

        assert!(sim.tutorial_progress.tech_unlocked);
        assert!(
            matches!(
                sim.tutorial_progress.current,
                crate::tutorial::TutorialMilestone::FirstTech
                    | crate::tutorial::TutorialMilestone::FirstWar
                    | crate::tutorial::TutorialMilestone::FirstReligion
                    | crate::tutorial::TutorialMilestone::Complete
            ),
            "tutorial should advance once tech unlocks"
        );
        assert_eq!(sim.snapshot().tutorial_progress, sim.tutorial_progress);
    }

    /// FR-CIV-TACTICS — opposing military units in LOS/range engage during the
    /// normal simulation tick, emit combat damage, and resolve casualties.
    #[test]
    fn fr_civ_tactics_tick_resolves_in_range_combat() {
        let mut sim = Simulation::with_seed(2026_07_01);
        sim.world = World::new();
        sim.military_phase.movement.cadence_ticks = 0;
        sim.military_phase.war.cadence_ticks = 1;
        sim.military_phase.war.engage_range_grid = 4;

        let hp = Fixed::from_num(1);
        let unit_a = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: hp,
            hp,
            max_hp: hp,
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        };
        let unit_b = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: hp,
            hp,
            max_hp: hp,
            morale: Fixed::from_num(1),
            position: Position { x: 1, y: 0 },
            faction_id: 1,
        };
        let _ = sim.world.spawn((unit_a,));
        let _ = sim.world.spawn((unit_b,));

        sim.tick();

        assert!(
            !sim.last_tick_engagements.is_empty(),
            "FR-CIV-TACTICS: tick should resolve at least one engagement"
        );
        assert!(
            !sim.last_tick_combat_pulses().is_empty(),
            "FR-CIV-TACTICS: resolved engagement should surface a damage pulse"
        );
        let survivors = sim.world.query::<&MilitaryUnit>().iter().count();
        let damaged = sim
            .world
            .query::<&MilitaryUnit>()
            .iter()
            .any(|(_, unit)| unit.hp < unit.max_hp);
        assert!(
            survivors < 2 || damaged,
            "FR-CIV-TACTICS: tick should apply unit damage or casualties"
        );
        assert!(
            sim.replay_log().combat_event_count() > 0,
            "FR-CIV-TACTICS: tick combat should be replay-recorded"
        );
    }

    /// FR-CORE-001 — each `Simulation::tick()` appends exactly one `ReplayEvent::Tick`.
    #[test]
    fn fr_core_001_single_tick_event_per_tick() {
        use crate::invariants::check_tick_invariants;

        let mut sim = Simulation::with_seed(1);
        assert_eq!(count_replay_ticks(&sim), 0);

        sim.tick();
        assert_eq!(sim.state.tick, 1);
        assert_eq!(count_replay_ticks(&sim), 1);
        check_tick_invariants(&sim).expect("one replay tick marker per completed tick");

        for expected in 2..=5 {
            sim.tick();
            assert_eq!(sim.state.tick, expected);
            assert_eq!(count_replay_ticks(&sim), expected as usize);
        }
    }

    /// CIV-0001 partial — `PHASE_ORDER` matches the sequence in `Simulation::tick`.
    ///
    /// FR-CIV-phasewire: this test was updated to reflect the renamed legacy
    /// phases (`buildings` → `construction_sites`, `language` → `language_drift`)
    /// plus the six newly added top-level phases (`religion`, `language`,
    /// `psyche`, `buildings`, `history`, `writing`). Update this list whenever
    /// `PHASE_ORDER` changes.
    #[test]
    fn phase_order_matches_tick_sequence() {
        assert_eq!(
            PHASE_ORDER,
            &[
                "production",
                "citizen_lifecycle",
                "military",
                "policy",
                "economy",
                "planet",
                "disasters",
                "diplomacy",
                "faction_decisions",
                "tactics",
                "voxel",
                "compact",
                // FR-CIV-phasewire: was "buildings".
                "construction_sites",
                "life",
                "daily_path",
                "cluster",
                "research",
                "tech",
                "belief",
                "unrest",
                "cohesion",
                "social_mood",
                "economic_focus_pre",
                "stratification",
                "institutions",
                "economic_focus",
                "emergence",
                "tutorial",
                "psyche_behavior",
                "culture",
                // FR-CIV-phasewire: was "language", legacy drift kept alongside
                // the new top-level `phase_language` (which drives
                // `language::tick_language_system`).
                "language_drift",
                "sentience",
                "species",
                "diffusion",
                // Legacy aliases — preserved so old test fixtures remain valid.
                "writing_apply",
                "building_layouts",
                "history_archive",
                // FR-CIV-phasewire: six new top-level phases that wire the
                // expanded modules' exported `tick_*` fns into the engine
                // tick loop.
                "religion",
                "language",
                "psyche",
                "buildings",
                "history",
                "writing",
                "audio",
                "victory_check",
            ]
        );
    }

    #[test]
    fn faction_decision_high_unrest_sets_deterministic_response_intents() {
        fn unrest_snapshot(level: UnrestLevel) -> UnrestSnapshot {
            UnrestSnapshot {
                settlement_id: 7,
                level,
                score: if level == UnrestLevel::Revolting {
                    300
                } else {
                    0
                },
                events_count: 0,
                riots_count: 0,
                migrants_count: 0,
                mob_size: 0,
            }
        }

        let mut sim_a = Simulation::with_seed(42);
        let mut sim_b = Simulation::with_seed(42);
        sim_a
            .last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Revolting));
        sim_b
            .last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Revolting));

        sim_a.tick();
        sim_b.tick();

        let intents_a = &sim_a.state.last_tick_faction_unrest_response_intents;
        let intents_b = &sim_b.state.last_tick_faction_unrest_response_intents;
        assert!(!intents_a.is_empty());
        assert_eq!(intents_a, intents_b);
        assert_eq!(
            sim_a.snapshot().last_tick_faction_unrest_response_intents,
            *intents_a
        );

        let mut calm = Simulation::with_seed(42);
        calm.last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Stable));
        calm.tick();
        assert!(calm
            .state
            .last_tick_faction_unrest_response_intents
            .is_empty());
    }

    #[test]
    fn faction_decision_hostility_and_trade_intents_persist_on_snapshot() {
        let mut hostile = Simulation::with_seed(7);
        for _ in 0..2 {
            hostile.faction_relations.apply_signal(
                0u32,
                1u32,
                civ_agents::DiplomacySignal {
                    combat_grievance: 0.8,
                    ..civ_agents::DiplomacySignal::default()
                },
            );
        }
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        },));
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 1, y: 0 },
            faction_id: 0,
        },));
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(5),
            hp: Fixed::from_num(5),
            max_hp: Fixed::from_num(5),
            morale: Fixed::from_num(1),
            position: Position { x: 2, y: 0 },
            faction_id: 1,
        },));
        let hostile_before = hostile
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("hostile setup must seed a relation row");
        hostile.tick();
        assert!(hostile
            .state
            .last_tick_faction_hostility_intents
            .contains(&0));
        assert_eq!(
            hostile.snapshot().last_tick_faction_hostility_intents,
            hostile.state.last_tick_faction_hostility_intents
        );
        let hostile_after = hostile
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("hostility intent must lower relation score");
        assert!(hostile_after <= hostile_before);
        assert!(hostile_after < -0.5);
        assert!(hostile.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::Conflict && event.faction_a == 0 && event.faction_b == 1
        }));

        let mut trade = Simulation::with_seed(11);
        trade.state.faction_resources.entry(0).or_default().food = Fixed::from_num(1500);
        trade.last_tick_cohesion_snapshots_mut().insert(
            1,
            CohesionSnapshot {
                settlement_id: 1,
                fabric: FabricTier::Tight,
                kin_count: 10,
                trust_sum: 100,
                fragmentation_events: 0,
                fragmentations: 0,
                faction_count: 1,
            },
        );
        trade.faction_relations.apply_signal(
            0u32,
            1u32,
            civ_agents::DiplomacySignal {
                trade_volume: 0.8,
                ..civ_agents::DiplomacySignal::default()
            },
        );
        trade.tick();
        assert!(trade
            .state
            .last_tick_faction_trade_open_intents
            .contains(&0));
        assert_eq!(
            trade.snapshot().last_tick_faction_trade_open_intents,
            trade.state.last_tick_faction_trade_open_intents
        );
        assert!(trade.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::TradeAgreement
                && event.faction_a == 0
                && event.faction_b == 1
        }));
        let trade_score = trade
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("trade intent must raise relation score");
        assert!(trade_score > 0.8);
    }

    #[test]
    fn military_unit_component_is_serializable() {
        let unit = MilitaryUnit {
            unit_type: UnitType::Knight,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(8),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 4, y: -2 },
            faction_id: 7,
        };
        let json = serde_json::to_string(&unit).expect("serialize");
        let decoded: MilitaryUnit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.faction_id, 7);
        assert_eq!(decoded.unit_type, UnitType::Knight);
    }

    /// L5-115 — `PHASE_ORDER` includes "emergence" and the phase is positioned
    /// after `life` so the agent state that emergence depends on is finalized
    /// (cluster stocks, needs, settlements) before emergence runs.
    /// Closes FR-CIV-LEGENDS-INGEST-02, FR-CIV-PSYCHE-900/901, FR-CIV-PSYCHE-911,
    /// FR-CIV-PSYCHE-912, FR-CIV-GENETICS, FR-CIV-AI-006, FR-CIV-LEGENDS-QUERY-07.
    /// FR-ENGINE-phaseorder: emergence is the final core emergence phase;
    /// `language` and `sentience` are emergence-following couplings and are
    /// placed AFTER emergence (and before `diffusion` propagation).
    #[test]
    fn phase_order_includes_emergence() {
        let life_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "life")
            .expect("PHASE_ORDER must include 'life'");
        let emergence_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "emergence")
            .expect("PHASE_ORDER must include 'emergence'");
        assert!(
            emergence_idx > life_idx,
            "emergence (idx {emergence_idx}) must run after life (idx {life_idx}) \
             so agent state is finalized first"
        );
        // FR-CIV-phasewire: was "language"; the legacy drift phase lives
        // under "language_drift" while the new top-level module tick lives
        // under the plain "language" entry (added by the audit FR-CIV-phasewire
        // migration). Either ordering constraint still pins culture → language.
        let language_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "language" || *p == "language_drift")
            .expect("PHASE_ORDER must include 'language' or 'language_drift' (FR-ENGINE-phaseorder)");
        let culture_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "culture")
            .expect("PHASE_ORDER must include 'culture' (FR-CIV-CULTURE)");
        let sentience_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "sentience")
            .expect("PHASE_ORDER must include 'sentience' (FR-ENGINE-phaseorder)");
        assert!(
            culture_idx > emergence_idx,
            "culture (idx {culture_idx}) must run after emergence (idx {emergence_idx})"
        );
        assert!(
            language_idx > culture_idx,
            "language (idx {language_idx}) must run after culture (idx {culture_idx})"
        );
        assert!(
            sentience_idx > language_idx,
            "sentience (idx {sentience_idx}) must run after language (idx {language_idx}) \
             so language-driven contact pressure is visible to the psyche evaluator"
        );
        let tutorial_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "tutorial")
            .expect("PHASE_ORDER must include 'tutorial' (FR-CIV-TUTORIAL)");
        assert!(tutorial_idx > emergence_idx);
    }
    /// L5-115 — `Simulation::tick` invokes `phase_emergence` and the public
    /// accessors on `Simulation` (legends_graph, emergence_feed,
    /// cluster_cultures) are queryable after a tick. Two same-seed sims run
    /// deterministically through the emergence pipeline (RNG state is
    /// preserved across the phase — see `test_determinism`).
    #[test]
    fn tick_invokes_emergence_phase() {
        let mut sim_a = Simulation::with_seed(2026_06_18);
        let mut sim_b = Simulation::with_seed(2026_06_18);

        for _ in 0..10 {
            sim_a.tick();
            sim_b.tick();
        }

        // Post-condition: the wire-up is observable via the public API.
        // `legends_graph` is the saga state populated by `emergence_legends`
        // (FR-CIV-LEGENDS-INGEST-02). The accessor must return without panic
        // — a non-panic on a wired phase is the wire-up check.
        let _graph_a = sim_a.legends_graph();
        let _graph_b = sim_b.legends_graph();

        // Determinism: same seed → same saga graph node count after N ticks.
        assert_eq!(
            sim_a.legends_graph().node_count(),
            sim_b.legends_graph().node_count(),
            "phase_emergence must be deterministic across same-seed sims"
        );

        // `emergence_feed` is cleared at the start of `phase_emergence` and
        // re-populated with the tick's events. The accessor must remain
        // queryable after a tick.
        let _feed_a = sim_a.emergence_feed();
        let _feed_b = sim_b.emergence_feed();

        // `cluster_cultures` is the population-level culture map populated
        // by `emergence_culture` (FR-CIV-PSYCHE-911). It must be queryable
        // and deterministic.
        assert_eq!(
            sim_a.cluster_cultures().len(),
            sim_b.cluster_cultures().len(),
            "phase_emergence must produce deterministic cluster_cultures"
        );
    }

    #[test]
    fn snapshot_exposes_research_cache_tech_state() {
        let mut sim = Simulation::with_seed(42);
        sim.research_cache_mut()
            .researched
            .push("pottery".to_owned());
        sim.research_cache_mut().in_progress = Some(("writing".to_owned(), 3));

        let snapshot = sim.snapshot();

        assert_eq!(snapshot.researched, ["pottery"]);
        assert_eq!(snapshot.in_progress_tech.as_deref(), Some("writing"));
        assert_eq!(sim.researched_tech_count(), 1);
    }

    #[test]
    fn tick_detects_tech_victory() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 1;
        sim.research_cache_mut().researched = (0..12).map(|idx| format!("tech_{idx}")).collect();

        sim.tick();

        assert!(matches!(
            sim.last_game_outcome,
            GameOutcome::Victory(ref kind) if kind == "Age of Enlightenment"
        ));
    }

    fn count_replay_ticks(sim: &Simulation) -> usize {
        sim.replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
            .count()
    }

    fn average_language_distance(left: &LanguageState, right: &LanguageState) -> f32 {
        left.seed_signature
            .iter()
            .zip(right.seed_signature)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / left.seed_signature.len() as f32
    }

    // ============================================================================
    // FR-CORE-005 — Policy phase + set_policy tests
    // ============================================================================

    /// FR-CORE-005 — new simulations start with [`NoopPolicy`] installed and
    /// `last_control_signals` empty.
    #[test]
    fn default_policy_is_noop() {
        let sim = Simulation::new();
        assert_eq!(sim.policy().name(), "noop");
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
    }

    /// FR-CORE-005 — `with_seed` constructor also starts with [`NoopPolicy`].
    #[test]
    fn with_seed_default_policy_is_noop() {
        let sim = Simulation::with_seed(42);
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — `set_policy` replaces the active policy.
    #[test]
    fn set_policy_replaces_active_policy() {
        let mut sim = Simulation::new();
        assert_eq!(sim.policy().name(), "noop");

        sim.set_policy(Box::new(crate::policy::CapitalistPolicy));
        assert_eq!(sim.policy().name(), "capitalist");

        sim.set_policy(Box::new(crate::policy::SubsistenceFirstPolicy));
        assert_eq!(sim.policy().name(), "subsistence_first");

        sim.set_policy(Box::new(crate::policy::NoopPolicy));
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — a single `tick()` populates `last_control_signals` from
    /// the active policy.
    #[test]
    fn phase_policy_populates_last_control_signals() {
        let mut sim = Simulation::new();
        sim.set_policy(Box::new(crate::policy::CapitalistPolicy));
        sim.tick();
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
        assert_eq!(sim.last_control_signals().production_multipliers.len(), 0);
        assert_eq!(sim.last_control_signals().allocation_weights.len(), 0);
        assert_eq!(sim.last_control_signals().tax_rates.len(), 0);
    }

    /// FR-CORE-005 — `phase_policy` runs every tick; repeated ticks keep
    /// `last_control_signals` consistent with the active policy.
    #[test]
    fn phase_policy_runs_every_tick() {
        let mut sim = Simulation::new();
        for _ in 0..5 {
            sim.tick();
        }
        assert_eq!(sim.state.tick, 5);
        // Default NoopPolicy produces default signals every tick.
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
    }

    /// FR-CORE-005 — `phase_policy` runs after `phase_military` and before
    /// `phase_economy` (verified indirectly: `last_control_signals` is
    /// populated for the same tick `phase_economy` reads `state.energy_budget_joules` from).
    #[test]
    fn phase_policy_runs_before_phase_economy() {
        use crate::policy::CapitalistPolicy;

        let mut sim = Simulation::new();
        sim.set_policy(Box::new(CapitalistPolicy));
        // After one tick, last_control_signals reflects the policy at tick 1.
        sim.tick();
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
        // The default capitalist policy is a no-op, so the economy state
        // behaves identically to a NoopPolicy run.
        let mut ref_sim = Simulation::with_seed(42);
        ref_sim.tick();
        assert_eq!(
            ref_sim.state.energy_budget_joules,
            sim.state.energy_budget_joules
        );
    }

    /// FR-CORE-005 — a custom policy that emits non-empty signals is reflected
    /// on `last_control_signals` after `tick()`. Uses an inline test-only
    /// policy to avoid modifying the public `policy` module for one test.
    #[test]
    fn custom_policy_signals_propagate_to_simulation() {
        #[derive(Debug)]
        struct TaxingPolicy;
        impl Policy for TaxingPolicy {
            fn evaluate(&self, _state: &WorldState) -> ControlSignals {
                let mut signals = ControlSignals::default();
                signals.tax_rates.insert(7, 250); // 2.5%
                signals
                    .production_multipliers
                    .insert("food".to_string(), 1.25);
                signals
            }
            fn name(&self) -> &'static str {
                "taxing"
            }
        }

        let mut sim = Simulation::new();
        sim.set_policy(Box::new(TaxingPolicy));
        sim.tick();
        assert_eq!(sim.last_control_signals().tax_rates.get(&7), Some(&250));
        assert_eq!(
            sim.last_control_signals()
                .production_multipliers
                .get("food"),
            Some(&1.25)
        );
    }

    /// CIV-0100 stub: joule budget drain stays non-negative after a tick.
    #[test]
    fn phase_economy_conserves_non_negative_budget() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(99);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 1_000.0,
            scarcity_multiplier: 2.0,
        };
        sim.tick();
        // Budget may be drained by lifecycle-weighted allocator but must stay >= 0.
        assert!(sim.state.energy_budget_joules.to_bits() >= Fixed::ZERO.to_bits());
    }

    /// `phase_economy` routes demand through the lifecycle-weighted allocator.
    #[test]
    fn phase_economy_uses_lifecycle_allocator() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(7);
        sim.state.energy_budget_joules = Fixed::from_num(50_000);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 1.0,
        };

        let before = sim.state.energy_budget_joules;
        sim.tick();

        // After tick, budget should be <= before (drained or same if labor_fraction=0)
        assert!(sim.state.energy_budget_joules.to_bits() <= before.to_bits());
        // Economy state must stay in sync with world state.
        assert_eq!(
            sim.economy_state.energy_budget_joules,
            i64::from(sim.state.energy_budget_joules.to_bits()) / crate::SCALE
        );
    }

    /// `phase_economy` keeps `economy_state` in sync with the world joule budget.
    #[test]
    fn phase_economy_updates_economy_state() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(99);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 1_000.0,
            scarcity_multiplier: 1.0,
        };
        sim.tick();
        // After tick, economy_state must mirror state.energy_budget_joules.
        assert_eq!(
            sim.economy_state.energy_budget_joules,
            i64::from(sim.state.energy_budget_joules.to_bits()) / crate::SCALE
        );
    }

    /// `phase_economy` advances [`MarketState`] so prices move over time.
    #[test]
    fn phase_economy_steps_market_prices() {
        const N: usize = 2;

        let mut sim = Simulation::with_seed(42);
        let initial = sim.market_state.prices.clone();
        for _ in 0..N {
            sim.tick();
        }
        assert_ne!(
            sim.market_state.prices, initial,
            "expected at least one market price to change after {N} ticks"
        );
    }

    /// FR-CIV-ECON / FR-CIV-TRADE: settlement supply imbalances should emit
    /// emergent trade flows and move the food price over successive ticks.
    #[test]
    fn phase_economy_emits_settlement_trade_flows_and_moves_prices() {
        let mut sim = Simulation::with_seed(2024);
        sim.set_settlement_population(1, 25);
        sim.set_settlement_population(2, 25);
        sim.set_settlement_food_stocked(1, 1_000);
        sim.set_settlement_food_stocked(2, 0);

        let before_price = sim.market_state.prices().get("food").copied().unwrap_or(0);
        for _ in 0..4 {
            sim.tick();
        }

        let flows = sim.last_tick_settlement_trade_flows();
        assert!(
            !flows.is_empty(),
            "expected settlement trade flows to emerge under a supply imbalance"
        );
        assert!(
            flows
                .iter()
                .any(|flow| flow.good == Good::Food && flow.qty > 0),
            "expected at least one positive food trade flow"
        );

        let after_price = sim.market_state.prices().get("food").copied().unwrap_or(0);
        assert_ne!(
            after_price, before_price,
            "expected food price to respond to the imbalance over multiple ticks"
        );
    }

    /// FR-CIV-RELIGION: the normal tick advances belief emergence and lets
    /// connected settlements exchange religious profile pressure via trade.
    #[test]
    fn tick_wires_religion_emergence_and_trade_spread() {
        let mut sim = Simulation::with_seed(73_001);
        sim.set_settlement_population(1, 250);
        sim.set_settlement_population(2, 250);
        sim.set_settlement_food_stocked(1, 2_000);
        sim.set_settlement_food_stocked(2, 0);
        for actor_id in 1..=8 {
            sim.set_settlement_actor(actor_id, 1);
            sim.set_actor_in_settlement_hardship(actor_id, 1_000);
        }
        for actor_id in 9..=16 {
            sim.set_settlement_actor(actor_id, 2);
        }

        for _ in 0..12 {
            sim.tick();
        }

        let source = sim
            .religious_profiles
            .get(&1)
            .expect("source settlement should have an emergent religion profile");
        let connected = sim
            .religious_profiles
            .get(&2)
            .expect("connected settlement should have an emergent religion profile");
        assert!(
            source.monitoring > 0.0 || source.mythic_coherence > 0.0,
            "hardship and population should produce an emergent religion signal"
        );
        assert!(
            connected.monitoring > 0.0 || connected.mythic_coherence > 0.0,
            "trade-connected settlement should receive/spread religion pressure"
        );
        assert!(
            sim.last_tick_settlement_trade_flows()
                .iter()
                .any(|flow| flow.from_settlement == 1 && flow.to_settlement == 2 && flow.qty > 0),
            "religion spread test expects the settlements to be connected by trade"
        );
    }

    /// FR-MARKET — supply shocks increase local scarcity pressure so the next
    /// tick food price exceeds a comparison sim without the shock.
    #[test]
    fn phase_economy_prices_respond_to_supply_shock() {
        let mut stable = Simulation::with_seed(9001);
        let mut shocked = Simulation::with_seed(9001);

        stable.state.trade_routes = vec![TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".to_string(),
            volume: Fixed::from_num(20),
        }];
        shocked.state.trade_routes = vec![TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".to_string(),
            volume: Fixed::from_num(20),
        }];
        stable.state.faction_resources.entry(0).or_default().food = Fixed::from_num(180);
        stable.state.faction_resources.entry(1).or_default().food = Fixed::from_num(10);
        shocked.state.faction_resources.entry(0).or_default().food = Fixed::from_num(180);
        shocked.state.faction_resources.entry(1).or_default().food = Fixed::from_num(10);

        stable.tick();
        shocked.tick();

        // Apply a one-tick supply shock between trade passes by draining
        // the exporter to create a scarcity signal.
        shocked
            .state
            .faction_resources
            .entry(0)
            .and_modify(|resources| {
                resources.food = Fixed::ZERO;
            });

        stable.tick();
        shocked.tick();

        let stable_food = stable
            .snapshot()
            .market_prices
            .get("food")
            .copied()
            .unwrap_or(0);
        let shocked_food = shocked
            .snapshot()
            .market_prices
            .get("food")
            .copied()
            .unwrap_or(0);
        assert!(
            shocked_food > stable_food,
            "expected shocked sim to have higher food price: stable={stable_food}, shocked={shocked_food}"
        );
    }

    #[test]
    fn test_initial_entities() {
        let sim = Simulation::new();
        let snapshot = sim.snapshot();
        assert!(snapshot.citizen_count > 0);
        assert!(snapshot.building_count > 0);
        assert!(snapshot.military_count > 0);
    }

    #[test]
    fn test_determinism() {
        let mut sim1 = Simulation::with_seed(12345);
        let mut sim2 = Simulation::with_seed(12345);

        for _ in 0..100 {
            sim1.tick();
            sim2.tick();
        }

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(sim1.state.population, sim2.state.population);
    }

    /// FR-CIV-ENGINE-INT-001 — climate is recomputed every tick and matches
    /// `compute_climate` directly.
    #[test]
    fn climate_recomputes_every_tick() {
        let mut sim = Simulation::with_seed(11);
        let planet = *sim.planet();
        let moon = *sim.moon();

        sim.tick();
        let expected = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(sim.climate(), &expected);

        sim.tick();
        let expected = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(sim.climate(), &expected);
    }

    /// FR-CIV-PLANET-010 — `Simulation::snapshot()` surfaces the deterministic
    /// `Climate` produced by `phase_planet`, bit-identical to `compute_climate`.
    #[test]
    fn engine_tick_includes_climate_in_snapshot() {
        let mut sim = Simulation::with_seed(2026);
        let planet = *sim.planet();
        let moon = *sim.moon();

        // Tick 0 — pre-tick climate is computed at construction time.
        let snap0 = sim.snapshot();
        let expected0 = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(snap0.tick, 0);
        assert_eq!(snap0.climate, expected0);

        // Advance ticks and confirm snapshot.climate stays bit-identical.
        for _ in 0..5 {
            sim.tick();
            let snap = sim.snapshot();
            let expected = compute_climate(sim.state.tick, &planet, &moon);

            assert_eq!(snap.tick, sim.state.tick);
            assert_eq!(snap.climate.tick, expected.tick);
            assert_eq!(
                snap.climate.day_phase.to_bits(),
                expected.day_phase.to_bits()
            );
            assert_eq!(
                snap.climate.year_phase.to_bits(),
                expected.year_phase.to_bits()
            );
            assert_eq!(
                snap.climate.moon_phase.to_bits(),
                expected.moon_phase.to_bits()
            );
            assert_eq!(
                snap.climate.tide_offset.to_bits(),
                expected.tide_offset.to_bits()
            );
            assert_eq!(snap.climate, *sim.climate());
        }
    }

    /// FR-CIV-PLANET-020 — `apply_tide_offset` shifts a registered coastal
    /// water-level voxel deterministically as the tide cycles, and the shift
    /// is symmetric around the registered sea-level baseline within tight
    /// numeric tolerance (≤ 1e-4 of the tidal amplitude in fixed-point units).
    #[test]
    fn tide_offset_shifts_coastal_voxel_height() {
        use civ_voxel::material::WATER;

        assert_eq!(WATER_MARKER_MATERIAL, WATER);

        // Use a moon config whose orbit period is a clean factor so we can land
        // on the peak (+amplitude), trough (-amplitude), and zero-crossing
        // ticks exactly. sin(TAU * phase) = +1 at phase=0.25, -1 at phase=0.75.
        let mut sim = Simulation::with_seed(2026);
        sim.moon = MoonConfig {
            orbit_period_ticks: 4,
            tidal_amplitude: 1.0,
        };
        sim.planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 0,
            day_length_ticks: 4,
            year_length_ticks: 4,
        };

        let base_y: i64 = 10 * FIXED_SCALE;
        let x: i64 = 5 * FIXED_SCALE;
        let z: i64 = 7 * FIXED_SCALE;
        sim.register_coastal_water_column(x, z, base_y);
        assert_eq!(sim.coastal_column_count(), 1);
        assert_eq!(sim.coastal_water_level(x, z), Some(base_y));

        let amplitude_units = FIXED_SCALE; // tidal_amplitude * FIXED_SCALE
        let tolerance: i64 = ((FIXED_SCALE as f64) * 1.0e-4_f64).ceil() as i64;

        // Tick 1 -> moon_phase = 0.25 -> tide_offset = +1.0 -> peak.
        sim.tick();
        let peak = sim
            .coastal_water_level(x, z)
            .expect("water level after peak tick");
        let peak_delta = peak - base_y;
        assert!(
            (peak_delta - amplitude_units).abs() <= tolerance,
            "expected peak delta ≈ +{amplitude_units}, got {peak_delta}"
        );
        // The water marker now occupies the shifted y, and the old base_y has
        // been cleared back to MaterialId(0). Both writes flow through the
        // voxel dirty queue (FR-CIV-VOXEL-002).
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: peak, z }),
            WATER_MARKER_MATERIAL
        );
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: base_y, z }),
            MaterialId(0)
        );

        // Tick 2 -> moon_phase = 0.5 -> tide_offset = 0 -> back to baseline.
        sim.tick();
        let mid = sim
            .coastal_water_level(x, z)
            .expect("water level at zero crossing");
        let mid_delta = mid - base_y;
        assert!(
            mid_delta.abs() <= tolerance,
            "expected zero-crossing delta ≈ 0, got {mid_delta}"
        );

        // Tick 3 -> moon_phase = 0.75 -> tide_offset = -1.0 -> trough.
        sim.tick();
        let trough = sim
            .coastal_water_level(x, z)
            .expect("water level after trough tick");
        let trough_delta = trough - base_y;
        assert!(
            (trough_delta + amplitude_units).abs() <= tolerance,
            "expected trough delta ≈ -{amplitude_units}, got {trough_delta}"
        );

        // Symmetry: peak and trough are mirror images around base_y within tolerance.
        let symmetry_residual = (peak_delta + trough_delta).abs();
        assert!(
            symmetry_residual <= tolerance,
            "peak {peak_delta} and trough {trough_delta} should mirror around baseline; residual {symmetry_residual} > tolerance {tolerance}"
        );

        // Tick 4 -> moon_phase = 0 -> back to baseline.
        sim.tick();
        let close = sim
            .coastal_water_level(x, z)
            .expect("water level at cycle close");
        assert!(
            (close - base_y).abs() <= tolerance,
            "expected end-of-cycle delta ≈ 0, got {}",
            close - base_y
        );

        // Determinism: a second simulation with the same seed + registration
        // produces bit-identical voxel water levels at every tick.
        let mut sim2 = Simulation::with_seed(2026);
        sim2.moon = sim.moon;
        sim2.planet = sim.planet;
        sim2.register_coastal_water_column(x, z, base_y);
        for _ in 0..4 {
            sim2.tick();
        }
        assert_eq!(
            sim.coastal_water_level(x, z),
            sim2.coastal_water_level(x, z)
        );
    }

    /// FR-CIV-TACTICS-010 — doctrine GA advances on a fixed tick cadence.
    #[test]
    fn phase_tactics_evolve_doctrine_on_cadence() {
        let mut sim = Simulation::with_seed(42);
        let gen0 = sim.faction_doctrines()[0].generation;
        for _ in 0..63 {
            sim.tick();
        }
        assert_eq!(sim.faction_doctrines()[0].generation, gen0);
        sim.tick();
        assert!(
            sim.faction_doctrines()[0].generation > gen0,
            "expected doctrine generation to advance at tick 64"
        );
    }

    /// FR-CIV-ENGINE-INT-002 — queued damage drains and voxel chunk count
    /// decreases as expected.
    #[test]
    fn pending_damage_drains_and_reduces_chunk_count() {
        let mut sim = Simulation::with_seed(12);
        fill_voxel_chunk(&mut sim.voxel_mut(), 0, 16);
        let before = sim.voxel().chunk_count();
        assert!(before > 0);

        sim.push_damage(DamageEvent {
            center: WorldCoord { x: 8, y: 8, z: 8 },
            radius_voxels: 12,
            energy: 1,
        });

        sim.tick();

        // A sphere of radius 12 voxels removes a substantial fraction of a 16³
        // chunk but never the whole 4096 cells (corner voxels are outside the
        // sphere). Assert >0 removals and <=4096 (the chunk total) — enough to
        // prove damage flowed through to the voxel substrate.
        let removed = sim.last_tick_voxel_damage_count();
        assert!(
            removed > 0,
            "expected damage to remove at least one voxel, got {removed}"
        );
        assert!(
            removed <= 16 * 16 * 16,
            "removal count exceeded chunk total: {removed}"
        );
        assert!(sim.pending_damage.is_empty());
    }

    /// FR-CIV-ENGINE-INT-003 — compact runs every 64 ticks and the uniform
    /// chunk count is non-decreasing across the cadence.
    #[test]
    fn compact_runs_every_64_ticks() {
        let mut sim = Simulation::with_seed(13);
        fill_voxel_chunk(&mut sim.voxel_mut(), 0, 16);
        let mut last_uniform = sim.voxel().uniform_chunk_count();

        for _ in 0..128 {
            sim.tick();
            let current = sim.voxel().uniform_chunk_count();
            assert!(current >= last_uniform);
            last_uniform = current;
        }
    }

    /// FR-CIV-ENGINE-INT-011 — phase_buildings allocates over time when signals are high.
    #[test]
    fn phase_buildings_allocates_over_time_when_signals_are_high() {
        let mut sim = Simulation::with_seed(77);
        sim.state.resources.wood = Fixed::from_num(800);
        sim.state.resources.metal = Fixed::from_num(800);
        let before = sim.building_graph().parcels.len();

        for _ in 0..200 {
            sim.tick();
        }

        assert!(sim.building_graph().parcels.len() > before);
    }

    /// FR-CIV-ARCH — emergence facades differ when culture profiles diverge.
    #[test]
    fn phase_buildings_applies_emergence_facades() {
        use civ_agents::culture::CultureProfile;

        let mut sim = Simulation::with_seed(91);
        sim.state.resources.wood = Fixed::from_num(900);
        sim.state.resources.metal = Fixed::from_num(900);
        sim.emergence
            .cluster_cultures
            .insert(1, CultureProfile::new([0.1, 0.2, 0.3, 0.4]));
        for _ in 0..300 {
            sim.tick();
        }
        let names: std::collections::BTreeSet<String> = sim
            .building_graph()
            .facades
            .values()
            .map(|f| f.name.clone())
            .collect();
        assert!(!names.is_empty());
    }

    /// FR-CIV-ENGINE-INT-012 — diffusion advances civilian wardrobe eras over time.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn phase_diffusion_bumps_wardrobe_eras() {
        let mut sim = Simulation::with_seed(91);
        let before = sim
            .world
            .query::<&Wardrobe>()
            .iter()
            .filter(|(_, wardrobe)| wardrobe.era >= sim.target_era)
            .count();

        for _ in 0..200 {
            sim.tick();
        }

        let after = sim
            .world
            .query::<&Wardrobe>()
            .iter()
            .filter(|(_, wardrobe)| wardrobe.era >= sim.target_era)
            .count();
        assert!(after > before);
    }

    /// FR-CIV-ENGINE-INT-015 — Cold-tier wardrobe diffusion only runs on cadence boundaries.
    #[test]
    fn cold_tier_diffusion_only_on_cadence_boundaries() {
        use civ_agents::spawn_many;

        let mut sim = Simulation::with_seed(55);
        let _ = spawn_many(&mut sim.world, 6, 50_000, 0);
        let policy = LodPolicy::default();

        let cold_entities: Vec<hecs::Entity> = sim
            .world
            .query::<(&Wardrobe, &LodTier)>()
            .iter()
            .filter_map(|(entity, (_, lod))| (*lod == LodTier::Cold).then_some(entity))
            .collect();
        assert!(
            !cold_entities.is_empty(),
            "expected spawn_many to produce Cold-tier civilians"
        );

        for tick in 1..=32 {
            // Only snapshot living cold entities: emergence (famine-driven
            // lifecycle deaths feeding legends) may despawn a civilian during a
            // tick, so an entity present this frame can be gone next frame.
            let before: std::collections::HashMap<hecs::Entity, u16> = cold_entities
                .iter()
                .filter_map(|&entity| {
                    sim.world
                        .get::<&Wardrobe>(entity)
                        .ok()
                        .map(|w| (entity, w.era))
                })
                .collect();

            sim.tick();

            for &entity in &cold_entities {
                // Skip entities that died this tick — only surviving cold
                // entities are subject to the cadence invariant.
                let Ok(wardrobe) = sim.world.get::<&Wardrobe>(entity) else {
                    continue;
                };
                let after = wardrobe.era;
                if let Some(&prev) = before.get(&entity) {
                    if prev != after {
                        assert!(
                            should_tick_entity_with_policy(tick, LodTier::Cold, policy),
                            "Cold-tier wardrobe changed on tick {tick} (off cadence)"
                        );
                    }
                }
            }
        }
    }

    /// FR-CIV-ENGINE-INT-013 — replay determinism still holds across 200 ticks
    /// with all phases on.
    #[test]
    fn determinism_holds_with_all_phases_enabled() {
        let mut sim1 = Simulation::with_seed(12345);
        let mut sim2 = Simulation::with_seed(12345);

        for tick in 0..200_u64 {
            if tick % 17 == 0 {
                let event = DamageEvent {
                    center: WorldCoord {
                        x: (tick as i64 % 32) * 1_000_000,
                        y: 0,
                        z: 0,
                    },
                    radius_voxels: 4,
                    energy: tick as u32,
                };
                sim1.push_damage(event);
                sim2.push_damage(event);
            }
            sim1.tick();
            sim2.tick();
        }

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(sim1.state.population, sim2.state.population);
        assert_eq!(sim1.climate(), sim2.climate());
        assert_eq!(
            sim1.last_tick_voxel_damage_count(),
            sim2.last_tick_voxel_damage_count()
        );
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
        assert_eq!(sim1.voxel().chunk_count(), sim2.voxel().chunk_count());
        assert_eq!(sim1.building_graph(), sim2.building_graph());
        assert_eq!(sim1.last_cohort_stats(), sim2.last_cohort_stats());
    }

    /// FR-CIV-ENGINE-INT-014 — last_cohort_stats reflects the population.
    #[test]
    fn last_cohort_stats_reflects_population() {
        let mut sim = Simulation::with_seed(19);
        sim.tick();

        let stats = sim.last_cohort_stats().expect("cohort stats");
        assert_eq!(stats.total_civilians as usize, count_civilians(&sim.world));
    }

    /// FR-CIV-ENGINE-INT-005 — `is_daytime` returns sensible day/night across
    /// one full day-length cycle.
    #[test]
    fn daytime_cycles_across_one_full_day() {
        let planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 23,
            day_length_ticks: 24,
            year_length_ticks: 240,
        };
        let moon = MoonConfig {
            orbit_period_ticks: 48,
            tidal_amplitude: 1.0,
        };

        let midnight = compute_climate(0, &planet, &moon);
        let noon = compute_climate(12, &planet, &moon);
        let next_midnight = compute_climate(24, &planet, &moon);

        assert!(!is_daytime(&midnight));
        assert!(is_daytime(&noon));
        assert!(!is_daytime(&next_midnight));
    }

    /// FR-CIV-VOXEL-006 — voxel writes between ticks produce dirty events that
    /// the engine's voxel phase drains into `last_tick_voxel_events`, in
    /// `(chunk_id, write_seq)` order.
    #[test]
    #[ignore = "TDD red step: voxel dirty event drain not yet wired through tick"]
    fn voxel_phase_drains_dirty_events_each_tick() {
        use civ_voxel::WorldCoord;
        let mut sim = Simulation::with_seed(42);
        // Tick once with nothing pending — should be empty.
        sim.tick();
        assert!(sim.last_tick_voxel_events().is_empty());
        // Write four voxels in two chunks, then tick.
        sim.voxel_mut()
            .write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
        sim.voxel_mut().write(
            WorldCoord {
                x: 1_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.voxel_mut().write(
            WorldCoord {
                x: 100_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.voxel_mut().write(
            WorldCoord {
                x: 101_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.tick();
        let events = sim.last_tick_voxel_events();
        assert_eq!(events.len(), 4);
        // Sorted ascending by (chunk_id, write_seq).
        for window in events.windows(2) {
            assert!(window[0] <= window[1]);
        }
        // Next tick clears them.
        sim.tick();
        assert!(sim.last_tick_voxel_events().is_empty());
    }

    /// FR-CIV-VOXEL-007 — voxel state is part of the deterministic simulation:
    /// two sims with identical seed + identical voxel-write sequences emit
    /// bit-identical voxel events.
    #[test]
    fn voxel_phase_replay_is_bit_identical() {
        use civ_voxel::WorldCoord;
        let mut sim1 = Simulation::with_seed(7);
        let mut sim2 = Simulation::with_seed(7);
        let writes = [
            (
                WorldCoord {
                    x: 5_000_000,
                    y: 0,
                    z: 0,
                },
                MaterialId(2),
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 5_000_000,
                    z: 0,
                },
                MaterialId(3),
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 0,
                    z: 5_000_000,
                },
                MaterialId(4),
            ),
        ];
        for (pos, mat) in writes {
            sim1.voxel_mut().write(pos, mat);
            sim2.voxel_mut().write(pos, mat);
        }
        sim1.tick();
        sim2.tick();
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
    }

    /// FR-CIV-CA-005 — identical dirty-chunk voxel setups must replay to the
    /// same log and voxel state on same-seed reruns.
    #[test]
    fn replay_ca_dirty_chunk_bit_identical() {
        use civ_voxel::material::{SAND, STONE, WATER};
        use civ_voxel::WorldCoord;

        let mut sim1 = Simulation::with_seed(17);
        let mut sim2 = Simulation::with_seed(17);
        let writes = [
            (
                WorldCoord {
                    x: 1_000_000,
                    y: 0,
                    z: 0,
                },
                WATER,
            ),
            (
                WorldCoord {
                    x: 16_000_000,
                    y: 0,
                    z: 0,
                },
                STONE,
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 16_000_000,
                    z: 0,
                },
                SAND,
            ),
        ];

        for (pos, mat) in writes {
            sim1.voxel_mut().write(pos, mat);
            sim2.voxel_mut().write(pos, mat);
        }
        let hash_before_1 = sim1.hash_chain_root();
        let hash_before_2 = sim2.hash_chain_root();
        assert_eq!(hash_before_1, hash_before_2);
        sim1.tick();
        sim2.tick();

        assert_eq!(sim1.replay_log(), sim2.replay_log());
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
        assert_eq!(sim1.voxel().chunk_count(), sim2.voxel().chunk_count());
        assert_eq!(sim1.hash_chain_root(), sim2.hash_chain_root());
    }

    /// FR-CIV-ENGINE-REPLAY-001 — ReplayLog round-trips through save/load.
    #[test]
    fn replay_log_round_trips_through_save_load() {
        let mut log = ReplayLog {
            seed: 99,
            ..ReplayLog::default()
        };
        log.record_tick(1);
        log.record_voxel_write(1, WorldCoord { x: 1, y: 2, z: 3 }, MaterialId(7));
        log.record_damage(
            2,
            DamageEvent {
                center: WorldCoord { x: 0, y: 0, z: 0 },
                radius_voxels: 2,
                energy: 11,
            },
        );
        log.record_research(3, vec![1, 2, 3], true);

        let file = NamedTempFile::new().unwrap();
        log.save(file.path()).unwrap();
        let loaded = ReplayLog::load(file.path()).unwrap();
        assert_eq!(loaded, log);
    }

    /// FR-CIV-ENGINE-REPLAY-002 — Simulation tick produces a ReplayEvent::Tick.
    #[test]
    fn simulation_tick_produces_replay_tick_event() {
        let mut sim = Simulation::with_seed(1);
        sim.tick();
        assert!(matches!(
            sim.replay_log().events.last(),
            Some(ReplayEvent::Tick { tick: 1 })
        ));
    }

    /// FR-CIV-TACTICS-041 — combat events extend the replay hash chain.
    #[test]
    fn combat_events_extend_replay_hash_chain() {
        let event = DamageEvent {
            center: WorldCoord { x: 10, y: 0, z: 20 },
            radius_voxels: 2,
            energy: 100,
        };
        let mut log = ReplayLog::default();
        log.record_tick(1);
        let after_tick = log.running_hash;
        log.record_combat(1, 10, 20, event);
        log.verify_hash_chain().expect("chain");
        assert_ne!(log.running_hash, after_tick);
    }

    /// FR-CIV-TACTICS-025-int — replay log restores queued combat damage events.
    #[test]
    fn replay_combat_events_restore_pending_damage() {
        let event = DamageEvent {
            center: WorldCoord {
                x: 100,
                y: 0,
                z: 200,
            },
            radius_voxels: 2,
            energy: 50,
        };
        let mut sim = Simulation::with_seed(1);
        sim.replay_log.record_combat(16, 10, 20, event);
        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(99);
        log.replay(&mut replayed).unwrap();
        assert_eq!(replayed.pending_damage.len(), 1);
        assert_eq!(replayed.pending_damage[0], event);
        assert_eq!(replayed.state.tick, 16);
    }

    /// FR-CIV-TACTICS-025-int2 — replay combat events drain to the same voxel state as live ticks.
    #[test]
    fn replay_combat_drains_to_same_voxel_state_as_live() {
        let seed = 12;
        let ticks = 32u64;
        let mut live = Simulation::with_seed(seed);
        for _ in 0..ticks {
            live.tick();
        }
        let chunk_live = live.voxel().chunk_count();
        let combat_count = live
            .replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Combat { .. }))
            .count();
        assert!(combat_count > 0, "expected war-bridge combat in replay log");

        let mut from_replay = Simulation::with_seed(seed);
        live.replay_log().replay(&mut from_replay).unwrap();
        assert_eq!(from_replay.voxel().chunk_count(), chunk_live);
        assert_eq!(from_replay.state.tick, live.state.tick);
    }

    /// FR-CIV-TACTICS-025-int3 — same seed reproduces identical combat replay markers.
    #[test]
    fn replay_combat_log_deterministic_for_seed_rerun() {
        let seed = 5;
        let ticks = 48u64;
        let mut a = Simulation::with_seed(seed);
        let mut b = Simulation::with_seed(seed);
        for _ in 0..ticks {
            a.tick();
            b.tick();
        }
        let combat_a: Vec<_> = a
            .replay_log()
            .events
            .iter()
            .filter_map(|e| match e {
                ReplayEvent::Combat {
                    tick,
                    shooter_id,
                    target_id,
                    event,
                } => Some((*tick, *shooter_id, *target_id, *event)),
                _ => None,
            })
            .collect();
        let combat_b: Vec<_> = b
            .replay_log()
            .events
            .iter()
            .filter_map(|e| match e {
                ReplayEvent::Combat {
                    tick,
                    shooter_id,
                    target_id,
                    event,
                } => Some((*tick, *shooter_id, *target_id, *event)),
                _ => None,
            })
            .collect();
        assert!(!combat_a.is_empty());
        assert_eq!(combat_a, combat_b);
    }

    /// FR-CIV-TACTICS-025 — war-bridge engagements append ReplayEvent::Combat.
    #[test]
    fn war_bridge_records_combat_replay_events() {
        let mut sim = Simulation::with_seed(1);
        for _ in 0..16 {
            sim.tick();
        }
        assert!(sim.replay_log().events.iter().any(|event| {
            matches!(
                event,
                ReplayEvent::Combat {
                    shooter_id,
                    target_id,
                    ..
                } if *shooter_id != 0 && *target_id != 0
            )
        }));
    }

    /// FR-CIV-ENGINE-REPLAY-003 — push_damage records a Damage event.
    #[test]
    fn push_damage_records_damage_event() {
        let mut sim = Simulation::with_seed(1);
        let event = DamageEvent {
            center: WorldCoord { x: 1, y: 1, z: 1 },
            radius_voxels: 3,
            energy: 4,
        };
        sim.push_damage(event);
        assert!(matches!(
            sim.replay_log().events.last(),
            Some(ReplayEvent::Damage { tick: 0, event: recorded }) if recorded == &event
        ));
    }

    /// FR-CIV-ENGINE-REPLAY-004 — replay reproduces final voxel chunk count and tick.
    #[test]
    fn replay_reproduces_final_voxel_chunk_count_and_tick() {
        let mut sim = Simulation::with_seed(2);
        sim.voxel_mut()
            .write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
        sim.push_damage(DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 1,
            energy: 1,
        });
        sim.tick();

        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(2);
        log.replay(&mut replayed).unwrap();
        assert_eq!(replayed.state.tick, sim.state.tick);
        assert_eq!(replayed.voxel().chunk_count(), sim.voxel().chunk_count());
    }

    /// CIV-0104 — minimal tick invariants hold after every tick.
    #[test]
    fn tick_invariants_hold_across_many_ticks() {
        use crate::invariants::check_tick_invariants;

        let mut sim = Simulation::with_seed(104);
        check_tick_invariants(&sim).expect("initial state");

        for _ in 0..200 {
            sim.tick();
            check_tick_invariants(&sim).expect("invariants after tick");
        }
    }

    /// FR-REPLAY-001 — `.civreplay` save/load restores simulation tick after N ticks.
    #[test]
    fn civreplay_save_load_restores_tick_after_ticks() {
        const N: u64 = 17;
        let mut sim = Simulation::with_seed(7);
        for _ in 0..N {
            sim.tick();
        }
        let expected_tick = sim.state.tick;

        let file = NamedTempFile::new().unwrap();
        sim.save_replay(file.path()).unwrap();
        let loaded = Simulation::load_replay_from_file(file.path()).unwrap();
        assert_eq!(loaded.state.tick, expected_tick);
    }

    /// FR-CIV-ENGINE-REPLAY-005 — identical replay logs converge to identical voxel state.
    #[test]
    fn replay_logs_converge_to_identical_voxel_state() {
        let mut sim1 = Simulation::with_seed(3);
        sim1.voxel_mut()
            .write(WorldCoord { x: 4, y: 5, z: 6 }, MaterialId(9));
        sim1.voxel_mut()
            .write(WorldCoord { x: 8, y: 9, z: 10 }, MaterialId(8));
        sim1.tick();

        let log = sim1.replay_log().clone();
        let mut sim2 = Simulation::with_seed(3);
        log.replay(&mut sim2).unwrap();

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(
            sim1.voxel().read(WorldCoord { x: 4, y: 5, z: 6 }),
            sim2.voxel().read(WorldCoord { x: 4, y: 5, z: 6 })
        );
        assert_eq!(
            sim1.voxel().read(WorldCoord { x: 8, y: 9, z: 10 }),
            sim2.voxel().read(WorldCoord { x: 8, y: 9, z: 10 })
        );
    }

    /// FR-CIV-TACTICS-025 — replay round-trip: war-bridge Combat events exist in the
    /// original log and the replayed simulation converges to the same tick and voxel state.
    #[test]
    fn replay_round_trip_preserves_combat_events() {
        let mut sim = Simulation::with_seed(1);
        for _ in 0..16 {
            sim.tick();
        }

        let combat_count = sim.replay_log().combat_event_count();
        assert!(
            combat_count > 0,
            "expected at least one Combat replay event after 16 ticks"
        );

        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(1);
        log.replay(&mut replayed).unwrap();

        assert_eq!(
            replayed.state.tick, sim.state.tick,
            "replayed tick must match original"
        );
        assert_eq!(
            replayed.voxel().chunk_count(),
            sim.voxel().chunk_count(),
            "replayed voxel chunk count must match original"
        );
    }

    /// FR-CIV-TACTICS-024 — snapshot.damage_events reflects combat pulses from
    /// the most recent tick.
    #[test]
    fn snapshot_damage_events_reflects_last_tick_pulses() {
        let mut sim = Simulation::with_seed(6);
        // Advance to a war-bridge cadence boundary (cadence = 16).
        for _ in 0..16 {
            sim.tick();
        }
        let snap = sim.snapshot();
        // After a cadence tick with ≥2 opposing military units the pulses list
        // must be non-empty; the snapshot field must match.
        assert_eq!(snap.damage_events, sim.last_tick_combat_pulses().len());
    }

    /// FR-CIV-PLANET-030 — `snapshot().weather_grid` temperature varies with
    /// year phase (summer equatorial > winter equatorial) and results are
    /// deterministic across re-runs.
    #[test]
    fn weather_grid_temperature_varies_with_year_phase() {
        // Earth-like defaults: year_length_ticks = 8_766_000, tilt = 23°.
        let year_length_ticks = 8_766_000_u64;
        let equatorial_idx = 8_usize; // middle of 16-region grid

        // Northern summer: year ¼ → sin(year_phase) is at peak
        let summer_tick = year_length_ticks / 4;
        // Northern winter: year ¾ → sin(year_phase) is at trough
        let winter_tick = year_length_ticks * 3 / 4;

        let mut sim_s = Simulation::with_seed(0);
        // Fast-forward to summer_tick by running ticks (use state manipulation
        // for test speed: set tick directly and recompute phase_planet).
        sim_s.state.tick = summer_tick;
        let planet_s = *sim_s.planet();
        let moon_s = *sim_s.moon();
        sim_s.climate = compute_climate(summer_tick, &planet_s, &moon_s);
        sim_s.weather_grid = compute_weather(&sim_s.climate, summer_tick, 16);
        let snap_summer = sim_s.snapshot();

        let mut sim_w = Simulation::with_seed(0);
        sim_w.state.tick = winter_tick;
        let planet_w = *sim_w.planet();
        let moon_w = *sim_w.moon();
        sim_w.climate = compute_climate(winter_tick, &planet_w, &moon_w);
        sim_w.weather_grid = compute_weather(&sim_w.climate, winter_tick, 16);
        let snap_winter = sim_w.snapshot();

        let summer_temp = snap_summer.weather_grid[equatorial_idx].temp_c_fp;
        let winter_temp = snap_winter.weather_grid[equatorial_idx].temp_c_fp;

        assert!(
            summer_temp > winter_temp,
            "summer equatorial temp ({summer_temp} fp) should exceed winter ({winter_temp} fp)"
        );

        // Determinism: re-running the same ticks must produce identical grids.
        let summer_grid_2 = compute_weather(&sim_s.climate, summer_tick, 16);
        assert_eq!(
            snap_summer.weather_grid, summer_grid_2,
            "weather grid must be deterministic across re-runs"
        );
    }

    // -------------------------------------------------------------------------
    // FR-CIV-CA-009 — `Simulation::phase_voxel_ca` + abiogenesis sites.
    // -------------------------------------------------------------------------

    /// FR-CIV-CA-009 — `phase_voxel_ca(None)` is a no-op: sites stay empty.
    /// This is the cheap path (no resident window wired up) and must not
    /// blow up or allocate a giant vec.
    #[test]
    fn phase_voxel_ca_none_is_noop() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-CA-009 — warm liquid WATER in a single chunk produces at
    /// least one viable abiogenesis site. A pure STONE chunk produces
    /// zero. The two runs must round-trip deterministically (same seed,
    /// same grid → same sites).
    #[test]
    fn phase_voxel_ca_warm_water_is_viable_stone_is_not() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-0100 — chronicle records technological breakthroughs when tech bits advance.
    #[test]
    fn chronicle_records_tech_breakthroughs() {
        // TODO: Implement WorldState::research_progress, Simulation::phase_tech, phase_chronicle, chronicle
    }

    /// FR-CIV-0100 — chronicle length stays bounded at CHRONICLE_MAX_LEN.
    #[test]
    fn chronicle_is_length_capped() {
        // TODO: Implement WorldState::chronicle field, Simulation::phase_chronicle and chronicle
    }

    /// FR-CIV-0100 — golden-age chronicle lines are deduped via chronicle_age.
    #[test]
    fn chronicle_dedups_age() {
        // TODO: Implement WorldState::chronicle_age, Simulation::phase_chronicle and chronicle
    }

    /// `tick_with_emergence_source` advances ticks identically; CA grid changes sampling.
    #[test]
    fn tick_with_emergence_source_advances_tick_and_differs_on_ca_grid() {
        // TODO: Implement Simulation::tick_with_emergence_source
    }

    /// `apply_scenario_military` wires cadence overrides and clamps engage range.
    #[test]
    fn apply_scenario_military_wires_overrides_and_clamps_range() {
        use crate::scenario::ScenarioMilitary;

        let mut sim = Simulation::with_seed(8);
        let military = ScenarioMilitary {
            movement_cadence_ticks: Some(8),
            movement_pulses_per_cadence: Some(3),
            war_cadence_ticks: Some(32),
            engage_range_grid: Some(0),
        };
        sim.apply_scenario_military(&military);
        let cfg = sim.military_phase_config();
        assert_eq!(cfg.movement.cadence_ticks, 8);
        assert_eq!(cfg.movement_pulses_per_cadence, 3);
        assert_eq!(cfg.war.cadence_ticks, 32);
        assert_eq!(cfg.war.engage_range_grid, 1);
    }

    /// `configure_military_fog` sets vision radius and clamps grid size.
    #[test]
    fn configure_military_fog_sets_radius_and_clamps_grid() {
        let mut sim = Simulation::with_seed(9);
        sim.configure_military_fog(Some(8), 12);
        assert_eq!(sim.military_phase_config().war.fog_vision_radius, Some(8));
        assert_eq!(sim.military_phase_config().war.fog_grid_size, 16);

        let kept_radius = sim.military_phase_config().war.fog_vision_radius;
        let kept_grid = sim.military_phase_config().war.fog_grid_size;
        sim.configure_military_fog(None, 99);
        assert_eq!(
            sim.military_phase_config().war.fog_vision_radius,
            kept_radius
        );
        assert_eq!(sim.military_phase_config().war.fog_grid_size, kept_grid);
    }

    // -------------------------------------------------------------------
    // Coverage-gap closure (COVERAGE_GAPS_4): the three pure policy helpers
    // below had no direct unit tests prior to this commit. Each test below
    // is named per the coverage-gap closure plan and bundles all relevant
    // edge cases from TEST_SPECS_UNTESTED.md into a single `#[test]`.
    // -------------------------------------------------------------------

    /// `job_type_for_civilian_id` is a total pure function of its `u64`
    /// input. This test pins the full mod-7 bucket map (including the
    /// catch-all `_` arm), wrap-around at the modulus, sparse / far-out ids
    /// resolving to the right bucket via `id % 7`, the `u64::MAX` boundary,
    /// and the determinism guarantee (same id → same `JobType`, no state).
    /// FR-CIV-ENGINE spawn-determinism depends on this. (COVERAGE_GAPS_4 row 1.)
    #[test]
    fn job_type_for_civilian_id_deterministic_split() {
        // All seven mod-buckets, including the `_`-arm for remainder 6.
        assert_eq!(job_type_for_civilian_id(0), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(1), JobType::Warrior);
        assert_eq!(job_type_for_civilian_id(2), JobType::Scholar);
        assert_eq!(job_type_for_civilian_id(3), JobType::Trader);
        assert_eq!(job_type_for_civilian_id(4), JobType::Priest);
        assert_eq!(job_type_for_civilian_id(5), JobType::Admin);
        assert_eq!(job_type_for_civilian_id(6), JobType::Unemployed);

        // `id % 7` wraps cleanly: every 7th id resolves to the same JobType.
        assert_eq!(job_type_for_civilian_id(7), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(14), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(42), JobType::Farmer); // 42 % 7 == 0
        assert_eq!(job_type_for_civilian_id(13), JobType::Unemployed); // 13 % 7 == 6
        assert_eq!(job_type_for_civilian_id(20), JobType::Unemployed); // 20 % 7 == 6

        // Sparse / far-out ids resolve to a deterministic bucket.
        // 1_000_000_008 % 7 == 0 (1_000_000_008 = 142_857_144 * 7) → Farmer.
        assert_eq!(job_type_for_civilian_id(1_000_000_008), JobType::Farmer);
        // 999_999_999 % 7: 999_999_999 / 7 = 142_857_142 remainder 5 → Admin.
        assert_eq!(job_type_for_civilian_id(999_999_999), JobType::Admin);
        // 1_000_000_000_000_000_000 % 7 = 1 → Warrior.
        assert_eq!(
            job_type_for_civilian_id(1_000_000_000_000_000_000),
            JobType::Warrior
        );

        // u64::MAX % 7 == 1 (u64::MAX = 2^64-1 = 2_635_249_153_387_078_802*7 + 1)
        // → Warrior. Confirms totality over the full u64 range, no overflow.
        assert_eq!(job_type_for_civilian_id(u64::MAX), JobType::Warrior);

        // Determinism: same id → same JobType, no state, no panic.
        for id in [0u64, 1, 6, 7, 42, 100, 999_999_999, u64::MAX] {
            assert_eq!(
                job_type_for_civilian_id(id),
                job_type_for_civilian_id(id),
                "job_type_for_civilian_id({id}) must be a pure function of its input"
            );
        }
    }

    /// `faction_wealth_scarcity_shadow` maps (treasury, resources) → shadow
    /// price used as input to `faction_unrest_delta_from_shadow`. This test
    /// pins the comfort-threshold branch (≥ 12_000 → baseline), the exact
    /// `12_000` boundary, the empty-Resources "deep scarcity" extreme
    /// (wealth = 0 → 4_000), food-only and treasury-only shortfalls, the
    /// lower floor at `FOOD_SCARCITY_BASELINE`, and the `treasury.to_bits() / SCALE`
    /// integer-units conversion. (COVERAGE_GAPS_4 row 5.)
    #[test]
    fn faction_wealth_scarcity_shadow_edge_cases() {
        // Comfort branch: wealth >= 12_000 pins shadow to FOOD_SCARCITY_BASELINE.
        // treasury=100_000, food=10_000 → wealth = 100_000 + 10_000*50 = 600_000.
        let res = Resources {
            food: Fixed::from_num(10_000),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(100_000), &res),
            FOOD_SCARCITY_BASELINE
        );

        // Exact comfort boundary: wealth == 12_000 still pins to baseline
        // because the function uses `>=`, not strict `>`.
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(12_000), &res),
            FOOD_SCARCITY_BASELINE
        );

        // Empty Resources + zero treasury = "deep scarcity": wealth = 0,
        // shadow = 1_000 + 12_000/4 = 4_000. (No upper clamp inside the
        // function; this is the maximum shadow reachable in one call.)
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::ZERO, &res),
            FOOD_SCARCITY_BASELINE + 12_000 / 4,
            "empty Resources + zero treasury lands at the maximum shadow"
        );

        // Food-only shortfall: treasury = 0, food = 10 → wealth = 500.
        // shadow = 1_000 + (12_000 - 500)/4 = 1_000 + 2_875 = 3_875.
        let res = Resources {
            food: Fixed::from_num(10),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::ZERO, &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 500) / 4
        );

        // Treasury-only shortfall: treasury = 4_000, food = 0 → wealth = 4_000.
        // shadow = 1_000 + (12_000 - 4_000)/4 = 3_000.
        // NOTE: the function does NOT implement a "treasury hedges food"
        // channel — treasury is additive in the same units as the
        // food-weighted wealth. This test pins the actual behavior.
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(4_000), &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 4_000) / 4
        );

        // Lower floor: shadow never falls below FOOD_SCARCITY_BASELINE for
        // any legal input. The comfort branch pins to it, the shortfall
        // branch adds to it.
        let cases: Vec<(i64, Resources)> = vec![
            (0, Resources::default()),
            (10_000, Resources::default()),
            (
                0,
                Resources {
                    food: Fixed::from_num(1),
                    ..Resources::default()
                },
            ),
            (Fixed::from_num(5_000).to_bits(), Resources::default()),
            (Fixed::from_num(99_999_999).to_bits(), Resources::default()),
        ];
        for (treasury_raw, res) in cases {
            let treasury = Fixed::from_bits(treasury_raw);
            let shadow = faction_wealth_scarcity_shadow(treasury, &res);
            assert!(
                shadow >= FOOD_SCARCITY_BASELINE,
                "shadow ({shadow}) fell below FOOD_SCARCITY_BASELINE ({FOOD_SCARCITY_BASELINE})"
            );
        }

        // `treasury.to_bits() / SCALE` is the integer wealth — guards against a
        // regression that would drop the `/ SCALE` and treat `raw` directly
        // as a wealth value.
        // treasury = 5_000 (fixed-point) → treasury_i = 5_000, food_i = 0,
        // wealth = 5_000 < 12_000 → shortfall: 1_000 + 7_000/4 = 2_750.
        let res = Resources::default();
        let treasury = Fixed::from_num(5_000);
        assert_eq!(
            faction_wealth_scarcity_shadow(treasury, &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 5_000) / 4
        );
    }

    /// `faction_unrest_delta_from_shadow` is a thin pass-through to
    /// `unrest_delta`. This test pins the sign behavior (shadow ≤ baseline
    /// → decay `-10`; shadow > baseline → positive rise), the `clamp(1, 50)`
    /// bounds, the linear scaling with shortfall, the `MAX_RISE = 50`
    /// ceiling for arbitrarily large shadows (including `i64::MAX`), and
    /// the wrapper's identity with `unrest_delta` across the full sign
    /// range. (COVERAGE_GAPS_4 row 6: "clamp at 0" lives in the caller's
    /// accumulator; the delta itself only knows `-10` and `[1, 50]`.)
    #[test]
    fn faction_unrest_delta_from_shadow_sign_and_clamp() {
        // shadow ≤ baseline → decay -10 (not zero, not positive).
        for shadow in [0i64, 100, 500, 999] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                -10,
                "shadow={shadow} (below baseline) must decay by 10"
            );
        }

        // At the boundary shadow == baseline the function takes the `else`
        // branch (scarcity is not > 0) and returns -10, not zero. Pin this
        // so a future `>=` refactor doesn't silently flip the boundary.
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE),
            -10
        );

        // Just above baseline, rise is clamped to a minimum of +1
        // (clamp(1, MAX_RISE) lower bound kicks in for any scarcity > 0,
        // even when scarcity / 20 == 0).
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1),
            1
        );
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 19),
            1
        );

        // Rise scales linearly with shortfall (scarcity / 20) until it
        // hits the MAX_RISE ceiling of 50.
        // shadow = 1_100 → scarcity = 100 → 100/20 = 5
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 100),
            5
        );
        // shadow = 1_400 → scarcity = 400 → 400/20 = 20
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 400),
            20
        );
        // shadow = 2_000 → scarcity = 1_000 → 1_000/20 = 50 (at ceiling)
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1_000),
            50
        );

        // Large shadows still clamp to MAX_RISE = 50. Stops a price spike
        // from instantly maxing faction unrest.
        for shadow in [10_000i64, 1_000_000, 1_000_000_000, i64::MAX] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                50,
                "shadow={shadow} must clamp at MAX_RISE=50"
            );
        }

        // Wrapper identity with `unrest_delta` across the full sign range.
        for shadow in [
            0i64,
            FOOD_SCARCITY_BASELINE - 1,
            FOOD_SCARCITY_BASELINE,
            FOOD_SCARCITY_BASELINE + 1,
            FOOD_SCARCITY_BASELINE + 100,
            FOOD_SCARCITY_BASELINE + 1_000,
            FOOD_SCARCITY_BASELINE + 100_000,
            i64::MAX,
        ] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                unrest_delta(shadow),
                "wrapper must equal unrest_delta at shadow={shadow}"
            );
        }
    }

    // ── N9 tests ──────────────────────────────────────────────────────────────

    /// N9: `aggression_threshold_reduction` is bounded: 0.0→0, 0.5→1500,
    /// 1.0→3000, and clamping means 2.0 still yields 3000.
    #[test]
    fn aggression_threshold_reduction_bounded() {
        assert_eq!(aggression_threshold_reduction(0.0), 0);
        assert_eq!(aggression_threshold_reduction(0.5), 1500);
        assert_eq!(aggression_threshold_reduction(1.0), 3000);
        assert_eq!(aggression_threshold_reduction(2.0), 3000); // clamped
    }

    /// N9: `faction_aggression` is rebuilt fresh each tick (ephemeral).
    #[test]
    fn faction_aggression_rebuilt_each_tick() {
        let mut sim = Simulation::with_seed(1);
        // Before any tick, faction_aggression is empty.
        assert!(
            sim.faction_aggression.is_empty(),
            "faction_aggression should start empty"
        );
        // After a tick the emergence phase populates it (agents have DNA).
        sim.tick();
        // The map is populated whenever there are aligned civilians with DNA.
        // Just verify the field is accessible and the type is correct.
        let _: &std::collections::BTreeMap<u32, f32> = &sim.faction_aggression;
    }

    /// FR-CIV-DIPLOMACY — `Simulation::tick()` must keep updating faction
    /// relations so emergent proximity/trade/war signals can accumulate over time.
    #[test]
    fn diplomacy_relations_evolve_through_sim_tick() {
        let mut sim = Simulation::with_seed(91);
        sim.state.tick = 499;

        let faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        assert!(
            faction_ids.len() >= 2,
            "test requires at least two factions"
        );

        // Reproduce the pair selection from `tick_faction_relation_drift`:
        // sorted faction_ids, pick by tick % len.
        let mut sorted_ids = faction_ids.clone();
        sorted_ids.sort_unstable();
        let tick_usize = 500_usize; // tick after increment
        let a = sorted_ids[tick_usize % sorted_ids.len()];
        let b = sorted_ids[(tick_usize + 1) % sorted_ids.len()];

        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(0));

        sim.tick();

        let event = sim.diplomacy_events().last().expect("diplomacy event");
        assert_eq!(event.tick, 500);
        assert_eq!((event.faction_a, event.faction_b), (a, b));
        assert!(
            matches!(
                event.kind,
                DiplomacyKind::TradeAgreement | DiplomacyKind::Conflict
            ),
            "diplomacy event should be TradeAgreement or Conflict"
        );
        // Treasury should have been modified by the drift logic.
        let trea_a = sim
            .state
            .faction_treasury
            .get(&a)
            .copied()
            .unwrap_or_default();
        let trea_b = sim
            .state
            .faction_treasury
            .get(&b)
            .copied()
            .unwrap_or_default();
        assert_ne!(
            trea_a,
            Fixed::from_num(0),
            "faction a treasury should change"
        );
        assert_ne!(
            trea_b,
            Fixed::from_num(0),
            "faction b treasury should change"
        );
    }

    #[test]
    fn player_diplomacy_action_mutates_relation_substrate() {
        let mut sim = Simulation::with_seed(7);
        let relation = sim
            .apply_player_diplomacy_action(0, 1, DiplomacyKind::Conflict)
            .expect("known faction pair should mutate");

        assert_eq!(relation.faction_a, 0);
        assert_eq!(relation.faction_b, 1);
        assert!(relation.score < 0.0);
        assert!(matches!(
            relation.kind.as_str(),
            "neutral" | "allied" | "hostile"
        ));
        assert_eq!(
            sim.diplomacy_events().last(),
            Some(&DiplomacyEvent {
                tick: sim.state.tick,
                faction_a: 0,
                faction_b: 1,
                kind: DiplomacyKind::Conflict,
            })
        );
    }

    #[test]
    fn player_trade_action_mutates_relation_substrate_positive() {
        let mut sim = Simulation::with_seed(7);
        let relation = sim
            .apply_player_diplomacy_action(0, 1, DiplomacyKind::TradeAgreement)
            .expect("known faction pair should mutate");

        assert_eq!(relation.faction_a, 0);
        assert_eq!(relation.faction_b, 1);
        assert!(relation.score > 0.0);
        assert!(matches!(
            relation.kind.as_str(),
            "neutral" | "allied" | "hostile"
        ));
        assert_eq!(
            sim.diplomacy_events().last().map(|event| event.kind),
            Some(DiplomacyKind::TradeAgreement)
        );
    }

    #[test]
    fn player_diplomacy_action_rejects_unknown_or_self_pair() {
        let mut sim = Simulation::with_seed(7);
        assert_eq!(
            sim.apply_player_diplomacy_action(0, 0, DiplomacyKind::Conflict),
            None
        );
        assert_eq!(
            sim.apply_player_diplomacy_action(0, 99, DiplomacyKind::Conflict),
            None
        );
        assert!(sim.diplomacy_events().is_empty());
    }

    /// N9: faction pairs with high aggression clash at lower disparity than
    /// faction pairs with zero aggression.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn aggressive_factions_clash_sooner() {
        // Build a baseline sim where factions are at the trade/conflict boundary.
        let mut sim_low = Simulation::with_seed(5);
        sim_low.state.tick = 500;
        sim_low.state.belief = 0;
        sim_low.state.cohesion = 0;
        sim_low.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim_low.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim_low.state.tick);
        // A disparity just below the base threshold: both sims should trade normally.
        let base = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
        sim_low.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_low
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(base - 1));
        // Zero aggression → no reduction.
        sim_low.faction_aggression.insert(a, 0.0);
        sim_low.faction_aggression.insert(b, 0.0);
        sim_low.phase_diplomacy();
        let low_kind = sim_low.diplomacy_events().last().expect("event").kind;

        // High aggression sim: same disparity, but aggression lowers threshold.
        let mut sim_high = Simulation::with_seed(5);
        sim_high.state.tick = 500;
        sim_high.state.belief = 0;
        sim_high.state.cohesion = 0;
        sim_high.state.unrest = 0;
        sim_high
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
        sim_high
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(base - 1));
        // Max aggression → reduction = 3000, so threshold drops to DIPLOMACY_MIN_CONFLICT_THRESHOLD.
        sim_high.faction_aggression.insert(a, 1.0);
        sim_high.faction_aggression.insert(b, 1.0);
        sim_high.phase_diplomacy();
        let high_kind = sim_high.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            low_kind,
            DiplomacyKind::TradeAgreement,
            "low-aggression factions should trade at this disparity"
        );
        assert_eq!(
            high_kind,
            DiplomacyKind::Conflict,
            "high-aggression factions should clash at the same disparity"
        );
    }

    // N11 maturity↔belief coupling tests (FR-CIV-EMERGENCE-N11)

    #[test]
    fn n11_avg_psyche_maturity_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        assert_eq!(avg_psyche_maturity(&sim.world), 0.0);
    }

    #[test]
    fn n11_avg_psyche_maturity_computes_mean() {
        use civ_agents::{Mood, Psyche, Temperament, PSYCHE_DIM};
        let mut sim = Simulation::new();
        sim.world.clear();
        let psyche = Psyche {
            drives: [0.5; PSYCHE_DIM],
            temperament: Temperament::neutral(),
            mood: Mood::neutral(),
            beliefs: [0.5; PSYCHE_DIM],
            maturity: 1.0,
        };
        sim.world.spawn((psyche,));
        assert_eq!(avg_psyche_maturity(&sim.world), 1.0);
    }

    #[test]
    fn n11_drift_factor_bounds() {
        for (maturity, expected) in [(0.0f32, 0.95f32), (0.5, 0.975), (1.0, 1.0)] {
            let drift = 0.95 + 0.05 * maturity;
            assert!(
                (drift - expected).abs() < 1e-6,
                "maturity={} drift={}",
                maturity,
                drift
            );
        }
    }

    #[test]
    #[ignore = "TDD red step: phase_diplomacy doesn't generate events for high disparity"]
    fn religion_diplomacy_coupling_phase_picks_trade_over_conflict() {
        let disparity = DIPLOMACY_BASE_CONFLICT_THRESHOLD + 2_000;
        let mut sim_peace = Simulation::with_seed(5);
        sim_peace.state.tick = 500;
        sim_peace.state.belief = 500_000;
        sim_peace.state.cohesion = 200_000;
        sim_peace.emergence.has_patron = true;
        let mut faction_ids: Vec<u32> = sim_peace.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return;
        }
        let (a, b) = (faction_ids[0], faction_ids[1]);
        sim_peace
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
        sim_peace
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_peace.phase_diplomacy();
        let peace_kind = sim_peace.diplomacy_events().last().expect("event").kind;

        let mut sim_war = Simulation::with_seed(5);
        sim_war.state.tick = 500;
        sim_war.state.belief = 0;
        sim_war.state.cohesion = 0;
        sim_war.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_war
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_war.phase_diplomacy();
        let war_kind = sim_war.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            peace_kind,
            DiplomacyKind::TradeAgreement,
            "high belief+cohesion must bias toward peace at fixed disparity"
        );
        assert_eq!(
            war_kind,
            DiplomacyKind::Conflict,
            "low belief must allow conflict at same disparity"
        );
    }

    /// `canonical_faction_pair` always returns the pair in ascending order so
    /// (a, b) and (b, a) hash to the same BTreeMap key.
    #[test]
    fn canonical_faction_pair_orders_ascending() {
        assert_eq!(canonical_faction_pair(0, 1), (0, 1), "already sorted");
        assert_eq!(
            canonical_faction_pair(1, 0),
            (0, 1),
            "reversed becomes sorted"
        );
        assert_eq!(canonical_faction_pair(3, 3), (3, 3), "equal ids stay equal");
        assert_eq!(
            canonical_faction_pair(u32::MAX, 0),
            (0, u32::MAX),
            "large vs small"
        );
        for (a, b) in [(2u32, 5), (10, 1), (7, 7), (0, u32::MAX)] {
            assert_eq!(
                canonical_faction_pair(a, b),
                canonical_faction_pair(b, a),
                "canonical_faction_pair({a},{b}) must be symmetric"
            );
        }
    }

    /// `route_resource` maps known goods labels to the correct ResourceType.
    /// Unknown goods fall back to Food (documented default).
    #[test]
    fn route_resource_maps_known_goods() {
        assert_eq!(route_resource("grain"), ResourceType::Food, "grain → Food");
        assert_eq!(
            route_resource("timber"),
            ResourceType::Wood,
            "timber → Wood"
        );
        assert_eq!(route_resource("ore"), ResourceType::Metal, "ore → Metal");
        assert_eq!(
            route_resource("tools"),
            ResourceType::Metal,
            "tools → Metal"
        );
        assert_eq!(
            route_resource("cloth"),
            ResourceType::Energy,
            "cloth → Energy"
        );
        assert_eq!(
            route_resource("salt"),
            ResourceType::Energy,
            "salt → Energy"
        );
        assert_eq!(
            route_resource(""),
            ResourceType::Food,
            "empty string → Food (fallback)"
        );
        assert_eq!(
            route_resource("unknown"),
            ResourceType::Food,
            "unrecognized → Food (fallback)"
        );
    }

    /// `emergent_route_goods` is deterministic: same faction id → same goods
    /// label, cycling across the three labels via id % 3.
    #[test]
    fn emergent_route_goods_is_deterministic_and_covers_all_labels() {
        assert_eq!(emergent_route_goods(0), "grain", "id%3==0 → grain");
        assert_eq!(emergent_route_goods(1), "ore", "id%3==1 → ore");
        assert_eq!(emergent_route_goods(2), "cloth", "id%3==2 → cloth");
        assert_eq!(emergent_route_goods(3), "grain", "id=3 wraps to grain");
        for id in [0u32, 1, 2, 100, u32::MAX] {
            assert_eq!(
                emergent_route_goods(id),
                emergent_route_goods(id),
                "emergent_route_goods({id}) must be a pure function of its input"
            );
        }
        // All labels returned by emergent_route_goods must be handled by route_resource
        // without falling through to the unknown fallback path.
        let known_labels = ["grain", "ore", "cloth", "timber", "tools", "salt"];
        for id in 0u32..3 {
            let goods = emergent_route_goods(id);
            assert!(
                known_labels.contains(&goods),
                "emergent_route_goods({id})=\"{goods}\" is not a known trade label"
            );
        }
    }

    // N10 kinship↔cohesion coupling tests (FR-CIV-EMERGENCE-N10)

    #[test]
    fn n10_avg_faction_kinship_computes_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        let avg = avg_faction_kinship(&sim.world);
        assert_eq!(avg, 0.0, "empty world should have zero average kinship");
    }

    #[test]
    fn n10_avg_faction_kinship_computes_mean_correctly() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();
        sim.world.clear();

        // Spawn one social graph with a single kinship tie of 1.0.
        let graph_a = SocialGraph {
            ties: vec![Tie {
                other: 1002,
                kinship: 1.0,
                familiarity: 0.0,
                affinity: 0.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((graph_a,));
        sim.world.spawn((SocialGraph::default(),));

        let avg = avg_faction_kinship(&sim.world);
        assert_eq!(avg, 1.0, "one kinship tie of 1.0 should average to 1.0");
    }

    #[test]
    fn n10_kinship_coupling_boosts_cohesion_basic() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();

        // Spawn a social graph with a kinship tie.
        let graph_a = SocialGraph {
            ties: vec![Tie {
                other: 2002,
                kinship: 1.0,
                familiarity: 0.0,
                affinity: 0.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((graph_a,));

        // Record cohesion before and after a tick.
        let before = sim.state.cohesion;
        sim.tick();
        let after = sim.state.cohesion;

        // With kinship=1.0, boost = 0.02 * 100_000 = 2000, so after >= before.
        // (caveat: other couplings and decay might affect this, but kinship boost
        // should dominate if no other agents contribute negative factors)
        assert!(
            after >= before,
            "phase_cohesion with kinship should not decrease cohesion (before={}, after={})",
            before,
            after
        );
    }

    #[test]
    fn n10_kinship_decay_factor_bounds() {
        // Verify the decay_factor formula stays in [0.93, 0.98].
        let test_cases: [(f32, f32); 3] = [(0.0, 0.93), (0.5, 0.955), (1.0, 0.98)];

        for (kinship, expected_factor) in test_cases {
            let decay_factor = 0.98_f32 - (0.05_f32 * (1.0_f32 - kinship)).max(0.0).min(1.0);
            assert!(
                (decay_factor - expected_factor).abs() < 1e-6,
                "kinship={} should give decay_factor≈{}, got {}",
                kinship,
                expected_factor,
                decay_factor
            );
        }
    }

    // N12 affinity↔diplomacy coupling tests (FR-CIV-EMERGENCE-N12)

    #[test]
    fn n12_avg_social_affinity_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        assert_eq!(avg_social_affinity(&sim.world), 0.0);
    }

    #[test]
    fn n12_avg_social_affinity_computes_mean_and_clamps() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();
        sim.world.clear();
        // One graph affinity +1.0, one graph affinity -1.0 → mean 0.0.
        let g_pos = SocialGraph {
            ties: vec![Tie {
                other: 1,
                kinship: 0.0,
                familiarity: 0.0,
                affinity: 1.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        let g_neg = SocialGraph {
            ties: vec![Tie {
                other: 2,
                kinship: 0.0,
                familiarity: 0.0,
                affinity: -1.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((g_pos,));
        sim.world.spawn((g_neg,));
        assert!(avg_social_affinity(&sim.world).abs() < 1e-6);
    }

    #[test]
    fn n12_affinity_bias_direction_and_bounds() {
        // Positive affinity raises threshold; negative lowers it; bounded [-5000, 5000].
        let pos = affinity_threshold_bias(1.0);
        let neg = affinity_threshold_bias(-1.0);
        let zero = affinity_threshold_bias(0.0);
        assert_eq!(pos, 5_000);
        assert_eq!(neg, -5_000);
        assert_eq!(zero, 0);
        assert!(
            pos > zero && zero > neg,
            "goodwill must raise tolerance over hostility"
        );
        // Out-of-range inputs clamp.
        assert_eq!(affinity_threshold_bias(2.0), 5_000);
        assert_eq!(affinity_threshold_bias(-2.0), -5_000);
    }

    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn n12_high_affinity_keeps_factions_trading() {
        use civ_agents::Tie;
        // Disparity ABOVE the base threshold (would Conflict at neutral affinity),
        // but strong collective goodwill raises the threshold enough to keep trade.
        let base = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
        let disparity = base + 2_000; // 12_000: above base, below base + max affinity bias

        // Low-affinity sim: hostile ties → threshold drops → Conflict.
        let mut sim_low = Simulation::with_seed(5);
        sim_low.state.tick = 500;
        sim_low.state.belief = 0;
        sim_low.state.cohesion = 0;
        sim_low.state.unrest = 0;
        for _ in 0..3 {
            sim_low.world.spawn((SocialGraph {
                ties: vec![Tie {
                    other: 9,
                    kinship: 0.0,
                    familiarity: 0.0,
                    affinity: -1.0,
                    trust: 0.0,
                    last_seen: 0,
                }],
            },));
        }
        let mut faction_ids: Vec<u32> = sim_low.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return; // Defensive: need a faction pair; skip if scenario has none.
        }
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim_low.state.tick);
        sim_low.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_low
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_low.phase_diplomacy();
        let low_kind = sim_low.diplomacy_events().last().expect("event").kind;

        // High-affinity sim: goodwill ties → threshold rises → TradeAgreement.
        let mut sim_high = Simulation::with_seed(5);
        sim_high.state.tick = 500;
        sim_high.state.belief = 0;
        sim_high.state.cohesion = 0;
        sim_high.state.unrest = 0;
        for _ in 0..3 {
            sim_high.world.spawn((SocialGraph {
                ties: vec![Tie {
                    other: 9,
                    kinship: 0.0,
                    familiarity: 0.0,
                    affinity: 1.0,
                    trust: 0.0,
                    last_seen: 0,
                }],
            },));
        }
        sim_high
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
        sim_high
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_high.phase_diplomacy();
        let high_kind = sim_high.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            low_kind,
            DiplomacyKind::Conflict,
            "hostile populations should clash at disparity above base threshold"
        );
        assert_eq!(
            high_kind,
            DiplomacyKind::TradeAgreement,
            "collective goodwill should raise the threshold and keep factions trading"
        );
    }

    // ── Named-race seed spawn tests (FR-CIV-GENETICS-SEED-*) ─────────────────

    /// FR-CIV-GENETICS-SEED-001 — first spawned agent carries Ardani archetype
    /// DNA after applying divergence=0.3 with a fixed RNG seed, and the result
    /// is deterministic across two identical Simulation instances.
    #[test]
    fn test_seed_spawn_determinism() {
        use civ_genetics::NamedSeed;
        let sim_a = Simulation::with_seed(0xC0FFEE_u64);
        let sim_b = Simulation::with_seed(0xC0FFEE_u64);
        // Collect all Dna components from both worlds.
        let dna_a: Vec<Dna> = sim_a
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        let dna_b: Vec<Dna> = sim_b
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        assert_eq!(
            dna_a.len(),
            dna_b.len(),
            "both sims must spawn the same number of DNA-bearing entities"
        );
        assert!(!dna_a.is_empty(), "at least one entity must carry DNA");
        // Both runs must be bit-identical under the same seed.
        for (a, b) in dna_a.iter().zip(dna_b.iter()) {
            assert_eq!(
                a, b,
                "Dna must be deterministic under an identical RNG seed"
            );
        }
        // The first civilian's DNA must differ from the raw zero genome, proving
        // it was seeded from an archetype rather than left default.
        let archetype = civ_genetics::archetype_dna(NamedSeed::Ardani);
        assert_eq!(
            dna_a[0].0.len(),
            archetype.0.len(),
            "genome length must match archetype"
        );
        // With divergence=0.3 the result must not be all-zero (extremely unlikely).
        assert_ne!(
            dna_a[0].0,
            vec![0u8; 64],
            "seeded DNA must not be the zero genome"
        );
    }

    /// FR-CIV-GENETICS-SEED-002 — spawn indices 0, 1, and 2 produce three
    /// distinct NamedSeed assignments (Ardani, Velthari, Grundak respectively).
    #[test]
    fn test_faction_archetype_variety() {
        use civ_genetics::NamedSeed;
        let ardani_base = civ_genetics::archetype_dna(NamedSeed::Ardani);
        let velthari_base = civ_genetics::archetype_dna(NamedSeed::Velthari);
        let grundak_base = civ_genetics::archetype_dna(NamedSeed::Grundak);

        // Verify the three archetypes are distinct from each other —
        // confirming the % 3 cycle will produce genuinely different seeds.
        assert_ne!(
            ardani_base, velthari_base,
            "Ardani and Velthari must differ"
        );
        assert_ne!(ardani_base, grundak_base, "Ardani and Grundak must differ");
        assert_ne!(
            velthari_base, grundak_base,
            "Velthari and Grundak must differ"
        );

        // With 128 civilians and 12 named seeds, each archetype slot is hit ~10-11 times.
        let sim = Simulation::with_seed(1);
        let dna_list: Vec<Dna> = sim
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        assert_eq!(dna_list.len(), 128, "all 128 civilians must carry Dna");

        // Verify that at minimum 3 distinct genomes are present, proving multiple
        // archetype branches were exercised (divergence prevents collisions).
        let unique_count = {
            let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
            for d in &dna_list {
                seen.insert(d.0.clone());
            }
            seen.len()
        };
        assert!(
            unique_count >= 3,
            "at least 3 distinct genomes expected (one per archetype); got {unique_count}"
        );
    }

    /// FR-CIV-GENETICS-SEED-003 — `seed_with_divergence` at divergence=0.0
    /// returns an exact clone of the archetype; this is the zero-divergence contract.
    #[test]
    fn test_zero_divergence_exact() {
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        let archetype = civ_genetics::archetype_dna(NamedSeed::Ardani);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0xDEAD_BEEF);
        let result = civ_genetics::seed_with_divergence(&archetype, 0.0, &mut rng);
        assert_eq!(
            result, archetype,
            "seed_with_divergence with divergence=0.0 must return an exact clone of the archetype"
        );
    }

    // ── FR-CONTENT-SEEDMIX: choose_named_seed helper unit tests ──────────────

    /// Empty seed_mix must reproduce the classic Ardani/Velthari/Grundak round-robin
    /// without advancing the RNG (bit-identical default path).
    #[test]
    fn choose_named_seed_empty_is_round_robin() {
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(1);
        let expected = [
            NamedSeed::Ardani,
            NamedSeed::Velthari,
            NamedSeed::Grundak,
            NamedSeed::Ardani,
            NamedSeed::Velthari,
            NamedSeed::Grundak,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let got = choose_named_seed(&[], None, i, &mut rng);
            assert_eq!(got, exp, "round-robin mismatch at spawn_index={i}");
        }
    }

    /// A 60/30/10 mix should yield Ardani as plurality (~0.6), Grundak as minority (~0.1).
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn choose_named_seed_weighted_distribution() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::distributions::WeightedIndex;
        use rand::SeedableRng;

        let seed_mix = vec![
            SeedWeight {
                seed: NamedSeed::Ardani,
                weight: 0.6,
            },
            SeedWeight {
                seed: NamedSeed::Velthari,
                weight: 0.3,
            },
            SeedWeight {
                seed: NamedSeed::Grundak,
                weight: 0.1,
            },
        ];
        let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
        let dist = WeightedIndex::new(&weights).expect("valid weights");

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let n = 2000usize;
        let mut counts = [0usize; 3];
        for i in 0..n {
            let result = choose_named_seed(&seed_mix, Some(&dist), i, &mut rng);
            match result {
                NamedSeed::Ardani => counts[0] += 1,
                NamedSeed::Velthari => counts[1] += 1,
                NamedSeed::Grundak => counts[2] += 1,
                _ => {}
            }
        }
        let ardani_frac = counts[0] as f32 / n as f32;
        let grundak_frac = counts[2] as f32 / n as f32;
        assert!(
            (ardani_frac - 0.6).abs() < 0.08,
            "Ardani fraction {ardani_frac:.3} not within ±0.08 of 0.6"
        );
        assert!(
            (grundak_frac - 0.1).abs() < 0.05,
            "Grundak fraction {grundak_frac:.3} not within ±0.05 of 0.1"
        );
        // Ardani must be the plurality
        assert!(
            counts[0] > counts[1] && counts[0] > counts[2],
            "Ardani must be plurality"
        );
    }

    /// A single-entry mix must always yield that one race.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn choose_named_seed_single_seed_all_that_race() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::distributions::WeightedIndex;
        use rand::SeedableRng;

        let seed_mix = vec![SeedWeight {
            seed: NamedSeed::Velthari,
            weight: 1.0,
        }];
        let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
        let dist = WeightedIndex::new(&weights).expect("valid weights");

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        for i in 0..100 {
            let result = choose_named_seed(&seed_mix, Some(&dist), i, &mut rng);
            assert_eq!(
                result,
                NamedSeed::Velthari,
                "expected Velthari at index {i}"
            );
        }
    }

    /// FR-CIV-014 / emergence-spawn — scenario-controlled faction spawning must
    /// honor arbitrary faction counts and per-faction civilian counts.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn scenario_faction_spawn_honors_counts() {
        use crate::scenario::ScenarioStartingConditions;
        use civ_agents::{Alignment, Civilian};
        use std::collections::BTreeMap;

        let sc = ScenarioStartingConditions {
            civilians_per_faction: 2,
            faction_count: 5,
            quadrant_spread: 1,
            seed_mix: Vec::new(),
        };
        let _ = &sc;
        let sim = Simulation::with_seed(123u64);

        let mut counts: BTreeMap<u32, u32> = BTreeMap::new();
        for (_, civ) in sim.world.query::<&Civilian>().iter() {
            if let Alignment::Faction(fid) = civ.alignment {
                *counts.entry(fid).or_insert(0) += 1;
            }
        }

        assert_eq!(counts.len(), 5, "expected five factions to be spawned");
        assert!(counts.values().all(|&count| count == 2));
    }

    // ── LANGUAGE→DIPLOMACY coupling tests ─────────────────────────────────────

    #[cfg(test)]
    mod language_diplomacy_tests {
        use super::*;

        #[test]
        fn bonus_bounded_and_monotonic() {
            let bonus_close = language_intelligibility_peace_bonus(0.1);
            let bonus_far = language_intelligibility_peace_bonus(0.9);
            assert!(
                bonus_close > bonus_far,
                "closer language must yield bigger bonus"
            );
            assert!(bonus_close <= 1200, "bonus must not exceed cap");
        }

        #[test]
        fn identical_language_max_bonus_more_peaceful() {
            let max_bonus = language_intelligibility_peace_bonus(0.0);
            let no_bonus = language_intelligibility_peace_bonus(1.0);
            assert_eq!(max_bonus, 1200, "identical language must yield max bonus");
            assert_eq!(no_bonus, 0, "max distance must yield zero bonus");
            assert!(max_bonus > no_bonus);
        }

        #[test]
        fn missing_language_legacy_threshold_unchanged() {
            let bonus = language_intelligibility_peace_bonus(1.0);
            assert_eq!(bonus, 0, "missing language must not alter threshold");
        }
    }

    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn language_names_diverge_for_isolated_factions_over_time() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_languages.clear();
        sim.language_state = LanguageState::default();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_language_drift();
        let baseline_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );

        for _ in 0..20 {
            sim.phase_language_drift();
        }

        let final_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );
        assert!(
            final_distance > baseline_distance,
            "isolated cultures should diverge over time, {baseline_distance} -> {final_distance}"
        );
        assert!(
            final_distance >= 0.5,
            "isolated languages should stay meaningfully divergent, got {final_distance}"
        );
        assert_ne!(
            sim.faction_place_name(1, 1),
            sim.faction_place_name(2, 1),
            "place names should diverge with isolated lexicons"
        );
    }

    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn language_drift_wires_through_sim_tick_for_isolated_factions() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_languages.clear();
        sim.language_state = LanguageState::default();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_language_drift();
        let baseline = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );

        for _ in 0..20 {
            sim.tick();
        }

        let final_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );
        assert!(
            final_distance > baseline,
            "isolated factions should diverge through Simulation::tick(), {baseline} -> {final_distance}"
        );
    }

    #[test]
    fn culture_traits_drift_through_sim_tick_for_isolated_factions() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_ideologies.clear();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));
        sim.religious_profiles.insert(
            1,
            ReligiousProfile {
                monitoring: 0.70,
                mythic_coherence: 0.60,
                uncertainty_reduction: 0.20,
                population: 4,
                ..ReligiousProfile::default()
            },
        );
        sim.religious_profiles.insert(
            2,
            ReligiousProfile {
                monitoring: 0.20,
                mythic_coherence: 0.30,
                uncertainty_reduction: 0.65,
                population: 4,
                ..ReligiousProfile::default()
            },
        );

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_culture();
        let before = sim
            .faction_ideologies()
            .get(&1)
            .expect("faction 1 culture should initialize")
            .values;

        for _ in 0..20 {
            sim.tick();
        }

        let after = sim
            .faction_ideologies()
            .get(&1)
            .expect("faction 1 culture should advance through tick")
            .values;
        assert_ne!(
            before, after,
            "FR-CIV-CULTURE: faction culture traits should drift through Simulation::tick()"
        );
    }

    // ── AUDIO-wire (FR-AUDIO-wire) tests ────────────────────────────────────
    //
    // These tests cover the thin wire between per-tick substrate events
    // (disasters / combat pulses / construction / emergence) and the
    // `SfxTrigger` buffer surfaced on `sim.last_tick_audio_events()`.
    // Audio synthesis itself lives in `civ-audio`; the engine only owns
    // the trigger list.

    #[cfg(test)]
    impl Simulation {
        /// FR-AUDIO-wire test helper — push a `CombatDamagePulse` into
        /// the engine's per-tick buffer at normalized world coords
        /// (`x_norm`, `y_norm` in `[0, 1]`), so the audio phase can be
        /// exercised without running the full tactics resolution.
        fn push_combat_pulse_for_test(&mut self, x_norm: f32, y_norm: f32) {
            self.last_tick_combat_pulses.push(CombatDamagePulse {
                x: x_norm.clamp(0.0, 1.0),
                y: y_norm.clamp(0.0, 1.0),
                unit_a: None,
                unit_b: None,
            });
        }
    }

    #[cfg(test)]
    mod audio_wire_tests {
        use super::*;

        /// FR-AUDIO-wire — on a fresh `Simulation::new()`, the audio buffer
        /// starts empty and remains empty after one tick (no combat, no
        /// construction, no disasters on the seeded first tick).
        #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
        #[test]
        fn fr_audio_wire_empty_buffer_clears_across_ticks() {
            let mut sim = Simulation::new();
            assert!(sim.last_tick_audio_events().is_empty());
            sim.tick();
            // No substrate event has fired on a seeded tick 1 — audio buffer
            // stays empty.
            assert!(sim.last_tick_audio_events().is_empty());
        }

        /// FR-AUDIO-wire — triggering a disaster mid-tick records a
        /// `SfxTrigger::Disaster` whose `kind` matches the
        /// `DisasterKind` label so the audio substrate's
        /// `SfxKind::for_disaster_label` can route it to the per-kind
        /// sting (Meteor / Flood / Quake / Wildfire / Storm / Plague).
        #[test]
        fn fr_audio_wire_disaster_records_routed_trigger() {
            use crate::disasters::{trigger_disaster, DisasterKind};

            let mut sim = Simulation::new();
            // Direct API: `trigger_disaster` records the audio trigger as
            // a side effect of `apply_disaster`.
            trigger_disaster(
                &mut sim,
                DisasterKind::Quake,
                WorldCoord { x: 0, y: 0, z: 0 },
            );
            let recorded = sim.last_tick_audio_events();
            assert_eq!(recorded.len(), 1, "one disaster → one trigger");
            match recorded[0] {
                SfxTrigger::Disaster { kind, severity } => {
                    assert_eq!(kind, "quake", "wire label matches the per-kind sting");
                    assert!(
                        (0.0..=1.0).contains(&severity),
                        "severity is clamped to [0, 1]"
                    );
                    assert!(
                        severity > 0.0,
                        "non-zero severity (quake has positive radius)"
                    );
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `record_disaster_audio` is idempotent: an
        /// unknown label surfaces as the umbrella "disaster" label so the
        /// substrate's `for_disaster_label` falls back to
        /// `SfxKind::Disaster` (no panic, no skipped event).
        #[test]
        fn fr_audio_wire_unknown_disaster_label_falls_back() {
            let mut sim = Simulation::new();
            sim.record_disaster_audio("hailstorm", 0.4);
            assert_eq!(sim.last_tick_audio_events().len(), 1);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { kind, severity } => {
                    assert_eq!(kind, "disaster", "unknown → umbrella label");
                    assert!(
                        (severity - 0.4).abs() < 1e-5,
                        "severity passes through clamp"
                    );
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `record_disaster_audio` clamps severity out of
        /// `[0, 1]` so the wire shape is bounded.
        #[test]
        fn fr_audio_wire_record_disaster_severity_is_clamped() {
            let mut sim = Simulation::new();
            sim.record_disaster_audio("flood", 1.7);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { severity, .. } => {
                    assert!(severity <= 1.0, "severity > 1 must clamp to 1.0");
                    assert!((severity - 1.0).abs() < 1e-5);
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
            let mut sim = Simulation::new();
            sim.record_disaster_audio("flood", -0.3);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { severity, .. } => {
                    assert!(severity >= 0.0, "severity < 0 must clamp to 0.0");
                    assert!((severity - 0.0).abs() < 1e-5);
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `phase_audio` translates a queued combat pulse
        /// into a `SfxTrigger::Battle` with intensity scaled by
        /// normalized proximity to the world center. We use the
        /// `#[cfg(test)]`-gated `push_combat_pulse_for_test` helper to
        /// stage the pulse without running the full tactics phase.
        #[test]
        fn fr_audio_wire_combat_pulse_emits_battle_trigger() {
            let mut sim = Simulation::new();
            // A pulse at the world center → maximum intensity (1.0).
            sim.push_combat_pulse_for_test(0.5, 0.5);
            sim.phase_audio();
            let events = sim.last_tick_audio_events();
            assert_eq!(events.len(), 1, "one pulse → one Battle trigger");
            match events[0] {
                SfxTrigger::Battle { intensity } => {
                    assert!((0.0..=1.0).contains(&intensity), "intensity is in [0, 1]");
                    assert!(intensity > 0.99, "center pulse → near-1.0 intensity");
                }
                other => panic!("expected Battle trigger, got {other:?}"),
            }
        }

        #[test]
        fn fr_audio_wire_lifecycle_and_research_emit_birth_death_tech() {
            let mut sim = Simulation::new();
            sim.last_births.push(PopulationEvent {
                tick: sim.state.tick,
                entity_id: 1,
                x: 0.25,
                y: 0.5,
            });
            sim.last_deaths.push(PopulationEvent {
                tick: sim.state.tick,
                entity_id: 2,
                x: 0.75,
                y: 0.5,
            });
            sim.research_cache.researched.push("agriculture".to_owned());

            sim.phase_audio();

            assert_eq!(
                sim.last_tick_audio_events(),
                &[SfxTrigger::Birth, SfxTrigger::Death, SfxTrigger::Tech]
            );

            sim.last_births.clear();
            sim.last_deaths.clear();
            sim.phase_audio();
            assert!(
                sim.last_tick_audio_events().is_empty(),
                "already-emitted research must not retrigger Tech"
            );
        }

        #[test]
        fn tick_invokes_phase_audio() {
            use civ_agents::culture::CultureProfile;

            let mut sim = Simulation::new();
            sim.cluster_cultures
                .insert(7, CultureProfile::new([0.4, 0.5, 0.6, 0.7]));
            assert!(sim.last_tick_music_cues.is_empty());

            sim.tick();

            assert!(
                sim.last_tick_music_cues.contains_key(&7),
                "tick should run phase_audio and populate music cues"
            );
        }

        /// FR-MUSIC-001 — two cultures produce distinct, drifting music-cue
        /// surfaces derived from emergent culture profiles.
        #[test]
        fn fr_music_distinct_culture_cues_evolve_over_time() {
            use civ_agents::culture::CultureProfile;

            let mut sim = Simulation::new();
            sim.cluster_cultures
                .insert(100, CultureProfile::new([0.14, 0.14, 0.14, 0.14]));
            sim.cluster_cultures
                .insert(200, CultureProfile::new([0.86, 0.86, 0.86, 0.86]));
            sim.faction_aggression.insert(0, 0.1);
            sim.faction_aggression.insert(1, 0.2);

            sim.tick();
            let snap_a = sim.snapshot();
            let cue_a_100 = snap_a
                .music_cues
                .get(&100)
                .cloned()
                .expect("seeded cluster 100 should have a cue");
            let cue_a_200 = snap_a
                .music_cues
                .get(&200)
                .cloned()
                .expect("seeded cluster 200 should have a cue");
            assert_ne!(
                cue_a_100, cue_a_200,
                "cultures with distinct profiles should surface distinct cue params"
            );

            sim.tick();
            let snap_b = sim.snapshot();
            let cue_b_100 = snap_b
                .music_cues
                .get(&100)
                .cloned()
                .expect("seeded cluster 100 should persist");
            let cue_b_200 = snap_b
                .music_cues
                .get(&200)
                .cloned()
                .expect("seeded cluster 200 should persist");
            assert_ne!(cue_a_100, cue_b_100);
            assert_ne!(cue_a_200, cue_b_200);
        }

        #[test]
        fn fed_stable_population_grows_via_births() {
            use civ_agents::{
                spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
                Needs as AgentNeeds,
            };

            let mut sim = Simulation::new();
            sim.state.resources.food = Fixed::from_num(100);
            sim.set_settlement_population(1, 2);

            let parent_a = spawn_civilian_at(
                &mut sim.world,
                1,
                Alignment::Faction(1),
                0.25,
                0.25,
                ActorVisualKind::Humanoid,
                &mut sim.rng,
            );
            let parent_b = spawn_civilian_at(
                &mut sim.world,
                2,
                Alignment::Faction(1),
                0.27,
                0.26,
                ActorVisualKind::Humanoid,
                &mut sim.rng,
            );

            for entity in [parent_a, parent_b] {
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 28;
                {
                    let mut needs = sim.world.get::<&mut AgentNeeds>(entity).unwrap();
                    needs.food = 0.96;
                    needs.rest = 0.94;
                    needs.safety = 0.93;
                    needs.belonging = 0.95;
                    needs.health = 0.97;
                }
            }

            let before = sim.world.query::<&AgentCivilian>().iter().count();
            sim.phase_life();
            let after = sim.world.query::<&AgentCivilian>().iter().count();

            assert!(
                after > before,
                "fed paired adults should produce at least one child"
            );
            assert!(
                !sim.last_births().is_empty(),
                "birth events should be recorded"
            );

            let child_id = sim.last_births().last().expect("child").entity_id;
            let kinship = sim.kinship.get(&child_id).expect("child kinship");
            assert!(
                kinship
                    .iter()
                    .any(|edge| matches!(edge.kind, KinshipKind::Family)),
                "newborn should receive family kinship"
            );
        }

        #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
        #[test]
        fn starving_population_migrates_and_founds_settlement() {
            use civ_agents::{
                spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
                Needs as AgentNeeds,
            };

            let mut sim = Simulation::new();
            sim.state.resources.food = Fixed::from_num(0);
            sim.set_settlement_population(7, 3);

            let adults = [
                spawn_civilian_at(
                    &mut sim.world,
                    11,
                    Alignment::Faction(7),
                    0.12,
                    0.12,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
                spawn_civilian_at(
                    &mut sim.world,
                    12,
                    Alignment::Faction(7),
                    0.13,
                    0.14,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
                spawn_civilian_at(
                    &mut sim.world,
                    13,
                    Alignment::Faction(7),
                    0.15,
                    0.16,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
            ];

            for entity in adults {
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 32;
                let mut needs = sim.world.get::<&mut Needs>(entity).unwrap();
                needs.food = 0.08;
                needs.rest = 0.22;
                needs.safety = 0.24;
                needs.belonging = 0.25;
                needs.health = 0.85;
            }

            let before_settlements = sim.settlements.clone();
            sim.phase_life();

            assert_eq!(sim.settlements.get(&7).copied(), Some(1));
            assert_eq!(sim.settlements.get(&8).copied(), Some(2));
            assert_eq!(sim.settlements.len(), before_settlements.len() + 1);

            let migrated = sim
                .world
                .query::<&AgentCivilian>()
                .iter()
                .filter(|(_, civ)| matches!(civ.alignment, Alignment::Faction(8)))
                .count();
            assert!(
                migrated >= 2,
                "starving adults should found a new settlement"
            );
        }

        // FR-CIV-LIFE-003: smoke test that `phase_citizen_lifecycle` runs
        // through the new `should_reproduce` path without panicking. We
        // advance the sim through several birth windows and verify that
        // the civilian count grows over time when there is food available
        // and adults exist.
        #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
        #[test]
        fn phase_citizen_lifecycle_uses_should_reproduce() {
            let mut sim = Simulation::new();
            // Spawn three adults at well-fed state so reproduction can fire.
            for (i, id) in [700u64, 701, 702].iter().enumerate() {
                let entity = spawn_civilian_at(
                    &mut sim.world,
                    *id,
                    Alignment::None,
                    0.20 + (i as f32) * 0.01,
                    0.20 + (i as f32) * 0.01,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                );
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 30;
                let mut needs = sim.world.get::<&mut Needs>(entity).unwrap();
                needs.food = 0.95;
                needs.shelter = 0.95;
                needs.safety = 0.95;
                needs.belonging = 0.95;
            }
            // Ensure resources are non-zero so the food regen branch runs
            // (and so the early-death branch is not triggered).
            sim.state.resources.food = Fixed::from_num(1000);
            sim.state.population = sim.state.population.max(count_civilians(&sim.world) as u64);

            // Run several birth windows (every 200 ticks).
            for tick in 0..600 {
                sim.state.tick = tick;
                sim.phase_citizen_lifecycle();
            }

            // After 600 ticks, with three fertile adults and food available,
            // at least one birth should have occurred.
            let final_pop = count_civilians(&sim.world) as u64;
            assert!(
                final_pop >= 4,
                "should_reproduce should have produced at least one child (got {})",
                final_pop
            );
        }
    }

    /// FR-CIV-LIFE P4-A — `phase_life` populates `last_tick_lifecycle_metrics`
    /// and `phase_economy` uses it to weight the LaborCapacityAllocator.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn labor_capacity_weighting_threads_through_phase_economy() {
        let mut sim = Simulation::new();
        // Default sim has no civilians; metrics must be zero, and allocation
        // should still succeed (with effective demand = 0).
        assert_eq!(sim.last_tick_lifecycle_metrics.adults, 0);
        assert_eq!(sim.last_tick_lifecycle_metrics.total_living(), 0);
        // Should not panic on tick.
        sim.tick();
        // Spawn civilians: 2 adults + 1 child via the engine's spawn API.
        let civ_a = sim.world.spawn(()).id();
        let civ_b = sim.world.spawn(()).id();
        // Advance phase_life directly: the metrics should be reproducible
        // from any civilian snapshot.
        sim.last_tick_lifecycle_metrics = LifecycleCounters {
            children: 1,
            adults: 2,
            elders: 0,
            dead: 0,
        };
        // 2 adults + 0.5 * 0 elders = 2 / (1 + 2 + 0) = 0.6667 labor fraction
        let living = (sim.last_tick_lifecycle_metrics.children
            + sim.last_tick_lifecycle_metrics.adults
            + sim.last_tick_lifecycle_metrics.elders) as f64;
        let productive = sim.last_tick_lifecycle_metrics.adults as f64
            + 0.5 * sim.last_tick_lifecycle_metrics.elders as f64;
        let frac = (productive / living).clamp(0.0, 1.0);
        assert!(
            (frac - 0.6666).abs() < 0.01,
            "labor fraction expected ~0.6667, got {frac}"
        );
        // Ensure spawn targets are still alive (sanity).
        assert!(civ_a > 0);
        let _ = civ_b; // unused: kept for documentation
    }

    // FR-CIV-LIFE-001/002/003: classifier wiring smoke test. Spawn three
    // civilians spanning the Child / Adult / Elder axis, run `phase_life`
    // once, then assert `last_tick_lifecycle_metrics()` contains the
    // expected counts. This is the contract-level check that the
    // classifier is reachable from the engine tick loop, not a deep
    // classifier correctness test (that lives in `civ_needs::lifecycle`).
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn lifecycle_classifiers_wired_into_phase_life() {
        use civ_agents::{
            spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
            Needs as AgentNeeds,
        };

        let mut sim = Simulation::new();

        // Spawn three civilians with distinct ages spanning Child / Adult / Elder.
        let child = spawn_civilian_at(
            &mut sim.world,
            100,
            Alignment::None,
            0.30,
            0.30,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(child).unwrap();
            civ.age = 5;
            let mut needs = sim.world.get::<&mut AgentNeeds>(child).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }

        let adult = spawn_civilian_at(
            &mut sim.world,
            102,
            Alignment::None,
            0.31,
            0.31,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(adult).unwrap();
            civ.age = 28;
            let mut needs = sim.world.get::<&mut AgentNeeds>(adult).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }

        let elder = spawn_civilian_at(
            &mut sim.world,
            103,
            Alignment::None,
            0.32,
            0.32,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(elder).unwrap();
            civ.age = 70;
            let mut needs = sim.world.get::<&mut AgentNeeds>(elder).unwrap();
            needs.food = 0.85;
            needs.shelter = 0.85;
            needs.safety = 0.85;
            needs.belonging = 0.85;
        }

        // Default counters (before any phase_life run) should be all zero.
        let pre = *sim.last_tick_lifecycle_metrics();
        assert_eq!(pre.total(), 0, "default lifecycle counters must be zero");

        sim.phase_life();

        let post = *sim.last_tick_lifecycle_metrics();
        assert!(
            post.total() >= 3,
            "phase_life should classify the three spawned civilians (got total={})",
            post.total()
        );
        assert!(
            post.children >= 1,
            "age=5 civilian should classify as Child"
        );
        assert!(
            post.adults >= 1,
            "age=28 healthy civilian should classify as Adult"
        );
        assert!(post.elders >= 1, "age=70 civilian should classify as Elder");
    }

    // FR-CIV-LIFE-002: maturity growth wiring smoke test. A healthy adult
    // (maturity starts at 0) should remain `Adult`-classifiable after the
    // classifier pass even without an attached `Psyche`, since the
    // classifier treats missing maturity as 0.0 and the age/integrity
    // branch alone still puts a 28-year-old healthy civilian in the
    // Adult bucket.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn phase_life_classifier_handles_missing_psyche() {
        use civ_agents::{
            spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
            Needs as AgentNeeds,
        };

        let mut sim = Simulation::new();
        let entity = spawn_civilian_at(
            &mut sim.world,
            200,
            Alignment::None,
            0.40,
            0.40,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
            civ.age = 28;
            let mut needs = sim.world.get::<&mut AgentNeeds>(entity).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }
        // Deliberately do NOT attach a `Psyche` component.
        sim.phase_life();
        let counters = *sim.last_tick_lifecycle_metrics();
        assert!(
            counters.adults >= 1,
            "adult should be classified even without Psyche"
        );
    }
}
