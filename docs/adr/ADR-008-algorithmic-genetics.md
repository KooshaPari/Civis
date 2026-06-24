# ADR-008: Genetics, Mutation, and Speciation Are Algorithmic (No LLM)

**Date:** 2026-05-22
**Status:** PROPOSED
**Author:** Civis 3D Extension

---

## Context

The Civis 3D extension introduces **multi-species + algorithmic genetics**
(WorldBox + Dinosaur-sim lineage). Civilians, fauna, flora, and special creatures all
carry DNA. The owner has stated explicitly that the **how-progression-happens** model
splits cleanly:

- LLM is appropriate for novel tech / lore / faction-diplomacy chatter (creative leaps
  with grounded validation).
- Algorithm is appropriate for genetics + speciation (selection pressures, mutation,
  recombination — well-understood and naturally deterministic).

Mixing LLM into the genetic loop would defeat replay (ADR-004, ADR-006) and add little
value: emergent species drift is already rich under classical genetic-algorithm dynamics.

## Decision

All genetics and speciation runs through **pure algorithmic code** in `crates/genetics`
and `crates/species`. No LLM in the genetic loop.

Core model:

1. **DNA is a fixed-length byte vector** per organism class (humanoid, quadruped,
   silicate, etc.). Class membership is data-driven; new classes do not require code
   changes.
2. **Mutation** is point + indel + recombination, parameterised per class, seeded from
   `civ-engine`'s `ChaCha8Rng`.
3. **Fitness** is computed per environment from a deterministic phenotype expression
   (`crates/species`) that maps DNA → morphology + behavior weights.
4. **Speciation triggers** when reproductive isolation (gene-flow rate * compatibility
   threshold) crosses a configured cutoff. The trigger spawns a new species record;
   the existing record continues unchanged.
5. **Cultural / wardrobe / tools drift** is layered on top via `crates/diffusion`
   (Bass/Rogers S-curve) — LLM is permitted there only for *flavor text* (names,
   descriptions), never for the propagation mechanics themselves.

## Consequences

- **Replay-safe by construction** — genetics never breaks `.civreplay` (ADR-004).
- **Modder-friendly** — DNA layouts, mutation rates, fitness functions are data-driven
  (RON), so mods add classes without code changes.
- **Predictable cost** — no LLM tokens spent on every reproductive event.
- **Visual species drift is emergent** — paired with the phenotype mapping in
  `crates/species`, organisms slowly *look* different as DNA drifts, which feeds the
  3D rendering layer.

## Alternatives Considered

- **LLM-proposes-mutations.** Adds nothing the classical GA can't do, breaks replay,
  costs tokens. Rejected.
- **LLM-narrates-evolution-events.** Acceptable as a *post-hoc lore overlay* gated
  through ADR-006 (cached events) — but the mechanics underneath stay algorithmic.
  This refinement may land later as a `civ-research`-adjacent feature.

## Cross-references

- ADR-004 (deterministic replay) — protected by this decision.
- ADR-006 (LLM event sourcing) — defines where LLM IS used; this ADR explicitly carves
  genetics out of that surface.
- `crates/genetics`, `crates/species`, `crates/diffusion`.
