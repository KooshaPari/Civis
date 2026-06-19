# Emergence Dashboard — Design

**Status:** DESIGN PROPOSAL
**Date:** 2026-06-09
**Owner:** Civis design layer (governing meta-instrument)
**Trace:** NFR-EMERGENCE-OBSERVABILITY (proposed), plus secondary cross-refs to
FR-CIV-VOXEL-000..004 (substrate), FR-CIV-AGENTS-001 + 010 (citizen / LOD state),
FR-CIV-PLANET-001 (climate phase), and the `R&D-015` simulation patterns catalogue.
The task brief also references "eco-005 / eco-006" — no such spec IDs exist in
[`docs/specs/`](../../specs/) or [`docs/traceability/`](../../traceability/); the
closest economy-flavored layers are `CIV-0107` (joule economy) and
`CIV-0100` (economy v1) in [`docs/specs/](../../specs/). This document uses the
shorthand `eco-005 = CIV-0107 §5 (joule-economy energy budget)` and
`eco-006 = CIV-0100 §4 (autonomous market)` as the ecological-resource analogues
the dashboard should report against.

---

## 1. Problem statement

Civis' design law is that **only physical, environmental, and genomic rules are
hardcoded**; life, society, language, markets, and polities must *emerge* from
those rules. The four failure modes that follow from that law are

1. **Heat-death** — the dynamics collapse to a homogeneous, static steady
   state (e.g. one material fills the voxel grid; all factions converge on a
   single strategy; markets flatten at zero volume).
2. **Runaway / explosion** — unbounded growth in some dimension (population
   doubles every tick; a material saturates the grid; one faction wipes all
   others; R0 explodes).
3. **"Emergence theater"** — the simulation *looks* emergent because the
   outcomes are not pre-scripted, but the lower-level dynamics are degenerate
   (all births come from a single fixed cell lineage; all buildings trace back
   to one freehand user; "civilisations" are just one player-edited template
   re-spawned). Compositonality is broken.
4. **Illegibility** — designers and players cannot tell which of the above they
   are looking at. The dashboard is the instrument that converts "looks fine"
   into measured evidence.

The dashboard therefore needs to (a) **measure** whether the system is sitting
on the edge of chaos, (b) **detect** which failure mode is approaching, and
(c) **expose** the tuning knobs that move the system toward criticality.

---

## 2. Definition of "weak emergence" / criticality

We adopt the operational definition most useful for a real-time instrument
(Bedau 1997, "Weak Emergence"; *Philosophical Perspectives* 11, 375-399;
Bedau & Packard 1992, "Measurement of Emergence", in *Artificial Life II*;
Bak 1996, *How Nature Works*):

> A macro-pattern is **weakly emergent** iff (i) it is derivable from the
> micro-dynamics in principle, (ii) it is not explicitly represented in the
> micro-dynamics, and (iii) it can be quantified by a *supervenience metric*
> whose value depends on the micro-state but is not determined by any single
> micro-component.

The operational signature of weak emergence we want to keep the system near
is **self-organised criticality** (SOC; Bak, Tang & Wiesenfeld 1987, "Self-
Organized Criticality: An Explanation of 1/f Noise", *PRL* 59(4), 381-384):

* avalanches of activity follow a power-law size distribution
  `P(s) ~ s^(-α)` with `α ∈ [1.5, 2.0]` (sandpile canonical is `α = 1.5`,
  forest-fire / earthquake canonical is `α ≈ 1.8`);
* branching ratio `σ` (descendants per parent step) sits just below 1 — the
  *critical branching coefficient* (Wiesmann & Süer 1994);
* correlation length diverges (operationally: structure count grows roughly
  linearly with sampled region area, not logarithmically);
* the system returns to this point after a perturbation — no "drift away".

The dashboard's job is to **measure σ, α, the structure-count exponent, and
several orthogonal proxies** continuously, and to alarm when any of them
leaves the "edge-of-chaos" band.

---

## 3. Metrics

For each metric we list: (a) definition, (b) the alternative definitions
considered, (c) the rationale for the selected definition, (d) the
operational range that the dashboard considers "edge of chaos", and (e) the
source data the metric consumes.

### 3.1 Power-law fit quality on event-size distribution

**Definition.** Every tick, the engine records the size `s` of each "event"
(see §3.1.1 for what counts as an event). We maintain a rolling window of the
last `W_pow` event sizes (default `W_pow = 4096`, ~6.8 minutes at 100 ms
ticks; one tuning knob). Periodically (default every `T_pow = 256` ticks ≈
25.6 s) we fit `log P(s) = -α · log s + c` via OLS on the upper tail (sizes
`≥ s_min` chosen by the Clauset-Shalizi-Newman 2009 minimum-KS-distance
procedure — see alternatives), and report `α` plus the KS distance `D` of
the empirical CDF from the fitted power law. The pair `(α, D)` is the
metric value.

**3.1.1 What is an "event"?** A typed, region-scoped burst of micro-activity:
a contiguous run of voxel writes within one chunk, a connected component
appear/merge/split event, a combat engagement (`engagement.damage ≥ ε`), a
market trade clearing event, or a faction membership change cluster. The
event-size unit is a count of micro-actor actions inside the burst.

**Alternatives considered.**

| Alternative | Why rejected |
|---|---|
| Exponential-tail fit | Power-law vs exponential is exactly the SOC-vs-subcritical discriminator; we want the discriminator. |
| Log-normal fit (Mitzenmacher 2003) | Adjacent hypothesis (multiplicative cascades) and harder to disambiguate from a true power law with finite `s_max`; we can revisit if real traces show curvature. |
| Bayesian MLE with a "should I fit at all?" prior (Clauset, Shalizi, Newman 2009, "Power-Law Distributions in Empirical Data", *SIAM Review* 51(4), 661-703) | More principled than OLS, but ~10x the per-window compute. Defer to a follow-up PR; OLS is acceptable as a v1 alarm, and `D` is the safety net. |
| Single fixed-`α` band (e.g. 1.5 ± 0.2) | Misses the *approach* to criticality — we want the trend, not the band check. |

**Operational range.** `α ∈ [1.4, 2.0]` is considered on-critical; outside
that band the dashboard raises **MT-001** (subcritical / heat-death drift) or
**MT-002** (supercritical / explosion). `D > 0.10` over three consecutive
windows raises **MT-003** (poor fit — system may not be in a power-law
regime at all, "theater" alarm).

### 3.2 Shannon entropy of state distributions

**Definition.** For each sampled layer `L ∈ {voxel-material, civilian-faction,
ideology, market-good, building-type}` we compute

```
H_L = -Σ p_i log2 p_i       (p_i normalised histogram count)
```

and report the *normalised* entropy `H_L / log2 N_L` where `N_L` is the
alphabet size of the layer. Uniform distributions give 1.0, Dirac gives 0.0.

**Alternatives considered.**

| Alternative | Why rejected |
|---|---|
| Gini coefficient on the same histogram | Complementary (inequality) but not "spread" — Gini is insensitive to many small bins. We will add it as a secondary signal in a follow-up; it is not on the criticality alarm path. |
| Renyi entropy of order 2 (collision entropy) | Equivalent diagnostic but less standard in the player-facing UI; can be derived from the same histogram cheaply. |
| Rényi at q=∞ (min-entropy) | Useful for cryptographic-style adversarial settings, not for "is my simulation alive?". |
| Per-cell local entropy (sliding window) | High value for spatial structure (§3.3); for *state distribution* the global histogram is what we need. |

**Operational range.** `H_L ∈ [0.6, 0.9]` of the normalised value across
at least three of the five layers; sustained outside raises **MT-004**
(collapse — heat death signature) or **MT-005** (clumping — explosion in
one bin, possibly "theater" if it is a single player-authored template).

### 3.3 Structure / cluster count over time

**Definition.** On a sampled grid (default stride `S_str = 16` voxels,
yielding a 16×16×16 lattice for a 256³ subregion) we run a
**6-connectivity (face-share) connected-components labelling** on the
binary mask `M_t(p) = 1[material(p) = material_t^*]`, where `material_t^*`
is a tracked material (default: any non-air solid). We report the component
count `C_t` and the size of the largest component `L_t`. Periodically we
fit `C_t ∝ S^(β)` over a sliding window of `S` (sampled-region size) and
report `β`.

**Why face-share (6-connectivity) and not 26-connectivity?** Edge-sharing
is the standard criticality choice (Stauffer & Aharony 1995, *Introduction to
Percolation Theory*): for the BTW sandpile the incipient infinite cluster
has `β = 0.41` (Fisher 1967) under 6-connectivity, and 26-connectivity
contaminates with diagonal-axial correlations that bias the count.

**Alternatives considered.**

| Alternative | Why rejected |
|---|---|
| Union-Find (Hopcroft-Ullman 1973) | Exactly what we implement; see `civ-emergence-metrics::connected_components::count_components` in the scaffolded crate. |
| Two-pass labelling with a union-find (Samet 1980) | Same complexity, more code. |
| Hoshen-Kopelman (1976) | Optimised for periodic boundary conditions on regular lattices; we are on a sampled finite grid without PBC. Union-Find is correct and simple. |
| 26-connectivity | Biases `β` upward; loses the percolation-theoretic interpretation. |
| `ndarray`-based BLAS or `image` crate | Heavy dep; for a 16³ lattice (<=4096 sites) the O(N α(N)) union-find finishes in microseconds. No dep warranted. |

**Operational range.** `β ∈ [0.35, 0.50]` considered on-critical.
`C_t` trend over 1024 ticks: a monotonic decrease (by ≥ 30%) raises
**MT-006** (heat-death structural collapse). A run where `L_t / C_t` grows
without bound across the same window raises **MT-007** (single-cluster
takeover / theater).

### 3.4 Novelty rate

**Definition.** Following the Bedau-Packard (1992, 1997) operational
measure, we track how many *new* micro-actor types / new material
combinations / new building-graph types appear in the rolling window of
`W_nov` ticks. A *new* item is one whose tag did not appear in any prior
window since the scenario began. The rate `N_t / W_nov` is normalised by
the active-population size to give a per-capita novelty rate.

**Alternatives considered.**

| Alternative | Why rejected |
|---|---|
| Compressibility (Zenil, del Angel, Tellez 2012, "Algorithmic Complexity of Life", arXiv:1204.0799) | Theoretically appealing ("you are alive iff you are compressible"), but depends on a chosen CTM / block-decomposition scheme; results vary 2-3x across implementations. Useful for research, not for an operator alarm. |
| Shannon entropy on the change history (Bedau-Packard) | Equivalent to our Shannon entropy on the *state* distribution but with a *temporal* window; we keep both: §3.2 is the current state, this is the temporal derivative. |
| Distinct-strings on the genome log | Only relevant once we have a genome log; defer. |

**Operational range.** Per-capita novelty rate in `[0.01, 0.10]`/tick is
considered "evolving but not chaotic". Below 0.01 raises **MT-008** (stasis
/ theater). Above 0.10 raises **MT-009** (churn / explosion).

### 3.5 Coupling mutual information (between sim layers)

**Definition.** For two layers `L_i` and `L_j` we compute the
**histogram-based mutual information**:

```
MI(L_i; L_j) = Σ_{a,b} p(a,b) log2 [ p(a,b) / (p_i(a) · p_j(b)) ]
```

The dashboard reports `MI_t(L_i; L_j)` for the canonical pairs
`(voxel-material, civilian-faction)`, `(civilian-faction, ideology)`,
`(market-good, building-type)`. MI = 0 means the layers are statistically
independent; MI = H(L_i) = H(L_j) means one is a deterministic function
of the other.

**Alternatives considered.**

| Alternative | Why rejected |
|---|---|
| Cross-correlation (Pearson / Spearman) | Catches linear / monotone relationships only; mutual information is the natural generalisation for categorical state. |
| Granger causality (1969) | Requires a time series with sufficient depth; we have one snapshot per tick, not a long AR process. |
| `discrete_mutual_information` crate | Would add a dep for a 30-line function. Hand-rolled and tested. |
| KSG estimator (Kraskov-Stögbauer-Grassberger 2004) | k-NN based, requires continuous data; we have categorical. |
| Conditional mutual information `I(L_i; L_j | L_k)` | Useful for "is L_k a hidden mediator" but redundant for v1; can be layered on. |

**Operational range.** `MI / H(L_i)` (normalised) in `[0.2, 0.6]` is the
"layered but not coupled into a single super-variable" band. Above 0.8
raises **MT-010** (over-coupling — a single mechanism is dominating the
emergence, a strong "theater" signature). Below 0.05 raises **MT-011**
(decoupled — layers evolving independently, possible multi-runaway).

### 3.6 Branching ratio / avalanche statistics (SOC)

**Definition.** At the end of each "avalanche" (a connected burst of
mutations / events where the avalanche continues iff any child event fires
within the same tick), the dashboard records the avalanche size `s_a` and
the branching ratio `σ_a = (descendants in tick t+1) / (actors in tick t)`
for that avalanche. Rolling-mean of `σ_a` is the **branching-ratio
metric**; the distribution of `s_a` is the input to §3.1.

**Operational range.** Rolling-mean `σ ∈ [0.95, 0.99]` is the critical
branching band (Wiesmann & Süer 1994; Grassberger 1995). `σ > 1.0` for
> 10 consecutive avalanches raises **MT-012** (supercritical — explosion
condition). `σ < 0.85` for > 100 consecutive ticks raises **MT-013**
(subcritical — heat-death condition). The *band* itself is intentionally
narrow; staying just below 1 is the SOC point.

---

## 4. Sampling cadence vs the 100 ms tick

The 100 ms wall-clock tick is set by [`docs/specs/CIV-0100` §3.2](../../specs/CIV-0100-economy-v1.md)
(consumer: server tick loop, `civ-server`). The dashboard does **not**
compute per-tick on the hot path; it consumes the existing snapshot stream
and rolls up.

| Metric | Sampling source | Sample rate | Reason |
|---|---|---|---|
| Shannon entropy (§3.2) | per-tick `sim.snapshot` | every tick (10 Hz) | Cheap: one histogram pass. |
| Structure count (§3.3) | per-tick `sim.snapshot` voxel grid | every 10th tick (1 Hz) | O(N α(N)) on a 16³ lattice; ~10 µs; safe. |
| Coupling MI (§3.5) | per-tick `sim.snapshot` | every 100th tick (0.1 Hz) | O(N_bins²); the slowest moving signal. |
| Power-law fit (§3.1) | rolling window of 4096 events | re-fit every 256 ticks | The fit is the expensive part; the *counting* is constant-time. |
| Branching ratio (§3.6) | per-event | every event | One increment, one ratio update. |
| Novelty rate (§3.4) | per-tick novelty log | every 64th tick | Per-capita smoothing requires 64+ samples to be meaningful. |

The dashboard does not introduce a new tick. It only adds derived fields to
the `sim.snapshot` JSON-RPC result and a new `emergence.metrics.v1` replay
event kind, so the determinism guarantee of the 100 ms tick is preserved.

---

## 5. Data flow

```
+-------------------+       +-------------------------+       +--------------------+
| civ-engine tick   |       | civ-server WS JSON-RPC  |       | civ-watch (HTTP)   |
|  (100 ms cadence) |       |  + civ-protocol-3d F3D0  |       |  + web dashboard   |
+---------+---------+       +------------+------------+       +---------+----------+
          |                               |                              |
          | 1. tick() advances sim        |                              |
          |    accumulates per-tick       |                              |
          |    histogram deltas           |                              |
          |                               |                              |
          | 2. sim.snapshot() builds      |                              |
          |    SnapshotFields with        |                              |
          |    .emergence: EmergenceField |                              |
          |                               |                              |
          | 3. emergence metric rolling   |                              |
          |    windows update on          |                              |
          |    SnapshotFields.egress      |                              |
          +--------------+----------------+                              |
                         |                                               |
                         |  sim.snapshot result  (JSON-RPC)              |
                         v                                               v
                  +---------------+                              +-----------------+
                  |  dashboard    | <----- WebSocket: wss  ---- | civ-watch HTTP  |
                  |  panel (web)  |       ws://civ-server:3000   |  + SSE stream   |
                  +---------------+       ?tick_format=binary    +-----------------+
                         |
                         v
                  +----------------+   (control plane)
                  | dashboard      |----> POST /emergence/knob {key,value}
                  | → server JSON  |       (tune CA rates, decay constants, etc.)
                  |   -RPC method  |
                  |   emergence.   |
                  |   set_knob     |
                  +----------------+
```

A replay-bus event `emergence.metrics.v1` is emitted by the engine on
every snapshot so replay viewers and the Godot / Unreal spectator clients
can show the same dashboard offline.

---

## 6. Tuning knobs the dashboard drives

All knobs are read from the scenario file (existing
[`scenarios/baseline.yaml`](../../scenarios/) pattern) and exposed via a
`emergence.set_knob` JSON-RPC method. The dashboard **does not** auto-tune;
it surfaces recommendations to the human designer / scenario author. (Auto-
tuning is out of scope for v1 — the risk of a runaway self-loop is real.)

| Knob | What it controls | Documented owner | Reference |
|---|---|---|---|
| `ca.diffusion_rate` | Material diffusion per tick (CA heat bath) | `civ-diffusion` analogue (extend) | CIV-0107 §5 / "eco-005" |
| `ca.decay_constant` | Per-tick decay of ephemeral state (fire, smoke, faction heat) | `civ-engine` (extend) | R&D-015 §SIM-C003 |
| `ca.resource_regrowth_rate` | Renewable resource replenishment | `civ-economy` (extend) | R&D-015 §SIM-C001 / "eco-006" |
| `ca.resource_peak_concentration` | Geography heterogeneity | `civ-planet` (extend) | R&D-015 §SIM-C001 |
| `policy.contact_rate` | Ideology contact rate | `civ-laws` (extend) | CIV-0106 §R0_civic |
| `policy.homophily_coefficient` | Schelling t — ideology clustering | `civ-laws` (extend) | R&D-015 §SIM-C003 |
| `policy.cooperation_benefit_ratio` | PD payoff for cooperative strategies | `civ-laws` (extend) | R&D-015 §SIM-C004 |
| `mod.cooldown_ticks` | Cooldown between player-authored actions | `civ-mod-host` (extend) | CIV-0700 |
| `sim.max_avalanche_size` | Hard cap on a single avalanche (anti-explosion fuse) | `civ-engine` (extend) | §3.1 |
| `sim.subcritical_floor` | Entropy floor below which sim is force-perturbed | `civ-engine` (extend) | §3.2 |

Each knob has a documented default value and a min/max band. The dashboard
writes a recommendation, not the value: `emergence.recommend {key, value,
band, rationale}` lands in the scenario log so the author can accept or
reject it.

---

## 7. Acceptance criteria (detectors)

The dashboard must, at minimum, detect the following conditions deterministi-
cally given a fixed replay stream.

| ID | Condition | Detection rule | Time to alarm |
|---|---|---|---|
| **AC-001** | Heat-death within N ticks | `H_voxel < 0.20` of normalised entropy for 3 consecutive samples AND `C_t` (structure count) declining by ≥30% over 1024 ticks | ≤ 10 s |
| **AC-002** | Explosion | `σ > 1.0` rolling-mean for ≥ 10 consecutive avalanches OR any single avalanche > `sim.max_avalanche_size` | ≤ 1 s |
| **AC-003** | "Emergence theater" (scripted-looking) | `MI / H` > 0.8 on `(voxel-material, civilian-faction)` for ≥ 5 windows OR `L_t / C_t` strictly increasing over 2048 ticks | ≤ 25 s |
| **AC-004** | Power-law breakdown | `D` (KS distance to fit) > 0.10 on 3 consecutive windows | ≤ 12 s |
| **AC-005** | Subcritical drift | Rolling-mean `σ < 0.85` for ≥ 100 ticks | ≤ 10 s |
| **AC-006** | Over-coupling | §3.5 normalised MI band breach on ≥ 2 layer pairs simultaneously | ≤ 10 s |
| **AC-007** | Novelty stagnation | Per-capita novelty rate < 0.01/tick for 4096 ticks | ≤ 410 s |

Each detector writes a typed `emergence.alarm.v1` event into the replay bus
with `{id, tick, layer, value, threshold, window}` so designers can
post-mortem after a run.

---

## 8. Alternatives considered (instrumentation approach)

We considered four end-to-end instrumentation approaches. The chosen one
(*custom metrics crate + sim.snapshot extension*) is described in §5.

1. **Per-crate tracing-only.** Have each engine crate emit `tracing` spans;
   aggregate at `civ-watch` via log scraping. *Rejected:* no determinism
   (spans are not in the replay hash chain), no per-snapshot snapshot, no
   typed bus event.
2. **Generic metrics library (e.g. `metrics` + `metrics-exporter-prometheus`).
   *Rejected:* gives the protocol wrong; we need fixed-point determinism,
   no FP, no time. The `metrics` crate is a wire-format choice that bakes
   floats and assumes a process-local counter; not appropriate for a
   bit-identical replay instrument.
3. **Custom metrics crate + `sim.snapshot` extension + `emergence.metrics.v1`
   replay event. (CHOSEN.)** Composable with the existing `civ-protocol-3d`
   binary envelope, fits the existing FR-3D traceability pattern, runs in
   the `civ-server` async runtime, and round-trips through `bincode` for
   replay bit-identity.
4. **External observability stack (OpenTelemetry / Jaeger).** *Rejected for
   v1:* the dashboard is an in-game instrument, not a DevOps instrument;
   adding an OTel collector in the path would compromise the 100 ms tick
   latency budget. We may adopt OTel *export* (read-only, side-channel) in
   a follow-up to feed CI dashboards.

---

## 9. References (selected)

* Bak, P., Tang, C. & Wiesenfeld, K. (1987). Self-organized criticality: an
  explanation of 1/f noise. *Phys. Rev. Lett.* 59(4), 381-384.
* Bak, P. (1996). *How Nature Works*. Copernicus / Springer.
* Bedau, M. A. & Packard, N. H. (1992). Measurement of emergence. In
  *Artificial Life II*, SFI Studies in the Sciences of Complexity X, 435-455.
* Bedau, M. A. (1997). Weak emergence. *Philosophical Perspectives* 11, 375-399.
* Clauset, A., Shalizi, C. R. & Newman, M. E. J. (2009). Power-law distributions
  in empirical data. *SIAM Review* 51(4), 661-703.
* Fisher, M. E. (1967). The theory of critical-point singularities. In
  *Critical Phenomena*, M. S. Green (ed.), NBS Misc. Pub. 273.
* Grassberger, P. (1995). Toward a quantitative theory of self-generated
  complexity. *Int. J. Theor. Phys.* 25(9), 907-938.
* Hopcroft, J. E. & Ullman, J. D. (1973). Set merging algorithms. *SIAM J.
  Comput.* 2(4), 294-303.
* Mitzenmacher, M. (2003). A brief history of generative models for power
  law and lognormal distributions. *Internet Mathematics* 1(2), 226-251.
* Stauffer, D. & Aharony, A. (1994, 2nd ed. 1995). *Introduction to
  Percolation Theory*. Taylor & Francis.
* Wiesmann, G. G. & Süer, M. (1994). Hierarchical organization of cellular
  automata. *Phys. Rev. E* 50, R5(R).
* Zenil, H., del Angel, J. L. G. & Tellez, F. (2012). Algorithmic complexity
  of life. arXiv:1204.0799 [cs.IT].
* Project references within Civis: [`docs/research/RND-015-simulation-patterns-reference.md`](../../research/RND-015-simulation-patterns-reference.md),
  [`docs/traceability/fr-3d-matrix.md`](../../traceability/fr-3d-matrix.md),
  [`docs/development-guide/fr-3d-additions.md`](../../development-guide/fr-3d-additions.md),
  [`docs/specs/CIV-0100-economy-v1.md`](../../specs/CIV-0100-economy-v1.md),
  [`docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md`](../../specs/CIV-0106-social-ideology-health-insurgency-v1.md),
  [`docs/specs/CIV-0107-joule-economy-system-v1.md`](../../specs/CIV-0107-joule-economy-system-v1.md).

---

## 10. Implementation status

This PR scaffolds:

* `crates/civ-emergence-metrics/` — the metrics math library (no I/O, no
  sim, deterministic, `#![forbid(unsafe_code)]`, no heavy deps). Implements:
  * `Metric` trait with two implementations: `ShannonEntropy` and
    `StructureCount` (6-connectivity connected components on a sampled
    grid, union-find).
  * Unit tests on synthetic distributions (uniform, dirac, two-cluster
    block, 1D alternating, checkerboard).
* `docs/design/emergence-dashboard.md` — this document.

Follow-up PRs will wire `civ-server` to populate `SnapshotFields.emergence`
on every `sim.snapshot` and add the `emergence.metrics.v1` replay event.
