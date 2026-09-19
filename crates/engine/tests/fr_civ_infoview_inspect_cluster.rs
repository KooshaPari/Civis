//! FR-CIV-INFOVIEW-905 / -911 / -912 / -914 and FR-CIV-INSPECT-902 / -920 -
//! real behavioural oracles.
//!
//! Replaces the auto-generated placeholders
//! (`fr_fr_civ_infoview_905.rs`, `.._911.rs`, `.._912.rs`, `.._914.rs`,
//! `fr_fr_civ_inspect_902.rs`, `fr_fr_civ_inspect_920.rs`), which asserted only
//! `WorldState::default().tick == 0` and therefore could not fail if any of
//! these requirements regressed.
//!
//! Requirement text: `docs/specs/requirements/FR-CIV-INFOVIEW.md`,
//! `docs/specs/requirements/FR-CIV-INSPECT.md`, `docs/design/info-views.md`
//! (`section 4` priority table + `section 9` traceability), `docs/design/onboarding-qol.md` section 6.
//!
//! ## What each ID is anchored to (engine-reachable surface)
//!
//! * **FR-CIV-INFOVIEW-905** - "BLIND overlays are registered and
//!   availability-gated; greyed in panel, never fabricate data, auto-activate
//!   on data presence." The *panel* half of that lives in `crates/hud`
//!   (`overlay_registry`, `env_overlay`) and the clients, neither of which is
//!   a `civ-engine` dependency. What is engine-reachable - and what the panel
//!   binds to - is the measurement availability contract: a data source with
//!   no measurement is surfaced as *absent* (`Option::None` / empty collection),
//!   never as a zeroed frame, and it lights up on the tick *after* its data
//!   appears without any re-registration.
//! * **FR-CIV-INFOVIEW-911** - "Resource overlays: natural resources
//!   (ore/wood/fertile/energy), production/supply flow ... deposits +
//!   utilization shown." Engine surface: the voxel material field (the deposit
//!   substrate) read back through the read-only `inspect.probe` query, the
//!   measured faction `Resources` stock (`resource_stock_units`), the
//!   per-cluster stockpile map (`cluster_stocks`), and the per-tick settlement
//!   supply flow (`last_tick_settlement_trade_flows`).
//! * **FR-CIV-INFOVIEW-912** - "Population & well-being overlays: population
//!   density, age, happiness, wealth, health, education ... per-cell
//!   aggregation." Engine surface: the operational hex cell
//!   (`lod::operational_hex_snapshot`) + strategic rollup
//!   (`lod::aggregate_strategic`), the per-settlement well-being frames
//!   (`last_tick_mood` / `last_tick_mood_all`, whose sub-scores are documented
//!   functions of measured inputs), and the age/lifecycle counters
//!   (`last_tick_lifecycle_metrics`).
//! * **FR-CIV-INFOVIEW-914** - "Infrastructure overlays: roads/traffic,
//!   building level, service coverage, transport lines ... reads emergent
//!   architecture/road graph." Engine surface: the building tier engine
//!   (`building_tiers`, the "building level" datum), the layout coverage model
//!   (`building_layouts`: capacity, per-tick infrastructure needs, road
//!   adjacency requirement, efficiency/coverage falloff), and the emergent
//!   facade/parcel assignment (`building_emergence::apply_emergence_facades`
//!   over `civ-build`'s deterministic cluster layout).
//! * **FR-CIV-INSPECT-902** - "Settlement/polity inspector SHALL show emergent
//!   membership (cluster overlap, NOT a faction id), population, economy
//!   summary, culture/ideology, dominant language." Engine surface: cluster
//!   identity is a `u64` `ClusterId` derived from co-location with no faction
//!   meaning (`civ_agents::ClusterMember`, `culture_traits_for_cluster`,
//!   `settlement_count`, `cluster_stocks`), population is the measured
//!   settlement registration (recoverable exactly out of the well-being
//!   frame), the economy summary is the per-cluster stockpile map (keyed by
//!   cluster, not faction), and culture/ideology is `faction_ideologies`.
//! * **FR-CIV-INSPECT-920** - "Inspector SHALL support follow-cam (lock camera
//!   to selected agent) and a 'trace lineage/history' jump." Engine surface:
//!   selection -> live entity resolution (`agent_entity`), which is what
//!   follow-cam locks onto and must never resolve to a stale/aliased entity,
//!   plus the chronicle/timeline jump targets (`history::{EraHistory, Timeline,
//!   HistoryLog}`) and the lineage walk (`language::LanguageFamilyTree`).
//!
//! ## Gaps found while writing this file (reported, not papered over)
//!
//! 1. **FR-CIV-INFOVIEW-911** - the requirement names `civ-laws` as a data
//!    source; `civ-engine` has no `civ-laws` dependency, and no per-cell
//!    "deposit/utilisation" accessor exists at engine level. The overlay
//!    sampler + legend live in `crates/hud` / `clients/bevy-ref`. The tests
//!    below pin the measured substrate the sampler has to read (voxel material
//!    deposits + measured stocks) and the *absence* of fabricated utilisation.
//! 2. **FR-CIV-INFOVIEW-914** - "roads/traffic" and "transport lines" have no
//!    engine representation: `civ-traffic`'s `TrafficGraph` is not a
//!    `civ-engine` dependency and there is no road graph in the engine. A road
//!    exists only as the `"road"` adjacency *requirement* string on a layout.
//! 3. **FR-CIV-INSPECT-902** - "dominant language": `faction_languages()`
//!    carries *measured* vocabulary per faction, but the readings are
//!    faction-keyed and `language::LinguaFranca` (which does carry `dominance`)
//!    is owned by no simulation state, so no settlement/polity has a dominant
//!    language reading. Also `Simulation::settlement_member_counts()` is a
//!    documented stub returning an empty map, so the membership overlap set
//!    currently has no engine accessor.
//! 4. **FR-CIV-INSPECT-920** - follow-cam camera control itself is client-side;
//!    the engine-reachable half is the agent -> entity resolution tested here.
//!    Kinship edges are recorded (`register_kinship`) but there is no public
//!    ancestor-chain query for *agent* lineage (only the language family tree
//!    exposes `get_lineage`), so the agent-lineage half of "trace lineage" is
//!    unimplemented at engine level.
//!
//! Absence guards for (1)-(4) are marked `TODO(<ID>)` so they fail loudly when
//! the missing data actually lands.

use std::collections::BTreeMap;

