# Civis source-coverage ledger — pass 1

Program `gaming-pair-20260929`; observed 2026-09-29. Analyzed source **b3cd62a7394878cc64d024fbfcfd398b8bb88bf1**; registry **85d7cd00cf59c379c05b740e8130a85b0d5bd31b**. Specification commits are not a qualified implementation candidate.

## Open denominator

This inventories meaningful source families, not every file. **File-level source coverage is unknown; no percentage is legitimate.** Full Git history/branches and useful deleted predecessors are not exhausted. The available clone failed DNS, native cargo is absent, and some search responses returned unrelated repositories; those results were discarded. A failed or truncated collection is not a resolved source.

`Read` and `resolved` are independent. Resolution needs meaning, authority, contradictions, obligations or explicit non-normative judgment, reachable implementation implications, stage/journey implications and verification consequences. Families remain open until all meaningful descendants are inventoried and resolved. Counts of files, old FRs, prompts or tests cannot substitute for this denominator.

## Source families

| ID | Source / inspected extent | Classification and meaning | Obligations / contradictions / surfaces | Stage and oracle implication | Resolution |
|---|---|---|---|---|---|
| C-S01 | Current user assignment, full | Normative program policy | Exactly Civis and Dino; mature-first, no quota, authority-aware traces, independent evidence, draft-only | No third product or fabricated completion | Resolved for current assignment |
| C-S02 | Conversation retrieval: Feb 19-21 CivLab; Aug 25/29 Civis; Sep 16 machine verification | Retrieved USER INTENT; assistant proposals kept distinct | Origins/proto-life to civilization, intervention, observable gameplay and research lineage. Generic later deterministic descriptions are not automatically an authorized reversal of explicit charter | Preserve origin mode; distinguish playable outcome from research validity | Partial; original transcript and later explicit decisions still needed |
| C-S03 | PRD.md:1-230, blob eef889b10cec5f0268ab9f32669e30060027f71e | Historical design, labeled APPROVED 2026-02-21 | CivLab headless deterministic multi-client game/research platform. Performance/uniqueness claims lack qualifying evidence. Conflicts with May charter | Do not impose bit-identical replay or multiplayer on v1 from old PRD | Partial; recover approval and supersession scope |
| C-S04 | docs/guides/emergence-charter.md, full; blob 0eaa1a0399947bf85ced5833489685e117a25de1 | Governing design as recorded; original actor authorization not recovered | Only substrate laws authored; life/culture/markets/polities emerge. May29 correction explicitly drops global deterministic replay; May31 v1 excludes multiplayer/co-op/spectator, includes actor animation | Snapshot persistence is required; future random trajectories need not match. Same-process/multi-renderer transport is not multiplayer | Read; normative conflict review remains open |
| C-S05 | docs/adr/ADR-determinism-dropped.md, full; blob aa5fa9ae804452c77754a4a27d52cf12ee42ab0b | Accepted design AS RECORDED, dated 2026-05-30 | Explicitly supersedes seed-only/bit-identical requirements for main simulation; subsystem opt-in allowed | RNG-stream replay tests only qualify an explicitly opted-in replay subsystem, not the whole product | File meaning resolved; acceptance provenance and cross-doc repair open |
| C-S06 | README.md:1-240, blob bb9bf1090ad5b3b92060223f8f8432ca9363b44f | Supporting documentation and current-state claims | 95% alpha claim is not accepted evidence; deterministic framing and fixed tech/victory language conflict with charter; standalone/server routes documented | Startup/rendering labels do not close player or save journeys | Partial |
| C-S07 | AGENTS.md:1-180, blob 029b0855521331554498e41f250785f40e454540 | Contributor policy / historical status | Several client/mod/replay gates and partial mod-host boundaries; maturity labels not proof | No complete mod marketplace or extra client program invented from instructions | Partial; follow linked gate implementations |
| C-S08 | Current main commit b3cd62a, metadata/message; docs/audits provenance correction referenced | Historical work receipt / author assertion | Commit reports removing nonexistent specs and wrongly rebound FR-UX IDs; RNG/save assertions require source and contract checks | Test names and trace matrix rows cannot establish semantic trace correctness | Partial: read correction artifacts, verifier code and changed assertions |
| C-S09 | Useful-history search for CivLab: 87c1c45, 5425512, 17192ce, 99a5db5, a4bc05f, af913fb | Historical author claims; not runtime evidence | April25 README replaced civic-infra fiction; May/June integration records show standalone/server and charter evolution, dropped determinism, broadened emergence | Distinguish identity recovery from implementation progress; no historical test run rebound to current candidate | Partial; commit bodies sampled, diffs/full lineage not exhausted |
| C-S10 | .gitmodules, full; blob c4540115a271d0f0b3eb09f3efdeeaf55d511ad2 | Current dependency declaration | vendor/phenodocs points to KooshaPari/phenodocs; short historical pin in registry is not enough | Resolve full gitlink SHA before consuming dependency source | Declaration resolved; gitlink/source review open |
| C-S11 | crates/engine/src/save_bundle.rs:1-260 and 300-620, blob 458383c00f5db14075d4fd93878fa35543e2ff29 | Current implementation | Format4 sidecars; complete save_dir/load_dir inspected. Metadata absence selects legacy v1. World state restored after replay load; state duplicated across sidecars/live fields | Snapshot conservation/consistency and explicit legacy classification are primary recovery oracles | Partial; C-F02/C-F03 below; archive remainder/helpers/callers not yet inspected |
| C-S12 | engine state/phase/scheduler/RNG/replay modules located through source and PLAN search | Current implementation to inspect | WorldState vs Simulation-owned state vs ECS/projection identity; conditional replay and emergent phase ownership | Faithful current-state restore differs from same-future replay; wrong simulation instance is a negative control | Open |
| C-S13 | Planet/material/voxel/fluid/laws/genetics/species source families named in charter and history | Current implementation + authored model | Enumerate rules, units, conservation, boundaries, genome/phenotype/speciation mappings; reject unsupported reality claims | Need material/biology model oracles and LOD boundary experiments | Open |
| C-S14 | Agents/needs/psyche/culture/language/legends/economy/institutions/warfare/technology | Current implementation + possibly conflicting designed outcomes | Distinguish authored primitive, behavioral rule, measured label, scenario intervention and hardcoded social outcome | Emergence cannot be graded by merely seeing a predetermined enum/event | Open |
| C-S15 | CLI/MCP/HTTP/JSON-RPC/WebSocket/control paths documented, dispatch not fully read | Interface claims/current implementation | Locate mounted handlers and validate authorization, typed errors, scope/run identity, mutation visibility and recovery | An API green against civ-server does not qualify a civ-standalone process | Open |
| C-S16 | clients/bevy-ref standalone/live attach; Godot/Unreal/Web clients; UI routes/screens/components | UI/runtime implementation | Trace actual menu -> settings -> worldgen -> command -> state -> rendering -> save flow; other clients may be experimental | No renderer multiplication to inflate mature or v1 completion | Open |
| C-S17 | Save database, migrations, archives, guest snapshots, settings and config | Persistence surfaces partially located | Cross-artifact integrity, atomic commit/rollback, migration compatibility and missing-state treatment | Restart and malformed/partial archive fixtures needed | Open beyond C-S11 |
| C-S18 | Mod-host/SDK/signing/capabilities/Wasmtime referenced in AGENTS/history | Extensibility implementation/policy | Determinism is not a universal admission requirement; sandbox permissions/resource limits still matter | Signature != safe behavior; restore guest and host state consistently | Open |
| C-S19 | Asset/import/animation/audio/rendering/shaders/GPU feature flags | Implementation / quality overlays | Asset rights, actual rig playback, fallback visibility, frame-time and feature-specific evidence | A render module existing is not reachable/animated/accessible gameplay | Open |
| C-S20 | Tests, BDD, native smoke, visual capture, traceability and existing oracle tools | Candidate evidence and implementation | Discover real assertions/skips, wrong-ID collisions and exact executed configuration; no native run this cycle | Zero cases, skipped gates, optional clients and unrelated screenshots cannot become greens | Open |
| C-S21 | CI/quality manifest/verifiers/release/deployment/security/supply chain | Operational implementation, mostly located through historical claims | Candidate binding, optional-gate aggregation, policy ownership, artifact integrity and stable approval require audit | Historical message says missing manifest became warning; verify current code before alleging current behavior | Open |
| C-S22 | Registry docs/intent/Civis.md, full; blob 33fb13a4e2c8ab39fb9dc3cfea850a09f076666e | Placeholder + binding index | 8 prompts and 3 responses are not accepted requirements; all listed bindings require recovery | No coverage credit for binding counts | File meaning resolved; underlying corpus OPEN |
| C-S23 | Registry docs/boundary/Civis.md, full; blob 07a4bd7b7153c9c2ec746eeca63ee0ff034d98e4 | Historical/stale boundary | Says archived/read-only/no successor; current repository active with same-day main changes. Archive rationale included missing local cargo and workspace size | Do not infer product should not exist from a host toolchain failure | Contradiction established; dated registry correction required |
| C-S24 | Registry projects/Civis.json and CivLab audit hits | Supporting registry assertions | Active CivLab engine description contradicts archived boundary within same registry snapshot | Registry-local truth must converge without deleting historical evidence | Partial |
| C-S25 | External research: WorldBox, Mesa, Bevy, Wasmtime, ODD | EXTERNAL PRIOR ART | Best absent-product alternatives differ for play vs research; standards help describe models but do not validate their realism | Pre-build alternatives and post-build pilot remain distinct | Partial; see registry research passes |
| C-S26 | UX/onboarding/docs/support/accessibility/external pilots | Product surface not fully inventoried | Closed user journeys, honest scenario assumptions, discoverability and operating burden | No external product validation inferred from commits or internal screenshots | Open |

