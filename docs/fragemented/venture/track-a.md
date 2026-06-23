# Venture Track A Artifact + Determinism

Source mirror: `./venture/TRACK_A_ARTIFACT_DETERMINISM_SPEC.md`

# Venture Track A Artifact IR and Determinism Closure

## Scope
Closes Track A gaps for artifact contracts, deterministic build/replay, and Veo/NanoBanana compiler behavior.

## Artifact IR Family
1. `SlideSpec`: deck layout, slide graph, style tokens, source references.
2. `DocSpec`: sections, constraints, citations, output channels.
3. `TimelineSpec`: scenes, timing, transitions, narration anchors.
4. `AudioSpec`: voice config, script segments, timing, loudness profile.
5. `BoardSpec`: whiteboard objects, connectors, layering, animation steps.

All IR objects require: `schema_version`, `content_hash`, `inputs_hash`, `policy_bundle_id`, `created_at`.

## Deterministic Build/Replay Contract
1. Idempotency key: hash of `(ir_hash, toolchain_version, policy_bundle_id, target_surface)`.
2. Cache key equals idempotency key plus explicit renderer version.
3. Provenance signature emitted for each export artifact.
4. Replay must reproduce byte-identical outputs when toolchain and dependencies are pinned.

## Veo/NanoBanana Scene Compiler Contract
1. `TimelineSpec -> scene plan -> provider prompt pack -> render jobs -> verification`.
2. Provider fallback order is policy-driven by quality tier and budget envelope.
3. All provider calls emit signed provenance and event records.
4. Non-deterministic providers require artifact fingerprint plus semantic-equivalence validator.

## Data/DB Additions
1. `artifact_ir(id, ir_type, schema_version, content_hash, payload_json, created_at)`
2. `artifact_builds(id, ir_id, idempotency_key, toolchain_version, status, created_at)`
3. `artifact_provenance(id, build_id, provider, model, signature, created_at)`

## Events
1. `artifact.ir.registered.v1`
2. `artifact.build.started.v1`
3. `artifact.build.completed.v1`
4. `artifact.provenance.attested.v1`
5. `artifact.replay.verified.v1`

## Acceptance Checks
1. Schema validation passes for all IR families.
2. Deterministic replay passes for pinned toolchain builds.
3. Fallback routing obeys policy tier constraints and budget caps.

## Related Specs

- `ARTIFACT_COMPILER_SPEC.md` — Full Artifact Compiler System specification including IR schemas, compiler pipeline, validation engine, headless execution model, and multi-format export
- `TECHNICAL_SPEC.md` — Venture-Autonomy Control Plane (artifact compiler is a subsystem)
- `API_EVENTS_SPEC.md` — Event topics and envelope format for artifact build completion, validation, and export events