use civ_engine::building_emergence::{
    apply_emergence_facades, architecture_tile_sets, culture_traits_for_cluster,
    emergent_style_key_for_sim, resource_stock_units,
};
use civ_engine::building_layouts::{
    compute_efficiency, generate_layout, tick_building_layouts, validate_placement, BuildingLayout,
    LayoutCatalog,
};
use civ_engine::building_tiers::{
    BuildingTier, BuildingTierConfig, BuildingTierEngine, UpgradeError,
};
use civ_engine::godtools::{
    GodToolReceipt, GodToolRequest, InspectRequest, LifeRequest, ProbeReport, ProbeRequest,
    SpawnCivSeedRequest,
};
use civ_engine::history::{EraHistory, EventType, HistoricalEvent, HistoryLog, Timeline, TimelineEvent};
use civ_engine::language::LanguageFamilyTree;
use civ_engine::lod::{aggregate_strategic, operational_hex_snapshot, HexCellSnapshot};
use civ_engine::social_types::{MOOD_CRIME_BASE, MOOD_MAX, MOOD_MIN};
use civ_engine::{CivAge, Fixed, InstitutionKind, Simulation, WorldCoord};
use civ_build::{BuildingId, DemandSignals};
use civ_voxel::ORE;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Independent re-derivation of the documented institution bonuses that
/// `phase_social_mood` adds on top of the measured sub-scores.
fn institution_bonus(sim: &Simulation, settlement_id: u32) -> (i64, i64) {
    let mut temple = 0_i64;
    let mut garrison = 0_i64;
    for institution in sim.institutions().get(&settlement_id).into_iter().flatten() {
        match institution.kind {
            InstitutionKind::Temple => temple = 25 + 25 * i64::from(institution.level),
            InstitutionKind::Garrison => garrison = 15 + 15 * i64::from(institution.level),
        }
    }
    (temple, garrison)
}

/// Independent re-derivation of the documented mood sub-scores.
fn expected_mood(
    sim: &Simulation,
    settlement_id: u32,
    population: u32,
    housing_capacity: u32,
    crime_pressure: i32,
    food_stocked: i64,
) -> (i64, i64, i64, i64) {
    let (temple, garrison) = institution_bonus(sim, settlement_id);
    let food_score = (food_stocked / 200).clamp(MOOD_MIN, MOOD_MAX);
    let housing_score = (i64::from(housing_capacity) - i64::from(population))
        .saturating_mul(2)
        .clamp(MOOD_MIN, MOOD_MAX);
    let crime_score = (MOOD_CRIME_BASE - 4 * i64::from(crime_pressure)).clamp(0, MOOD_CRIME_BASE);
    let total = (food_score + housing_score + crime_score + temple + garrison)
        .clamp(MOOD_MIN, MOOD_MAX);
    (food_score, housing_score, crime_score, total)
}