## C-F01 — correction to the initial recovery hypothesis

The initial README/PRD-based hypothesis treated deterministic replay as a universal mature spine. **Evidence falsified that hypothesis.** The May charter correction and Accepted ADR explicitly drop that requirement and exclude multiplayer/co-op/spectator from v1. Working recovery honors those explicit scope corrections over generic older descriptions. Later user authorization and all contradictory FRs still require archaeology; no new mandatory deterministic research mode is invented here.

A missing RNG stream position is therefore **not automatically a whole-product blocker**. It may block an explicitly promised subsystem replay contract. Faithful restoration of actual persisted state, identity, model/configuration and pending operations remains a different, important obligation.

## C-F02 — snapshot classification and cross-file consistency risk

In the inspected complete load_dir path, absent metadata is interpreted as legacy version1; required environment/stock/institution sidecars are enforced only for version>=4. The method parses metadata but uses its format_version, not its spec_id/tick, to select this path. It reconstructs via replay and then replaces state and restores sidecars. This creates a **source-observed ambiguity to test**: removing metadata from a damaged v4 bundle can select legacy behavior, and conflicting independently valid sidecars may need stronger consistency checks.

This is not a native reproduced corruption report. Legacy support may be intentional; the contract must distinguish explicitly supported legacy data from incomplete current-format saves. Required controls: remove metadata, change spec_id, mismatch ticks, omit world_state, pair sidecars from two saves, preserve the original bytes after failed migration. Do not 'fix' by banning legitimate legacy saves without acceptance review.

