"""Verdicts for the miscellaneous requirement tags that sit on data containers.

Every declaration in `SITES` below is a POD/record/enum. Under the audit rule
for this pass ("a tag on a struct does not discharge a behavioral requirement
merely because a field is plausibly named"), none of them can carry the
behavior its id demands, with two partial exceptions noted per site. Each id
was checked against its authoritative requirement text (quoted in the reason)
and against the real code in `crates/`, and the true implementing symbol is
named wherever one exists.

FR-CIV-MOD-000 carries a namespace collision and is handled specially: the
CIV-0700 series has no `-000` at all, so the two mod-host sites are the only
candidates for the modding-platform reading, and neither reads it correctly.
The two definitions are also self-contradictory about the format (RON primary
per the index row at modding-platform.md:26 and :91; JSON parallel per the
section body at :92-94), so no struct can satisfy both without knowing which
is normative. Both tags come off.

Verdicts here are removal-only: ids judged IMPLEMENTED are omitted from
`SITES` and listed in `KEEP` with the evidence that keeps them.
"""


KEEP = {
    "FR-SAVE-009": (
        "Genuinely implemented. The requirement is that the BLAKE3 hash chain tail SHALL be "
        "serialized and restored on load, enabling the chain to continue unbroken from the "
        "saved tick (docs/specs/CIV-1000-save-load-persistence-spec.md:2808). ReplayLog."
        "running_hash is a serde-derived Option<[u8; HASH_LEN]> field "
        "(crates/engine/src/replay.rs:171) that is written and read back through the "
        "replay.civreplay container, and load-time verification is real code: "
        "ReplayLog::verify_hash_chain (crates/engine/src/replay.rs:690) recomputes the chain "
        "from the recorded tick events via recompute_running_hash (:650) and raises "
        "ReplayError::HashChainMismatch (:210) when the stored tail disagrees, so a bad load "
        "is rejected rather than silently resumed. The tag on crates/engine/src/replay.rs:156 "
        "stays."
 ),
}