fn probe(sim: &mut Simulation, pos: WorldCoord) -> ProbeReport {
    match sim
        .apply_god_tool(GodToolRequest::Inspect(InspectRequest::Probe(ProbeRequest {
            pos,
        })))
        .expect("inspect.probe must succeed")
    {
        GodToolReceipt::Inspect { report } => report,
        other => panic!("inspect.probe must return an Inspect receipt, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-INFOVIEW-905 - BLIND data is absent, never fabricated; activates on data
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INFOVIEW-905.
///
/// "Never fabricate data": every reading whose data source has never been
/// measured must be *absent* from the client surface, not zero-filled. If the
/// engine started emitting defaulted frames, the overlay would grey-out nothing
/// and would paint a fabricated world - the bug this test exists to catch.
#[test]
fn infoview_905_unmeasured_readings_are_absent_not_zero_filled() {
    let mut sim = Simulation::with_seed(0x905_B11D);

    // Nothing has been measured yet: no frames, no stocks, no parcels, no sample.
    assert!(
        sim.last_tick_mood_all().is_empty(),
        "no settlement is measured, so no well-being frame may exist"
    );
    assert!(sim.last_tick_mood(7).is_none());
    assert!(sim.cluster_stocks().is_empty());
    assert!(sim.building_graph().parcels.is_empty());
    assert_eq!(sim.building_graph().completed_count(), 0);
    assert!(sim.snapshot().in_progress_tech.is_none());
    assert!(sim.snapshot().researched.is_empty());
    assert!(sim.state.emergence_sample.is_none());

    for _ in 0..6 {
        sim.tick();
    }

    // Ticking more does not conjure data: a BLIND reading stays absent.
    assert!(
        sim.last_tick_mood_all().is_empty(),
        "ticking must not invent well-being frames for unmeasured settlements"
    );
    assert!(
        sim.last_tick_mood(7).is_none(),
        "field lookups must stay None for an unmeasured settlement"
    );
    assert!(
        sim.building_graph().parcels.is_empty(),
        "no parcel has been allocated, so the infrastructure overlay has nothing to draw"
    );
    assert!(
        sim.state.emergence_sample.is_none(),
        "no emergence sample has been measured on this fixture, so the reading must stay absent"
    );

    // The one measurement that *does* exist is the co-location cluster the seeded
    // agents formed. It is surfaced 1:1 with the emergent settlement roster (one
    // stockpile per measured settlement) and never for an unmeasured cluster: a
    // fabricated row would break this reconciliation.
    let mut cluster_sizes: BTreeMap<u64, u32> = BTreeMap::new();
    for (_, member) in sim.world.query::<&civ_agents::ClusterMember>().iter() {
        *cluster_sizes.entry(member.cluster.0).or_insert(0) += 1;
    }
    assert_eq!(
        sim.cluster_stocks().len() as u32,
        sim.settlement_count(),
        "one measured stockpile per emergent settlement, no more"
    );
    for cluster_id in sim.cluster_stocks().keys() {
        assert!(
            cluster_sizes.get(cluster_id).copied().unwrap_or(0) >= 2,
            "stockpile {cluster_id} must belong to a measured multi-member cluster"
        );
    }
}

/// Covers FR-CIV-INFOVIEW-905.
///
/// "Auto-activate on data presence": registering the backing data source makes
/// the reading appear on the next tick, with no re-registration and no code
/// change - and the twin simulation that never registered the source stays
/// blind. This is the availability gate seen from the engine side.
#[test]
fn infoview_905_reading_activates_when_its_data_source_appears() {
    const SEED: u64 = 0x905_A11B;
    let mut blind = Simulation::with_seed(SEED);
    let mut live = Simulation::with_seed(SEED);

    live.set_settlement_population(21, 40);
    live.set_settlement_housing_capacity(21, 40);
    // 1000 stockpiled food = food_score 5. Deliberately NOT 2000: with the
    // institution bonus below at 190, a food_score of 10 would reach exactly
    // MOOD_MAX and saturate, making both the headroom guard and the live-update
    // assertion below vacuous.
    live.set_settlement_food_stocked(21, 1_000);
    // Crime pressure is chosen so the well-being total is NOT saturated at
    // MOOD_MAX: a saturated total would hide a frozen reading.
    live.set_settlement_crime_pressure(21, MOOD_CRIME_BASE as i32 / 4);

    blind.tick();
    live.tick();

    assert!(
        blind.last_tick_mood(21).is_none(),
        "the data-less twin must stay blind"
    );
    let first_mood = *live
        .last_tick_mood(21)
        .expect("the reading must appear once its data source exists");
    let frame = first_mood;
    assert_eq!(frame.settlement_id, 21);
    assert_eq!(frame.food_score, 5, "1000 food units / 200 per score point");
    assert_eq!(frame.housing_score, 0, "population 40 against capacity 40");
    assert_eq!(
        frame.crime_score, 0,
        "crime at the documented baseline saturates the inverse crime sub-score"
    );
    let (temple, garrison) = institution_bonus(&live, 21);
    let expected_first = (frame.food_score
        + frame.housing_score
        + frame.crime_score
        + temple
        + garrison)
        .clamp(MOOD_MIN, MOOD_MAX);
    assert!(
        expected_first < MOOD_MAX,
        "fixture must stay below saturation, otherwise the live-update assertion is vacuous"
    );
    assert_eq!(
        frame.mood, expected_first,
        "the activated reading must equal the measured inputs, not a default"
    );

    // Live update: the same reading tracks the next measurement with no
    // re-registration (this is what "auto-activate" buys the panel).
    // 4000 food = food_score 20, which lifts the total past the 195 measured
    // above and therefore past the first frame's mood.
    live.set_settlement_food_stocked(21, 4_000);
    live.tick();
    let updated = *live.last_tick_mood(21).expect("reading stays live");
    assert_eq!(updated.food_score, 20, "the new stockpile must be read");
    assert!(
        updated.mood > frame.mood,
        "the reading must follow its data source, not freeze at first read \
         (mood {} did not move past {})",
        updated.mood,
        frame.mood
    );
    assert_eq!(
        updated.mood_delta,
        updated.mood - frame.mood,
        "the trend must be measured against the previous frame"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-INFOVIEW-911 - resource deposits + utilisation
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INFOVIEW-911.
///
/// The deposit layer reads the measured voxel material field. Oracle: the
/// read-only probe returns exactly the material that the world holds (no
/// interpolation, no default ore), probing is read-only, and a deposit is local
/// - a cell far away from the write is untouched.
///
/// Note on granularity (observed, reported to the maintainers rather than
/// asserted): the voxel kernel materialises a write at leaf granularity, so a
/// coordinate 1024 units away from the written cell also reads the new
/// material, while a coordinate 1_000_000 units away does not. The write is
/// therefore not a single-voxel stamp in this kernel revision. That behaviour
/// lives in `phenotype-voxel` (a git dependency), not in `civ-engine`, so this
/// file pins the map-scale locality contract and reports the granularity
/// observation instead of encoding it as a requirement.
#[test]
fn infoview_911_deposit_reads_track_the_measured_voxel_material_field() {
    let mut sim = Simulation::with_seed(0x911_D0E5);
    let pos = WorldCoord {
        x: 640,
        y: 900,
        z: 640,
    };
    let far = WorldCoord {
        x: 640 + 1_000_000,
        y: 900,
        z: 640,
    };

    let before = probe(&mut sim, pos);
    assert_eq!(before.pos, pos, "the probe must report the cell it read");
    assert_ne!(before.material, ORE, "fixture must start without a deposit");
    let far_before = probe(&mut sim, far);
    assert!(
        before.nearest_agent.is_none(),
        "an empty patch of world must not report a fabricated nearby agent"
    );

    let replay_before = sim.replay_log().events.len();
    sim.push_voxel_write(pos, ORE);

    let after = probe(&mut sim, pos);
    assert_eq!(
        after.material, ORE,
        "the deposit reading must equal the material actually written"
    );
    assert_eq!(
        sim.replay_log().events.len(),
        replay_before + 1,
        "the only recorded event is the write itself"
    );

    // Read-only: probing twice is idempotent and mutates nothing.
    let repeat = probe(&mut sim, pos);
    assert_eq!(repeat, after, "probing must be a pure read");
    assert_eq!(
        sim.replay_log().events.len(),
        replay_before + 1,
        "a probe must not record an edit"
    );
    assert_eq!(sim.voxel().read(pos), ORE);

    // Locality at map scale: a deposit is a feature of the region it was written
    // into, not a global flag on the whole world.
    let far_after = probe(&mut sim, far);
    assert_eq!(
        far_after.material, far_before.material,
        "a deposit must not paint the far side of the map"
    );
}

/// Covers FR-CIV-INFOVIEW-911.
///
/// "Deposits + utilization shown": the overlay reads measured stocks, and a
/// resource draw-down can never drive a stockpile negative. The supply-flow
/// series must be empty until something actually measures a flow - an overlay
/// that shows flow where the engine measured none is fabricating data.
#[test]
fn infoview_911_utilisation_reads_measured_stocks_and_invents_no_flow() {
    let mut sim = Simulation::with_seed(0x911_570C);
    sim.state.resources.wood = Fixed::from_num(1_234_i64);
    sim.state.resources.metal = Fixed::from_num(56_i64);

    assert_eq!(
        resource_stock_units(&sim.state.resources),
        (1_234, 56),
        "the resource overlay must read the measured wood/metal stocks verbatim"
    );
    assert_eq!(
        sim.snapshot().resources,
        sim.state.resources,
        "the client snapshot must carry the same measured stocks the overlay binds to"
    );

    // No settlement market is registered yet, so the supply-flow series is empty
    // rather than zero-filled rows: an overlay must not draw a trade line the
    // engine never measured.
    assert!(sim.last_tick_settlement_trade_flows().is_empty());
    for _ in 0..4 {
        sim.tick();
    }
    assert!(
        sim.last_tick_settlement_trade_flows().is_empty(),
        "with no registered settlement market, no flow row may be emitted"
    );

    // TODO(FR-CIV-INFOVIEW-911): the engine has no per-cell deposit/utilisation
    // accessor and no `civ-laws` dependency, so the overlay's deposit + usage
    // sampler is not implemented at this layer. What is measured today is the
    // per-cluster stockpile map, and it must stay proportional to the measured
    // emergent roster instead of inventing rows.
    let mut cluster_sizes: BTreeMap<u64, u32> = BTreeMap::new();
    for (_, member) in sim.world.query::<&civ_agents::ClusterMember>().iter() {
        *cluster_sizes.entry(member.cluster.0).or_insert(0) += 1;
    }
    assert_eq!(
        sim.cluster_stocks().len() as u32,
        sim.settlement_count(),
        "one measured stockpile per emergent settlement, no more"
    );
    for (cluster_id, stockpile) in sim.cluster_stocks() {
        assert!(
            cluster_sizes.get(cluster_id).copied().unwrap_or(0) >= 2,
            "stockpile {cluster_id} must belong to a measured multi-member cluster"
        );
        assert!(
            stockpile.get(civ_economy::Good::Food) >= 0,
            "utilisation must never be presented as a negative stockpile"
        );
    }

    // Utilisation clamp: a draw-down below zero saturates at zero, so the
    // overlay can never render a negative stockpile.
    let mut stocks = civ_economy::Stocks::default();
    stocks.add(civ_economy::Good::Food, 5);
    assert_eq!(stocks.get(civ_economy::Good::Food), 5);
    assert_eq!(
        stocks.add(civ_economy::Good::Food, -2),
        -2,
        "the applied delta must report what was actually withdrawn"
    );
    assert_eq!(stocks.get(civ_economy::Good::Food), 3);
    assert_eq!(
        stocks.add(civ_economy::Good::Food, -10),
        -3,
        "an over-draw must remove only what was there"
    );
    assert_eq!(stocks.get(civ_economy::Good::Food), 0);
}

// ---------------------------------------------------------------------------
// FR-CIV-INFOVIEW-912 - population & well-being overlays
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INFOVIEW-912.
///
/// Per-cell aggregation: the operational hex cell carries this cell's measured
/// population *and* its resource stock, and the strategic rollup is exactly the
/// sum of the visible cells - so zooming out can never change the number.
#[test]
fn infoview_912_per_cell_population_aggregates_exactly_into_the_region_rollup() {
    let per_cell_population = [12_u32, 0, 500, 3, 7];
    let cells: Vec<HexCellSnapshot> = per_cell_population
        .iter()
        .map(|population| operational_hex_snapshot(*population, population * 2))
        .collect();

    for (index, cell) in cells.iter().enumerate() {
        assert_eq!(cell.population, per_cell_population[index]);
        assert_eq!(
            cell.resources,
            per_cell_population[index] * 2,
            "a cell must carry its own resource stock, not a region total"
        );
    }

    let expected_total: u64 = per_cell_population
        .iter()
        .map(|population| u64::from(*population))
        .sum();
    assert_eq!(
        u64::from(aggregate_strategic(&per_cell_population)),
        cells.iter().map(|cell| u64::from(cell.population)).sum::<u64>(),
        "the region rollup must equal the sum of the per-cell drill-down"
    );
    assert_eq!(u64::from(aggregate_strategic(&per_cell_population)), expected_total);

    // Narrowing the view changes the total by exactly the removed cell.
    let mut narrowed = per_cell_population.to_vec();
    let removed = narrowed.pop().expect("fixture has a cell to remove");
    assert_eq!(
        aggregate_strategic(&per_cell_population) - removed,
        aggregate_strategic(&narrowed)
    );
    assert_eq!(
        aggregate_strategic(&[]),
        0,
        "an empty view aggregates to zero, not to a fabricated default"
    );

    // The region rollup reconciles with the settlements the engine actually
    // measures: every registered settlement shows up in the frame set.
    let mut sim = Simulation::with_seed(0x912_C311);
    for (index, population) in per_cell_population.iter().enumerate() {
        sim.set_settlement_population(index as u32 + 1, *population);
        sim.set_settlement_housing_capacity(index as u32 + 1, *population);
    }
    sim.tick();
    let measured: Vec<u64> = sim
        .last_tick_mood_all()
        .iter()
        .filter(|frame| frame.settlement_id >= 1 && frame.settlement_id <= 5)
        .map(|frame| i64::from(frame.settlement_id) as u64)
        .collect();
    assert_eq!(
        measured.len(),
        per_cell_population.len(),
        "each measured settlement must contribute exactly one per-cell reading"
    );
    assert_eq!(aggregate_strategic(&per_cell_population), expected_total as u32);
}

/// Covers FR-CIV-INFOVIEW-912.
///
/// "Happiness, wealth, age" are measured, per settlement, and update live: each
/// well-being frame equals an independent re-derivation of the documented
/// sub-score formulas from that settlement's own measured inputs, the trend
/// tracks the previous frame, and the age/lifecycle counters partition the
/// living population instead of inventing an age mix.
#[test]
fn infoview_912_well_being_and_age_frames_are_measured_per_settlement() {
    const SEED: u64 = 0x912_0E11;
    // (settlement, population, housing capacity, crime pressure)
    let scripted = [
        (1_u32, 5_u32, 40_u32, 0_i32),
        (2, 12, 12, 40),
        (3, 4, 2, 75),
    ];

    let mut sim = Simulation::with_seed(SEED);
    for (id, population, capacity, crime) in scripted {
        sim.set_settlement_population(id, population);
        sim.set_settlement_housing_capacity(id, capacity);
        sim.set_settlement_crime_pressure(id, crime);
        sim.set_settlement_food_stocked(id, 0);
    }
    sim.tick();

    let mut previous: BTreeMap<u32, i64> = BTreeMap::new();
    for (id, population, capacity, crime) in scripted {
        let frame = *sim
            .last_tick_mood(id)
            .expect("every measured settlement must produce a well-being frame");
        let (food, housing, crime_score, total) =
            expected_mood(&sim, id, population, capacity, crime, 0);
        assert_eq!(frame.food_score, food, "settlement {id}: food sub-score");
        assert_eq!(frame.housing_score, housing, "settlement {id}: housing sub-score");
        assert_eq!(frame.crime_score, crime_score, "settlement {id}: crime sub-score");
        assert_eq!(frame.mood, total, "settlement {id}: total well-being");
        assert_eq!(
            frame.mood_delta, total,
            "settlement {id}: first frame trends against zero"
        );
        // Population is recoverable exactly from the frame: the overlay's
        // per-settlement population reading is the measured registration.
        assert_eq!(
            i64::from(capacity) - frame.housing_score / 2,
            i64::from(population),
            "settlement {id}: population must be recoverable from the measured frame"
        );
        previous.insert(id, total);
    }

    // The frame set is keyed and ordered by settlement id (stable chart keying).
    let ids: Vec<u32> = sim
        .last_tick_mood_all()
        .iter()
        .filter(|frame| frame.settlement_id <= 3)
        .map(|frame| frame.settlement_id)
        .collect();
    assert_eq!(ids, vec![1, 2, 3], "frames must be sorted by settlement id");

    // Live update: new measurements move the frames and the trend. The
    // settlement's food stockpile is skimmed by the emergent settlement trade in
    // the same tick, so the well-being reading must follow the *measured*
    // post-trade stockpile (the exact flow row is asserted, not assumed).
    sim.set_settlement_food_stocked(2, 8_000);
    sim.set_settlement_crime_pressure(2, 0);
    sim.tick();
    let frame = *sim.last_tick_mood(2).expect("frame stays live");
    let outflow: i64 = sim
        .last_tick_settlement_trade_flows()
        .iter()
        .filter(|flow| flow.from_settlement == 2)
        .map(|flow| flow.qty)
        .sum();
    let inflow: i64 = sim
        .last_tick_settlement_trade_flows()
        .iter()
        .filter(|flow| flow.to_settlement == 2)
        .map(|flow| flow.qty)
        .sum();
    let measured_stocked = 8_000 - outflow + inflow;
    let (_, housing, crime_score, _) = expected_mood(&sim, 2, 12, 12, 0, measured_stocked);
    assert_eq!(
        frame.food_score,
        measured_stocked / 200,
        "the well-being reading must follow the measured post-trade stockpile \
         (8000 stocked, {outflow} traded out, {inflow} traded in)"
    );
    assert_eq!(frame.housing_score, housing);
    assert_eq!(frame.crime_score, crime_score);
    let (temple, garrison) = institution_bonus(&sim, 2);
    let total = (frame.food_score + housing + crime_score + temple + garrison)
        .clamp(MOOD_MIN, MOOD_MAX);
    assert_eq!(frame.mood, total);
    assert_eq!(
        frame.mood_delta,
        total - previous[&2],
        "the well-being trend must measure against the previous frame"
    );
    assert!(
        frame.food_score > 0,
        "the newly measured stockpile must be visible to the overlay"
    );

    // Age / lifecycle composition is measured, not authored: the labels
    // partition the population and every living civilian is counted once.
    let counters = *sim.last_tick_lifecycle_metrics();
    assert_eq!(
        counters.total(),
        counters.children + counters.adults + counters.elders + counters.dead,
        "the age overlay must partition the population into disjoint labels"
    );
    assert_eq!(
        counters.total(),
        counters.total_living() + counters.dead,
        "the dead must be disjoint from the living labels"
    );
    assert_eq!(
        counters.total_living() as usize,
        sim.all_agents().len(),
        "each living civilian must be classified exactly once by the age overlay"
    );
    assert!(
        (0.0..=1.0).contains(&counters.adult_fraction()),
        "the working-age fraction must be a measured fraction"
    );

    // Determinism: the same seed plus the same scripted measurements reproduce
    // the same frames bit-for-bit (charts replay exactly).
    let mut twin = Simulation::with_seed(SEED);
    for (id, population, capacity, crime) in scripted {
        twin.set_settlement_population(id, population);
        twin.set_settlement_housing_capacity(id, capacity);
        twin.set_settlement_crime_pressure(id, crime);
        twin.set_settlement_food_stocked(id, 0);
    }
    twin.tick();
    twin.set_settlement_food_stocked(2, 8_000);
    twin.set_settlement_crime_pressure(2, 0);
    twin.tick();
    assert_eq!(
        twin.last_tick_mood_all(),
        sim.last_tick_mood_all(),
        "identical measurements must reproduce identical well-being frames"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-INFOVIEW-914 - infrastructure overlays
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INFOVIEW-914.
///
/// The infrastructure overlay's "building level" reading is the tier engine:
/// level-ups are gated on measured tech/population/resources, a rejected
/// upgrade must not spend resources or bump the level, an accepted one costs
/// exactly the configured amount, and capacity grows with the level.
#[test]
fn infoview_914_building_level_progression_is_gated_and_exact() {
    let mut engine = BuildingTierEngine::new(BuildingTierConfig::new(0, 0, 10, 0.1));
    let id = engine.spawn(BuildingTier::Primitive, 1);

    assert_eq!(engine.get_building(id).expect("building").tier, BuildingTier::Primitive);
    assert_eq!(engine.get_building(id).expect("building").condition, 1.0);
    assert!(engine.get_building(id).expect("building").operational);
    let primitive_capacity = engine.total_capacity(1);
    assert_eq!(primitive_capacity, 1, "tier 0 contributes capacity ordinal+1");

    // Configs scale per tier: tech + 2/level, population + 10/level, cost + 50/level.
    let basic = engine.config_for(BuildingTier::Basic);
    assert_eq!((basic.tech_level, basic.population_min, basic.resource_cost), (2, 10, 60));

    // Gated: insufficient tech, then population, then resources.
    let mut treasury = 10_000_u32;
    assert_eq!(
        engine
            .upgrade(id, 1, 10, &mut treasury)
            .expect_err("tech 1 must not unlock tier 1 (needs 2)"),
        UpgradeError::InsufficientTech
    );
    assert_eq!(treasury, 10_000, "a rejected upgrade must not spend resources");
    assert_eq!(engine.get_building(id).expect("building").tier, BuildingTier::Primitive);

    assert_eq!(
        engine
            .upgrade(id, 2, 9, &mut treasury)
            .expect_err("population 9 must not unlock tier 1 (needs 10)"),
        UpgradeError::InsufficientPopulation
    );
    assert_eq!(engine.get_building(id).expect("building").tier, BuildingTier::Primitive);

    let mut poor = 5_u32;
    assert_eq!(
        engine
            .upgrade(id, 2, 10, &mut poor)
            .expect_err("5 resources must not buy a 60-cost level-up"),
        UpgradeError::InsufficientResources
    );
    assert_eq!(poor, 5, "a failed level-up must not partially spend");
    assert_eq!(engine.get_building(id).expect("building").tier, BuildingTier::Primitive);

    // Accepted: exact cost, level advanced, condition restored to pristine.
    engine
        .upgrade(id, 2, 10, &mut treasury)
        .expect("tech 2 / population 10 / 10000 resources must level up");
    assert_eq!(treasury, 10_000 - 60, "the level-up must cost exactly the tier cost");
    let upgraded = engine.get_building(id).expect("building").clone();
    assert_eq!(upgraded.tier, BuildingTier::Basic);
    assert_eq!(upgraded.age, 0, "a fresh level resets the age clock");
    assert_eq!(upgraded.condition, 1.0);
    assert_eq!(
        engine.total_capacity(1),
        primitive_capacity + 1,
        "a higher level must contribute more capacity to the settlement"
    );

    // Boundaries: the top level cannot be exceeded; the bottom cannot be stepped down.
    let maxed = engine.spawn(BuildingTier::Legendary, 1);
    assert_eq!(
        engine
            .upgrade(maxed, 1_000, 1_000_000, &mut treasury)
            .expect_err("Legendary is the top level"),
        UpgradeError::AlreadyMaxTier
    );
    assert_eq!(
        engine
            .upgrade(9_999, 100, 1_000, &mut treasury)
            .expect_err("unknown building"),
        UpgradeError::BuildingNotFound
    );

    engine.downgrade(id).expect("Basic must be downgradable");
    assert_eq!(engine.get_building(id).expect("building").tier, BuildingTier::Primitive);
    assert_eq!(
        engine.downgrade(id).expect_err("Primitive is the bottom level"),
        UpgradeError::AlreadyMaxTier
    );

    // Coverage decay: condition falls by maintenance * dt, and a depleted
    // building drops out of the operational (drawn) set.
    let mut decaying = BuildingTierEngine::new(BuildingTierConfig::new(0, 0, 10, 0.2));
    let decaying_id = decaying.spawn(BuildingTier::Primitive, 4);
    decaying.tick(1.0);
    let after_one = decaying.get_building(decaying_id).expect("building").condition;
    assert!(
        (after_one - 0.8).abs() < 1e-6,
        "one tick of maintenance 0.2 must leave 0.8, got {after_one}"
    );
    assert_eq!(decaying.get_building(decaying_id).expect("building").age, 1);
    decaying.tick(4.0);
    let depleted = decaying.get_building(decaying_id).expect("building");
    assert_eq!(depleted.condition, 0.0);
    assert!(
        !depleted.operational,
        "a depleted building must leave the operational (rendered/counting) set"
    );
    assert_eq!(
        decaying.total_capacity(4),
        0,
        "a non-operational building must stop contributing service capacity"
    );
}

/// Covers FR-CIV-INFOVIEW-914.
///
/// "Service coverage" and "reads emergent architecture": layouts declare the
/// infrastructure they need (including road adjacency), coverage uplift comes
/// only from a genuinely adjacent service type, the per-tick falloff is bounded
/// and pure, and parcels get emergent facades/cluster assignments from the
/// measured style key rather than an authored table.
#[test]
fn infoview_914_layout_coverage_and_road_adjacency_are_measured() {
    let house = LayoutCatalog::house();
    assert_eq!(house.capacity, 4, "the housing layout's capacity is a declared datum");
    assert_eq!(house.infrastructure_needs.food_per_tick, 1);
    assert_eq!(house.infrastructure_needs.energy_per_tick, 1);
    assert!(
        house
            .infrastructure_needs
            .adjacent_requirements
            .iter()
            .any(|requirement| requirement == "road"),
        "the road requirement is what the transport overlay draws a road edge for"
    );

    // Placement is measured against occupancy, not assumed free.
    let occupied = house.footprint.clone();
    assert!(
        validate_placement(&house, (10, 10), &occupied),
        "an empty anchor must accept the footprint"
    );
    assert!(
        !validate_placement(&house, (0, 0), &occupied),
        "an anchor over an occupied cell must be rejected"
    );
    assert!(
        !validate_placement(&house, (0, 0), &[(0, 0)]),
        "an anchor on a single occupied cell must be rejected"
    );

    // Layout generation is deterministic: the same (type, size, seed) yields the
    // same footprint/capacity, so the overlay can cache parcel geometry.
    let generated_a = generate_layout("house", 9, 7);
    let generated_b = generate_layout("house", 9, 7);
    assert_eq!(generated_a.footprint, generated_b.footprint);
    assert_eq!(generated_a.capacity, generated_b.capacity);
    assert!(
        generated_a.footprint.len() >= 2,
        "a generated layout must have a non-degenerate footprint"
    );
    assert!(
        validate_placement(&generated_a, (0, 0), &[]),
        "a generated layout must be placeable on empty ground"
    );

    // Coverage uplift only from a genuinely adjacent matching service type.
    let base = compute_efficiency(&house, &[]);
    assert_eq!(base, house.efficiency, "no adjacency must mean no uplift");
    assert!(
        (compute_efficiency(&house, &[String::from("water")]) - base).abs() < 1e-6,
        "an unrelated neighbour must not fabricate coverage"
    );
    let farm = LayoutCatalog::farm();
    assert!(
        (compute_efficiency(&farm, &[String::from("water")]) - (farm.efficiency + 0.1)).abs()
            < 1e-6,
        "one matching neighbour must add exactly one coverage step"
    );
    assert!(
        compute_efficiency(&house, &vec![String::from("market"); 64]) <= 2.0,
        "coverage must stay capped instead of growing without bound"
    );

    // Per-tick falloff: pure projection, floor at 0.1, no in-place mutation.
    let mut decaying = LayoutCatalog::farm();
    decaying.efficiency = 0.101;
    let after = tick_building_layouts(std::slice::from_ref(&decaying));
    assert!(
        (after[0].efficiency - 0.1).abs() < 1e-6,
        "the falloff floor is 0.1, got {}",
        after[0].efficiency
    );
    assert_eq!(
        decaying.efficiency, 0.101,
        "the projection must not mutate the live layout in place"
    );
    let floored = tick_building_layouts(&[BuildingLayout {
        efficiency: 0.1,
        ..LayoutCatalog::house()
    }]);
    assert_eq!(floored[0].efficiency, 0.1);

    // Emergent architecture: facade + parcel assignment is a deterministic
    // function of the measured style key, and parcels are grouped per cluster.
    let mut sim = Simulation::with_seed(0x914_FAC);
    let anchor = WorldCoord { x: 0, y: 0, z: 0 };
    let snapshot = sim.snapshot();
    let style = emergent_style_key_for_sim(&sim, None, &snapshot.geology_map, &anchor);
    let demand = DemandSignals {
        residential: 0.4,
        commercial: 0.2,
        industrial: 0.1,
        civic: 0.3,
    };
    let allocated = [BuildingId(1), BuildingId(2), BuildingId(3)];
    apply_emergence_facades(&mut sim, Some(5), style, demand, &allocated);

    assert!(
        !architecture_tile_sets().is_empty(),
        "the facade grammar needs a registered tile-set catalog"
    );
    for id in allocated {
        assert!(
            sim.building_graph().facades.contains_key(&id),
            "parcel {id:?} must receive an emergent facade"
        );
    }
    assert_eq!(
        sim.building_graph().parcels_in_cluster(5),
        &allocated[..],
        "allocated parcels must be grouped under their emergent cluster"
    );
    let first = sim.building_graph().facades[&BuildingId(1)].clone();
    assert_eq!(
        first,
        sim.building_graph().facades[&BuildingId(3)].clone(),
        "the same style key must yield the same facade for the same demand profile"
    );
    // A cluster is infrastructure-relevant only if its parcels are tracked: a
    // different cluster's parcels do not leak into this one.
    assert!(sim.building_graph().parcels_in_cluster(6).is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-INSPECT-902 - settlement / polity inspector
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INSPECT-902.
///
/// "Emergent membership (cluster overlap, NOT a faction id)": membership is the
/// co-location cluster the agents actually formed - a `u64` cluster id that no
/// faction table knows about - and the cluster's culture/economy readings are
/// keyed by that same emergent id. The reading must stay anchored to the
/// cluster's own emergent seed (bounded per-tick drift), not to a constant or to
/// some faction-keyed value.
#[test]
fn inspect_902_membership_reads_emergent_cluster_identity_not_a_faction() {
    let mut sim = Simulation::with_seed(0x902_C105);
    let founder = 900_001_u64;
    sim.apply_god_tool(GodToolRequest::Life(LifeRequest::SpawnCivSeed(
        SpawnCivSeedRequest {
            seed_civilian_id: founder,
            faction: 0,
            center: WorldCoord {
                x: 1_024,
                y: 0,
                z: 1_024,
            },
        },
    )))
    .expect("life.spawn_civ_seed must spawn a co-located nucleus");
    sim.tick();

    // Membership is emergent co-location, read off the live world.
    let memberships: Vec<(u64, u64)> = sim
        .world
        .query::<(&civ_agents::Civilian, &civ_agents::ClusterMember)>()
        .iter()
        .map(|(_, (civilian, member))| (civilian.id, member.cluster.0))
        .collect();
    let cluster = memberships
        .iter()
        .find(|(id, _)| *id == founder)
        .map(|(_, cluster)| *cluster)
        .expect("a spawned founder must belong to an emergent cluster");
    let member_ids: Vec<u64> = memberships
        .iter()
        .filter(|(_, id)| *id == cluster)
        .map(|(id, _)| *id)
        .collect();
    assert!(
        member_ids.len() >= 2,
        "an emergent settlement needs more than one member"
    );
    // The membership identity is derived from co-location (the minimum agent id
    // in the connected component), not assigned from a faction table.
    assert_eq!(
        cluster,
        *member_ids.iter().min().expect("members exist"),
        "the settlement id must be the co-location component's minimum agent id"
    );
    assert!(sim.settlement_count() >= 1);
    assert!(
        sim.cluster_stocks().contains_key(&cluster),
        "the economy summary must be keyed by the same emergent cluster id, not a faction"
    );
    assert!(
        sim.cluster_stocks()
            .get(&cluster)
            .expect("stockpile")
            .get(civ_economy::Good::Food)
            >= 0,
        "a measured stockpile may never be presented as negative"
    );

    // Culture reading: deterministic, in range, and anchored to this cluster's
    // own emergent id (the documented seed), within the per-tick drift budget.
    let reading = culture_traits_for_cluster(&sim, cluster);
    assert_eq!(
        culture_traits_for_cluster(&sim, cluster),
        reading,
        "a cluster reading must be deterministic"
    );
    let expected_seed = [
        ((cluster % 256) as f32) / 255.0,
        (((cluster >> 8) % 256) as f32) / 255.0,
        (((cluster >> 16) % 256) as f32) / 255.0,
        (((cluster >> 24) % 256) as f32) / 255.0,
    ];
    for (index, value) in reading.iter().enumerate() {
        assert!((0.0..=1.0).contains(value), "culture[{index}] = {value} out of range");
        assert!(
            (value - expected_seed[index]).abs() < 0.06,
            "cluster {cluster} culture[{index}] = {value} must stay anchored to its \
             emergent seed {} (bounded drift), not to a constant or a faction default",
            expected_seed[index]
        );
    }
    assert_ne!(
        reading, [0.25_f32; 4],
        "the cluster reading must not be the neutral authoring default"
    );
    // Distinct emergent clusters do not collapse into one identity.
    assert_ne!(
        culture_traits_for_cluster(&sim, 0xDEAD_0000_0000_0001),
        culture_traits_for_cluster(&sim, 0xDEAD_0000_00FF_0000)
    );

    // TODO(FR-CIV-INSPECT-902): `Simulation::settlement_member_counts()` is a
    // documented stub (empty map), so the membership *overlap set* has no
    // engine accessor yet. When it lands, replace this absence guard with an
    // assertion on the overlap counts themselves.
    assert!(
        sim.settlement_member_counts().is_empty(),
        "membership counts are still a stub: an empty map, not a fabricated roster"
    );
}

/// Covers FR-CIV-INSPECT-902.
///
/// "Population, economy summary, culture/ideology, dominant language": each
/// field reads a real owning-crate source. Population comes back out of the
/// measured settlement registration, the economy summary is keyed by emergent
/// cluster, ideology vectors are bounded measured signals, and the language
/// reading is a measured (faction-keyed) vocabulary - with the *per-settlement
/// dominant-language* field pinned as still absent.
#[test]
fn inspect_902_population_economy_and_ideology_fields_bind_to_measured_sources() {
    let mut sim = Simulation::with_seed(0x902_F1E1D);
    assert!(
        !sim.state.factions.is_empty(),
        "the inspector's polity fields are meaningless without factions"
    );

    // Population: the measured registration is what the frame reports, and it is
    // recoverable exactly (housing sub-score = 2 * (capacity - population)).
    let (population, capacity) = (250_u32, 300_u32);
    sim.set_settlement_population(9, population);
    sim.set_settlement_housing_capacity(9, capacity);
    sim.tick();
    let frame = *sim.last_tick_mood(9).expect("registration must be measurable");
    assert_eq!(frame.settlement_id, 9);
    assert_eq!(
        i64::from(capacity) - frame.housing_score / 2,
        i64::from(population),
        "the population field must be the measured registration"
    );

    // Economy summary: keyed by emergent co-location cluster id (u64), not by
    // faction. Every stockpile row must correspond to a cluster that actually
    // has two or more co-located members, so a faction-keyed (or fabricated)
    // summary would break this reconciliation.
    let mut cluster_sizes: BTreeMap<u64, u32> = BTreeMap::new();
    for (_, member) in sim.world.query::<&civ_agents::ClusterMember>().iter() {
        *cluster_sizes.entry(member.cluster.0).or_insert(0) += 1;
    }
    assert!(
        !sim.cluster_stocks().is_empty(),
        "an inhabited world with co-located agents must expose a per-cluster economy summary"
    );
    for cluster_id in sim.cluster_stocks().keys() {
        assert!(
            cluster_sizes.get(cluster_id).copied().unwrap_or(0) >= 2,
            "economy summary row {cluster_id} must belong to a measured multi-member cluster"
        );
    }

    // Culture/ideology: measured signals, all bounded to [0, 1].
    for _ in 0..24 {
        sim.tick();
    }
    let ideologies = sim.faction_ideologies();
    assert!(
        !ideologies.is_empty(),
        "an inhabited world must expose per-faction culture/ideology"
    );
    for (faction, ideology) in ideologies {
        for (index, value) in ideology
            .values
            .iter()
            .chain(ideology.norms.iter())
            .enumerate()
        {
            assert!(
                (0.0..=1.0).contains(value),
                "faction {faction} trait vector[{index}] out of bounds: {value}"
            );
        }
        for (label, signal) in [
            ("cooperation", ideology.cooperation),
            ("aggression", ideology.aggression),
            ("openness", ideology.openness),
            ("tradition", ideology.tradition),
        ] {
            assert!(
                (0.0..=1.0).contains(&signal),
                "faction {faction} {label} signal out of bounds: {signal}"
            );
        }
    }
    // Ideology is emergent, not uniform: at least one vector must have moved
    // off the neutral default, otherwise the inspector would show a constant.
    assert!(
        ideologies
            .values()
            .any(|ideology| ideology.values != [0.5_f32; 4] || ideology.norms != [0.5_f32; 4]),
        "measured culture must diverge from the neutral default"
    );

    // Dominant language: the faction-level language state carries *measured*
    // vocabulary (the reading an inspector binds to), but no per-settlement
    // dominance reading exists yet.
    let languages = sim.faction_languages();
    assert!(
        !languages.is_empty(),
        "an inhabited world must expose measured language state"
    );
    for (faction, language) in languages {
        assert!(
            !language.lexemes.is_empty(),
            "faction {faction} language state must carry measured vocabulary, not an empty stub"
        );
        assert!(
            language.drift_rate > 0.0,
            "faction {faction} must carry a measured drift rate"
        );
    }
    // TODO(FR-CIV-INSPECT-902): "dominant language" per settlement/polity is
    // still unimplemented - the readings are faction-keyed, and
    // `language::LinguaFranca` (which carries `dominance`) is owned by no
    // simulation state, so nothing on the wire can be called a settlement's
    // dominant language. This guard fails the moment such a field is surfaced.
    let wire = serde_json::to_value(sim.snapshot()).expect("snapshot serializes");
    for absent in ["dominant_language", "languages", "lingua_franca"] {
        assert!(
            wire.get(absent).is_none(),
            "TODO(FR-CIV-INSPECT-902): snapshot must not carry `{absent}` until the \
             per-settlement dominant-language reading is implemented"
        );
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-INSPECT-920 - follow-cam + history/lineage jump
// ---------------------------------------------------------------------------

/// Covers FR-CIV-INSPECT-920.
///
/// Follow-cam locks the camera to the *selected agent*. Oracle: the selection
/// resolves to the live entity carrying that agent id (never an alias), the
/// lock survives ticks and still resolves to the same agent, the locked entity
/// exposes a live position for the camera to track, and an unknown selection or
/// a despawned agent resolves to nothing instead of a stale entity.
#[test]
fn inspect_920_follow_cam_locks_to_the_selected_agent_and_never_a_stale_one() {
    let mut sim = Simulation::with_seed(0x920_CA40);
    let agents = sim.all_agents();
    assert!(!agents.is_empty(), "follow-cam needs at least one selectable agent");
    let selected = agents[0].id;

    let entity = sim
        .agent_entity(selected)
        .expect("a live agent id must resolve to its entity");
    // The resolved entity really carries the selected id (no aliasing).
    let resolved_ids: Vec<u64> = sim
        .world
        .query::<&civ_agents::Civilian>()
        .iter()
        .filter(|(candidate, _)| *candidate == entity)
        .map(|(_, civilian)| civilian.id)
        .collect();
    assert_eq!(resolved_ids, vec![selected]);

    // The camera needs a live position for the locked entity.
    let positions: Vec<WorldCoord> = sim
        .world
        .query::<(&civ_agents::Civilian, &civ_agents::Position3d)>()
        .iter()
        .filter(|(_, (civilian, _))| civilian.id == selected)
        .map(|(_, (_, position))| position.coord)
        .collect();
    assert_eq!(positions.len(), 1, "the locked agent must have exactly one live position");

    // The lock survives simulation ticks and still points at the same agent.
    for _ in 0..8 {
        sim.tick();
    }
    assert_eq!(
        sim.agent_entity(selected),
        Some(entity),
        "the follow-cam target must not drift to another entity across ticks"
    );

    // An unknown selection must not fabricate a lock.
    assert!(sim.agent_entity(u64::MAX).is_none());
    assert!(
        sim.agent_entity(selected + 1_000_000).is_none(),
        "an unselected agent id must not resolve to somebody else's entity"
    );

    // A despawned agent releases the lock instead of tracking a stale entity.
    sim.world.despawn(entity).expect("despawn the selected agent");
    assert!(
        sim.agent_entity(selected).is_none(),
        "follow-cam must release when its agent leaves the world"
    );
}

/// Covers FR-CIV-INSPECT-920.
///
/// "Trace lineage / history" jump: the chronicle exposes real, ordered,
/// tick-stamped jump targets; a jump to an exact tick includes the boundary
/// events; the retention window and the importance ordering are what the
/// inspector's "jump somewhere worth looking" list depends on; and the lineage
/// walk returns the full ancestor chain to the root.
#[test]
fn inspect_920_history_jump_targets_are_real_chronicle_entries() {
    // Era chronicle: tick-stamped, ordered, forward-only jump targets.
    let mut era = EraHistory::default();
    era.record_advance(3, 7, CivAge::Stone, CivAge::Bronze);
    era.record_advance(9, 7, CivAge::Bronze, CivAge::Iron);
    era.record_advance(10, 7, CivAge::Iron, CivAge::Stone); // not an advance
    let transitions = era.transitions();
    assert_eq!(transitions.len(), 2, "a non-advance must not create a jump target");
    assert_eq!(transitions[0].tick, 3);
    assert_eq!(transitions[1].tick, 9);
    assert!(
        transitions[0].tick < transitions[1].tick,
        "jump targets must be ordered by tick"
    );
    assert_eq!(transitions[1].from, CivAge::Bronze);
    assert_eq!(transitions[1].to, CivAge::Iron);
    assert_eq!(era.chronicle().len(), 2);
    assert!(era.chronicle()[0].contains("tick 3"));

    // Timeline: exact-tick jumps include the boundaries; importance ordering is
    // descending so the inspector can offer "the good bits".
    let mut timeline = Timeline::new();
    let minor = TimelineEvent {
        id: 0,
        tick: 5,
        event_type: EventType::Trade,
        title: "Caravan arrives".to_owned(),
        description: String::new(),
        actors: vec![1],
        consequences: Vec::new(),
        importance: 0.2,
    };
    let major = TimelineEvent {
        id: 0,
        tick: 12,
        event_type: EventType::War,
        title: "War declared".to_owned(),
        description: String::new(),
        actors: vec![1, 2],
        consequences: vec!["mobilisation".to_owned()],
        importance: 0.9,
    };
    assert_eq!(timeline.add_event(minor), 0, "ids auto-increment from zero");
    assert_eq!(timeline.add_event(major), 1);
    assert_eq!(
        timeline.get_event(1).map(|event| event.title.as_str()),
        Some("War declared")
    );
    assert_eq!(
        timeline.events_in_range(5, 5).len(),
        1,
        "a jump to the exact tick must include the boundary event"
    );
    assert_eq!(timeline.events_in_range(5, 12).len(), 2);
    assert_eq!(timeline.events_in_range(6, 11).len(), 0);
    let top = timeline.most_important(1);
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].event_type, EventType::War);
    assert_eq!(timeline.most_important(10).len(), 2);

    // History log: jump filtered by event type and inclusive tick range, and the
    // visible window stays bounded (eviction keeps the newest entries).
    let mut log = HistoryLog::with_capacity(3);
    log.record_event(HistoricalEvent {
        tick: 2,
        event_type: EventType::Trade,
        description: "trade".to_owned(),
        ..HistoricalEvent::default()
    });
    log.record_event(HistoricalEvent {
        tick: 4,
        event_type: EventType::War,
        description: "war".to_owned(),
        consequences: vec!["losses".to_owned()],
        ..HistoricalEvent::default()
    });
    log.record_event(HistoricalEvent {
        tick: 6,
        event_type: EventType::Trade,
        description: "trade again".to_owned(),
        ..HistoricalEvent::default()
    });
    assert_eq!(log.query_events(EventType::Trade, 0, 6).len(), 2);
    assert_eq!(log.query_events(EventType::Trade, 5, 6).len(), 1);
    assert_eq!(log.query_events(EventType::War, 0, 3).len(), 0);
    assert!(
        log.summarize_era("Bronze", 0, 6).contains('3'),
        "the era summary must count the recorded events"
    );
    assert!((0.0..=1.0).contains(&log.compute_historical_impact(0, 6)));

    log.record_event(HistoricalEvent {
        tick: 8,
        event_type: EventType::Religion,
        description: "faith".to_owned(),
        ..HistoricalEvent::default()
    });
    assert_eq!(log.events.len(), 3, "the jump window must stay bounded");
    assert_eq!(
        log.events[0].tick, 4,
        "eviction must drop the oldest entry so jump targets stay recent"
    );

    // Lineage walk: the ancestor chain reaches the root, and a node's own
    // history is a prefix of its descendant's.
    let mut tree = LanguageFamilyTree::new("proto");
    let child = tree.diverge(0, "child", 5);
    let grandchild = tree.diverge(child, "grandchild", 12);
    assert_eq!(tree.get_lineage(grandchild), vec![grandchild, child, tree.root_id]);
    assert_eq!(
        tree.get_lineage(child),
        vec![child, tree.root_id],
        "the parent chain must be the tail of the child's chain"
    );
    assert_eq!(
        tree.families[grandchild].divergence_tick, 12,
        "a lineage jump needs a tick to jump to"
    );

    // TODO(FR-CIV-INSPECT-920): agent-level lineage has no public ancestor
    // query - `register_kinship` records edges but no accessor walks them. When
    // one lands, assert the agent's ancestor chain here instead of the language
    // tree above.
}
