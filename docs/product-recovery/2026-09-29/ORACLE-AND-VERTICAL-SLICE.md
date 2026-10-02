# Civis — journey/oracle and vertical slice contract v0.1

Date2026-09-29. Source **b3cd62a7394878cc64d024fbfcfd398b8bb88bf1**. All native cases below **NOT RUN**. Draft anchors are excluded from product grading pending semantic acceptance.

## Journeys and one coherent slice

C-J01: player/modeler creates a bounded world under declared model/configuration -> sees/inspects its actual state -> performs one supported intervention -> observes a traceable meaningful effect in that same world.

C-J02: player saves the changed world -> exits/restarts -> selects the save -> returns to a faithful present state with functioning inspection/control. No global requirement that future random trajectories be bit-identical.

C-J03: player/modeler follows a declared emergent behavior -> inspects relevant state/rules/history -> perturbs relevant conditions -> distinguishes modeled cause/measurement from scripted label or unsupported reality claim.

C-J04: invalid command, damaged save or unsupported/unauthorized extension is presented -> state is protected or explicit recovery occurs -> user understands which operation failed without a false green.

CVP closes J01/J02 plus the failure controls applicable to them. MVP widens coupled behavior and J03/J04. Native UI and machine interface must resolve to the same actual world. One bounded origin/substrate scenario remains in scope where required by accepted origin-to-civilization intent; do not replace the mature concept with a disposable city-counter demo.

The proposed slice uses actual simulation persistence and a real mounted human/machine interface, not mock routes or static screenshots. Select the exact intervention after tracing existing mounted code. Prefer a supported small material/environment or agent intervention with an independently measurable effect and non-default saved state; do not invent a new high-level subsystem just to create a demo.

## State manifest required before save qualification

Enumerate every authoritative state owner: WorldState fields, Simulation-owned stores, ECS state, environmental/planetary state, stockpiles, institutions, guest state, queued accepted operations/events and configuration/model identities. Classify each as persist directly, reconstruct under a justified invariant, or intentionally ephemeral. Record duplicate mirrors and their precedence. Unknown fields remain unknown; serialization deriving is not an exhaustive manifest.

Load acceptance compares declared present-state observations/identity and state consistency. An explicitly chosen replay subsystem may additionally qualify its own RNG/schedule semantics. Do not use a whole-game replay test to reverse the May charter.

## Adversarial oracle matrix

| Case | Stimulus | Expected acceptance behavior | False green exposed |
|---|---|---|---|
| Real intervention | Mounted UI/machine command on known world | State and user-visible effect match command, world and candidate | UI/HTTP success without live mutation |
| Same-world save/return | Non-default environment/stocks/institutions/agents/guest state where supported | Present-state contract restored; fresh runtime identity linked to durable world | Replay/default reconstruction loses sidecar or non-replay state |
| Current-format metadata loss | Delete metadata from a known format4 fixture | Explicit corruption/classification outcome; no silent downgrade-qualified success | Missing metadata interpreted as legacyv1 without integrity basis |
| Genuine legacy | Supported old fixture with declared origin/version | Correct explicit migration, limitations shown, original bytes preserved | Fixing metadata issue by indiscriminately rejecting valid old saves |
| Cross-world sidecars | Mix individually valid files from two worlds/ticks/models | Reject inconsistent world or identify explicit repair requirements | Independent JSON parsing counted as save integrity |
| Missing authoritative data | Remove required world/environment/stock/institution/guest component | Reject or formally allowed reconstruction only | Optional-file fallback silently drops actual state |
| Crash at write boundary | Interrupt each save and slot replacement step | Previous committed save or complete new save recoverable | Happy-path roundtrip ignores destructive partial writes |
| Invalid/unauthorized command | Wrong scope, malformed values, missing capability | Deny with unchanged authoritative state and useful failure | Agent has endpoint access and mutates any world |
| Stale/disconnected client | Render old snapshot or control another process | Stale/foreign observation cannot qualify current command | Standalone screenshot qualifies civ-server or vice versa |
| Emergence ablation | Remove claimed causal coupling, leave labels/feed/UI | Domain oracle detects lost behavior | Predetermined label mistaken for emergent result |
| Resolution transfer | Promote/demote regions while effects cross boundary | Declared conserved quantities, event/entity ownership and approximation hold | Graphics LOD assumed to prove simulation scalability |
| Dependency failure | Storage full, plugin trap, unavailable asset/collector | Explicit degraded/rejected state under accepted policy | Silent fallback counted as normal mature feature |
| Conflicting/stale evidence | Two incompatible observations or old candidate receipt | BLOCKED/CONFLICT, preserve both with provenance | Latest favorable result wins by default |
| Worker replacement | Replace development agent between attempts | Same accepted contract and durable effort/world evidence remain | Agent reset erases failed product assessments |

Numeric tolerances, sample design and failure budgets must be recovered/accepted for each subject. A statistically variable model needs statistical checks where relevant; an unbounded 'looks plausible' LLM judgment is not an oracle. Visual inspection is valuable for rendering/UX but not a substitute for authoritative state or a model-validity experiment.

## Bounded current implementation mapping

`crates/engine/src/save_bundle.rs` blob `458383c00f5db14075d4fd93878fa35543e2ff29`: ranges1-260 and300-620 read, including full save_dir/load_dir and archive-write entrypoint. Format4 writes metadata, mod state, mirrored world state, environment, stocks, institutions and replay. load_dir defaults metadata absence to v1, only enforces named sidecars for v4, reconstructs via replay then overwrites/mirrors snapshot state. The inspected method does not establish cross-file tick/spec/world consistency. save_dir writes sequentially and save_archive uses direct destination fs::write; callers may provide additional safeguards and remain uninspected.

These are source-observed risks, not native reproduced corruption reports. Do not assert all validation is absent. State ownership, save slot/database transaction wrapper, actual mounted user path and native failure injection remain to inspect. The latest main author's RNG/replay note is scoped against the corrected contract rather than automatically imported as a whole-product blocker.

Ledger continuation: exact phenodocs gitlink resolved to **35e0e90a19dfd93ce4e1a81210ade92088b2bbbc**, source contents unreviewed. `docs/research/RESEARCH_INDEX.md` fully read (blob `d5436a1a05ca1c0a99ff3c73072278544e585b68`); existing competitor/model/engine research needs primary-source and supersession checks, not wholesale replacement.

## Evidence architecture and controls

Policy and approved verifier identity live outside candidate-worker write access. Bind each evaluation to accepted contract/criterion, built candidate, model/mod/feature configuration, environment/runtime mode, world, run, timestamp and raw evidence. Build provenance, gameplay outcome, scientific model validity and user sentiment are distinct predicates; no compensating average for failed critical dimensions.

The shared synthetic predicate experiment validates a small fail-closed identity/conjunction model only. It provides no native Civis qualification, no collector-authentication proof and no independent fresh review. Protected acceptance-policy deployment and a fresh reviewer actively seeking alternative interpretations are still strict blockers.