SITES = [
    (
        "crates/agents/src/culture.rs",
        "CultureProfile",
        {
            "FR-CIV-CULT-001": (
                "NOT IMPLEMENTED. The requirement is a named entity with named fields: "
                "agileplus-specs/civ-009-culture-diffusion/spec.md:24 reads \"Culture entity "
                "`Culture { id, name, ideology_centroid: Fixed, spread_rate: Fixed, resistance: "
                "Fixed }` - each nation has a dominant culture; citizens have `culture_affinity: "
                "Fixed` to each known culture\". CultureProfile is none of that: `git grep -n "
                "\"struct Culture\\b\\|enum Culture\\b\" -- crates/` returns zero hits, so the "
                "Culture entity does not exist under any name. Its four required scalars "
                "(ideology_centroid, spread_rate, resistance) are absent - the crate's only "
                "spread_rate hits are fire/flood hazard fields in crates/climate/src/"
                "disaster_spread.rs, and `git grep -n \"culture_affinity\" -- crates/` returns "
                "zero hits, so the per-citizen affinity the requirement mandates is not "
                "modeled at all. What does exist is an unrelated per-population meme vector "
                "(traits/language/phonemes/contact/kinship, culture.rs:22-35) whose behavior is "
                "language drift and cluster distance, not the CULT-001 entity. No implementing "
                "symbol exists."
 ),
        },
        [
            "CultureProfile is genuinely used: culture_phase_drifts_cluster_profiles "
            "(crates/agents/src/culture.rs:138) mutates the trait vectors, and "
            "cluster_language_distance (:106) reads them. That real work is what "
            "FR-CIV-CULT-002/003 (culture diffusion, ideology convergence) are tagged on "
            "elsewhere; it does not make the type the CULT-001 entity.",
            "The sibling requirement ids in the same spec (CULT-002, CULT-003) do have code "
            "behind them in this file. CULT-001 is the entity-shape one, and no entity of that "
            "shape exists in the repository.",
        ],
    ),
    (
        "crates/agents/src/psyche.rs",
        "Mood",
        {
            "FR-CIV-PSYCHE-002": (
                "IMPLEMENTED-BY-BEHAVIOR. FR-CIV-PSYCHE-002 is not a mood requirement at all. "
                "docs/design/psyche-social.md:273 reads \"FR-CIV-PSYCHE-002 | Temperament "
                "emerges from DNA (reactivity/sociability/risk/impulsivity)\", and the same "
                "doc's AC-1 (line 254) scopes it to genetics->psyche projection. The mood "
                "requirement in that catalog is FR-CIV-PSYCHE-010 (\"Mood emerges from needs "
                "vector + decayed event memory\", psyche-social.md:277, formula at :142-148), "
                "and that is the id currently on the function that does the work. Mood is a "
                "two-field valence/arousal POD with one `neutral()` constructor "
                "(crates/agents/src/psyche.rs:48-64); it cannot emerge anything from anything. "
                "True implementing symbol for PSYCHE-002: psyche_from_dna "
                "(crates/agents/src/psyche.rs:152), which projects reactivity/sociability/"
                "risk_tol/impulsivity out of DNA byte slots via score_axis (:139) -> "
                "civ_genetics::sentience::cognition_score, and is the artifact AC-1 names. That "
                "coverage is not lost; it needs the tag, not this one."
 ),
        },
        [
            "The PSYCHE-002 binding looks authoritative only because "
            "docs/specs/requirements/FR-CIV-PSYCHE.md exists and looks like the spec for this "
            "file. It is a different catalog: it uses a 9xx series "
            "(PSYCHE-900/901/910/911/912/920/921) and contains no -002 or -003. The 0xx "
            "catalog that does define these ids is docs/design/psyche-social.md. Two catalogs, "
            "same prefix, disjoint numbering - the same shape of problem as the MOD-000 "
            "collision below.",
            "Worth noting for whoever re-tags: `git grep \"FR-CIV-PSYCHE-006\"` and `-005` were "
            "already unbound from `Psyche` and `PsychGenomeProfile` in this file, with reasons "
            "that cite the 9xx catalog. Those reasons and this one use different source docs "
            "for ids in the same prefix.",
        ],
    ),
    (
        "crates/agents/src/psyche.rs",
        "Temperament",
        {
            "FR-CIV-PSYCHE-003": (
                "IMPLEMENTED-BY-BEHAVIOR. docs/design/psyche-social.md:274 reads "
                "\"FR-CIV-PSYCHE-003 | Lifelong temperament plasticity gated by maturity\", and "
                ":134 spells out the gate: temperament drifts toward lived-experience "
                "statistics, \"gated by `(1 - maturity*0.8)` so children are plastic, adults "
                "stable\". That gate is real code - nudge_temperament (crates/agents/src/psyche.rs:"
                "188) computes `let plasticity = (1.0 - maturity * 0.8).clamp(0.0, 1.0);` at :194 "
                "and scales the learning rate by it at :195 before nudging reactivity and "
                "sociability at :196-200, with maturity advanced by tick_maturity (:171) from "
                "lived experience and stress. The maturity clock itself is a field on `Psyche` "
                "(:86) and lives on that record, not on Temperament. Temperament here is a "
                "four-f32 POD with only a `neutral()` constructor (psyche.rs:21-43); it has no "
                "maturity, no experience input, and no update rule, so the plasticity behavior "
                "is not on this declaration. True implementing symbol: nudge_temperament "
                "(crates/agents/src/psyche.rs:188). Note the tag directly above that function "
                "currently says FR-CIV-PSYCHE-010 (line 186), which is the mood id - the "
                "plasticity function is the one PSYCHE-003 needs, and PSYCHE-010 is already "
                "discharged by update_mood (:204). Coverage moves, it does not disappear."
 ),
        },
        [
            "A 4-field aggregate of f32 cannot be 'lifelong' or 'plastic'. Lifelong-ness is a "
            "property of the surrounding tick loop that feeds lived experience into the nudge, "
            "not of the vector.",
        ],
    ),
    (
        "crates/civ-traffic/src/lib.rs",
        "InfraProvenance",
        {
            "FR-CIV-ROAD-902": (
                "IMPLEMENTED-BY-BEHAVIOR, and the tagged declaration is the wrong half of it. "
                "docs/specs/requirements/FR-CIV-ROAD.md:13 reads \"All structures + roads SHALL "
                "carry shared data tags regardless of author (procedural vs player), via the "
                "building graph\", with the acceptance note \"`civ-protocol-3d` building graph "
                "tags provenance but exposes uniform query API\". The building-graph half is "
                "real and lives elsewhere: civ_build::BuildingProvenance (Procedural/Freehand, "
                "crates/build/src/lib.rs:43) is stored per parcel in BuildingGraph.provenance "
                "(crates/build/src/lib.rs:319), written by BuildingGraph::set_provenance "
                "(crates/build/src/lib.rs:353) from both the procedural allocator (:274) and the "
                "freehand authoring path (:710), and mapped onto the wire by map_build_"
                "provenance (crates/protocol-3d/src/lib.rs:586). The uniform query API exists "
                "too: civ_build::BuildingGraph::iter-like accessors never filter on provenance, "
                "and the traffic side exposes kind_between / speed_multiplier_at / iter_segments "
                "(crates/civ-traffic/src/lib.rs:309,323,355) that read every segment regardless "
                "of author. But InfraProvenance itself is a two-variant marker "
                "(Emergent/UserPlaced, lib.rs:53-58) and the requirement says structures, i.e. "
                "the building graph, not vehicles or road segments. The tag belongs on the "
                "civ-build / protocol-3d provenance types, not on a traffic-crate enum. True "
                "implementing symbols: civ_build::BuildingProvenance (crates/build/src/lib.rs:"
                "43) and BuildingGraph::set_provenance (crates/build/src/lib.rs:353)."
 ),
        },
        [
            "The enum is genuinely load-bearing inside this crate: record_traffic "
            "(lib.rs:247) branches on it to decide whether a segment may be promoted by traffic "
            "or only upgraded, and place_segment (lib.rs:277) stamps UserPlaced. That is real "
            "behavior, but it is road-growth policy, not the shared-tag/uniform-query contract "
            "the requirement describes.",
        ],
    ),
    (
        "crates/engine/src/building_tiers.rs",
        "BuildingTierEngine",
        {
            "FR-CIV-INFOVIEW-914": (
                "DESIGN-ONLY. docs/specs/requirements/FR-CIV-INFOVIEW.md:18 reads "
                "\"Infrastructure overlays: roads/traffic, building level, service coverage, "
                "transport lines. | Reads emergent architecture/road graph; coverage falloff "
                "shown. | UI (reads [EMERGENT])\", and the family closes with INFOVIEW-920: "
                "\"Each overlay SHALL show a legend (scale + units) and update live as the sim "
                "ticks\" (:19). This is a UI-surface requirement with a UI artifact. "
                "BuildingTierEngine is a headless sim aggregate - `buildings: Vec<Building>`, "
                "`configs: Vec<BuildingTierConfig>`, `next_id: u32` (building_tiers.rs:190-197) - "
                "whose methods are spawn, upgrade, and per-tick cost/maintenance decay. It "
                "renders nothing, has no legend, no per-cell coverage field, and no transport "
                "line representation. `git grep -n \"BuildingTierEngine\" -- crates/` returns "
                "only its own impl, its own unit tests (:395, :535, :557, :607) and two test "
                "files in crates/engine/tests/, so it is not even consulted by any overlay "
                "sampler. The overlay artifact that does exist is a registry row, not this "
                "engine: InfoOverlay { id: \"info_roads\", group: OverlayGroup::"
                "Infrastructure, render_kind: RenderKind::Gizmo } at "
                "crates/engine/src/info_views.rs:346-358, which is metadata about an overlay and "
                "does not read the graph either - and note the design doc assigns the roads "
                "overlay to FR-CIV-INFOVIEW-918 (docs/design/info-views.md:113), not 914. A "
                "sim engine cannot present a legend; that is why this is DESIGN-ONLY rather "
                "than NOT-IMPLEMENTED."
 ),
        },
        [
            "The id is also unstable across sources: docs/design/info-views.md:109 binds "
            "FR-CIV-INFOVIEW-914 to 'Needs Pressure (B2)', a Population-group overlay, while "
            "docs/specs/requirements/FR-CIV-INFOVIEW.md:18 binds it to infrastructure. Either "
            "way BuildingTierEngine is neither.",
        ],
    ),
    (
        "crates/engine/src/lod.rs",
        "ZoomLevel",
        {
            "FR-CIV-TERRAIN-004": (
                "NOT IMPLEMENTED. agileplus-specs/civ-014-terrain-playable-hardening/spec.md:44 "
                "reads \"Map2D zoom levels SHALL round-trip without voxel-data loss; `map2d.zoom` "
                "and `map2d.ux` (issue #2494) SHALL be expressed in the watch HTTP surface and "
                "the web dashboard\". The requirement is an end-to-end property of a Map2D "
                "type plus a watch HTTP field plus a dashboard, and the searched-for type does "
                "not exist: `git grep -n \"Map2D\" -- crates/` returns exactly one hit, and it "
                "is a test file header (crates/engine/tests/fr_fr_civ_terrain_004.rs:5), not a "
                "type. There is no round-trip function (no serialization of a zoom value exists "
                "anywhere) and there is no map2d.zoom field in the watch surface. ZoomLevel is a "
                "two-variant view enum (Strategic/Operational, crates/engine/src/lod.rs:12-17) "
                "whose only consumer is project_zoom (lod.rs:96), and that function is the "
                "identity `(state_tick, zoom) -> (state_tick, zoom)`. An identity function "
                "cannot round-trip anything, and the module header calls the whole file a "
                "'CIV-0101 two-zoom level-of-detail policy (stub)' (lod.rs:1). The file's own "
                "test asserts only that two variants exist and differ "
                "(two_levels_defined, lod.rs:106). No implementing symbol exists."
 ),
        },
        [
            "Note the mismatch between the two TERRAIN specs in play: "
            "agileplus-specs/civ-014 spells it Map2D, while the tag sits on a symbol described "
            "as CIV-0101 two-zoom LOD. Even a perfect identity projection would not satisfy the "
            "civ-014 wording, which names a watch HTTP surface and a web dashboard.",
        ],
    ),
    (
        "crates/genetics/src/lib.rs",
        "DnaClass",
        {
            "FR-CIV-GODTOOL-911": (
                "IMPLEMENTED-BY-BEHAVIOR. docs/specs/requirements/FR-CIV-GODTOOL.md:14 reads "
                "\"Life/spawn tools SHALL seed DNA-bearing organisms; outcome (survival, "
                "speciation, sentience) emerges, never scripted. | Spawn injects `civ-genetics` "
                "DNA; lineage then evolves under laws only\". The spawn tool is real and does "
                "inject DNA: Simulation::apply_god_tool (crates/engine/src/godtools.rs:1033) "
                "dispatches LifeRequest::SpawnOrganism (godtools.rs:326) to "
                "civ_agents::spawn_civilian_at (crates/agents/src/lib.rs:517), which builds a "
                "genome from an archetype seed with divergence - `spawn_genome_with_divergence"
                "(rng, &dna_class, &seed_def, 0.3)` at crates/agents/src/lib.rs:550 - and "
                "attaches it via spawn_civilian (:552). Emergence-not-scripting is real too: "
                "civ_genetics::mutate (crates/genetics/src/lib.rs:97) applies per-byte "
                "class-parameterised point mutation under a seeded ChaCha8Rng and recombine (:108) "
                "does uniform crossover, and speciation falls out of should_speciate (:162) "
                "Hamming distance rather than any authored branch. The requirement is met - by "
                "the spawn tool, not by the genome schema. DnaClass is a four-field config POD "
                "(name/length/mutation_rate/speciation_threshold, lib.rs:71-81) whose `Default` "
                "is the only construction anywhere in the workspace: every single call site uses "
                "`DnaClass::default()` (crates/agents/src/lib.rs:397,548,622; "
                "crates/emergence-oracle/src/oracles/genetics.rs:65,100,113; "
                "crates/engine/src/emergence.rs:229; and every test). A struct that is never "
                "configured and is read only by mutate/speciate cannot be the thing that 'seeds "
                "DNA-bearing organisms'. True implementing symbol: "
                "civ_agents::spawn_civilian_at (crates/agents/src/lib.rs:517), reached from "
                "Simulation::apply_god_tool (crates/engine/src/godtools.rs:1033) - that is the "
                "artifact the test file for this id already exercises "
                "(crates/engine/tests/fr_civ_godtool_cluster.rs:443)."
 ),
        },
        [
            "Not a false positive about capability - the requirement really is satisfied - but a "
            "false positive about attribution. This is the difference the IMPLEMENTED-BY-"
            "BEHAVIOR verdict exists for: the tag asserts that a config struct seeds organisms, "
            "and it does not.",
        ],
    ),
    (
        "crates/mod-host/src/lib.rs",
        "ModMeta",
        {
            "FR-CIV-MOD-000": (
                "DATA-SHAPE-ONLY. Two collisions land on this id, and it fails under both "
                "readings. (1) The CIV-0700 series does not define -000 at all: `git grep -n "
                "\"FR-CIV-MOD-00\" -- docs/specs/CIV-0700-modding-api-spec.md` returns "
                "FR-CIV-MOD-001 through -020 and no -000, so if the mod-host tags were minted "
                "from the spec this crate is named for, the id is a hallucination. Its own test "
                "file is the tell - crates/mod-host/tests/fr_fr_civ_mod_000.rs asserts only that "
                "the four ModType variants equal themselves, which is a tautology. (2) The only "
                "real definition is docs/design/modding-platform.md:26, \"Mod manifest schema "
                "(RON primary / JSON parallel), versioned\", and ModMeta is closer than anything "
                "else in the repo but still does not discharge it. The v1 schema in that doc is "
                "the union of its own two statements about the format - the index row says RON "
                "primary, the section body at modding-platform.md:92-94 says \"The loader keeps "
                "dual-format parsing already in `parse_manifest`\" with JSON already supported - "
                "and the repo has neither: `git grep -n \"ron::\\|manifest.ron\\|manifest.json\" "
                "-- crates/mod-host` returns zero hits and CIVMOD_MANIFEST_NAME is "
                "\"manifest.toml\" (crates/mod-host/src/lib.rs:233). The manifest table also "
                "carries no `sdk` semver-req, no `entrypoint`, no `requires`/`conflicts`/"
                "`load_after`/`load_before`/`priority`, no `provides` file globs, and no "
                "`compat { min_save_schema, max_save_schema }`; those field names have zero hits "
                "across the crate. `version` and `api_version` are plain Strings with no semver "
                "parse and no host-range check, so the only \"versioned\" claim is a field name. "
                "ModMeta is a serde Deserialize record for a [mod] table: it records required "
                "data, it cannot load, validate, version-check, or dual-parse anything."
 ),
        },
        [
            "For the collision, the more damning detail: modding-platform.md is dated 2026-05-30 "
            "and states at the top that it expands `crates/civlab-sdk` (manifest.rs, material.rs, "
            "building.rs) - a different crate from the `civ-mod-host` this tag sits in. The doc's "
            "own words say 'Do NOT duplicate those'. The tagged types are a TOML-only, "
            "post-dating-looking re-implementation of a manifest concept that its cited home "
            "already owns. Which definition wins is a spec-ownership decision and is escalated "
            "in docs/audits/triage-container-protocol-modhost.md; it does not change the verdict "
            "here, because ModMeta satisfies neither candidate.",
        ],
    ),
    (
        "crates/mod-host/src/lib.rs",
        "ModType",
        {
            "FR-CIV-MOD-000": (
                "CONTAINER-ONLY. Same collision as on ModMeta, and the weaker half of it. Under "
                "the CIV-0700 reading there is no FR-CIV-MOD-000 to point at (the series starts "
                "at -001, docs/specs/CIV-0700-modding-api-spec.md:2356). Under the only real "
                "definition, docs/design/modding-platform.md:26 \"Mod manifest schema (RON "
                "primary / JSON parallel), versioned\", ModType is a four-variant serde enum "
                "(Policy/Economic/Event/Scenario, crates/mod-host/src/lib.rs:56-65) that is one "
                "field of ModMeta (`pub mod_type: ModType`, lib.rs:80). It is a value in a "
                "schema, not a schema: no fields, no format handling, no version, no loader. It "
                "cannot express a manifest and the requirement does not ask it to. The tag is "
                "here because the enum name contains 'Mod' and the id ends in 'MOD-000' - the "
                "same name-matching failure mode that put 33 unrelated PvE session ids on "
                "SESSION_HISTORY_CAP in this crate (crates/server/src/session.rs:27-68) and "
                "three Python-package ids on a Rust version string (crates/build/src/lib.rs:50-"
                "63). What ModType really implements is CIV-0700 section 2's mod-type taxonomy, "
                "which is exercised for real: it is matched at lib.rs:352,363,675,722 to route "
                "policy vs economic guest invocations, and rendered by mod_type_label "
                "(lib.rs:986, duplicated at crates/watch/src/mods_api.rs:97)."
 ),
        },
        [
            "Because both mod-host sites carry the same id, both are removed on the same "
            "evidence. Neither is a data shape that satisfies the manifest requirement, and "
            "neither is in the crate the design doc says should own it.",
        ],
    ),
    (
        "crates/voxel/src/material.rs",
        "MaterialDef",
        {
            "FR-CIV-RENDER-002": (
                "DATA-SHAPE-ONLY. The requirement is a rendering pass, quoted from "
                "docs/guides/voxel-emergent-vision-and-migration.md:153: \"Translucent material "
                "pass: liquid and gas cells rendered with alpha-blended geometry; solid cells "
                "rendered opaque first.\" A pass that emits alpha-blended geometry and orders "
                "opaque before translucent does not exist in this repository: `git grep -rn -i "
                "\"translucent\\|alpha_blend\\|AlphaMode\\|transparent\" -- crates/voxel/src "
                "crates/render/src crates/asset-pipeline/src` returns four hits, all of them "
                "comments about uncovered texels in the GPU texture atlas "
                "(crates/voxel/src/atlas/gpu_atlas.rs:150,381,679,681). crates/render has ten "
                "modules (atlas, audio, camera, frame, gltf, hex_map, lib, lod, state, timeline) "
                "and none is a material pass. What exists is exactly the classification data a "
                "pass would read: MaterialDef carries `color: [u8; 4]` (material.rs:65) and "
                "`phase: Phase` (:35) with is_liquid/is_gas helpers (:94,100). That is the "
                "headless-checkable half, and the crate's own test says so in as many words - "
                "crates/voxel/tests/fr_civ_render_002_translucency.rs:19-23 states \"the geometry "
                "pass itself lives in the Bevy client, which needs a GPU and is not exercised "
                "here. What is verified is the classification the pass reads\". The test also "
                "records that the shipped palette does not even split strictly on phase (lava "
                "is an opaque liquid, glass a translucent solid, :128-136), so the data is not a "
                "faithful encoding of the rule either. MaterialDef records the data; the pass "
                "the requirement asks for is absent."
 ),
        },
        [
            "This is the clearest false positive of the batch in the sense that the tag is on the "
            "one type whose doc comment literally says 'RGBA render hint for engine adapters' "
            "(material.rs:64) - the phrasing makes the binding read as obvious, and it is still "
            "a data field standing in for a render pass that nobody wrote.",
        ],
    ),
    (
        "crates/voxel/src/window/mod.rs",
        "WindowPolicy",
        {
            "FR-CIV-RENDER-001": (
                "DATA-SHAPE-ONLY. The requirement is two clauses, quoted from "
                "docs/guides/voxel-emergent-vision-and-migration.md:152: \"Bevy chunk streamer "
                "loads and unloads chunks within a 3-chunk camera radius; no render-thread "
                "stalls (NFR-CIV-SCALE-002).\" The first half is genuinely implemented, and not "
                "by this struct's declaration: WindowPolicy::classify (crates/voxel/src/window/"
                "mod.rs:233) is the pure function that maps (coord, anchor, policy) to a "
                "ChunkState - Meshed inside mesh_ring, Fading through the seam band, Resident out "
                "to coarse_ring - and it is the load/unload decision the renderer's per-frame "
                "plan calls. That decision is exercised end to end by "
                "crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs, whose first test "
                "loads rings 0..=3 and unloads ring 4 under a three-chunk policy. WindowPolicy "
                "itself is a ten-field config POD whose only methods are `checked` (a validating "
                "constructor, :190) and the Default impl (:163); it performs no load, no unload, "
                "and owns no clock, so it cannot discharge the streaming contract on its own. "
                "The second half - 'no render-thread stalls' - has no artifact at all: that needs "
                "a frame loop and timing, and the test file concedes it at lines 14-16 (\"The "
                "geometry pass and the 'no render-thread stalls' timing clause need a GPU and a "
                "frame loop, which are out of scope here\"). Half the requirement is implemented "
                "by classify and half is unimplemented; the struct that holds the constants "
                "discharges neither half on its own, so the binding overstates what this "
                "declaration provides."
 ),
        },
        [
            "Borderline call, and the reason it lands on removal rather than keep: if the audit "
            "treated configuration structs as discharging the requirements they parameterize, "
            "half this repository's tags would survive. The rule set for this pass does not. The "
            "honest summary is that the streaming behavior lives in classify, the timing clause "
            "lives nowhere, and WindowPolicy is a knob panel.",
        ],
    ),
    (
        "crates/server/src/session.rs",
        "SharedSession",
        {
            "FR-CIV-SERVER-001": (
                "IMPLEMENTED-BY-BEHAVIOR, against the wrong id. agileplus-specs/"
                "civ-021-recovered-requirements/spec.md:221-222 is a rename table: "
                "`FR-CIV-SERVER-001` -> `FR-CIV-SERVER-001-WS`, with the note that the real "
                "WebSocket server is the hyphenated form. So the id on this declaration is a "
                "STALE-ID that the repo's own recovery spec retired; the live id is "
                "FR-CIV-SERVER-001-WS. And the tagged type is a per-connection record, not a "
                "server: SharedSession holds connection_id, connected_at, role, "
                "subscribed_frame_kinds, last_acked_tick, tick_broadcasts_received, closed "
                "(crates/server/src/session.rs:84-128). It cannot accept a connection. True "
                "implementing symbol for the WebSocket server: the bridge in "
                "crates/server/src/ws_bridge.rs, which binds a TcpListener (:588) at "
                "127.0.0.1:3800 (:150, default also in crates/server/src/main.rs:86) and "
                "services the upgrade. One caveat on that symbol, recorded so it is not lost: "
                "the spec text this id traces to names port 9876 (docs/specs/"
                "CIV-0001-core-simulation-loop.md:373, \"WebSocket connect to ws://localhost:"
                "9876/sim\"), and `git grep -rn \"9876\" -- crates/` returns zero hits - the "
                "bridge is on 3800, which crates/protocol-3d/src/lib.rs:67 already records for "
                "the sibling FR-CIV-PROTO-002. So the requirement is implemented under the wrong "
                "port; that is a spec/implementation drift to raise separately, not a reason to "
                "keep a stale id on a session record."
 ),
        },
        [
            "The test file for the live id is itself a stub and should not be counted as "
            "evidence: crates/server/tests/fr_fr_civ_server_001_ws.rs asserts only "
            "`SESSION_HISTORY_CAP > 0`, which is the constant that this crate already had 33 "
            "unrelated ids unbound from (session.rs:27-68).",
        ],
    ),
    (
        "crates/server/src/session.rs",
        "SessionSnapshot",
        {
            "FR-CIV-SERVER-002": (
                "IMPLEMENTED-BY-BEHAVIOR, against the wrong id and the wrong scope. "
                "agileplus-specs/civ-021-recovered-requirements/spec.md:223-224 is the same "
                "rename table: `FR-CIV-SERVER-002` -> `FR-CIV-SERVER-002-PROTO`, noting that "
                "the real protocol is the hyphenated form, so the id here is a retired STALE-ID. "
                "Even taking the un-retired reading, the requirement is a client/server message "
                "protocol, and `git grep -rn \"ClientMessage\\|ServerMessage\" -- crates/server/"
                "src/` returns zero hits: there are no such types. What the crate actually has is "
                "a JSON-RPC surface in crates/server/src/jsonrpc.rs, and this struct is a "
                "response envelope for one method. SessionSnapshot is a four-field record - "
                "connection_id, last_acked_tick, tick_broadcasts_received, "
                "snapshot: Option<SnapshotFields> (crates/server/src/session.rs:193-203) - with "
                "two constructors that copy session fields and wrap an engine payload "
                "(:208, :224). It defines no method, no variant, and no framing; it cannot be a "
                "protocol. True implementing symbols: the JSON-RPC request/response types in "
                "crates/server/src/jsonrpc.rs, surfaced over the ws_bridge transport. A stale id "
                "on a response DTO is strictly worse than no tag: it inflates the coverage count "
                "for a protocol that is implemented under a different id, and the id it "
                "advertises has no definition left in the repo."
 ),
        },
        [
            "Same stub-test caveat: crates/server/tests/fr_fr_civ_server_002_proto.rs also "
            "asserts only `SESSION_HISTORY_CAP > 0`.",
            "Both SERVER tags fail the same way, so both come off together: the recovery spec "
            "at civ-021 retired these exact ids, and each is parked on a per-connection data "
            "struct rather than on the transport or protocol that would discharge it.",
        ],
    ),
    (
        "crates/research/src/lib.rs",
        "ReplayMode",
        {
            "FR-CIV-TECH-007": (
                "IMPLEMENTED-BY-BEHAVIOR. docs/design/tech-engineering.md:231 reads \"Canonical "
                "saves never emit `LlmEvent`s; invention draws from canonical + diffused "
                "candidates only. | replay-mode test | AC-P1\". That gate is real and it is the "
                "match arm in replay_advance_llm_event (crates/research/src/lib.rs:182): "
                "`ReplayMode::Canonical => ReplayAdvanceOutcome::Refused(ReplayRefusal::"
                "CanonicalLlmEvent)` at :193, with Hybrid/Free requiring a cache hit at :194-200 "
                "and the early `if !is_replay` live-play escape at :188-190. The AI-side twin "
                "exists too (crates/ai/src/provenance.rs:111-112). ReplayMode is a "
                "three-variant serde enum (Canonical/Hybrid/Free, "
                "crates/research/src/lib.rs:115-122) and by construction it cannot refuse "
                "anything: the refusal is a property of the function that matches on it. Two "
                "secondary problems reinforce the removal. (1) The tag sits directly above the "
                "enum, but the file's own tags put FR-CIV-TECH-009 - the sibling requirement "
                "about exactly this replay path (\"Hybrid/Free replay reproduces accepted cards "
                "from cache; cold cache halts loudly (`HybridCacheMiss`)\", "
                "tech-engineering.md:233) - directly above the function that implements the "
                "cache-miss half (crates/research/src/lib.rs:176). TECH-007 is the mode "
                "declaration; TECH-009 is the behavior. One function serves both, and the "
                "existing tag is on the other id. (2) The requirement's second clause, that "
                "invention draws from canonical + diffused candidates only, is not discharged "
                "anywhere in this file: the tag names a mode enum, not a candidate-selection "
                "rule. True implementing symbol: replay_advance_llm_event "
                "(crates/research/src/lib.rs:182), verified by the replay-mode test the "
                "requirement names (crates/research/tests/fr_civ_tech_tests.rs:146,159)."
 ),
        },
        [
            "The distinction from FR-SAVE-009 on ReplayLog, kept in the same file: there the "
            "tagged type owns the hash-chain tail and a real verifier enforces it on load, so "
            "the binding stands. Here the enum only names a mode.",
        ],
    ),
]
