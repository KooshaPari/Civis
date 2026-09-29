"""Verdicts for the requirement tags on data-type declarations in `crates/tactics`.

Scope: requirement tags bound to declarations in `crates/tactics`. Each tag claims
"this symbol implements this requirement". The assigned nine were nine tags sitting
directly above a struct/enum/const declaration; eight of those claims are false and
are listed in SITES for removal, while one (FR-CIV-FOG-001 on `FogOfWar`) is
genuinely discharged and is recorded in KEEP.

Four further ids were adjudicated in a second pass at the coordinator's request:
the line-27 FR-CIV-WAR-030 tag on `score_doctrine_fitness` (a function, and judged
as a function), plus two ids surfaced by a dry run, FR-CIV-MORALE on `MoraleState`
and FR-CIV-TACTICS-024 on `CombatEngagement`. Those two are doc-comment bindings
(`/// ... (ID)`) rather than standalone tag lines, but `_apply_verdicts.is_tag_line`
returns True for them, so they are mechanically removable and were adjudicated on
the same merits. FR-CIV-MORALE is not a defined requirement id anywhere in the repo
and is therefore removed on different grounds than the rest.

  * FR-CIV-TACTICS-024 -- docs/traceability/fr-3d-matrix.md:115, "Per-soldier
    combat engagements on snapshot." Its own
    docs/traceability/fr-civ-tactics-024/fr-civ-tactics-024-spec.md is an empty
    auto-generated template (STATUS SPEC-TEMPLATE, "## Requirement" followed only
    by an HTML comment), so the matrix row is the only real statement of intent.
  * FR-CIV-MORALE -- no authoritative text exists. See the MoraleState entry.

Authoritative requirement text:
  * FR-CIV-FOG-001 -- agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47-50
  * FR-CIV-WAR-011 / -013 / -021 / -022 / -030 -- docs/design/warfare.md:83-85, 89-94,
    111-113, 114-117, 124-140. That document declares itself authoritative for the
    FR-CIV-WAR-* namespace at docs/design/warfare.md:4 and its header at
    docs/design/warfare.md:3 reads "Design / planner spec. No implementation."
    The agileplus-specs/ tree only defines FR-CIV-WAR-001..004 (civ-006-deep-combat);
    the 011..042 block exists solely in warfare.md, and every one of these test
    files self-describes as "Status: CODE-ONLY-no-spec".

Three ids now appear more than once. Every site is judged independently and every
removal is recorded; pair agreement is stated in the note lines for each
declaration.

Nothing under crates/ was modified to produce this file, and cargo was not run.
_apply_verdicts.py was not run, in any mode.
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
            "Only the FactionEngagementStats and score_doctrine_fitness entries are in "
            "SITES for this module. The score_doctrine_fitness tag at "
            "crates/tactics/src/doctrine_fitness.rs:27 was outside the assigned nine and "
            "was added on request in a second pass; its own SITES entry records the full "
            "reason.",
        ],
    ),
    (
        "crates/tactics/src/doctrine_fitness.rs",
        "score_doctrine_fitness",
        {
            "FR-CIV-WAR-030": (
                "DATA-SHAPE-ONLY, and in this case the function's own body is the "
                "clearest evidence that the tag is false. The requirement is to extend "
                "the doctrine fitness signal beyond tactical engagement stats to include "
                "operational and strategic outcomes -- net theater objective gain, supply "
                "efficiency, own attrition and routs, civilian grievance generated -- so "
                "that doctrines which win battles but lose the war are selected against, "
                "with per-cluster independent libraries and memetic diffusion across "
                "contact networks (docs/design/warfare.md:124-140, AC-WAR-7 at :178). The "
                "coordinator's hypothesis that a function might be the genuine binding "
                "where the struct is not does not survive reading it. "
                "score_doctrine_fitness at crates/tactics/src/doctrine_fitness.rs:28 "
                "takes (doctrine: &Doctrine, stats: &FactionEngagementStats) and returns "
                "the sum of exactly two terms: composition_balance, the mean of "
                "doctrine.unit_composition, and a 'battle' term computed solely from "
                "stats.net_pressure(), stats.engagements_as_shooter and "
                "stats.voxels_removed (:29-39). Every input is a tactical quantity. The "
                "function has no parameter through which an operational or strategic "
                "outcome could enter: no objective gain, no supply state, no attrition, "
                "no rout count, no grievance term exists on either argument. That is the "
                "definition of the pre-extension state the spec describes at "
                "docs/design/warfare.md:125, 'Doctrine fitness today reads tactical "
                "engagement stats only. Extend the fitness signal (spec; not yet coded)'. "
                "Tagging this function with FR-CIV-WAR-030 therefore asserts the exact "
                "opposite of what it does: it is the code the requirement says must be "
                "extended, and it has not been. Corroborating searches: git grep -n -i "
                "-E 'k_terr|k_supl|k_loss|k_grv|civilian_grievance|own_attrition|memetic|"
                "diffusion' -- crates/ returns no hit in the tactics crate (the only "
                "diffusion matches are unrelated civ-agents tech/wardrobe propagation); "
                "the module defines only two public functions, net_pressure (:19) and "
                "this one (:28), so there is no second scoring path elsewhere; and "
                "git grep -n 'score_doctrine_fitness' -- crates/ shows the only "
                "production call sites are crates/engine/src/engine/military_phases.rs:95 "
                "and crates/tactics/src/doctrine_evolution.rs:47, both of which pass "
                "nothing but a &Doctrine and a &FactionEngagementStats, so no operational "
                "or strategic input is available anywhere on the call path. The remaining "
                "references are re-exports and tests, including "
                "crates/tactics/tests/fr_fr_civ_war_030.rs, which asserts only that higher "
                "engagement stats raise fitness and that scoring is deterministic -- "
                "precisely the tactical-only behavior, never the extension."
            ),
        },
        [
            "Adjudicated as a function, not as a data container, at the coordinator's "
            "request. Being a function makes it a more plausible candidate for a "
            "behavioral requirement than any struct in this batch, and it is still false: "
            "it is the pre-extension fitness function the requirement is written to change, "
            "not an implementation of the change.",
            "All three FR-CIV-WAR-030 sites are now adjudicated and all three are false, so "
            "this id disagrees with itself nowhere: crates/tactics/src/lib.rs:106 on "
            "Doctrine, crates/tactics/src/doctrine_fitness.rs:6 on "
            "FactionEngagementStats, and crates/tactics/src/doctrine_fitness.rs:27 here. "
            "The requirement is real and specified, but the extension is unimplemented, "
            "so every tag asserting it is overstating coverage. Whoever implements the "
            "four extra terms can re-add the tag to this function, which is where it "
            "belongs.",
            "find_decl in _apply_verdicts.py:75-85 matches 'pub fn' as well as struct, "
            "enum, const, static, type and trait, and the tag at :27 sits directly above "
            "this pub fn with no intervening doc line, so this site is mechanically "
            "removable in the same way as the struct sites.",
        ],
    ),
    (
        "crates/tactics/src/morale.rs",
        "MoraleState",
        {
            "FR-CIV-MORALE": (
                "PHANTOM ID, NOT A REQUIREMENT. Unlike every other entry in this file, "
                "this id cannot be judged against requirement text because no such "
                "requirement is defined anywhere in the repo. FR-CIV-MORALE occurs in "
                "exactly two files, one of which is this audit file: "
                "crates/tactics/src/morale.rs, where it appears in the module doc at "
                ":1, in the struct doc at :139, and in six test doc comments at :358, "
                ":397, :430, :470, :490 and :520. Searches that came back empty: "
                "git grep -rn 'FR-CIV-MORALE' outside crates/tactics/src/morale.rs; "
                "git grep -n -A6 'FR-CIV-MORALE' -- docs/specs agileplus-specs "
                "docs/design docs/guides; a case-insensitive search for 'morale' in "
                "docs/traceability/index.md, which enumerates the repo's requirement ids "
                "and contains no MORALE row; a check for a "
                "docs/traceability/fr-civ-morale/ directory, which does not exist; and a "
                "scan of docs/audits/_id_inventory_v3.json, whose keys contain no "
                "MORALE entry. The only MORALE-adjacent hit anywhere in the spec tree is "
                "an unrelated constant MORALE_ATTRITION_FACTOR at "
                "docs/specs/CIV-0105-war-diplomacy-shadow-v1.md:232, which belongs to a "
                "different id scheme. So the binding cannot be wrong in the sense of "
                "misplacing a real requirement, because there is no requirement to place. "
                "It is a self-referential id invented so the morale module's own unit "
                "tests read as requirement coverage. That is the phantom-tag pattern this "
                "audit exists to strip, and the id should be struck rather than "
                "re-pointed at some real requirement: if morale behavior is meant to be "
                "tracked, a spec has to be written first. Note this id is a doc comment, "
                "/// Per-unit morale tracker (FR-CIV-MORALE)., not a standalone tag line; "
                "_apply_verdicts.is_tag_line still returns True for it, so it is "
                "mechanically removable."
            ),
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
            "FR-CIV-TACTICS-024": (
                "DATA-SHAPE-ONLY, and the true implementing symbol is a different one. "
                "The requirement is 'Per-soldier combat engagements on snapshot.' "
                "(docs/traceability/fr-3d-matrix.md:115), whose own spec file is an empty "
                "auto-generated template: "
                "docs/traceability/fr-civ-tactics-024/fr-civ-tactics-024-spec.md is "
                "marked 'Status: SPEC-TEMPLATE (auto-generated 2026-09-16)' at :2 and its "
                "'## Requirement' section at :8-10 contains only an HTML placeholder "
                "comment, so the matrix row is the only real statement of intent. Two "
                "things are demanded: that engagements be resolved per soldier, and that "
                "they appear on the snapshot. Both are genuinely implemented, but not by "
                "this struct. Per-soldier resolution is tick_war_bridge in "
                "crates/tactics/src/war_bridge.rs, driven from "
                "crates/engine/src/engine/military_phases.rs:175, and the snapshot "
                "surface is the per-soldier damage pulse vector at "
                "crates/engine/src/engine.rs:842, documented '(FR-CIV-TACTICS-024)', "
                "which holds SmallVec<[CombatDamagePulse; 8]> and is wired at :3498 as "
                "feeding doctrine fitness and the server /sim/state wire. The requirement "
                "is accepted by a real test, "
                "war_bridge_records_combat_replay_events at "
                "crates/engine/src/engine/engine_tests.rs:1807, which ticks a Simulation "
                "and asserts ReplayEvent::Combat entries with non-zero shooter and "
                "target ids appear in the replay log. CombatEngagement itself is a "
                "six-field per-engagement record (crates/tactics/src/war_bridge.rs:32) "
                "and does not resolve engagements or publish them on any snapshot. So "
                "the coverage is real and the binding is wrong: this is IMPLEMENTED-BY-"
                "BEHAVIOR, and the tag belongs on tick_war_bridge and on the "
                "last_tick_combat_pulses field at engine.rs:842, "
                "snapshot field, which already carries it. Because the behavior is real, "
                "removing this id loses no coverage as long as the engine.rs:842 binding "
                "is retained."
            ),
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
