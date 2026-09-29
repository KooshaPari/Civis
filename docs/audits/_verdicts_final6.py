"""Verdicts for the LAST 6 requirement-id tags bound to Rust data containers.

Scope: the six remaining sites where a requirement tag sits directly above a
struct / enum / const declaration. Every other container binding in the repo has
been adjudicated; this file closes the set. Each site was judged independently on
the merits -- the authoritative requirement text against the actual symbol -- with
no reliance on what a prior ruling concluded.

Outcome: 3 GENUINE (kept), 3 FALSE (removed). This file OVERTURNS three prior
rulings, two of which the coordinator explicitly asked to be re-examined and one
of which they did not flag. The overturns are documented with negative evidence
rather than left implicit.

  * FR-CIV-3D-011 on BiomeKind -- OVERTURNED from GENUINE to FALSE. The prior
    ruling in _verdicts_unclassified.py:23 ("BiomeKind's first six variants are
    literally the six biomes the terrain-generation coverage requirement names")
    reasons from the enum's variant names. The requirement is not about names.
  * FR-CIV-PROTO-006 on DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL -- OVERTURNED from
    GENUINE to FALSE. The prior in-file note at bundle.rs:39-41 accepted it on
    the argument that "a zstd level is exactly the data the requirement names".
    That argument establishes a data match, not an implementation.
  * FR-CIV-PROTO-006 on Frame3dBundleEncodeOptions -- OVERTURNED from GENUINE to
    FALSE, same reasoning, same file.

The three sites that survive were re-derived from source, not rubber-stamped:
  * FR-CIV-0104-003 on ConstraintState -- the sticky-flag logic is real, and the
    negative cases (Critical/Warning must NOT trip it) are tested.
  * FR-SAVE-009 on ReplayLog -- the chain tail is a serde field, is re-verified
    on load, and has a whole-bundle round-trip test plus a tamper-rejection test.
  * FR-CIV-FOG-001 on FogOfWar -- recomputes from scratch each call, applying
    exactly the three named inputs.

Authoritative requirement text, quoted in each entry:
  * FR-CIV-0104-003 -- docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1464-1467
  * FR-CIV-3D-011   -- docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968-1972
  * FR-CIV-PROTO-006 -- docs/specs/CIV-0200-client-protocol.md:1149-1152
  * FR-SAVE-009    -- docs/specs/CIV-1000-save-load-persistence-spec.md:2808
  * FR-CIV-FOG-001 -- agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47-50

Nothing under crates/ was modified to produce this file, and cargo was not run.
_apply_verdicts.py was run in DRY-RUN mode only; --apply was never passed.
"""

