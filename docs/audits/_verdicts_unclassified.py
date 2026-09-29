"""Verdicts for the ids `_unclassified.py` surfaced but the triage reports skipped.

`_apply_verdicts.py` refuses to edit a declaration carrying an id no verdict
module mentions. That guard is doing its job: it found seven live blocks holding
ids that no report had adjudicated. Five of the seven turned out to be real
bindings that should stay, which is exactly the outcome a silent
"remove everything the report flagged" pass would have destroyed.

Each site below was judged by reading the authoritative requirement text and the
declaration it sits on. The `__keep__` key names ids that stay on that
declaration; every other id in `removed` is taken off.
"""

KEEP = {
    "FR-ECON-005": "allocation.rs:171 doc comment already cites FR-ECON-005; the regime enum is the dispatch selector the requirement needs",
    # The five ids below are genuine bindings on the declaration they sit on. They
    # live here because the sim/domain module owns the FALSE ids on the same
    # blocks and its per-site keep set cannot name them. Each was confirmed by
    # reading the requirement text and the struct it tags; see the site notes.
    "FR-CIV-PLANET-020": "CoastalColumn.base_y/last_water_y are the tide-driven coastal water columns the requirement names",
    "FR-CIV-0104-003": "ConstraintState.ablation_mode is the field the requirement says a HALT violation sets permanently",
    "FR-CIV-INFOVIEW-900": "InfoOverlay is the per-overlay registration record the toggleable info-view layer system is built from",
    "FR-CIV-3D-011": "BiomeKind's first six variants are literally the six biomes the terrain-generation coverage requirement names",
    "FR-CIV-EMERG-001": "EmergenceDashboard computes the five metrics the requirement lists; the struct is the engine's hand-off shape",
}

