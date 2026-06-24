# Deferred domain crates (not on 3D critical path)

**Maturity audit:** [DX-06 / DX-07](../development-guide/fr-ax-dx-ux-maturity-audit.md)
**3D attach path:** Godot / Bevy / Unreal / web dashboard — see [client-attach-matrix.md](client-attach-matrix.md)

These crates support **L3–L4 strategy depth** (genetics, laws, research, diffusion) per [plan-3d-phases.md](../roadmap/plan-3d-phases.md). They are **not** required to ship the current **L1–L2** loop: civ-server WS attach, civ-watch terrain, `sim.snapshot` / `civ_pins`, spawn palette, and F3D0 throttle.

Finish **P0–P1** attach and protocol parity before expanding vertical slices here.

---

## DX-06 — Research (`civ-research`)

| Item | Detail |
|------|--------|
| **Crate** | [`crates/research/`](../../crates/research/) (`civ-research`) |
| **ADR** | [ADR-006 — LLM event sourcing](../adr/ADR-006-llm-event-sourcing.md) |
| **Phase** | P-R1 (hybrid research; depends on `civ-laws` + agents) |
| **Status** | Validator + card graph stubs; **not** wired into scenario load or `civis-3d-verify` gate |
| **Tests** | `cargo test -p civ-research` |
| **README** | [`crates/research/README.md`](../../crates/research/README.md) |

**Defer until:** laws DB and agent tick expose stable hooks for proposal → validation → canon.

---

## DX-07 — Domain stubs (genetics, laws, species, diffusion)

Pure-Rust domain logic with schema + unit tests. Engine may call diffusion during tick ([`crates/engine`](../../crates/engine/)), but **no client** needs these crates for terrain, pins, or spawn UX today.

| Crate | Path | FR namespace | Phase | Key tests / entry |
|-------|------|--------------|-------|-------------------|
| **civ-genetics** | [`crates/genetics/`](../../crates/genetics/) | `FR-CIV-GENETICS-*` | P-G1 | `genetics::mutation_deterministic`, `genetics::speciation_trigger` |
| **civ-species** | [`crates/species/`](../../crates/species/) | `FR-CIV-SPECIES-*` | P-G1 | `species::phenotype_deterministic` — uses `civ_genetics::Dna` |
| **civ-laws** | [`crates/laws/`](../../crates/laws/) | `FR-CIV-LAWS-*` | P-L1 | `laws::ron_roundtrip`, `laws::validator_rejects_incomplete` |
| **civ-diffusion** | [`crates/diffusion/`](../../crates/diffusion/) | `FR-CIV-DIFFUSION-*` | P-A1 | `diffusion::s_curve_adoption` — wardrobe/tools S-curve |
| **civ-agents** | [`crates/agents/`](../../crates/agents/) | `FR-CIV-AGENTS-*` | P-A1 | Composes genetics + species + diffusion (agent wardrobe/tools) |

**Traceability:** [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md) (rows for genetics, species, laws, diffusion).
**Spec addendum:** [`docs/development-guide/fr-3d-additions.md`](../development-guide/fr-3d-additions.md).
**Genetics ADR:** [ADR-008](../adr/ADR-008-algorithmic-genetics.md).

**Defer until:** one vertical slice is chosen (e.g. spawn API exposes species id + wardrobe era on `civ_pins`) and a client renders it.

---

## What *is* on the 3D critical path

| Layer | Crates / clients |
|-------|------------------|
| Authority | `civ-server`, `civ-engine`, `civ-protocol`, `civ-protocol-3d` |
| Terrain / HTTP | `civ-watch`, `civ-voxel`, `civ-planet`, `civ-build` |
| Clients | `clients/godot-ref`, `clients/bevy-ref`, `clients/unreal-show`, `web/dashboard` |
| Agent smoke | `ws_smoke`, `agent-smoke.ps1` — see [agent-smoke.md](agent-smoke.md) |

---

## Related

- [product-quality-ladder.md](../roadmap/product-quality-ladder.md) — L4 strategy vs L1–L2 attach
- [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) — crate inventory
- [fr-modding-roadmap.md](../development-guide/fr-modding-roadmap.md) — mod hooks (separate from DX-07)
