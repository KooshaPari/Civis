"""Verdicts for the requirement tags on data-type declarations in `crates/tactics`.

Scope: nine tags that sit directly above a struct/enum/const declaration. Each tag
claims "this symbol implements this requirement". Eight of the nine claims are
false and are listed in SITES for removal; one (FR-CIV-FOG-001 on `FogOfWar`) is
genuinely discharged and is recorded in KEEP.

Authoritative requirement text:
  * FR-CIV-FOG-001 -- agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47-50
  * FR-CIV-WAR-011 / -013 / -021 / -022 / -030 -- docs/design/warfare.md:83-85, 89-94,
    111-113, 114-117, 124-140. That document declares itself authoritative for the
    FR-CIV-WAR-* namespace at docs/design/warfare.md:4 and its header at
    docs/design/warfare.md:3 reads "Design / planner spec. No implementation."
    The agileplus-specs/ tree only defines FR-CIV-WAR-001..004 (civ-006-deep-combat);
    the 011..042 block exists solely in warfare.md, and every one of these test
    files self-describes as "Status: CODE-ONLY-no-spec".

Two ids appear twice each, on two different declarations. Both sites are judged
independently and both removals are recorded; the pair agreement is stated in the
note lines for each declaration.

Nothing under crates/ was modified to produce this file, and cargo was not run.
"""

KEEP = {
    "FR-CIV-FOG-001": (
        "FogOfWar genuinely discharges this one and the tag stays. The requirement "
        "is that visibility SHALL be a function of (unit position, vision radius, "
        "terrain LOS), visible iff an LOS check from any friendly unit passes, and "
        "deterministic given the same unit set and terrain (civ-015 spec:47-50). "
        "FogOfWar::update at crates/tactics/src/fog_of_war.rs:81 recomputes from "
        "scratch each call (self.visible.clear() at :86, so there is no incremental "
        "state to diverge), and per friendly unit it applies exactly the three "
        "named inputs: Euclidean range against self.vision_radius at :126-129, then "
        "line_of_sight_grid(voxel_world, ...) at :129, OR-ed across every unit of "
        "the faction. The struct is not a passive record: the type plus its impl "
        "carry the required function. Note that the sibling FOG-002..005 were "
        "already removed from this declaration by _verdicts_fog.py; FOG-001 is the "
        "one id in that block that is really discharged here, which is why it "
        "survives and the other four did not."
    ),
}

