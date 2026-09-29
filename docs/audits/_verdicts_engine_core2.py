"""Adjudicated false-binding removals for the engine/core data-container pass.

Scope: nine requirement-tag bindings sitting directly above Rust data-type
declarations in ``crates/engine``. Each was re-derived from the authoritative
requirement text (docs/specs, agileplus-specs, docs/design,
docs/reference/non-functional-requirements.md) and from the declaration's real
usage under ``crates/``. Auto-generated audit reports and
``docs/traceability`` matrix rows were treated as non-authoritative.

Verdict strings used, per the audit's own vocabulary:

* ``FR-CLIENT-003``       -> NOT-IMPLEMENTED
* ``FR-CIV-POLITY-004``   -> CONTAINER-ONLY
* ``FR-CIV-POLITY-003``   -> NOT-IMPLEMENTED
* ``FR-CIV-POLITY-001``   -> CONTAINER-ONLY
* ``FR-CIV-CORE-012``     -> IMPLEMENTED-BY-BEHAVIOR (tag OFF; real carrier named)
* ``NFR-CIV-DET-003``     -> MEASUREMENT-ONLY
* ``FR-CIV-3D-003``       -> MEASUREMENT-ONLY
* ``FR-CIV-VEHICLE-002``  -> DATA-SHAPE-ONLY
* ``FR-CIV-0104-003``     -> IMPLEMENTED (KEEP; independently re-verified, omitted
  from SITES per instructions)

``Fixed`` carries two false ids, so it is one SITES entry with two keys, not two
entries. ``KEEP`` records FR-CIV-0104-003 together with the evidence that
re-verified the earlier pass's finding.

Each reason is written as consecutive implicitly-concatenated string lines so it
reads as a paragraph in source while remaining a single ``str`` at runtime, which
is what ``_peek.py`` and ``_apply_verdicts.py`` require.

Running this module as a script prints the accounting and exits 0.
"""

# ---------------------------------------------------------------------------
# Bindings deliberately retained. A kept tag needs no SITES entry.
# ---------------------------------------------------------------------------
KEEP = {
    "FR-CIV-0104-003": (
        "Independently re-verified against the authoritative text at "
        "docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1464-1466: "
        "\"Any HALT violation sets `ablation_mode = true` on the run permanently\", "
        "verified by \"Trigger HALT violation; verify all subsequent snapshots have "
        "`ablation_mode: true`\". This is behavioral, not data-shape: the mandate is "
        "that a HALT violation FLIPS THE FLAG AND IT STAYS SET, and "
        "ConstraintState::update (crates/engine/src/constraints.rs:600-611) is the "
        "code that does exactly that -- it scans the check set for "
        "ConstraintCheck::Violated { severity: ViolationSeverity::Halt, .. } and "
        "latches the result under `if !self.ablation_mode`, so the write is one-way. "
        "Latching is the whole requirement, so a passive-looking field name would not "
        "be enough on its own; the `if !self.ablation_mode` guard is what makes the "
        "binding real, and it is present. Three real (non-stub) tests exercise the "
        "propagation and permanence: crates/engine/tests/fr_fr_civ_0104_003.rs:20 "
        "(HALT latches, survives a subsequent all-OK tick), :59 (Warning and Critical "
        "severities do NOT latch), :98 (five consecutive HALTs stay latched). The "
        "earlier pass's finding stands; I agree with it and the tag stays. Two "
        "sibling ids on the same struct (FR-CIV-0104-007, FR-CIV-0104-010) were "
        "already correctly removed by the prior lane, which is consistent with this "
        "struct being read per-id rather than swept wholesale."
    ),
}