KEEP = {
    "FR-CIV-0104-003": (
        "GENUINE, prior ruling CONFIRMED by re-derivation from source. The "
        "requirement is 'Any HALT violation sets `ablation_mode = true` on the run "
        "permanently', with the test clause 'Trigger HALT violation; verify all "
        "subsequent snapshots have `ablation_mode: true`' "
        "(CIV-0104-minimal-constraint-set-theorem.md:1464-1467). This is a "
        "behavioral requirement about a flag, and the type plus its impl carry it: "
        "ConstraintState owns `ablation_mode` (constraints.rs:574, documented at "
        ":573 as 'Whether a HALT violation has been observed (permanent once set)'), "
        "and ConstraintState::update at constraints.rs:593 implements permanence "
        "literally by guarding the assignment with `if !self.ablation_mode` at :601, "
        "so once set it can never be cleared by a later passing tick. The severity "
        "filter is exact rather than approximate: the setter at :602-610 matches "
        "only `ConstraintCheck::Violated { severity: ViolationSeverity::Halt, .. }`, "
        "so Warning and Critical do not trip it. That negative behavior is tested "
        "rather than assumed, in crates/engine/tests/fr_fr_civ_0104_003.rs:59 "
        "(non_halt_violations_do_not_set_ablation) and :98 "
        "(multiple_halts_keep_ablation_mode), alongside the positive "
        "halt_sets_ablation_mode_permanently at :20, which asserts the flag still "
        "reads true after a subsequent all-Ok tick at :50-54. That last assertion is "
        "the requirement's own 'verify all subsequent' clause. The default is false "
        "(:584), so the pre-condition is right too. Caveat worth recording, which "
        "does not change the verdict: ConstraintState has no production consumer "
        "outside constraints.rs itself (git grep -n 'ConstraintState\\|"
        "constraint_state' -- crates/ returns nothing outside that file and its "
        "tests), so `ablation_mode` does not yet reach a real sim.snapshot the way "
        "the test clause's word 'snapshots' implies. The flag semantics -- the "
        "substance of FR-CIV-0104-003 -- are correctly implemented; wiring it into "
        "the published snapshot is the separate, untagged concern of the Phase 5 "
        "integration in the same spec. This is a genuine binding on the tagged "
        "symbol, not on some other one."
    ),
    "FR-SAVE-009": (
        "GENUINE, prior ruling CONFIRMED by re-derivation from source. The "
        "requirement is 'The BLAKE3 hash chain tail SHALL be serialized and restored "
        "on load, enabling the chain to continue unbroken from the saved tick' "
        "(CIV-1000-save-load-persistence-spec.md:2808). Every clause of that sentence "
        "has a named implementing line, and the type is the one that owns them. "
        "Serialized: `running_hash: Option<[u8; HASH_LEN]>` at replay.rs:171 is a "
        "serde-derived field on a `#[derive(Serialize, Deserialize)]` struct "
        "(replay.rs:158), so it round-trips through the `replay.civreplay` RON "
        "codec written by ReplayLog::save (replay.rs:710, `ron::to_string`) and read "
        "back by ReplayLog::load (replay.rs:717, `ron::from_str`). Restored on load: "
        "ReplayLog::load calls log.verify_hash_chain() at replay.rs:721 before "
        "returning the log, so a corrupted or forked tail is rejected at the load "
        "boundary rather than trusted. Continues unbroken: ReplayLog::record_tick "
        "chains from the stored tail rather than from genesis, taking "
        "`self.running_hash.unwrap_or(GENESIS)` at replay.rs:644 and advancing it at "
        ":645, which is precisely the mechanism by which the chain survives the load "
        "boundary. verify_hash_chain at replay.rs:690 is a real verifier, not a "
        "stub: it recomputes the root from the event payloads "
        "(recompute_running_hash at :650, folding Tick/Combat/Climate/Research "
        "markers through chain_root_from_payloads) and compares against the stored "
        "value, erroring with ReplayError::HashChainMismatch at :698 on divergence. "
        "The evidence is behavioral, not structural: "
        "crates/engine/src/save_bundle.rs:2326 "
        "(fr_save_009_hash_chain_tail_survives_bundle_roundtrip) exercises the whole "
        "named path rather than the codec alone -- it ticks a real Simulation, "
        "records ticks, saves through CivSaveBundle::save_archive (:2348), reloads "
        "with load_archive (:2350), asserts the tail came back equal (:2357), and "
        "then proves the chain CONTINUES by recording tick 999 on both the "
        "pre-save and post-load logs and asserting the resulting roots match "
        "(:2370-2378). That is the spec's 'continue unbroken from the saved tick' "
        "clause as an executed assertion. A negative test at :2385 "
        "(fr_save_009_tampered_chain_tail_is_rejected_on_load) corrupts the last "
        "byte of the embedded tail and asserts the load fails, proving the field is "
        "load-bearing rather than decorative. This is the strongest binding in the "
        "final six: the tag names the type that genuinely owns the required state."
    ),
    "FR-CIV-FOG-001": (
        "GENUINE, CONFIRMED (also re-derived here rather than inherited). The "
        "requirement is 'Fog-of-war visibility SHALL be a function of (unit "
        "position, vision radius, terrain LOS). A target is \"visible\" iff the "
        "line-of-sight check from any friendly unit passes; vision state is "
        "deterministic given the same unit set and terrain' "
        "(agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:"
        "47-50). The spec names exactly three inputs and the implementation applies "
        "exactly those three, no more and no fewer. FogOfWar::update at "
        "crates/tactics/src/fog_of_war.rs:81 begins with `self.visible.clear()` at "
        ":86, so visibility is recomputed from scratch on every call and there is no "
        "incremental state that could diverge between two runs over the same inputs "
        "-- that is what discharges 'deterministic given the same unit set and "
        "terrain' structurally rather than by assertion. Per friendly unit it applies "
        "the three named inputs: unit position as the origin (ux, uy at :105-106), "
        "vision radius as a Euclidean range test against self.vision_radius "
        "(:123-127, which `continue`s past cells outside `r`), and terrain LOS via "
        "line_of_sight_grid(voxel_world, ...) at :129. The spec's 'iff' -- any "
        "friendly unit, not all -- is implemented as an OR across the faction's "
        "units: the loop at :104 filters `faction_units` by faction_id and every "
        "passing cell sets `bitmap[idx] = true` at :131, never clearing it, so "
        "union semantics are exact. The struct is not a passive record; the type plus "
        "its impl carry the required function, so a tag on the struct is the correct "
        "placement. Relationship to the sibling ids: FOG-002..005 were already "
        "removed from this same declaration by _verdicts_fog.py (see the "
        "'[unbound]' block at fog_of_war.rs:45-48), and this entry agrees with that "
        "file -- FOG-001 is the one id in the block the struct genuinely discharges, "
        "which is exactly why it is the one that survives. No prior ruling for this "
        "site was contested; it is listed here so the final six are complete in one "
        "place and so the confirmation is on the record rather than implied."
    ),
}

