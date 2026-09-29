"""Machine-readable transcription of docs/audits/triage-container-sim-domain.md.

Only FALSE verdicts are transcribed here: CONTAINER-ONLY and NOT-IMPLEMENTED.
DATA-SHAPE-ONLY and IMPLEMENTED-BY-BEHAVIOR rows are legitimate bindings and are
deliberately absent.

The audit describes the repository *before* a prior unbind pass. 70 of the 123
FALSE tags it lists no longer exist as live tag comments on a container:

  * 33 FR-SESSION-NNN tags above `const SESSION_HISTORY_CAP` in
    `crates/server/src/session.rs`
  * 37 tags above `struct WorldState` and `struct Simulation` in
    `crates/engine/src/engine.rs`

Those are recorded in ALREADY_REMOVED and are not transcribed. Every entry in
SITES corresponds to a tag comment that is still present in the source today,
grouped by the Rust declaration it sits above.
"""

SITES = [
    # ------------------------------------------------------------------
    # crates/agents
    # ------------------------------------------------------------------
    (
        "crates/agents/src/lib.rs",
        "LodTier",
        {
            "FR-CIV-3D-002": (
                'The requirement in agileplus-specs/civ-011-bevy-primary-client/spec.md is '
                '"LOD Budget Enforcement": the engine SHALL enforce a budget so detail is '
                'culled when the budget is exceeded. `LodTier` is a three-variant enum '
                '(Hot / Warm / Gestalt) that only names fidelity levels; it holds no budget '
                'field and implements no check. Grepping crates/agents/src for '
                '`lod_budget`, `LodBudget` and `budget_check` returns zero matches, so no '
                'budget enforcement exists anywhere in the crate and no implementing symbol '
                'does.'
            ),
        },
        [
            'A fidelity tier label is a shape, not an enforcement point. A budget check would '
            'have to live in whatever walks agents each tick, not on the enum itself.',
        ],
    ),
    (
        "crates/agents/src/psyche.rs",
        "Psyche",
        {
            "FR-CIV-PSYCHE-006": (
                "The requirement in docs/specs/requirements/FR-CIV-PSYCHE.md is "
                '"Cold agents SHALL collapse to cluster-level aggregates (mean mood, belief '
                'centroid, tie density)". `Psyche` is a per-agent struct of five scalar/'
                'array fields (drives, temperament, mood, beliefs, maturity). The crate has '
                'an `update_beliefs` function, but nothing in it reads agent temperature and '
                'emits cluster means, and no function anywhere produces a belief centroid or a '
                'tie-density aggregate, so no implementing symbol exists.'
            ),
        },
        [
            'The collapse path would have to run when an agent is culled by distance, which is '
            'a tick-loop behavior, not a field on the per-agent record.',
        ],
    ),
    (
        "crates/agents/src/psyche.rs",
        "PsychGenomeProfile",
        {
            "FR-CIV-PSYCHE-005": (
                "The requirement in docs/specs/requirements/FR-CIV-PSYCHE.md is "
                '"Cost O(MAX_TIES) bounded; decay SHALL be amortised (touch on access or '
                'periodic sweep)" over the social tie graph. `PsychGenomeProfile` is a data '
                'projection struct holding five DNA slot vectors; it is a DNA lookup shape and '
                'carries no tie graph, no decay counter, and no clock. Grepping psyche.rs for '
                '`decay` and `sweep` returns zero matches, so no decay schedule and no '
                'amortisation mechanism exists in the file and no implementing symbol does.'
            ),
        },
        [
            'The bound and the amortisation schedule belong on the tie-graph owner that walks '
            'edges; a genome projection cannot enforce either.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/ai
    # ------------------------------------------------------------------
    (
        "crates/ai/src/config.rs",
        "AiConfig",
        {
            "FR-CIV-AI-015": (
                'The requirement on the balance-analyst row of the agileplus-specs matrix is '
                '"Heuristic anomaly detection, then SLM triage": the system SHALL flag '
                'anomalies heuristically and route them to a small language model for triage. '
                '`AiConfig` is a plain environment-resolved config struct (model ids, '
                'concurrency cap). crates/ai contains no `anomal`, `balance` or `triage` code, '
                'so the detection heuristic and the triage call have no implementing symbol.'
            ),
        },
        [
            'The config names models the system would call; it does not perform detection or '
            'routing, so the tag asserts a pipeline that does not exist.',
        ],
    ),
    (
        "crates/ai/src/lib.rs",
        "EmbedRequest",
        {
            "FR-CIV-AI-013": (
                'The requirement on the culture-drift row of the agileplus-specs matrix is '
                '"Embeddings, then cosine drift, then a speciation threshold": culture drift '
                'SHALL be computed as cosine distance between embeddings and SHALL trigger '
                'speciation past a threshold. `EmbedRequest` is a request DTO holding a batch '
                'of texts and a snapshot hash. Grepping crates/ai/src for `cosine` returns '
                'zero matches, and nothing compares embedding vectors or applies a '
                'speciation threshold, so no implementing symbol exists.'
            ),
        },
        [
            'A request struct is the input half of the provider call; the drift comparison and '
            'the threshold decision are absent entirely.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/economy
    # ------------------------------------------------------------------
    (
        "crates/economy/src/allocation.rs",
        "JouleAllocator",
        {
            "FR-CIV-ECON-003": (
                "The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is "
                "that the economy SHALL perform numeraire selection: the good chosen as the "
                "unit of account SHALL be selected rather than assumed, and prices SHALL be "
                "expressed relative to it. `JouleAllocator` is a unit struct whose "
                "`AllocationEngine::allocate` implementation is a linear `demand.min(budget)` "
                "clamp. Grepping the crate for `numeraire` returns zero matches, so no "
                "numeraire is ever selected and no implementing symbol exists."
            ),
        },
        [
            'The allocator returns a quantity, not a price, so it has nothing in which a '
            'numeraire could be expressed even indirectly.',
        ],
    ),
    (
        "crates/economy/src/allocation.rs",
        "AllocationRegime",
        {
            "FR-CIV-MARKET-006": (
                'The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is '
                '"Polity coercion overlap SHALL flip the locale\'s regime to Planned": a '
                'locale under coercive polity overlap SHALL be forced into the planned '
                'allocation regime. `AllocationRegime` is a three-variant enum '
                '(Capitalist / Planned / Joule); the free function `allocate_with` merely '
                'dispatches on it. No coercion signal is ever fed into regime selection, so '
                'the coercion-to-regime flip has no implementing symbol.'
            ),
        },
        [
            'The enum supplies the vocabulary for the requirement but no code path ever changes '
            'a locale\'s regime as coercion rises.',
        ],
    ),
    (
        "crates/economy/src/lib.rs",
        "SCHEMA_VERSION",
        {
            "FR-CIV-ECON-001-MARKET": (
                'The `requirement` field bound to this id is an ID-hyphen-collapse '
                'decision-table row from the "RENAME candidates / Future" list in '
                'agileplus-specs/civ-002-economy-joule-system/spec.md, not a requirement '
                'sentence. `SCHEMA_VERSION` is a u32 bumped on breaking snapshot or '
                'ledger changes. It implements nothing a reader could discharge, so no '
                'implementing symbol exists for it.'
            ),
            "FR-CIV-MARKET-001": (
                'The `requirement` field bound to this id is a `Traceability:` header line '
                'from the traceability table in '
                'agileplus-specs/civ-002-economy-joule-system/spec.md, not a requirement '
                'sentence. `SCHEMA_VERSION` is a version marker only; it is a bare u32 '
                'constant with no behavior, so no implementing symbol exists for it.'
            ),
        },
        [
            'Both ids are traceability artifacts rather than requirements, so no code could '
            'ever satisfy them and the tag should be dropped rather than retargeted.',
        ],
    ),
    (
        "crates/economy/src/market.rs",
        "MarketState",
        {
            "FR-CIV-MARKET-002": (
                'The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is '
                '"Two projector functions SHALL produce the price field". `MarketState` is a '
                'struct wrapping a `BTreeMap<String, i64>` of per-good prices. Grepping '
                'crates/economy/src for `projector` and `price_field` returns zero matches; '
                'market.rs implements order-book mechanics (place_bid, place_ask, clear_all, '
                'ask_vwap, price_impact), so no projector function and no implementing symbol '
                'exist.'
            ),
            "FR-CIV-MARKET-003": (
                'The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is '
                '"Locales SHALL hold a soft membership over types (weights), not a hard '
                'switch". `MarketState` has no membership vector: it maps a good id directly '
                'to a single price, which is a hard switch. Grepping the crate for '
                '`membership_weights` returns zero matches, so no soft-membership model and no '
                'implementing symbol exist.'
            ),
            "FR-CIV-MARKET-004": (
                'The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is '
                '"Every priced locale SHALL run damped tatonnement as the baseline '
                'price-discovery dynamic". `MarketState` stores prices without recording how '
                'they were discovered. Grepping the whole crate for `tâtonnement` and '
                '`tatonnement` returns zero matches, so the damped-walrasian iteration and no '
                'implementing symbol exist.'
            ),
            "FR-CIV-MARKET-005": (
                'The requirement in agileplus-specs/civ-002-economy-joule-system/spec.md is '
                '"Where trust and trade volume are high AND the locale is near-camera or '
                'active, the locale SHALL upgrade to a CDA". `MarketState` has no trust, '
                'volume, camera-proximity or activity field, and no upgrade path. The only '
                '`CDA` hit in the crate is a doc comment in allocator.rs, so no trust-gated '
                'upgrade and no implementing symbol exist.'
            ),
        },
        [
            'The crate is 1149 lines of order-book machinery; having many functions is not '
            'evidence that the named market model exists.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/tactics
    # ------------------------------------------------------------------
    (
        "crates/tactics/src/fog_of_war.rs",
        "FogOfWar",
        {
            "FR-CIV-FOG-004": (
                'The requirement at '
                'agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:54 is '
                '"The web dashboard SHALL provide a tactics panel with unit selection, fog '
                'overlay, and jump-to-engagement". `FogOfWar` is a Rust struct holding grid '
                'size, vision radius and a packed visibility bit matrix; it is a server-side '
                'visibility structure and cannot render a client panel. web/dashboard/src has '
                'no tactics panel - the only fog UI is `updateTacticsOverlay` in scene3d.tsx, '
                'which toggles a three.js fog flag with no unit selection and no '
                'jump-to-engagement, so no implementing symbol exists.'
            ),
            "FR-CIV-FOG-005": (
                'The requirement at '
                'agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:57 is '
                '"The server SHALL filter visibility before transmission so an observer never '
                'receives hidden state". `FogOfWar` computes the visibility matrix but performs '
                'no transmission filtering, and the server path that would filter is '
                'unfiltered: `get_snapshot_for_session` in engine.rs returns the full snapshot '
                'while its own doc comment defers filtering to follow-up lanes. There is no '
                'observer mode and no implementing symbol.'
            ),
        },
        [
            'Both requirements are client- and transport-layer behaviors. The struct is the '
            'legitimate data source for them, so the tags point at the wrong crate entirely.',
        ],
    ),
    (
        "crates/tactics/src/movement.rs",
        "GridMove",
        {
            "FR-CIV-RTS-001": (
                'The requirement in agileplus-specs/civ-012-godot-secondary-client/spec.md is '
                '"Q - Move command (click target to confirm)": the client SHALL bind the Q key '
                'to a move command and SHALL require a click on the target to confirm it. '
                '`GridMove` is a three-field move intent (unit_index, new_grid_x, '
                'new_grid_y). It carries no keybinding and no confirmation state, and no '
                'keybinding or click-to-confirm handler exists anywhere in crates/, so no '
                'implementing symbol does.'
            ),
        },
        [
            'This is a client input-binding requirement filed against a server-side movement '
            'intent struct.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/build
    # ------------------------------------------------------------------
    (
        "crates/build/src/lib.rs",
        "SCHEMA_VERSION",
        {
            "FR-API-002": (
                'The requirement at agileplus-specs/civ-013-research-api/spec.md:26 is '
                '"A Python scenario runner, civlab.run_scenario(path, ticks=50), installed '
                'via pip install civlab". crates/build is pure Rust; `SCHEMA_VERSION` is the '
                'string "0.1.0-stub". No Python package named civlab exists in this '
                'repository, so the entry point, the install path and any implementing symbol '
                'are all absent.'
            ),
            "FR-API-003": (
                'The requirement at agileplus-specs/civ-013-research-api/spec.md:27 is '
                '"Policy parameter override, where invalid parameter names SHALL raise '
                'ValueError". That is Python-side behavior on a civlab runner. Nothing in '
                'crates/build reads policy parameter overrides and nothing raises ValueError, '
                'so no implementing symbol exists.'
            ),
            "FR-API-004": (
                'The requirement on the Phase 4 row of agileplus-specs/civ-013-research-api/'
                'spec.md is "Data export" as a pipeline stage of the research API. '
                '`SCHEMA_VERSION` is a version string constant and crates/build contains '
                'building and tileset-placement code with no export function, so no '
                'implementing symbol exists.'
            ),
        },
        [
            'All three ids describe a Python package that the Rust build crate can neither '
            'implement nor stand in for.',
        ],
    ),
    (
        "crates/build/src/lib.rs",
        "CultureEraWealthVector",
        {
            "FR-CIV-CLIENT-GODOT-001": (
                'The requirement on the Phase 1 row of '
                'agileplus-specs/civ-012-godot-secondary-client/spec.md is '
                '"WebSocket connection and handshake". `CultureEraWealthVector` is a '
                'three-field data DTO (culture, era, wealth) used to pick a tile-set family '
                'for procedural generation. It contains no transport code and crates/build has '
                'no client handshake, so no implementing symbol exists.'
            ),
            "FR-CIV-CLIENT-GODOT-002": (
                'The requirement on the Phase 2 row of '
                'agileplus-specs/civ-012-godot-secondary-client/spec.md is "3D scene '
                'rendering". `CultureEraWealthVector` is a plain serializable DTO and '
                'crates/build performs no rendering, so no implementing symbol exists.'
            ),
        },
        [
            'Both ids are client-lifecycle requirements filed against a procedural-generation '
            'input vector.',
        ],
    ),
    (
        "crates/build/src/lib.rs",
        "BuildingGraph",
        {
            "FR-CIV-BIO-001": (
                'The requirement on the Phase 1 row of '
                'agileplus-specs/civ-008-genetics-species/spec.md is "a species registry and '
                'YAML schema": organisms SHALL be registered under a schema that round-trips '
                'from YAML. `BuildingGraph` is a building graph (parcels, facades, provenance, '
                'completed buildings). It does round-trip RON, but the tagged type models '
                'structures, not species, and contains no genome, trait or species-registry '
                'entry, so no implementing symbol exists.'
            ),
        },
        [
            'The crate has a real serialization story, which is why the tag looks plausible, but '
            'the entity being serialized is the wrong one.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/civ-emergence-metrics
    # ------------------------------------------------------------------
    (
        "crates/civ-emergence-metrics/src/dashboard.rs",
        "EmergenceDashboard",
        {
            "FR-CIV-EMERGENCE-003": (
                'The requirement at agileplus-specs/civ-019-emergence-metrics-dashboard/'
                'spec.md:47 is "Metrics SHALL be exposed on sim.snapshot.emergence AND the '
                'emergence_metrics.v1 replay-bus event SHALL be emitted once per N ticks". '
                '`EmergenceDashboard` is a Copy struct of normalized f32 metric values; its '
                'own `compute` fills those fields and jsonrpc.rs does build a '
                '`sim.snapshot.emergence` block, so the RPC half is genuinely met. The '
                '`emergence_metrics.v1` replay-bus event has no type and no emitter anywhere, '
                'so the requirement is only half discharged: the real implementing symbol for '
                'the event half is none, because nothing emits it.'
            ),
            "FR-CIV-EMERG-004": (
                'The requirement at agileplus-specs/civ-019-emergence-metrics-dashboard/'
                'spec.md:51 is "The web dashboard SHALL provide an EmergencePanel component '
                'with a per-metric sparkline covering the last 120 ticks and a threshold-color '
                'chip". `EmergenceDashboard` is a Rust value struct and cannot render a React '
                'component. web/dashboard/src contains no EmergencePanel file - the panel set '
                'is agents, diplomacy, economy, religion, mods, perf, stats and tech_tree. A '
                'sparkline.tsx primitive exists but is not wired to emergence metrics, so no '
                'implementing symbol exists.'
            ),
            "FR-CIV-EMERG-005": (
                'The requirement at agileplus-specs/civ-019-emergence-metrics-dashboard/'
                'spec.md:55 is "The Bevy primary client SHALL provide a '
                'live_emergence_overlay HUD toggle on the E key with a glassmorphism chip '
                'group". `EmergenceDashboard` is a plain data struct with no HUD, keybinding or '
                'rendering. Grepping clients/ for `live_emergence_overlay` returns zero '
                'matches, so no implementing symbol exists.'
            ),
            "FR-CIV-EMERGENCE-012": (
                "The report's Spec file:line column for this row reads "
                '"(no spec text; requirement empty)" and no spec under docs/specs/ or '
                'agileplus-specs/ defines this id at all, so there is no requirement '
                "sentence to satisfy. `EmergenceDashboard` is a metric value struct, so the "
                "tag is unjustified in either direction and no implementing symbol exists."
            ),
            "FR-CIV-EMERGENCE-013": (
                "The report's Spec file:line column for this row reads "
                '"(no spec text; requirement empty)" and no spec under docs/specs/ or '
                'agileplus-specs/ defines this id at all, so there is no requirement '
                "sentence to satisfy. `EmergenceDashboard` is a metric value struct, so the "
                "tag is unjustified in either direction and no implementing symbol exists."
            ),
        },
        [
            'The RPC half of the metrics-exposure requirement is real and the tag is aimed '
            'near it; the event-emission half and both client panels are absent.',
        ],
    ),
    (
        "crates/civ-emergence-metrics/src/sample_snapshot.rs",
        "EmergenceSampleSnapshot",
        {
            "FR-CIV-EMERGENCE-011": (
                "The report's Spec file:line column for this row reads "
                '"(no spec text; requirement empty)" and no spec under docs/specs/ or '
                'agileplus-specs/ defines this id at all, so there is no requirement '
                'sentence to satisfy. `EmergenceSampleSnapshot` is '
                "a flat, transport-safe DTO mirroring the engine's EmergenceSample fields, so "
                "the tag is unjustified in either direction and no implementing symbol exists."
            ),
        },
        [
            'A data carrier tagged with an id that no specification ever defined.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/diplomacy
    # ------------------------------------------------------------------
    (
        "crates/diplomacy/src/shadow_networks.rs",
        "ShadowNetworkState",
        {
            "FR-DIPL-007": (
                "The report's Spec file:line column for this row reads "
                '"(no spec text; requirement empty)" and no spec under docs/specs/ or '
                'agileplus-specs/ defines this id at all, so no requirement sentence could be '
                'located to quote. `ShadowNetworkState` is '
                "a state record holding a config, per-pair leakage aggregates and a per-tick "
                "event buffer, so the tag is unjustified in either direction and no "
                "implementing symbol exists."
            ),
        },
        [
            'An empty requirement field means the binding was generated from id provenance with '
            'nothing to check against.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/engine
    # ------------------------------------------------------------------
    (
        "crates/engine/src/climate.rs",
        "WATER_MARKER_MATERIAL",
        {
            "FR-CIV-TERRAIN-005": (
                "The requirement in docs/specs/CIV-0102-climate-followup-v1.md is that water "
                "placement tools SHALL respect a single source of truth for the water marker. "
                "`WATER_MARKER_MATERIAL` is a `const MaterialId` alias equal to WATER, which "
                "does correctly name the one marker that the tool writes, and "
                "`register_coastal_water_column` uses it. But a type alias is not a tool "
                "behavior: nothing here intercepts a placement tool and validates it against "
                "the constant, so the single-source-of-truth enforcement has no implementing "
                "symbol."
            ),
        },
        [
            'The const is a reasonable thing to point at as the source of truth, but the '
            'requirement is about tools honoring it, and no tool lives here.',
        ],
    ),
    (
        "crates/engine/src/climate.rs",
        "CoastalColumn",
        {
            "FR-CIV-TERRAIN-002": (
                "The requirement in docs/specs/CIV-0101-two-zoom-lod-v1.md is that civ-voxel "
                "chunk seams SHALL be free of visible artifacts. `CoastalColumn` is a "
                "two-field water-level record (base_y, last_water_y) and contains no geometry, "
                "no mesh and no seam logic. No seam-hiding or seam-subtraction code exists "
                "anywhere in the crate, so no implementing symbol does."
            ),
        },
        [
            'A rendering property of the voxel client cannot be discharged by a tide-tracking '
            'record.',
        ],
    ),
    (
        "crates/engine/src/command_queue.rs",
        "CommandQueue",
        {
            "FR-CIV-CORE-008": (
                "The requirement at docs/specs/CIV-0001-core-simulation-loop.md:902 is "
                '"Commands from multiple clients SHALL be applied in deterministic order, via '
                'a priority queue". `CommandQueue` wraps a `VecDeque<Command>` and its own '
                'doc comment states commands are processed in FIFO order. There is no priority '
                'field, no sort and no `client_priority` on `Command`; the deterministic-order '
                'guarantee comes from FIFO alone, so the required priority queue and any '
                'implementing symbol are absent.'
            ),
            "FR-CIV-CORE-016": (
                "The requirement at docs/specs/CIV-0001-core-simulation-loop.md:917 is "
                '"Commands SHALL be prioritized by (client_priority, tick_received)". `Command` '
                "carries only client_id, seq, kind and tick_issued, and `CommandQueue` is a "
                "flat deque, so neither half of the sort key exists and no implementing symbol "
                "does."
            ),
            "FR-CIV-NOTIFY-921": (
                'The requirement in docs/specs/requirements/FR-CIV-NOTIFY.md is "a rebindable '
                'hotkey map". `CommandQueue` is a server-side data structure holding pending '
                'commands and a capacity bound; it holds no key, no binding and no input '
                'manager, so the rebinding behavior has no implementing symbol.'
            ),
        },
        [
            'Two of the three ids want an ordering key the queue never had, and the third is a '
            'client input concern.',
        ],
    ),
    (
        "crates/engine/src/constraints.rs",
        "ConstraintState",
        {
            "FR-CIV-0104-007": (
                "The requirement in docs/specs/CIV-0104-minimal-constraint-set-theorem.md is "
                '"The baseline SHALL be stable under the full constraint set": a baseline '
                "simulation SHALL remain stable when every constraint is enabled. "
                "`ConstraintState` is a per-tick tracker (ablation_mode, ticks_below_recovery_"
                "threshold, an append-only stability log) and computes no baseline. Grepping "
                "constraints.rs for `baseline` returns only doc comments, so no baseline "
                "stability assertion or implementing symbol exists."
            ),
            "FR-CIV-0104-010": (
                "The requirement in docs/specs/CIV-0104-minimal-constraint-set-theorem.md is "
                '"Recovery Window Tracking" as a tracker behavior. `ConstraintState` does own '
                "a `recovery_window` field, so the shape is present, but nothing in the crate "
                "advances or evaluates that window, and the tagged struct is a passive state "
                "bag. The tracking behavior therefore has no implementing symbol."
            ),
        },
        [
            'The recovery window is the clearest example of a field that looks like coverage '
            'but is never read.',
        ],
    ),
    (
        "crates/engine/src/engine.rs",
        "GOD_ACTION_AUDIT_CAP",
        {
            "FR-CIV-GODTOOL-921": (
                'The requirement in docs/specs/requirements/FR-CIV-GODTOOL.md is "God-tool '
                'actions SHALL support undo and a blueprint copy/paste of a region". '
                "`GOD_ACTION_AUDIT_CAP` is a bare `usize` bounding how many `GodActionRecord` "
                "entries are retained per tick. It records nothing and reverses nothing: "
                "grepping the engine for `fn undo` returns zero matches, and no blueprint or "
                "region copy/paste code exists, so no implementing symbol does."
            ),
        },
        [
            'An audit retention cap is retention bookkeeping; undo is an inverse operation on '
            'applied state and no such operation is written anywhere in the engine.',
        ],
    ),
    (
        "crates/engine/src/engine.rs",
        "SimulationSnapshot",
        {
            "FR-CIV-CORE-009": (
                "The requirement at docs/specs/CIV-0001-core-simulation-loop.md:907 is "
                '"The engine SHALL implement JSON-RPC 2.0 methods: handshake, command, '
                'snapshot and subscribe". `SimulationSnapshot` is a plain serializable state '
                "aggregate (tick, population, citizen_count, building_count) and dispatches "
                'nothing. The JSON-RPC surface does exist, but it lives in a different crate '
                'at `JsonRpcMethod` in crates/server/src/jsonrpc.rs, so the real implementing '
                'symbol is that dispatch enum and the tag simply points at the wrong symbol; '
                'this declaration implements none of the four methods.'
            ),
        },
        [
            'The behavior is genuinely implemented, just three crates away, so this is a '
            'retargeting problem rather than a missing one.',
        ],
    ),
    (
        "crates/engine/src/info_views.rs",
        "InfoOverlay",
        {
            "FR-CIV-INFOVIEW-916": (
                'The requirement on the A3 Temperature row of the docs/audits/fr-matrix is a '
                'temperature overlay. `InfoOverlay` is a catalog entry (id, name, group, '
                'render_kind, legend stops) and computes no overlay values. No function in '
                'info_views.rs derives a temperature value per cell, so no implementing symbol '
                'exists.'
            ),
            "FR-CIV-INFOVIEW-917": (
                'The requirement on the A8 Resource Deposits row of the docs/audits/fr-matrix '
                'is a resource-deposit overlay. `InfoOverlay` is a catalog entry and no '
                'deposit-overlay compute function exists in the file, so no implementing '
                'symbol exists.'
            ),
            "FR-CIV-INFOVIEW-918": (
                'The requirement on the E1 Roads row of the docs/audits/fr-matrix is a roads '
                'overlay. The matrix row itself notes that the traffic graph exists but that '
                'this is only the first Gizmo render-kind exemplar, i.e. the overlay is '
                'aspirational. `InfoOverlay` is a catalog entry and no roads-overlay compute '
                'function exists, so no implementing symbol exists.'
            ),
            "FR-CIV-INFOVIEW-919": (
                'The requirement on the C4 Wealth row of the docs/audits/fr-matrix is a wealth '
                'overlay, which the matrix rates NEAR priority. `InfoOverlay` is a catalog '
                'entry and no wealth-overlay compute function exists, so no implementing symbol '
                'exists.'
            ),
            "FR-CIV-INFOVIEW-921": (
                'The requirement on the B6 Migration Flow row of the docs/audits/fr-matrix is '
                'a migration-flow overlay, also rated NEAR. `InfoOverlay` is a catalog entry '
                'and no migration-flow overlay compute function exists, so no implementing '
                'symbol exists.'
            ),
        },
        [
            'The registry is the right place to hang a pointer to a future overlay compute '
            'function, but as written each tag claims an overlay that was never computed.',
        ],
    ),
    (
        "crates/engine/src/metrics.rs",
        "MetricsFixed",
        {
            "FR-CIV-METRICS-001-TIMESERIES": (
                'The `requirement` field bound to this id is a row of the docs/audits/'
                'fr-matrix ID-rename table that explicitly calls it a phantom alias and states '
                'that the non-hyphenated form is the real requirement, with the real '
                'hybrid-replay line living in PLAN.md. `MetricsFixed` is a Copy wrapper of four '
                'Fixed fields, so the phantom id cannot be discharged by it and no implementing '
                'symbol exists.'
            ),
        },
        [
            'This id should be reclassified as an alias in the ID inventory so it stops '
            'producing bindings at all.',
        ],
    ),
    (
        "crates/engine/src/perf.rs",
        "TickProfile",
        {
            "FR-CIV-PERF-004": (
                'The requirement on the WS Command Latency row of the docs/audits/fr-matrix is '
                'WebSocket command-latency measurement. `TickProfile` records per-phase tick '
                'timing and a total, and has zero references outside perf.rs - nothing in '
                'production constructs it - so no command-latency instrumentation is wired and '
                'no implementing symbol exists.'
            ),
            "FR-CIV-PERF-006": (
                'The requirement on the 10k Citizens row of the docs/audits/fr-matrix is a '
                'full-snapshot capability at 10k citizens. `TickProfile` is a struct of '
                'counters with zero references outside perf.rs, and there is no 10k-citizen '
                'snapshot test anywhere in the tick path, so no measurement and no implementing '
                'symbol exist.'
            ),
        },
        [
            'An orphaned pub API is the whole story here: the type looks like instrumentation, '
            'but no code path ever builds it.',
        ],
    ),
    (
        "crates/engine/src/social_types.rs",
        "StratBand",
        {
            "FR-CIV-POLITY-007": (
                'The requirement in agileplus-specs/civ-007-diplomacy-laws-government/spec.md '
                'is "a polity SHALL dissolve when its internal mean coordination falls below '
                'the anarchic floor for a sustained window". `StratBand` is a four-variant '
                'stratification enum (Poor / Middle / Rich / Elite) with a rank used for '
                'promotion and demotion. Grepping the crate for `anarch` and `dissolve` returns '
                'nothing, and the enum has no coordination value and no sustained-window '
                'logic, so no implementing symbol exists.'
            ),
        },
        [
            'The enum supplies an ordering, not a dissolution trigger, and nothing in the crate '
            'tracks a mean coordination value to threshold against.',
        ],
    ),
    (
        "crates/engine/src/social_types.rs",
        "UnrestLevel",
        {
            "FR-CIV-NOTIFY-901": (
                'The requirement in docs/specs/requirements/FR-CIV-NOTIFY.md is "alert rules '
                'in RON, for example happiness below X, which are measured rather than '
                'scripted". `UnrestLevel` is a four-variant enum (Stable / Restless / '
                'Rioting / Revolting) with a hardcoded `from_score` ladder of fixed score '
                'thresholds. No RON file is loaded and no threshold is data-driven, so the '
                "requirement's explicit intent - rules authored in RON rather than compiled "
                'into Rust - is violated outright and no implementing symbol exists.'
            ),
        },
        [
            'The hardcoded ladder produces plausible levels from the same inputs, which is why '
            'the tag survives review, but it is the opposite of the authored-rule model the '
            'requirement asks for.',
        ],
    ),
    (
        "crates/engine/src/tutorial.rs",
        "TutorialMilestone",
        {
            "FR-CIV-NOTIFY-920": (
                'The requirement in docs/specs/requirements/FR-CIV-NOTIFY.md is tutorial '
                'milestones as a behavior: milestones SHALL advance as the simulation reaches '
                'them. `TutorialMilestone` is a five-variant ordered enum naming the stages. '
                'The advancement does exist, but it is `advance_from_sim` further down the '
                'same file, which is therefore the real implementing symbol; the tag sits on '
                'the enum that only enumerates them, and that enum implements nothing.'
            ),
        },
        [
            'A retarget onto advance_from_sim would be correct today; the tag currently claims '
            'coverage from a bare enum.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/planet
    # ------------------------------------------------------------------
    (
        "crates/planet/src/geology.rs",
        "BiomeKind",
        {
            "FR-CIV-3D-015": (
                'The requirement on the Texture Atlas Completeness row of '
                'docs/specs/CIV-0101-two-zoom-lod-v1.md is an atlas table mapping every biome '
                'to its texture slots. `BiomeKind` enumerates biomes and is driven by a real '
                'elevation/temperature/moisture classifier, but it carries no texture or slot '
                'field. Grepping geology.rs for `atlas` returns zero matches, so the atlas '
                'table and any implementing symbol are absent.'
            ),
        },
        [
            'A complete biome enum is the necessary precondition for atlas completeness but does '
            'not establish it; the sibling biome-coverage id is the one this enum really '
            'discharges.',
        ],
    ),

    # ------------------------------------------------------------------
    # crates/research
    # ------------------------------------------------------------------
    (
        "crates/research/src/lib.rs",
        "TechCard",
        {
            "FR-CIV-RESEARCH-001-SCENARIO": (
                'The `requirement` field bound to this id is a row of the docs/audits/'
                'fr-matrix ID-rename table that says the real LLM cache and card-acceptance '
                'line is the non-hyphenated parent id and which marks this id a phantom alias. '
                '`TechCard` is a struct describing a '
                'proposed tech (id, era, inputs, energy_cost, byproducts, dependencies) and '
                'the rename row itself identifies `LlmEvent::cache_key` as the real '
                'implementing symbol, which is the symbol that should carry the tag instead.'
            ),
            "FR-CIV-RESEARCH-003-EXPORT": (
                'The `requirement` field bound to this id is a row of the docs/audits/'
                'fr-matrix ID-rename table whose real requirement is the hybrid-replay line at '
                'crates/research/src/lib.rs:616 and which explicitly marks this id as a '
                'phantom alias of a non-hyphenated parent. `TechCard` is a plain card '
                'description struct and implements no export, so no implementing symbol is '
                'tagged that can discharge the alias.'
            ),
        },
        [
            'Both ids are alias artifacts of the ID inventory rather than requirements; they '
            'should be reclassified so they stop generating bindings.',
        ],
    ),
    (
        "crates/research/src/lib.rs",
        "ValidationOutcome",
        {
            "FR-CIV-RESEARCH-002-SNAPSHOT": (
                'The `requirement` field bound to this id is a row of the docs/audits/'
                'fr-matrix ID-rename table that says the real canonical-replay line is '
                'crates/research/src/lib.rs:601 and which marks this id as a phantom alias of '
                'a non-hyphenated parent. `ValidationOutcome` is a two-variant enum '
                '(Accept, Reject(RejectReason)) describing a validator verdict. It performs no '
                'replay, so no implementing symbol is tagged that can discharge the alias.'
            ),
        },
        [
            'The canon-save replay behavior that the parent id names does exist and is exercised '
            'by the replay-mode tests, so this is a retargeting problem rather than a gap.',
        ],
    ),
]

# The 70 FALSE tags from the report that a prior unbind pass already deleted from
# the source. Recorded so the arithmetic in the self-check is auditable; these are
# intentionally NOT transcribed into SITES because the tag no longer exists on a
# container.
ALREADY_REMOVED = {
    # crates/server/src/session.rs - 33 tags on `const SESSION_HISTORY_CAP`
    "crates/server/src/session.rs": [
        "FR-SESSION-%03d" % n for n in range(1, 34)
    ],
    # crates/engine/src/engine.rs - 37 tags on `struct WorldState` (35) and
    # `struct Simulation` (2)
    "crates/engine/src/engine.rs": [
        # 26 FR-SOC-* dynamics tags on struct WorldState
        "FR-SOC-INS-001", "FR-SOC-INS-002", "FR-SOC-INS-003", "FR-SOC-INS-004",
        "FR-SOC-INS-005", "FR-SOC-INS-006", "FR-SOC-INS-007",
        "FR-SOC-CIV-001", "FR-SOC-CIV-002",
        "FR-SOC-INT-001", "FR-SOC-INT-002", "FR-SOC-INT-003", "FR-SOC-INT-004",
        "FR-SOC-COH-001", "FR-SOC-COH-002", "FR-SOC-COH-003", "FR-SOC-COH-004",
        "FR-SOC-INTG-001", "FR-SOC-INTG-002", "FR-SOC-INTG-003", "FR-SOC-INTG-004",
        "FR-SOC-INTG-005", "FR-SOC-INTG-006", "FR-SOC-INTG-007",
        # 9 more on struct WorldState
        "FR-CIV-ARCH-006", "FR-CIV-ARCH-NOSVG-001",
        "FR-CIV-CORE-002", "FR-CIV-CORE-004", "FR-CIV-CORE-019",
        "FR-CIV-PERF-RT-003", "NFR-CIV-PERF-002",
        "FR-PROT-001", "FR-PROT-002", "FR-PROT-003", "FR-PROT-005",
        # 2 on struct Simulation
        "FR-CIV-CORE-003", "FR-CIV-CORE-017",
    ],
}

# FALSE-row totals asserted by the report.
#
# The report's Verdict counts table gives 81 NOT-IMPLEMENTED + 42 CONTAINER-ONLY
# = 123 FALSE rows, and parsing its verdict table yields exactly 123 distinct
# FALSE ids with no id listed twice, so rows and ids coincide for this report.
REPORT_FALSE_ROWS = 123
REPORT_FALSE_IDS = 123
ALREADY_REMOVED_ROWS = sum(len(v) for v in ALREADY_REMOVED.values())


def self_check():
    """Return the four self-check numbers as a dict."""
    transcribed = {}
    for entry in SITES:
        _file, _decl, ids, _notes = entry
        for rid in ids:
            if rid in transcribed:
                raise AssertionError(
                    "%s appears in two SITES entries (%s and %s)"
                    % (rid, transcribed[rid], _file)
                )
            transcribed[rid] = _file

    decls = [(f, d) for f, d, _i, _n in SITES]
    if len(set(decls)) != len(decls):
        raise AssertionError("a (file, declaration) pair appears in more than one entry")

    removed = set(ALREADY_REMOVED["crates/server/src/session.rs"]) | set(
        ALREADY_REMOVED["crates/engine/src/engine.rs"]
    )
    overlap = set(transcribed) & removed
    if overlap:
        raise AssertionError("ids transcribed that are already removed: %s" % sorted(overlap))

    unclassified = REPORT_FALSE_IDS - len(removed) - len(transcribed)
    return {
        "false_rows_in_report": REPORT_FALSE_ROWS,
        "false_ids_in_report": REPORT_FALSE_IDS,
        "already_removed_from_source": len(removed),
        "transcribed": len(transcribed),
        "unclassified": unclassified,
        "sites": len(SITES),
    }


if __name__ == "__main__":
    r = self_check()
    print("Container-binding triage — sim/domain FALSE verdicts")
    print("  FALSE rows in the report .................. %d" % r["false_rows_in_report"])
    print("  distinct FALSE ids in the report .......... %d" % r["false_ids_in_report"])
    print("  already removed from source (skipped) ..... %d" % r["already_removed_from_source"])
    print("  transcribed (still tagged on a container) . %d" % r["transcribed"])
    print("  could not classify ....................... %d" % r["unclassified"])
    print("  SITES entries ............................ %d" % r["sites"])
    accounted = r["already_removed_from_source"] + r["transcribed"] + r["unclassified"]
    assert accounted == r["false_ids_in_report"], (
        "%d + %d + %d != %d"
        % (r["already_removed_from_source"], r["transcribed"], r["unclassified"],
           r["false_ids_in_report"])
    )
    assert r["unclassified"] == 0, "%d FALSE ids unaccounted for" % r["unclassified"]
    print("\nOK: %d transcribed + %d already removed = %d FALSE ids"
          % (r["transcribed"], r["already_removed_from_source"], r["false_ids_in_report"]))