## C-F03 — durable-save transaction boundary

save_dir writes multiple files sequentially to its destination; save_archive builds a temporary directory then writes compressed bytes directly to the destination with fs::write. The inspected entry points do not themselves establish an atomic replace of an existing accepted save. Higher-layer safeguards have not yet been inspected. Fault injection must interrupt each write and prove the prior committed save remains recoverable, or prove the caller provides the required transaction boundary. A successful round-trip test alone is insufficient.

## C-F04 — emergence and performance claims need different evidence

The charter's inference that reality-like rules yield reality-like outcomes and its 'disk primary, not compute' scale assertion are hypotheses, not qualification evidence. Separate: model validity, statistical variation, current-state integrity, gameplay intelligibility, and performance on a named workload/hardware/configuration. No scientific or scaling conclusion is certified by this pass.


## Pass 13-16 ledger continuation

| ID | Source | Classification | Resolution / consequence | Status |
|---|---|---|---|---|
| C-S27 | Prior Feb19 conversation + later May charter/ADR | USER INTENT + accepted later correction | Broad coupled politics/economics/war + macro/detail simulation is user horizon; older assistant deterministic LOD proposal is superseded on global replay semantics | Partial conversation corpus recovered |
| C-S28 | Recovery CI run36628556544 + artifact11062202963 | VERIFIED OBSERVATION on CivSaveBundle candidate | Economy policy resets, research lost, orphan guest memory survives without loaded mod identity | Resolved for exercised bundle subjects; mounted user journey open |
| C-S29 | Current main590fad06 audit spec-only-triage | SUPPORTING AUDIT / contradictory catalog evidence | Finds scanner blind spots, dead substrate counted, synthetic155-ID emergence range, namespace collisions, and authority questions | File reviewed; authority decisions open |
| C-S30 | recovery_state_manifest_prototype.rs | EXPERIMENT / architecture candidate | Test-only semantic manifest captures policy/research/active mod identity and orphan-memory compatibility; production save untouched | CI queued |
| C-S31 | Current main drift54d57589 ->590fad06 | CURRENT SOURCE DRIFT | One docs-only audit commit; no production code change, so reproduced behavior remains relevant but candidate identity stays exact | Resolved drift extent |