SITES = [
    (
        "crates/tactics/src/lib.rs",
        "DamageEvent",
        {
            "FR-CIV-WAR-013": (
                "DATA-SHAPE-ONLY. The requirement is that when two opposing formations "
                "close within engagement range the operational layer hands off the "
                "contact to the tactical war bridge, that operational supply/cohesion "
                "state parameterizes the tactical engagement (low ammo -> fewer shots, "
                "low cohesion -> earlier rout), and that far-from-camera contacts "
                "resolve via a statistical combat rollup logged to the event stream "
                "(docs/design/warfare.md:89-94). DamageEvent is a three-field record "
                "(center, radius_voxels, energy) at crates/tactics/src/lib.rs:72 and "
                "cannot decide a handoff, read supply or cohesion, or run a rollup. "
                "None of the three mandated behaviors exist anywhere: "
                "git grep -n -i -E 'rollup|statistical|aggregate_damage|far_from_camera|lod' "
                "-- crates/tactics/src/ returned no hits, and the parameterization half "
                "is contradicted at the only real consumer, tick_war_bridge at "
                "crates/tactics/src/war_bridge.rs (the call site at "
                "crates/engine/src/engine/military_phases.rs:175 passes only tick, "
                "config, samples, voxel world and fog -- no supply or cohesion argument "
                "reaches it, and grepping war_bridge.rs for 'cohesion|ammo|supply' "
                "returns nothing). The operational-to-tactical sequencing that does "
                "exist, in military_phases.rs, is the handoff scaffold, not the "
                "parameterization or the LOD rollup, so this is not a relocation "
                "candidate either."
            ),
            "FR-CIV-WAR-022": (
                "DATA-SHAPE-ONLY. The requirement is that DamageEvents reshape terrain "
                "via the voxel CA and that the altered voxel field changes LOS, cover "
                "and passability for subsequent ticks at all three layers -- a collapsed "
                "bridge cutting an operational supply line, a cratered wall opening a "
                "firing lane -- making destruction a cross-layer coupling rather than a "
                "cosmetic effect (docs/design/warfare.md:114-117). The first half is "
                "real and is implemented by apply_damage at crates/tactics/src/lib.rs:144, "
                "which carves the sphere into the VoxelWorld; DamageEvent is the payload "
                "it consumes, not the implementer. The cross-layer half is the part the "
                "tag asserts and it is absent. The only DamageEvent behavior in the "
                "crate is estimated_casualties at crates/tactics/src/lib.rs:97, which "
                "scales a casualty count by blast footprint and has nothing to do with "
                "LOS, cover or passability; git grep -n -i -E "
                "'cover|passab|collapse|re_?read|destruction_feedback' -- crates/tactics/src/ "
                "crates/voxel/src/ found no destruction-feedback re-read path, and "
                "nothing re-reads the mutated field to gate a subsequent tick. The test "
                "file crates/tactics/tests/fr_fr_civ_war_022.rs only asserts that a "
                "larger radius yields more casualties, which is a unit test of the struct "
                "and not evidence of cross-layer coupling."
            ),
        },
        [
            "One declaration, two false ids, one removal entry. DamageEvent is the "
            "correct data carrier for both requirements -- it is literally the "
            "voxel damage application both specs talk about -- which is exactly why a "
            "plausible-looking tag lands here. Neither requirement is about the shape "
            "of the record, so neither is discharged by it.",
            "The distinction between the two ids matters. FR-CIV-WAR-013 needs a handoff "
            "and a rollup, neither of which exists. FR-CIV-WAR-022 needs terrain "
            "reshaping, which does exist (apply_damage, lib.rs:144) but is implemented by "
            "that function rather than by this struct, and its distinguishing cross-layer "
            "re-read is missing. Neither site is IMPLEMENTED-BY-BEHAVIOR, because in "
            "both cases the behavior the requirement names is itself absent.",
        ],
    ),
    (
        "crates/tactics/src/lib.rs",
        "Doctrine",
        {
            "FR-CIV-WAR-030": (
                "CONTAINER-ONLY. The requirement is to extend the doctrine fitness "
                "signal beyond tactical engagement stats to include operational and "
                "strategic outcomes -- net theater objective gain, supply efficiency, "
                "own attrition and routs, civilian grievance generated -- so that "
                "doctrines which win battles but lose the war are selected against, "
                "with per-cluster independent libraries and memetic diffusion across "
                "contact networks (docs/design/warfare.md:124-140, AC-WAR-7 at :178). "
                "The spec itself says the existing mechanism is already shipped and the "
                "extension is 'spec; not yet coded' at docs/design/warfare.md:125. "
                "Doctrine at crates/tactics/src/lib.rs:108 is a three-field GA genome "
                "(id, unit_composition, score) and cannot compute a fitness signal at "
                "all. The actual fitness function, score_doctrine_fitness at "
                "crates/tactics/src/doctrine_fitness.rs:28, adds only composition_balance "
                "plus net engagement pressure and voxels removed -- purely tactical, "
                "precisely the pre-extension state the requirement says to move past. "
                "git grep -n -i -E "
                "'k_terr|k_supl|k_loss|k_grv|civilian_grievance|own_attrition|memetic|diffusion' "
                "-- crates/ returned no hits in tactics (the only diffusion matches are "
                "unrelated civ-agents tech/wardrobe propagation), and "
                "crates/tactics/src/doctrine_fitness.rs has exactly two public functions, "
                "net_pressure and score_doctrine_fitness, with no per-cluster selection "
                "and no doctrine-copying path."
            ),
        },
        [
            "Both FR-CIV-WAR-030 tags in this crate are false, so the pair agrees. The "
            "second one, on score_doctrine_fitness at "
            "crates/tactics/src/doctrine_fitness.rs:27, is also still present in "
            "source and is false for the same reason: the function computes exactly "
            "the tactical-only signal the requirement asks to extend, so tagging it "
            "with the extension requirement asserts the opposite of what it does.",
            "A coordination correction arrived mid-audit stating that the "
            "FactionEngagementStats site had already been removed from source. That is "
            "not what the file says: crates/tactics/src/doctrine_fitness.rs:6 still "
            "reads '// FR-CIV-WAR-030' directly above pub struct "
            "FactionEngagementStats, and the same file's line 27 carries the second "
            "tag. Both are recorded here rather than silently dropped. The tag on "
            "FactionEngagementStats is likewise CONTAINER-ONLY: the struct records "
            "engagement counts and voxels removed, which are the tactical inputs the "
            "spec identifies as the starting point, not the operational or strategic "
            "terms it requires.",
        ],
    ),
    (
        "crates/tactics/src/doctrine_fitness.rs",
        "FactionEngagementStats",
        {
            "FR-CIV-WAR-030": (
                "CONTAINER-ONLY. Same requirement as the tag on Doctrine: extend the "
                "doctrine fitness signal beyond tactical engagement stats to include "
                "operational and strategic outcomes -- net theater objective gain, supply "
                "efficiency, own attrition and routs, civilian grievance generated -- so "
                "doctrines that win battles but lose the war are selected against "
                "(docs/design/warfare.md:124-140, AC-WAR-7 at :178). The spec states the "
                "status plainly: 'Doctrine fitness today reads tactical engagement stats "
                "only. Extend the fitness signal (spec; not yet coded)' at "
                "docs/design/warfare.md:125, and §4.1 at :122 records the existing "
                "mechanism as shipped under FR-CIV-TACTICS-023. FactionEngagementStats at "
                "crates/tactics/src/doctrine_fitness.rs:8 is the worst possible carrier for "
                "this id: its three fields are exactly the tactical inputs the spec names as "
                "the starting point to be moved past (engagements_as_shooter, "
                "engagements_as_target, voxels_removed), and it has no field for any of the "
                "four operational or strategic terms. Its only method, net_pressure at "
                "crates/tactics/src/doctrine_fitness.rs:19, subtracts target engagements "
                "from shooter engagements -- the definition of tactical-only. "
                "git grep -n -i -E "
                "'k_terr|k_supl|k_loss|k_grv|civilian_grievance|own_attrition|memetic|diffusion' "
                "-- crates/ returned no hits in the tactics crate, and this module has "
                "exactly two public functions, net_pressure and score_doctrine_fitness, "
                "with no per-cluster selection and no doctrine-copying path. The test at "
                "crates/tactics/tests/fr_fr_civ_war_030.rs asserts only that higher "
                "engagement stats raise fitness and that scoring is deterministic, which "
                "confirms the pre-extension tactical behavior the requirement says to extend."
            ),
        },
        [
            "This is the site a mid-audit coordination note reported as already removed, "
            "asserting that no tag remained in source. That was not accurate when checked: "
            "crates/tactics/src/doctrine_fitness.rs:6 still reads '// FR-CIV-WAR-030' "
            "directly above 'pub struct FactionEngagementStats', re-verified with "
            "git grep and a raw line dump after the note arrived. The removal entry is "
            "therefore written, because the tag is present and the claim is false. The "
            "same module carries a second FR-CIV-WAR-030 tag at "
            "crates/tactics/src/doctrine_fitness.rs:27, above pub fn "
            "score_doctrine_fitness, which is likewise still present.",
            "The two FR-CIV-WAR-030 tags inside this module are the mirror image of the "
            "FR-CIV-WAR-021 pair: here the pair agrees outright. score_doctrine_fitness "
            "computes precisely the tactical-only signal FR-CIV-WAR-030 asks to extend, so "
            "tagging the function with the extension requirement asserts the opposite of "
            "what it does, and tagging the stats struct it consumes with the same id is "
            "equally unfounded. Both tags overstate coverage of a requirement the spec "
            "itself marks as not yet coded.",
            "Only the FactionEngagementStats entry is in SITES here. The "
            "score_doctrine_fitness tag at crates/tactics/src/doctrine_fitness.rs:27 was "
            "not in the assigned nine, so it is reported rather than removed; it is a "
            "false binding of the same kind and should be added to scope.",
        ],
    ),
    (
        "crates/tactics/src/morale.rs",
        "MoraleState",
        {
            "FR-CIV-WAR-021": (
                "DATA-SHAPE-ONLY, and worse, an orphan. The requirement is that "
                "per-soldier behavior reads emergent agent psyche -- morale, fear, "
                "in-group loyalty and fatigue modulate fire discipline, rout threshold "
                "and willingness to follow orders, with casualties and atrocities "
                "writing grievance and trauma back into psyche (docs/design/warfare.md:"
                "111-113). MoraleState models one of the four named inputs (morale) and "
                "a rout threshold, which is real coverage of that slice; none of the "
                "other three are present. git grep -n -i -E "
                "'fear|loyalty|grievance|trauma|ideolog' -- crates/tactics/src/ returned "
                "no hits at all, so there is no fear, loyalty, grievance, trauma or "
                "ideology term for the struct to read, and nothing anywhere writes "
                "grievance or trauma back. The decisive point is that the struct has no "
                "production consumer: git grep -n 'MoraleState' -- crates/ shows hits "
                "only inside morale.rs itself and in test files "
                "(crates/tactics/tests/fr_fr_civ_war_021.rs:9, "
                "crates/tactics/tests/fr_civ_tactics_tests.rs:6, "
                "crates/tactics/tests/fr_fr_civ_tactics_076.rs:8). No combat, movement "
                "or war-bridge path in the crate ever constructs one, so morale does "
                "not in fact modulate fire discipline, rout threshold or order-following "
                "in the running simulation. A tracker nothing reads cannot discharge a "
                "behavioral requirement."
            ),
        },
        [
            "The two FR-CIV-WAR-021 sites do not agree in kind, though both come off. "
            "MoraleState is a state record that is merely unread; UnitStance is a "
            "two-variant enum that can never modulate anything by construction, so it "
            "is the weaker binding of the pair. The orphaned-consumer problem is shared: "
            "the same greps show UnitStance is likewise referenced only in morale.rs and "
            "in tests, never in war_bridge.rs, movement.rs or military_phases.rs.",
            "MoraleState carries FR-CIV-MORALE in its own doc comment and is legitimately "
            "tagged FR-CIV-TACTICS-076 by its test coverage, so removing FR-CIV-WAR-021 "
            "leaves the type's real provenance intact and loses no true coverage. The "
            "test at crates/tactics/tests/fr_fr_civ_war_021.rs:15 asserts only that "
            "casualties flip stance from Standing to Routing on an isolated instance; "
            "that is a unit test of the struct, not evidence that psyche drives combat.",
        ],
    ),
    (
        "crates/tactics/src/morale.rs",
        "UnitStance",
        {
            "FR-CIV-WAR-021": (
                "CONTAINER-ONLY. The requirement is that emergent agent psyche modulates "
                "fire discipline, rout threshold and willingness to follow orders, and "
                "that casualties and atrocities write grievance and trauma back into "
                "psyche (docs/design/warfare.md:111-113). UnitStance at "
                "crates/tactics/src/morale.rs:87 is a two-variant marker enum "
                "(Standing, Routing) whose entire body is two unit variants. A marker "
                "cannot read psyche, modulate anything, or write anything back, so this "
                "is the weakest possible claim of the nine. The morale computation that "
                "does exist lives on the sibling type, MoraleState::stance at "
                "crates/tactics/src/morale.rs:211, and even that is only a comparison of "
                "one scalar against one threshold. git grep -n -E 'fear|loyalty|grievance|"
                "trauma|ideolog' -- crates/tactics/src/ returns nothing, so three of the "
                "four named psyche inputs and the entire write-back half of the "
                "requirement have no symbol anywhere in the crate."
            ),
        },
        [
            "Same id as the MoraleState entry, judged independently as instructed. The "
            "two sites differ in severity, not in verdict: MoraleState at least holds "
            "the morale scalar, UnitStance holds nothing but a classification. Neither "
            "is consumed outside morale.rs and test files, so the requirement's 'per-"
            "soldier behavior reads psyche' clause is unimplemented at both sites.",
            "If a future change wants to discharge FR-CIV-WAR-021, the correct home is "
            "not either of these declarations. It is the combat and movement path that "
            "would have to consult stance before firing or advancing, which today is "
            "WarBridge::resolve_combat in crates/tactics/src/war_bridge.rs and "
            "operational_movement_pulse in crates/tactics/src/movement.rs:51, neither "
            "of which reads morale at all.",
        ],
    ),
    (
        "crates/tactics/src/movement.rs",
        "OperationalMovementConfig",
        {
            "FR-CIV-WAR-011": (
                "DATA-SHAPE-ONLY. The requirement is that operational movement is driven "
                "by theater objectives plus supply gradients, so that formations advance "
                "toward objectives only while supplied and otherwise fall back to supply, "
                "with advance-to-contact and fall-back-to-supply emerging from one utility "
                "comparison (docs/design/warfare.md:83-85). OperationalMovementConfig at "
                "crates/tactics/src/movement.rs:11 is a two-field cadence struct "
                "(cadence_ticks, path_search_radius) and carries no objective vector, no "
                "supply state and no utility comparison. The movement it parameterizes "
                "actively contradicts the requirement: operational_movement_pulse at "
                "crates/tactics/src/movement.rs:51 steers every unit at the nearest enemy "
                "by Manhattan distance, unconditionally, with no supply term and no "
                "objective term in the choice. git grep -n -i -E "
                "'supply|objective|gradient' -- crates/tactics/src/movement.rs returned no "
                "hits, and the same search across all of crates/ for "
                "'objective_gain|supply_efficiency|supply_gradient|theater_objective' "
                "returned nothing at all, so no supply-gradient driver exists anywhere in "
                "the repo. The test at crates/tactics/tests/fr_fr_civ_war_011.rs asserts "
                "only that cadence_ticks > 0 and path_search_radius > 0, which validates "
                "the struct's defaults and nothing about maneuver policy."
            ),
        },
        [
            "The requirement is behavioral by construction: it is about which target a "
            "formation picks, and the only symbol that picks targets is "
            "operational_movement_pulse, not this config. That is a true observation "
            "about where the behavior would live, but it does not make this verdict "
            "IMPLEMENTED-BY-BEHAVIOR, because the supply-gradient utility comparison the "
            "requirement mandates does not exist in operational_movement_pulse either. "
            "The movement code is advance-to-contact only.",
            "docs/design/warfare.md:83 credits 'already in movement.rs' for operational "
            "movement generally, and that much is real and shipped. The credit does not "
            "extend to the objective-and-supply driving that FR-CIV-WAR-011 actually "
            "asks for, which is the gap the tag was papering over.",
        ],
    ),
    (
        "crates/tactics/src/war_bridge.rs",
        "CombatEngagement",
        {
            "FR-CIV-WAR-013": (
                "DATA-SHAPE-ONLY. The requirement is that the operational layer hands a "
                "contact off to the tactical war bridge, that operational supply and "
                "cohesion state parameterize the tactical engagement (low ammo -> fewer "
                "shots, low cohesion -> worse formation integrity and earlier rout), and "
                "that far-from-camera contacts resolve via a statistical combat rollup "
                "logged to the event stream rather than spawning per-soldier voxels "
                "(docs/design/warfare.md:89-94). CombatEngagement at "
                "crates/tactics/src/war_bridge.rs:32 is a per-engagement record "
                "(shooter, target, factions, damage, target_index); its own doc comment "
                "already attributes it to FR-CIV-TACTICS-024, and it is downstream of the "
                "handoff rather than an implementer of it. Of the three mandated "
                "behaviors, one is partially present: the per-soldier near-camera "
                "engagement loop is real and is driven by tick_war_bridge, called from "
                "crates/engine/src/engine/military_phases.rs:175. The two that define "
                "the requirement are absent. Parameterization: the call passes only "
                "tick, config, samples, voxel world and fog, so no supply or cohesion "
                "value reaches combat resolution, and git grep -n -i -E "
                "'cohesion|ammo|supply' -- crates/tactics/src/war_bridge.rs returned "
                "nothing. Rollup: git grep -n -i -E "
                "'rollup|statistical|aggregate_damage|far_from_camera|lod' -- "
                "crates/tactics/src/ returned no hits, so far-from-camera contacts are "
                "resolved by the same per-soldier path, the opposite of what the "
                "requirement demands. The test at "
                "crates/tactics/tests/fr_fr_civ_war_013.rs only constructs the struct and "
                "asserts a field round-trips."
            ),
        },
        [
            "The two FR-CIV-WAR-013 sites do not agree. DamageEvent in lib.rs is the "
            "damage payload and has no relationship to a handoff at all; CombatEngagement "
            "here is the genuine per-engagement record of the war bridge, so it is the "
            "more defensible of the two bindings and the closer to the requirement's "
            "vocabulary. It still fails, because the requirement's substance is the "
            "supply/cohesion parameterization and the statistical rollup, and CombatEngagement "
            "holds no field that feeds either.",
            "This is the closest call in the batch and the one most likely to be a false "
            "negative on removal. If FR-CIV-WAR-013 is ever partially credited, the "
            "near-camera handoff scaffold in military_phases.rs:175 plus this type is "
            "where it belongs, and the tag would need to move to tick_war_bridge in "
            "crates/tactics/src/war_bridge.rs rather than sit on a struct. Nothing in the "
            "current code justifies asserting full coverage here.",
        ],
    ),
]