SITES = [
    (
        "crates/planet/src/geology.rs",
        "BiomeKind",
        {
            "FR-CIV-3D-011": (
                "DATA-SHAPE-ONLY AND, DECISIVELY, NOT SATISFIED BY THE GENERATOR. "
                "This OVERTURNS a prior GENUINE ruling and the overturn rests on the "
                "requirement's own words. The requirement is 'The terrain generation "
                "system SHALL produce terrain featuring all six biome types (ocean, "
                "plains, forest, desert, tundra, mountain) when using the default "
                "scenario seed set', verified by 'generate terrain with 20 different "
                "seeds; assert all 6 biomes appear in at least 15 of the 20 generated "
                "maps' (CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:"
                "1968-1972). The prior ruling, recorded verbatim at "
                "docs/audits/_verdicts_unclassified.py:23, is that 'BiomeKind's first "
                "six variants are literally the six biomes the terrain-generation "
                "coverage requirement names'. That reasons from variant NAMES to a "
                "generation COVERAGE claim, which is the name-match pattern this "
                "audit exists to reject. A requirement about what a generator emits "
                "is not discharged by an enum that can name the values. Three "
                "independent lines of evidence show the behavior is absent or false. "
                "(1) THE ENUM CANNOT PRODUCE THE SIX. BiomeKind at "
                "crates/planet/src/geology.rs:29 has 17 variants, and the six the "
                "requirement names are a strict subset -- it is a superset, so it is "
                "not a type whose existence demonstrates six-biome coverage. "
                "(2) THE GENERATOR ACTUALLY FAILS THE DEFAULT-SEED CLAUSE. The only "
                "terrain generator, GeologyMap::seed at geology.rs:339, assigns each "
                "of 16 regions by latitude band and emits Desert only when "
                "`axial_tilt_deg < 10` (geology.rs:358-359) and Forest only when "
                "`axial_tilt_deg > 30` (geology.rs:360-361), with everything else "
                "equatorial falling through to Plains at :362-363. The default "
                "PlanetConfig sets `axial_tilt_deg: 23` at crates/planet/src/lib.rs:"
                "92, which is neither <10 nor >30. Under the default config the "
                "generator therefore emits ONLY FOUR of the six named biomes -- "
                "Ocean (:352), Tundra (:354), Mountain (:356), Plains (:363) -- and "
                "NEVER Desert and NEVER Forest. The requirement says all six 'when "
                "using the default scenario seed set'. The default seed set does not "
                "produce them, so the binding asserts coverage of a clause the code "
                "contradicts. (3) THE SPEC'S VERIFICATION DOES NOT EXIST. The clause "
                "names `crates/terrain/tests/biome_coverage_test.rs`; there is no "
                "crates/terrain crate at all (dir /b crates lists agents, ai, ... "
                "planet, ... voxel, watch -- no terrain), so the named test was never "
                "written. The only `biome_coverage` in the repo is an unrelated voxel "
                "helper at crates/voxel/src/procedural.rs:259 with its own test at "
                ":353. The sole FR-CIV-3D-011 test, "
                "crates/engine/tests/fr_fr_civ_3d_011.rs, is SYNTHETIC: "
                "all_six_biomes_exist at :16 builds a local array of six biome "
                "constants and asserts `biomes.len() == 6` at :25, which is a fact "
                "about the test's own array literal and never reads the enum, never "
                "constructs a GeologyMap, and never generates terrain; "
                "biomes_are_distinct at :30 only compares Debug strings of those same "
                "constants. Both tests would pass unchanged if BiomeKind were deleted, "
                "and they would equally pass if the generator emitted only Plains. "
                "The richer per-cell classifier classify_biome at geology.rs:115 can "
                "reach Desert/Forest, but it is driven by elevation/temperature/"
                "moisture rather than by a seed set, is not reachable from "
                "GeologyMap::seed, and no test asserts multi-seed biome coverage "
                "through it either. Remove the tag. The genuine precondition here is "
                "the enum's completeness, and the prior ruling was right about THAT "
                "and wrong to read it as the requirement. Implementing the clause "
                "means making GeologyMap::seed (or a new seeded terrain path) emit all "
                "six under defaults and adding the 20-seed test."
            ),
        },
        [
            "This is one of the two sites the coordinator asked for a second opinion "
            "on, having been previously ruled GENUINE. My verdict is that the prior "
            "ruling was WRONG, and the reason is not a technicality: the prior "
            "ruling's own stated grounds -- 'BiomeKind's first six variants are "
            "literally the six biomes the ... requirement names' "
            "(_verdicts_unclassified.py:23) -- are a name match, which the audit "
            "brief explicitly rules out as evidence.",
            "The requirement's central clause is generation coverage under the DEFAULT "
            "seed set. I checked the default rather than assuming it, and "
            "GeologyMap::seed at geology.rs:339 with the default axial_tilt_deg of 23 "
            "(crates/planet/src/lib.rs:92) emits four of six biomes: Desert and Forest "
            "are unreachable at that tilt. So the tag does not merely rest on weak "
            "evidence, it is contradicted by the code it claims to describe.",
            "The supporting evidence is independently bad too: the spec's named "
            "verification file does not exist (no crates/terrain), and the only test "
            "bound to this id is synthetic -- it asserts the length of an array the "
            "test itself declares. Nothing in this site survives contact with the "
            "instruction not to manufacture coverage.",
            "Note the sibling id already removed from this same declaration by an "
            "earlier pass, recorded at geology.rs:26: FR-CIV-3D-015 (Texture Atlas "
            "Completeness) was correctly removed because a complete biome enum is a "
            "precondition for atlas coverage but does not establish it. That "
            "reasoning applies verbatim here -- a complete biome enum is the "
            "precondition for biome COVERAGE and does not establish it. The two "
            "removals should stand together, and I think it is a sign the FR-CIV-3D-"
            "011 ruling was made without the FR-CIV-3D-015 precedent next to it.",
            "MECHANICAL BLOCKER ON THIS SITE, READ BEFORE TRUSTING THE DRY-RUN COUNT. "
            "The FR-CIV-3D-015 removal above was already applied in commit 8d719130, "
            "which wrote the marker line '// The following 1 requirement tags were "
            "removed from BiomeKind.' at geology.rs:19. _apply_verdicts.py:136-138 "
            "guards idempotence by scanning the WHOLE FILE for that header string, so "
            "it reports this site as 'already processed' and never reaches my "
            "FR-CIV-3D-011 removal. The dry run therefore prints 'total removed: 3' "
            "for this file when only 2 of the 3 removals here are reachable, and a "
            "green run is NOT evidence that this tag was handled. The verdict is "
            "sound and the tag at geology.rs:27 is still live in source as of this "
            "writing; applying it needs either a second removal pass for the same "
            "declaration (the guard is per-file, not per-id) or a manual edit of the "
            "one line. Flagging rather than working around it, because changing "
            "_apply_verdicts.py is outside this file's remit and would affect the "
            "other eleven verdict modules.",
        ],
    ),
    (
        "crates/protocol-3d/src/bundle.rs",
        "DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL",
        {
            "FR-CIV-PROTO-006": (
                "DATA-SHAPE-ONLY, AND SHAPELESS WITH RESPECT TO THE REQUIREMENT. This "
                "OVERTURNS a prior GENUINE ruling. The requirement is 'Support binary "
                "frames with zstd compression; unpack without errors', tested by "
                "'Subscribe with use_binary_frames=true; verify frames unpack "
                "correctly' (CIV-0200-client-protocol.md:1149-1152). The prior "
                "in-file note, written into this file's header at bundle.rs:39-41, "
                "kept the tag on the grounds that 'a zstd level is exactly the data "
                "the requirement names, so that one tag is legitimate on shape'. That "
                "is a data-match argument. The requirement is about a wire format and "
                "a round trip, and `pub const DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL: i32 "
                "= 1` at bundle.rs:48 is a single i32 naming a compression level. A "
                "constant is not a frame: it declares no magic bytes, no header "
                "layout, no length framing, no flag bits, and performs no encode or "
                "decode. It cannot be unpacked and cannot fail to unpack, because it "
                "does not participate in either direction of the round trip. The "
                "requirement's two halves -- 'Support binary frames' and 'unpack "
                "without errors' -- are both absent from this symbol, and neither can "
                "be supplied by an integer. The genuine implementation lives in "
                "neighboring symbols in this same file, where FR-CIV-PROTO-006 is "
                "correctly and independently tagged and where removing this tag loses "
                "no coverage: encode_frame3d_bundle at bundle.rs:182 writes the "
                "F3DB magic, version, flag byte, big-endian tick, frame count, and "
                "uncompressed/payload lengths (:209-216), and decode_frame3d_bundle at "
                ":225 parses exactly that header back and returns Frame3dBundleError "
                "on truncation, bad magic, and version mismatch (:226-234). Both "
                "carry the FR-CIV-PROTO-006 tag at :179 and :222. The constant is "
                "consumed by Frame3dBundleEncodeOptions::default at :113, which is "
                "the data flowing INTO the real implementer rather than the "
                "implementer itself. A default parameter value is the weakest "
                "possible claimant to a 'binary frame format' requirement. If the tag "
                "is left here it asserts that a magic number is a wire protocol."
            ),
        },
        [
            "The coordinator flagged that FR-CIV-PROTO-006 appears twice in this file "
            "and that one site might be genuine while the other is not. Having read "
            "both, the honest answer is the opposite of the expected shape: BOTH "
            "container sites are false, and the genuine binding is on neither of them "
            "but on the two functions further down the same file.",
            "This is the first of the pair and it is the weaker of the two. The "
            "constant is a bare i32 with no behavior of any kind, so the case for a "
            "container binding is not merely weak, it is absent. Note also that the "
            "prior note in the file header says it is kept 'on shape' -- but shape is "
            "precisely what the requirement is not about. The requirement has two "
            "behavioral halves and the constant discharges neither.",
            "The earlier removal recorded in the same header block is worth noting "
            "because it got this site almost right: FR-CIV-PROTO-005 was removed from "
            "this constant on the grounds that 'a magic constant and a zstd level "
            "filter nothing' (bundle.rs:44). That sentence describes this constant "
            "just as accurately, and it is the reasoning that should have been "
            "applied to FR-CIV-PROTO-006 as well. The sibling id was judged on the "
            "symbol; this one was judged on the type of the symbol. Same symbol, "
            "different standard.",
            "No coverage is lost by removing it: the real round trip is already "
            "tagged on the encode and decode functions and is exercised by "
            "crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:11, which encodes with "
            "compress: true at this exact constant and asserts the decoded bundle "
            "unpacks -- which is the spec's 'verify frames unpack correctly' clause, "
            "run through the constant rather than certified by it.",
        ],
    ),
    (
        "crates/protocol-3d/src/bundle.rs",
        "Frame3dBundleEncodeOptions",
        {
            "FR-CIV-PROTO-006": (
                "DATA-SHAPE-ONLY, AND THE SHAPE IS AN INPUT, NOT A FORMAT. This "
                "OVERTURNS a prior GENUINE ruling, which held this tag 'for the same "
                "reason as on the constant above' (bundle.rs:93). The requirement is "
                "'Support binary frames with zstd compression; unpack without errors', "
                "tested by 'Subscribe with use_binary_frames=true; verify frames unpack "
                "correctly' (CIV-0200-client-protocol.md:1149-1152). "
                "Frame3dBundleEncodeOptions at bundle.rs:102 is a two-field opt-in "
                "config struct -- `compress: bool` (:104) and `compress_level: i32` "
                "(:106) -- and its impl is a single Default (:109-116) returning "
                "compress: false. It is a bag of caller-supplied knobs. It does not "
                "encode, does not decode, does not describe a byte layout, and holds "
                "nothing that could be 'unpacked': there is no buffer, no header, no "
                "framing, and no round trip on this type. Even taken at its strongest "
                "it is the parameter object OF the frame format rather than the frame "
                "format. Two further points sharpen the verdict. First, the default "
                "DISABLES the very compression the requirement mandates: "
                "compress: false at :112 means an out-of-the-box "
                "Frame3dBundleEncodeOptions does not produce a zstd-compressed frame "
                "at all, so a client that constructs this type with Default::default() "
                "gets no binary-frame compression and therefore does not get "
                "FR-CIV-PROTO-006 behavior. A type whose default state contradicts the "
                "requirement is a weak binding for it. Second, the flags bit that "
                "actually records compression on the wire is a DIFFERENT type, "
                "Frame3dBundleFlags at bundle.rs:63, whose ZSTD_COMPRESSED bit 0 "
                "(:67) is what decode_frame3d_bundle reads at :237 to decide whether "
                "to decompress; that type carries other removed ids and not this one. "
                "The requirement is genuinely implemented, by "
                "encode_frame3d_bundle (bundle.rs:182, tagged at :179), "
                "encode_frame3d_bundle_from_f3d0 (:195, which builds the 23-byte "
                "header at :208-216), and decode_frame3d_bundle (:225, tagged at "
                ":222). This struct is passed into the first two as an argument "
                "(:184, :199) and read by maybe_compress at :202. Tagging it asserts "
                "that a struct of two switches is a binary frame format."
            ),
        },
        [
            "Second of the FR-CIV-PROTO-006 pair, judged separately as instructed. It "
            "is a closer call than the constant and it still fails, for a reason the "
            "constant does not even reach: this struct is at least ON the real code "
            "path, carried as an argument into the real encoder. Being on the path is "
            "not the same as being the implementation, and the requirement names a "
            "format and a round trip rather than a config knob.",
            "The decisive fact is the default. compress: false at bundle.rs:112 means "
            "the type as constructed by Default does not request zstd compression, "
            "and the requirement's first clause is 'Support binary frames with zstd "
            "compression'. So the tagged symbol's own default behavior runs against "
            "the requirement it is tagged with. I checked whether that made the tag a "
            "mis-bound-but-real feature flag instead; it does not, because the flag is "
            "only meaningful when the caller opts in and the opt-in does nothing by "
            "itself -- maybe_compress at :202 is what reads it, and that function is "
            "inside the tagged-on-correctly encoder.",
            "I want to be precise about the prior ruling's reasoning, because the "
            "coordinator may push back. The header note says FR-CIV-PROTO-006 'is "
            "kept here for the same reason as on the constant above', i.e. shape "
            "matching. Shape matching cannot establish a behavioral requirement: the "
            "requirement is satisfied by bytes surviving a round trip, and no field of "
            "this struct is a byte. The genuine coverage for this id is at bundle.rs:"
            "179 and :222 and in the round-trip test at "
            "crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:11, and none of that is "
            "lost when this tag comes off.",
            "Both FR-CIV-PROTO-006 container sites in this file therefore come off, "
            "while the two function sites at :182 and :225 stay. That is the 'one "
            "genuine, one not' shape the coordinator anticipated, relocated: the "
            "genuine binding is real, it just is not on either container.",
        ],
    ),
]
