# ADR-006: LLM Outputs as Hash-Keyed Cached Events for Replay-Safe Hybrid Progression

**Date:** 2026-05-22
**Status:** PROPOSED
**Author:** Civis 3D Extension

---

## Context

The Civis 3D extension introduces a **hybrid deterministic + LLM-driven progression
model**. Three modes are exposed per-save:

- **Canonical** — historical tech tree only, no LLM, fully deterministic.
- **Hybrid (default)** — canonical tree as backbone; LLM proposes side-tech branches
  when in-game research teams meet prerequisites and stall on the canonical path.
- **Free** — LLM may propose alt-physics and alt-biology, still gated by the physics-law
  DB (`civ-laws`, see ADR — pending).

LLMs are non-deterministic by nature. ADR-004 (deterministic replay) requires that any
`.civreplay` produces bit-identical state. Without intervention, an LLM call breaks
this guarantee.

## Decision

Treat every LLM invocation as an **event-sourced, hash-keyed cached event** with the
following on-the-wire shape:

```rust
struct LlmEvent {
    seed: u64,
    prompt_hash: [u8; 32],          // blake3 of prompt template + variables
    model_id: String,
    model_version: String,
    input_snapshot_hash: [u8; 32],  // blake3 of snapshot region the call observed
    output_hash: [u8; 32],          // blake3 of serialized output
    output: LlmOutput,              // the validated, schema-typed result
    tick: u64,
}
```

**Replay rules:**

- **Canonical mode:** any `LlmEvent` in the event log is treated as a corruption
  marker. Replay refuses to advance until a canonical replacement (deterministic
  fallback) is supplied by the engine.
- **Hybrid / Free mode:** during replay, the cache is keyed on
  `(prompt_hash, input_snapshot_hash, model_id, model_version)`. A hit reuses
  `output` verbatim — no live LLM call. A miss in replay is a hard error; replay
  refuses to advance until the original log is restored.
- **Live runs:** cache hits short-circuit. Cache misses trigger a real LLM call,
  result is validated against the typed schema (`crates/research`), and the resulting
  `LlmEvent` is appended to the event log.

**Validation gating** — every LLM-proposed tech card must declare
`{inputs, energy_cost, byproducts, dependencies}` and be validated against the
versioned physics-law DB (`civ-laws`) before becoming canon. Free mode permits typed
extensions (`kind: fictional_extension`), but extensions must still expose measurable
inputs/outputs/losses/dependencies.

## Consequences

- **Replay determinism preserved** under all three modes; Canonical never advances on
  a stochastic event, Hybrid/Free advance only when the cache resolves bit-identically.
- **Audit surface extends to LLM** — every model+version+prompt is recorded.
- **Cost** — the event log carries LLM outputs verbatim. For long campaigns this could
  grow large; out-of-band blob storage for `output` keyed by `output_hash` is allowed
  (the log only needs the hash for canonicity).
- **Schema versioning required** — `model_id`/`model_version` are first-class so
  replays of older saves can detect provider drift and fall back to Canonical mode.

## Alternatives Considered

- **No LLM (Canonical only).** Eliminates the problem but discards the emergent novelty
  the owner explicitly wants.
- **Live LLM without caching.** Trivially breaks ADR-004.
- **PRNG-seeded local model only.** Possible (e.g., temperature 0 + fixed seed) but
  brittle across hosted-API provider updates; cache-by-hash is robust.

## Cross-references

- ADR-004 (deterministic replay) — this ADR extends its scope to LLM events.
- `civ-research` crate — owns the cache + validator + replay-refusal logic.
- `civ-laws` crate — physics-law DB consumed by validation.
- Plan: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md` § Hybrid deterministic + LLM progression.