# ---------------------------------------------------------------------------
# False bindings. One tuple per (file, declaration); several false ids on one
# declaration are keys of the same dict.
# ---------------------------------------------------------------------------
SITES = [
    (
        "crates/engine/src/command_queue.rs",
        "CommandKind",
        {
            "FR-CLIENT-003": (
                "The requirement is at agileplus-specs/civ-010-multi-client-protocol/spec.md:30: "
                "\"Role authorization -- research clients cannot issue build or policy "
                "commands; unauthorized -> JSON-RPC error -32603 with role information; "
                "enforced by integration tests covering all three role tiers\". That is a "
                "behavioral authorization requirement with three named deliverables "
                "(a rejection path, a specific error code, and tests across three role "
                "tiers). `CommandKind` is a flat 6-variant enum of the actions a client "
                "may issue (Pause/Resume/SetSpeed/SaveReplay/LoadReplay/PolicyOverride, "
                "command_queue.rs:15-22). It carries no role, no tier, no authorization "
                "field and no permission table, so it cannot reject anything: "
                "CommandQueue::push (command_queue.rs:58) accepts every variant from "
                "every client id unconditionally. Nothing in the enum discharges any "
                "clause of the requirement, and the enum is only a vocabulary, not a "
                "gate. The workspace's own test file already admits the absence in "
                "writing: crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:743-754 "
                "is headed \"FR-CLIENT-003 -- Client role authorization enforcement "
                "(ABSENT)\" and states \"no crate in the workspace defines a client role, "
                "and the engine's command envelope carries no role at all, so a policy "
                "override from an unknown client id is accepted verbatim\". The only "
                "-32603 in the server (crates/server/src/jsonrpc.rs:25, "
                "INTERNAL_ERROR) is the generic JSON-RPC internal-error constant, not a "
                "role rejection. `git grep -rn \"-32603\" -- crates/` returns only that "
                "constant, its two test literals, and an unrelated civis-cli MCP reply; "
                "no role-gated rejection exists anywhere. No implementing symbol exists, "
                "so the tag is removed."
            ),
        },
        [
            "CommandKind is the command vocabulary for a client, which is why the tag",
            "was plausible: it names the actions the requirement wants to restrict. But",
            "a list of what a client MAY do is the opposite of a gate on what a given",
            "client tier may do, and the enum has no role dimension to gate on.",
        ],
    ),
    (
        "crates/engine/src/diplomacy.rs",
        "DeepDiplomacyState",
        {
            "FR-CIV-POLITY-004": (
                "The requirement is at docs/design/polities-markets.md:68-74: "
                "\"FR-CIV-POLITY-004 -- Regime read-out from graph topology. A polity's "
                "*displayed* shape is a pure function of its internal edge structure\", "
                "with a table mapping topological signatures (mean coordination, "
                "reciprocity, degree distribution, coercion asymmetry) to read-out labels "
                "Anarchic / Networked / Collective / Hierarchical / Dominant. The "
                "mandated behavior is a pure function: read the edge structure, classify "
                "the regime. `DeepDiplomacyState` is an 8-field passive aggregate "
                "(alliance_manager, active_negotiations, active_exchanges, "
                "faction_cultures, faction_resources, active_wars, war_start_ticks, "
                "war_casualties; diplomacy.rs:22-39) with a derived Default and no impl "
                "block at all -- it holds no edge structure, computes no topology, and "
                "has no classification function to call. It cannot do what is asked. "
                "`git grep -rni \"Anarchic|Networked|Collective|Hierarchical|regime\" "
                "-- crates/` returns the unrelated branching-regime classifier "
                "(crates/civ-emergence-metrics/src/branching.rs:180, "
                "classify_regime over sigma-bar) and the unrelated economy allocation "
                "regimes (crates/economy/src/allocation.rs:177); neither reads polity "
                "edge topology, and the allocation enum's own removal note concedes \"no "
                "code path ever changes a locale's regime as coercion rises\". There is "
                "no graph-topology classifier in the workspace, so the read-out has no "
                "implementing symbol and the container cannot stand in for one. "
                "Collateral observation: the only test file for this id, "
                "crates/engine/tests/fr_fr_civ_polity_004.rs, asserts that the struct's "
                "default is empty and that it is Clone. A test that checks emptiness "
                "and clonability is evidence about the container, not about regime "
                "read-out, and is a stub in spirit even though it compiles."
            ),
        },
        [
            "DeepDiplomacyState is a real state container that is genuinely threaded",
            "through WorldState (crates/engine/src/engine.rs:492, 702, 1097, 1273, 1547)",
            "and does model alliances, negotiations and wars. That it is wired in is not",
            "the question. FR-CIV-POLITY-004 asks for a computed classification of a",
            "polity from its edge structure, and this struct has no edges to read.",
        ],
    ),
    (
        "crates/engine/src/diplomacy.rs",
        "DiplomacyEvent",
        {
            "FR-CIV-POLITY-003": (
                "The requirement is at docs/design/polities-markets.md:54-61: "
                "\"FR-CIV-POLITY-003 -- Coercion derives, never declared. `coercion(i->j)` "
                "measures i's capacity to compel j, computed from substrate only\": a "
                "clamp01 of relative capability power(i)/(power(i)+power(j)+eps) times "
                "proximity(i,j) times (1 - relation_score(i,j).max(0)). The defining "
                "clause is \"derives, never declared\" -- the coercion value must be "
                "COMPUTED from power, proximity and relations on every read, and must "
                "not be a stored attribute. `DiplomacyEvent` is a 4-field event record "
                "(tick, faction_a, faction_b, kind: DiplomacyKind; diplomacy.rs:162-167) "
                "that is emitted on state change; it holds no coercion value, no power, "
                "no proximity, and no relation score, and it performs no computation. "
                "Not only does it not discharge the requirement, its shape is the "
                "opposite of it: a persisted event row is exactly the \"declared\" "
                "storage the spec forbids. `git grep -rn \"fn coercion|fn compute_coercion\" "
                "-- crates/` returns nothing; the only 'coercion' identifiers in the "
                "workspace are the unrelated bounded-coercion constraint check "
                "(crates/engine/src/constraints.rs:310) and a removed MARKET-006 note "
                "that itself records \"no code path ever changes a locale's regime as "
                "coercion rises\". The derivation has no implementing symbol. Separately, "
                "the doc comment immediately above the tag (diplomacy.rs:159) cites "
                "\"FR-CIV-DIPLOMACY\", an id that `git grep` does not find in "
                "docs/specs, agileplus-specs or docs/design at all -- it is a phantom id, "
                "and under RULES a doc comment claiming coverage is a claim, not evidence."
            ),
            # Not in my remit: FR-CIV-DIPLOMACY is a phantom id (no authoritative
            # requirement text anywhere in the repo) that sits in the doc comments
            # at diplomacy.rs:151 and :159, i.e. inside the same block the apply
            # engine sweeps. It is deferred to whichever lane owns phantom-id
            # triage rather than removed here. Listed under __keep__ so the engine
            # classifies it instead of blocking this site.
            "__keep__": ("FR-CIV-DIPLOMACY",),
        },
        [
            "DiplomacyEvent is a legitimate event type in its own right; the problem is",
            "only that it is tagged with a requirement about deriving a scalar from",
            "substrate. Note the parallel: a genuine diplomacy event enum does exist",
            "elsewhere (crates/diplomacy/src/effects.rs:121) and is actually constructed",
            "and returned by effect appliers, so the workspace can emit diplomacy",
            "events -- just not by deriving coercion, and not from this type.",
        ],
    ),
    (
        "crates/engine/src/diplomacy.rs",
        "FactionRelations",
        {
            "FR-CIV-POLITY-001": (
                "The requirement is at docs/design/polities-markets.md:37-44: "
                "\"FR-CIV-POLITY-001 -- Cohesion graph. A polity surface is computed as a "
                "weighted graph over clusters (nodes = ClusterId, edges = directed "
                "coordination weight)\", with each edge weight a monotone blend of five "
                "named terms: coord(i->j) = w_colo*colocation + w_kin*kinship_overlap + "
                "w_cult*(1 - culture_distance) + w_econ*max(0, payoff_if_coordinated) + "
                "w_coer*coercion(i->j). The mandated artifact is a weighted GRAPH whose "
                "nodes are clusters and whose edges carry a computed five-term weight. "
                "`FactionRelations` is a single `rows: BTreeMap<(u32,u32), "
                "FactionRelationRecord>` where each record is just { score: f32, "
                "samples: u32 } (diplomacy.rs:61-63, 53-56). Its nodes are faction "
                "integer ids, not ClusterId; it has no edge weight; it has none of the "
                "five terms (no colocation, kinship overlap, culture distance, payoff, or "
                "coercion); and `apply_signal` (diplomacy.rs:83-101) computes only a "
                "single scalar relation score from trade_volume and combat_grievance. "
                "The struct's own doc comment calls it a \"Stub faction-relation matrix\" "
                "(diplomacy.rs:58) and engine.rs:802 calls it \"a stub: an empty "
                "FactionRelations until DiplomacyMatrix schema is ...\". A one-scalar "
                "relation score between factions is not a weighted cohesion graph over "
                "clusters. `git grep -rni \"coordination_weight|CohesionGraph|colocation|"
                "culture_distance|kinship_overlap|reciprocity\" -- crates/` returns only "
                "cluster_by_colocation (crates/agents/src/cluster.rs:71), which "
                "partitions positions into clusters and never computes an edge weight; "
                "no cohesion-graph builder exists, so the requirement has no implementing "
                "symbol. The id's own test file concedes the gap: "
                "crates/engine/tests/fr_fr_civ_polity_001.rs:5 says the FR \"captures: "
                "Diplomacy relations matrix\" rather than the cohesion graph, and its "
                "strongest assertion is that a default world has zero rows."
            ),
        },
        [
            "This is the clearest CONTAINER-ONLY in the batch: a 2-field record in a",
            "BTreeMap, self-described as a stub, carrying a single undirected relation",
            "score, sitting under a tag that promises a directed five-term weighted",
            "graph over ClusterId nodes. The struct does real work for a different",
            "requirement (FR-CIV-POLITY-002, tagged on apply_signal at diplomacy.rs:84),",
            "which is exactly why the binding looks plausible and is still false.",
        ],
    ),
    (
        "crates/engine/src/fixed_math.rs",
        "Fixed",
        {
            "FR-CIV-CORE-012": (
                "The requirement is at docs/specs/CIV-0001-core-simulation-loop.md:922-924: "
                "\"FR-CIV-CORE-012: Fixed-Point Arithmetic -- No floating-point in money, "
                "resources, or energy; all i64\", verified by \"Clippy linter detects "
                "floating-point in sim logic; compilation fails\". The mandate has two "
                "halves: the representational one (money/resources/energy are i64, not "
                "float) and the enforcement one (a build-time gate that fails compilation "
                "on float in sim logic). `pub struct Fixed(pub(crate) i64)` "
                "(fixed_math.rs:14) is genuinely the right carrier for the first half and "
                "it is heavily used, so the requirement IS really implemented -- but not "
                "here. The tag is therefore off the declaration and the true implementing "
                "artifact is named: the enforcing gate is a workspace Clippy lint "
                "configuration, not a Rust type, and it is ABSENT. There is no "
                "`#![deny(clippy::float_arithmetic)]` at the top of crates/engine/src/lib.rs "
                "(`findstr /n \"deny\"` over that file returns nothing), the repo's "
                "clippy.toml sets only msrv and avoid-breaking-exported-api and contains no "
                "float-related key, and `git grep -rn \"float_arithmetic\" -- .github/workflows` "
                "returns nothing, so no CI job turns the lint into a build failure either. "
                "The only text asserting the gate is the doc comment at fixed_math.rs:8-11, "
                "which claims \"Float arithmetic is blocked at the lint level via `cargo "
                "clippy -D clippy::float_arithmetic` in CI\" -- a claim the configuration "
                "contradicts, and under RULES a doc comment is a claim, not evidence. Worse "
                "for the binding, the type does not actually discharge the requirement: "
                "`Fixed` stores an i64 scaled by 1_000 (fixed_math.rs:6, 31, 39, 47, 55), "
                "and the crate implements `FixedFromNum for f32` (fixed_math.rs:61) and a "
                "`to_f64` conversion, so floats flow in and out of the type by design. The "
                "money/resource/energy path that would need to be float-free in fact runs "
                "through crates/engine/src/metrics.rs, which is built on f64 fields "
                "(metrics.rs:7-10) and an f64 signature (metrics.rs:15), i.e. one of the "
                "exact modules the spec's companion NFR names as float-banned. So the type "
                "is a plausible-looking carrier for a gate that is not configured. "
                "Retained for contrast, and deliberately NOT listed as false: the earlier "
                "lane's triage note (docs/audits/triage-container-engine-core.md:119) "
                "called this the \"one clearly legitimate container tag in the batch\". "
                "That reading is defensible on the representation half and I am removing "
                "the tag on the enforcement half plus the 1_000-vs-10^6 mismatch below. If "
                "the operator prefers to keep FR-CIV-CORE-012 on `Fixed` as a marker for "
                "the i64 representation, this one key can be dropped from the dict without "
                "affecting the other six entries here."
            ),
            "NFR-CIV-DET-003": (
                "The requirement is at docs/reference/non-functional-requirements.md:158-166: "
                "\"NFR-CIV-DET-003 -- Fixed-Point Arithmetic Enforcement: All "
                "state-mutating simulation paths (production, allocation, market clearing, "
                "taxation, metrics computation) SHALL use the project's `Fixed` type (i64 "
                "scaled by 10^6) exclusively; floating-point types SHALL NOT appear in any "
                "state-mutation code path\", with \"Measurable Target: Zero occurrences of "
                "f32 or f64 in state-mutation modules "
                "(crates/engine/src/{production,allocation,market,taxation,metrics}.rs), "
                "verified statically\" and \"Verification Method: Clippy lint rule "
                "no_float_in_sim_core ... or #[deny(clippy::float_arithmetic)] scope; "
                "property test tests/fixed_point_float_agreement\". A numeric type "
                "declaration cannot discharge this, and saying so explicitly is the point: "
                "this NFR is a MEASURED PROPERTY over a set of modules, enforced by a lint "
                "configuration plus a property test, and it is verified in "
                "docs/reference/non-functional-requirements.md:560 as a CI-GATE "
                "(\"Clippy deny float + tests/fixed_point_float_agreement\"). A single "
                "`struct Fixed(i64)` is not the artifact that satisfies it, and no "
                "plausibly-named field or impl can stand in for a static zero-occurrence "
                "scan. Three independent confirmations that the NFR is unmet and that the "
                "type is not the carrier. (1) SCALE MISMATCH: the NFR mandates i64 scaled "
                "by 10^6; `Fixed` is scaled by 1_000 (fixed_math.rs:6), so the type the "
                "NFR names is not this type. (2) NAMED MODULES DO NOT EXIST: of the five "
                "state-mutation modules the NFR enumerates, production.rs, allocation.rs, "
                "market.rs and taxation.rs are absent from crates/engine/src/ (only "
                "metrics.rs exists), so the measurable target names files that are not "
                "there. (3) THE GATE IS ABSENT AND THE PATH VIOLATES IT: as recorded for "
                "FR-CIV-CORE-012 above there is no float deny in crates/engine/src/lib.rs, "
                "clippy.toml or .github/workflows, and crates/engine/src/metrics.rs -- the "
                "one named module that does exist -- is f64 throughout (metrics.rs:7-10, "
                "15). The genuine artifact is the property test, which lives in a test "
                "file rather than on the type: crates/engine/tests/fr_nfr_civ_det_003.rs:16 "
                "(fixed_point_float_agreement, asserting 1e-6 tolerance) and :49 "
                "(state_economy_quantities_are_integer_backed), with a second coverage "
                "test at crates/engine/tests/fr_engine_metrics_replay_tests.rs:160. The "
                "test file's own header (fr_nfr_civ_det_003.rs:7-10) concedes the point: "
                "\"The 'zero f32/f64 in sim state-mutation modules' half of this NFR is a "
                "lint gate (clippy float deny) rather than a runtime assertion\". The tag "
                "is a structural marker on a type, asserting coverage the type does not "
                "provide."
            ),
            # Not in my remit: NFR-C-03 appears in the doc comment at fixed_math.rs:8,
            # inside the same block the apply engine sweeps, and has no authoritative
            # requirement text outside generated audit artifacts. Deferred to a
            # phantom-id triage lane; listed so the engine classifies rather than
            # blocks this site.
            "__keep__": ("NFR-C-03",),
        },
        [
            "Two different requirement kinds sit on one declaration and both are false,",
            "which is the main lesson of this site: FR-CIV-CORE-012 is a representational",
            "rule whose real carrier is a missing lint gate, and NFR-CIV-DET-003 is a",
            "measured property whose real carrier is a property test. A 64-bit integer",
            "newtype is a reasonable and widely used engineering artifact; it is simply",
            "not the evidence either requirement asks for.",
            "The doc comment on this type also cites a third id, NFR-C-03 (line 8), which",
            "has no authoritative text outside docs/audits and the generated id inventory;",
            "it is left in place under __keep__ for a phantom-id triage lane, not judged",
            "here.",
        ],
    ),
    (
        "crates/engine/src/lib.rs",
        "SCALE",
        {
            "FR-CIV-3D-003": (
                "The requirement is at "
                "docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1904-1908: "
                "\"FR-CIV-3D-003 -- Frame Rate -- Primary Hardware. SHALL: The 3D web client "
                "SHALL maintain a minimum of 45 frames per second at 1080p resolution when "
                "rendering a Zoom 2 view (12x12 hex cell grid) on M2 MacBook hardware\", "
                "with \"Verification: Automated benchmark via headless Chromium with "
                "performance.now frame timing. Run on CI hardware with Apple Silicon "
                "runner. Pass threshold: 95th percentile frame time < 22.2ms over a "
                "300-frame sample\". That is a pure MEASUREMENT requirement: a wall-clock "
                "frame-time percentile produced by a headless-Chromium benchmark on "
                "specific hardware. `pub const SCALE: i64 = 1_000` (lib.rs:109) is a "
                "fixed-point scaling factor for joules; it is an integer literal that "
                "says nothing about frame rate, and a constant cannot discharge a "
                "benchmark. Worse, the test pinned to this id is a pure tautology: "
                "crates/engine/tests/fr_fr_civ_3d_003.rs:20 computes "
                "`Fixed::from_num(1000) / Fixed::from_num(1000)` -- literally 1000/1000 -- "
                "and asserts only `fps_45 > Fixed::ZERO`, which holds for every value of "
                "every constant, so it passes unconditionally and measures nothing; the "
                "companion test at :27 asserts only `SCALE == 1_000`. `git grep -rni "
                "\"performance.now|headless chromium|95th percentile\" -- crates/` returns "
                "only an unrelated server-side latency histogram "
                "(crates/server/src/metrics.rs:219, p95 of a metrics histogram) and an "
                "unrelated NFR-S-02 p95 test. The only real frame-budget code in the "
                "workspace is crates/render/src/frame.rs, whose own header says \"Frame "
                "budget planning for 60 fps rendering (CIV-0500; no FR-PERF-003 id)\" -- "
                "60 fps planning arithmetic, not a 45 fps measured benchmark, and it is in "
                "civ-render, not in civ-engine. The companion file makes the same point "
                "for FR-PERF-003 at crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:14: "
                "\"NOT engine-testable (GPU + civ-render is not a dependency of civ-engine)\". "
                "No implementing benchmark artifact exists, so the tag is removed. The id "
                "is additionally a scope mismatch on its face: a render-frame-rate "
                "criterion cannot be discharged by a crate that does not depend on the "
                "renderer, and lib.rs:46-102 is a module list with no rendering path."
            ),
        },
        [
            "SCALE is a genuinely used constant -- crates/engine/src/economy_engine.rs:7",
            "and :15 divide joules by it at the economy boundary -- but that is a joule",
            "conversion, and a used constant is still not a frame-rate measurement. Note",
            "the id collision hazard: FR-CIV-3D-003 (45 fps on M2, headless Chromium) and",
            "FR-PERF-003 (60 fps on reference GPU, civ-render) are different requirements",
            "with similar targets, and the test file for this one is a tautology dressed",
            "as coverage.",
        ],
    ),
    (
        "crates/engine/src/vehicle_types.rs",
        "VehicleArchetype",
        {
            "FR-CIV-VEHICLE-002": (
                "The requirement is at docs/design/vehicles-logistics.md:105-106: "
                "\"FR-CIV-VEHICLE-002 -- Removing a required material from a locale's stock "
                "makes new builds of that kind fail while existing instances persist "
                "(capability is per-build, not retroactive)\". The mandated behavior has "
                "two halves, and the tag is on the half that implements neither. "
                "`VehicleArchetype` is a 9-field catalog data row (kind, medium, "
                "requires_traits, requires_materials, build_cost, capacity, "
                "base_speed_mult, era_hint; vehicle_types.rs:69-89) whose own doc comment "
                "is \"A data row in the vehicle archetype catalog\". It is the "
                "declarative requirement set, not the gate: it stores "
                "`requires_materials` and does nothing with them. No method on the type "
                "evaluates the gate, and the only two functions in the file, "
                "check_build_capability (vehicle_types.rs:106) and effective_speed (:142), "
                "are free functions taking `&VehicleArchetype` rather than methods on it. "
                "Both halves of FR-CIV-VEHICLE-002 are therefore outside the tagged "
                "declaration. The first half (new builds fail) is really implemented, by a "
                "different symbol: crates/engine/src/vehicle_types.rs:106 "
                "check_build_capability walks archetype.requires_materials against the "
                "locale's set and returns CapabilityGate::Fail { missing }, exercised for "
                "real at crates/engine/tests/fr_fr_civ_vehicle_002.rs:14-35, which removes "
                "\"livestock\" and asserts the fail names it. The second half (existing "
                "instances persist / not retroactive) has NO implementing symbol at all: "
                "there is no placed-vehicle or vehicle-instance type in the crate, and "
                "`git grep -rni \"instance\" -- crates/engine/src/vehicle_types.rs` returns "
                "exactly one hit, the doc comment \"How a vehicle instance was created\" "
                "at :210 on the unrelated `InfraProvenance` enum. Nothing anywhere stores "
                "a built vehicle, so nothing can persist. The test file for this id does "
                "not cover the persistence half at all: fr_fr_civ_vehicle_002.rs:5 claims "
                "to verify \"Removing a required material makes new builds fail; existing "
                "instances persist\", but the body only tests the first half, so the file's "
                "own header overstates what it checks."
            ),
        },
        [
            "This is the one site in the batch where the true implementing symbol",
            "exists and is easy to name, which makes it the most useful removal: the",
            "capability gate is check_build_capability at vehicle_types.rs:106, and",
            "FR-CIV-VEHICLE-001 already tags the CapabilityGate enum directly above it",
            "(vehicle_types.rs:91). FR-CIV-VEHICLE-002 should be recorded against that",
            "function once the non-retroactive half is built, not against the data row",
            "that merely stores the requirement set.",
        ],
    ),
]


if __name__ == "__main__":
    n_sites = len(SITES)
    n_removed = sum(len(d) for _r, _d, d, _n in SITES)
    print(f"sites: {n_sites}")
    print(f"false bindings to remove: {n_removed}")
    print(f"bindings deliberately kept: {len(KEEP)}")
    print()
    for rel, decl, rem, note in SITES:
        print(f"  {rel} :: {decl}")
        for tag, why in rem.items():
            flat = " ".join(why.split())
            print(f"      {tag}: {flat[:150]}...")
        for line in note:
            print(f"      note: {' '.join(line.split())[:110]}")
    print()
    for tag, why in KEEP.items():
        print(f"  KEEP {tag}: {' '.join(why.split())[:150]}...")