SITES = [
    (
        "crates/economy/src/allocation.rs",
        "AllocationRegime",
        {
            "__keep__": ("FR-ECON-005",),
        },
        [
            "FR-ECON-005 stays. The requirement (agileplus-specs/civ-002-economy-joule-system/spec.md:28) asks for an allocation algorithm that",
            "distributes goods by priority and the doc comment at allocation.rs:171 already cites the id next to the regime enum that",
            "routes every allocation through allocate_with. Keeping the tag makes an existing true binding explicit rather than incidental.",
        ],
    ),
    (
        "crates/engine/src/climate.rs",
        "CoastalColumn",
        {
            "__keep__": ("FR-CIV-PLANET-020",),
        },
        [
            "FR-CIV-PLANET-020 stays. The requirement (agileplus-specs/civ-021-recovered-requirements/spec.md:85) is tide-driven coastal",
            "water columns that shift each tick. CoastalColumn is that record: base_y and last_water_y, iterated from a BTreeMap for",
            "determinism, written by Simulation::phase_planet (climate.rs:38-44) on the climate tide offset. The doc comment at",
            "climate.rs:17 cites the id. This is a genuine binding, and it is the clearest example on this declaration of a report's",
            "NOT-IMPLEMENTED verdict being wrong about a neighbouring id on the same struct.",
        ],
    ),
    (
        "crates/engine/src/constraints.rs",
        "ConstraintState",
        {
            "__keep__": ("FR-CIV-0104-003",),
        },
        [
            "FR-CIV-0104-003 stays. The requirement (docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1464-1466) is that a HALT",
            "violation sets ablation_mode = true permanently. ConstraintState.ablation_mode (constraints.rs:567) is that exact field,",
            "documented as 'permanent once set', and defaulted false in Default::default. The requirement's acceptance criterion is",
            "satisfied by the field being the thing the requirement names.",
        ],
    ),
    (
        "crates/engine/src/info_views.rs",
        "InfoOverlay",
        {
            "__keep__": ("FR-CIV-INFOVIEW-900",),
        },
        [
            "FR-CIV-INFOVIEW-900 stays. The requirement (docs/specs/requirements/FR-CIV-INFOVIEW.md:12) is a toggleable info-view",
            "layer system with a panel of selectable overlays. InfoOverlay is the registration record for one overlay (id, name,",
            "group, render_kind, availability, legend) and the doc comment at info_views.rs:88 names the id as its origin. The",
            "toggle and mutual-exclusion behaviour live in the registry above it, which is where the other INFOVIEW ids on this",
            "block are aimed.",
        ],
    ),
    (
        "crates/planet/src/geology.rs",
        "BiomeKind",
        {
            "__keep__": ("FR-CIV-3D-011",),
        },
        [
            "FR-CIV-3D-011 stays. The requirement (docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968-1972) is that",
            "terrain generation produces all six biome types (ocean, plains, forest, desert, tundra, mountain). BiomeKind's first",
            "six variants are literally those six, in that order, and the doc comment at geology.rs:14-16 says they are the",
            "geology-seed archetypes. The tag names the coverage surface itself.",
        ],
    ),
    (
        "crates/civ-emergence-metrics/src/dashboard.rs",
        "EmergenceDashboard",
        {
            "__keep__": ("FR-CIV-EMERG-001",),
        },
        [
            "FR-CIV-EMERG-001 stays. The requirement (agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:39) is that the",
            "engine compute five named metrics per (tick, seed, scenario) in read-only fashion. EmergenceDashboard is the struct",
            "that holds exactly those five (cluster_entropy, ideology_homophily, sentience_fraction, psyche_stability,",
            "diplomacy_tension), each computed by a named helper in this same file. The module doc comment at dashboard.rs:1",
            "states it is the summary-metrics layer for the five dashboard tiles and cites the id.",
        ],
    ),
    (
        "crates/engine/src/command_queue.rs",
        "CommandQueue",
        {
            "FR-CIV-NOTIFY-921": (
                "Requirement FR-CIV-NOTIFY-921 (docs/specs/requirements/FR-CIV-NOTIFY.md:16) is 'A full, rebindable hotkey map",
                "SHALL cover camera, tools, overlays, speed, and selection', verified by 'conflict detection'. CommandQueue is a",
                "VecDeque<Command> with a max_pending bound (command_queue.rs:31-34): it orders inbound commands and nothing else.",
                "There is no key binding, no action, no rebind, and no conflict check in this type or anywhere in the crate. The id",
                "appears to have been attached because the queue handles SetSpeed, which is one of the things the hotkey map is",
                "meant to drive, but a queue that can carry a speed command does not implement a rebindable hotkey map. The real",
                "hotkey contract, if one exists, is exercised in crates/engine/tests/fr_civ_notify_cluster.rs; that test file is",
                "the implementing evidence, not this struct.",
            ),
        },
        [
            "FR-CIV-NOTIFY-921 removed from CommandQueue. It was the only unclassified id on the block besides the two CORE ids",
            "the engine/core report already adjudicated as false.",
        ],
    ),
    (
        "crates/engine/src/engine.rs",
        "Simulation",
        {
            "FR-CIV-CORE-006": (
                "FR-CIV-CORE-006 (docs/specs/CIV-0001-core-simulation-loop.md:892) forbids system time in the simulation.",
                "Simulation is the tick-loop owner, so a tag asserting that the loop reads no wall clock belongs on it. The",
                "detector and the prior pass both treat this as already handled; recorded here so the block's full id set is",
                "accounted for rather than merely not-mentioned.",
            ),
            "FR-CIV-CORE-007": (
                "FR-CIV-CORE-007 (docs/specs/CIV-0001-core-simulation-loop.md:897) is snapshot serialization; Simulation owns the",
                "state that is serialized. Same reasoning as CORE-006.",
            ),
            "FR-CIV-CORE-011": (
                "FR-CIV-CORE-011 (docs/specs/CIV-0001-core-simulation-loop.md:917) is replay-determinism verification; the",
                "simulation whose determinism is verified is Simulation. Same reasoning.",
            ),
            "FR-CIV-CORE-013": (
                "FR-CIV-CORE-013 (docs/specs/CIV-0001-core-simulation-loop.md:927) is phase-schedule integrity, and Simulation runs the",
                "phase schedule. Same reasoning.",
            ),
        },
        [
            "Simulation carries four ids that no report classified: CORE-006, -007, -011, -013. All four are tick-loop properties",
            "of the simulation rather than data shapes, and an earlier pass explicitly wrote them into a 'tags that remain and why",
            "they stay' list, so the intent was already a decision to keep them. This entry records that decision as a verdict so the",
            "block is fully accounted for and a later pass cannot re-flag it as unexamined.",
        ],
    ),
]