### Catalog quarantine

The generated FR-CIV-EMERGENCE-100..254 range, reserved/report-only IDs, template-only rows, namespace collisions, and dead/unmounted symbol bindings are ineligible for mature-contract grading unless independently recovered as authored accepted obligations. Audit inventory counts are not the mature denominator.

### Authority blockers

Do not silently decide whether root FUNCTIONAL_REQUIREMENTS.md, docs/design per-ID catalogues, or historical civlab batch-analysis/run-management surfaces are normative. Continue archaeology; if explicit accepted/user evidence remains absent, request a user decision before final contract closure.


## C-S27 — current main audit and authority

Current `main` observed 2026-09-30: `590fad0643eb85cae89edd9e64ed6b991461de6e`. Compared with inspected code revision `54d5758970249c8d1f24688ea45920b530e77299`, it is exactly one commit ahead and adds only `docs/audits/spec-only-triage-2026-09-29.md` (+339 lines). No code changed, so `54d57589...` remains the effective current implementation snapshot for the mapped surfaces while `590fad06...` is the current repository/document snapshot.

The new audit is classified **supporting audit / authority-contested catalogue analysis**:
- useful evidence of scanner blind spots, dead-substrate false coverage, generated/range-manufactured IDs, namespace collisions and missing tests;
- its 205-row verdict set is NOT imported as accepted product requirements;
- its rule treating per-ID acceptance criteria in design documents as REAL-GAP requires independent intent/authority review;
- FUNCTIONAL_REQUIREMENTS.md is Draft and predates explicit May determinism/multiplayer supersession.

Detailed resolution: PASS-22-AUTHORITY-SUPERSESSION.md.

Resolution state: PARTIAL. The audit file meaning is understood; its underlying 205 sources/authority decisions are not thereby resolved.
