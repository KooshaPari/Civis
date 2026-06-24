# Religion Emergence — Norenzayan Big-Gods as a Gradient-Coupled Substrate

> **Status:** Design spec (research / design-only, no code changes in this PR).
> Owner: Emergence design.
> **Governing constraints:**
> [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) (only
> physical/environmental/genomic laws are authored; everything else EMERGES) and
> [`docs/design/PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md)
> (the only coupling channel is the six-field substrate `{T, M, E, F, P, B}` —
> inter-layer API calls are prohibited).
>
> **Companions (do not duplicate):**
> - [`docs/adr/ADR-016-religion-emergence-from-needs-vector.md`](../adr/ADR-016-religion-emergence-from-needs-vector.md)
>   decides *what* religion couples to (needs vector + group size, Norenzayan
>   Big Gods framing). This spec is the concrete *mechanism*: which fields, which
>   gradients, which per-tick caps, which invariants, how it shows up in
>   `crates/engine/src/religion.rs`, how `civ-laws` consumes it.
> - [`docs/adr/ADR-018-emergence-systems-coupling.md`](../adr/ADR-018-emergence-systems-coupling.md)
>   rows 9–11 (religion → cohesion / ritual cohesion / sentience-crossings pulse)
>   and [`docs/adr/ADR-011-n-series-emergence-coupling.md`](../adr/ADR-011-n-series-emergence-coupling.md)
>   contract. This spec adds the row and tightens its bounds; it does **not**
>   replace the contract.
> - [`docs/design/PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md)
>   §3.1 row 5 (`religion × {T, M, E, F, B}`) and §4.4 (the rain-dance
>   ritual example). This spec is the *religion-side* implementation of that row.
> - [`docs/design/legends-engine.md`](legends-engine.md) §3.4 — the
> `EventKind::IdeologyShift` / `CulturalSpeciation` row already says religion
> feeds the saga graph as a producer. We obey that contract here.

---

## 0. What this spec fixes

The current `crates/engine/src/religion.rs:1-158` is **partially emergent but
authored at the type level**. The substrate decision (`BeliefConcept`) is an
enum with five named variants:

```
crates/engine/src/religion.rs:4-10
pub enum BeliefConcept {
    NaturalAgent,
    MoralOverseer,
    Afterlife,
    Taboo { action: String },
    Ritual { cost: f32 },
}
```

This is exactly the *authored pantheon* the emergence charter forbids:

1. **Authored theology.** `NaturalAgent` / `MoralOverseer` / `Afterlife` are
   hand-named doctrinal *kinds* — the model ships with a thesis about which
   religious *forms* exist. A culture that produces a sun-worshipping ancestor
   cult would have to be retrofitted as a new enum variant; that is doctrine by
   enumeration, not by emergence.
2. **Bool-gated emergence.** `emerge_belief` (lines 42–67) maps the
   `hardship: f32` scalar to a `BeliefConcept` through if/else branches
   (`group_size > 50 → MoralOverseer`, `hardship > 0.7 && bias >= 0.5 →
   NaturalAgent`). The concept assignment is a hard switch on scalar
   thresholds, not a continuous gradient; two populations with
   `hardship = 0.69` and `hardship = 0.71` produce different religions despite
   identical conditions. That is the **theatre** failure mode
   (`PHYSICS_COUPLING_SUBSTRATE.md` §5.3) at the model layer.
3. **Abstract scalar inputs.** The function signature is
   `emerge_belief(hardship: f32, group_size: u32, agent_detection_bias: f32)`.
   `hardship` and `agent_detection_bias` are hand-plumbed by the engine (see
   `PHYSICS_COUPLING_SUBSTRATE.md` §1.1 row "religion" — "abstract"). There is
   no path from the substrate's `|∇T|`, `|∇B|`, or `|∇P|` to those scalars.
4. **No coupling-down path.** `spread_religion` (lines 69–98) is
   *self-contained*: it reads its own `Religion` struct and mutates
   `rel.cohesion`. It does not write to `PhysicsFields` (no `E` from ritual
   fires, no `F` from burnt offerings). The substrate never knows religion
   happened.

This spec replaces all four with a **gradient-coupled, mechanism-only** model:
no named deities, no doctrinal enum, continuous gradients, substrate writes
through the `civ-physics-substrate` setter, per-tick caps, and a hook into
`civ-laws` so religious emergence mints *cohesion + law compliance* rather than
*cohesion + narrative*.

---

## 1. Charter alignment

| Charter rule | How this spec obeys it |
|---|---|
| *Model the rule, not the outcome.* | The religion model emits **cohesion deltas, law-compliance deltas, and substrate writes**; it authors *zero* theological content. There is no `BeliefConcept` enum in the spec. |
| Everything that emerges has continuous state. | Religion state is a vector of three real-valued scalars (`monitoring`, `mythic_coherence`, `uncertainty_reduction`) in `[0, 1]` — no booleans, no enum. |
| Only the substrate mediates between layers. | Religion reads `{∇T, ∇M, ∇B, ∇P}` and writes `{E, F, P, B}` through `PhysicsFields::set` typed setters. No `religion.notify(faction)`. |
| Determinism not required (charter §"Determinism NOT a requirement"). | This spec uses floats and `thread_rng` where it serves richer emergence (cultural drift of `mythic_coherence`). Save/load persists state, not replay. |
| Loud, not silent (repo CLAUDE.md). | Every coupling tripwire (cap exceeded, substrate write refused, deterministic-drift mismatch) logs a named warning. |

---

## 2. The three religion scalars (state, not theology)

A religion is not a set of beliefs; it is a **measured profile of three
continuous scalars** describing what the religion is *doing* in its
population. The triple is the only religion state the engine writes; legends,
inspector, and inspector tooltips read it directly.

```
struct ReligiousProfile {
    monitoring:        f32,  // [0, 1]  Big-Gods: how much a population watches / sanctions / punishes its members.
                              //          High monitoring = "Moralizing High God" regime in Norenzayan's terms.
                              //          Rises with group_size and |∇unrest|; falls with kinship density.
    mythic_coherence:  f32,  // [0, 1]  Internal narrative integration: shared stories / myths / ritual canon.
                              //          High coherence = "afterlife + moral accounting + ancestor cult" regime.
                              //          Rises with sustained monitoring + low migration; frays on contact.
    uncertainty_reduction: f32,  // [0, 1]  Anxiety-quotient: how much of the population's experienced uncertainty
                                  //          the religion absorbs. This is the *demand* side of the supply/demand
                                  //          loop: the more uncertainty, the more "relief" a religion provides.
                                  //          Returns toward zero when the underlying driver (unrest) falls.
    age_ticks:         u64,  // monotonically increasing; one per phase_emergence call.
    population:        u32,  // member count (cluster size); bounded by the same `MIN_AGENTS = 2` rule as
                              // other cluster reads (avoids lone wanderers driving religion).
    last_drift_seed:   u64,  // blake3 of (cluster_id, age_ticks) → re-seeded for non-deterministic mythic
                              // drift on save/load churn. Determinism not required; this is for replays
                              // of a *given saved state*, not bit-identical reseed.
}
```

Three scalars, not five. They are **independent axes of one emergent
phenomenon**, chosen so that:

- a low-monitoring / low-coherence / high-uncertainty-reduction profile is the
  Norenzayan **animist / shamanic** form (private coping rituals, episodic);
- a high-monitoring / high-coherence / low-uncertainty-reduction profile is the
  Norenzayan **Big-Gods** form (moralizing, omnipresent, norm-enforcing);
- the rest is a continuum between, with named regimes as inspector tooltips
  (read-only labels derived from the triple — never authored as the model's
  primary state).

A religion that wants a "sun god" must emerge *through* these three scalars;
the inspector tooltip can then derive a *display label* like "moralizing sun
cult" from `monitoring = 0.8, mythic_coherence = 0.6, dominant resource =
climate`. That label is **derived**, not stored. Storing it would be authoring
theology.

> **Why not more scalars?** Each additional axis would have to justify itself
> as a continuous gradient read from the substrate (per ADR-011 "shared
> gradient, not API call"). The three we have cover the three substrate
> sources religion actually reads: human monitoring pressure (∇P),
> integration under stress (∇T, ∇B), and uncertainty absorption (unrest).
> Adding more axes (e.g. `sacrificial_intensity`, `doctrinal_literalism`)
> without a substrate gradient to drive them would be authoring content by
> another name. We will *not* add them.

---

## 3. The five substrates religion couples to

Per `PHYSICS_COUPLING_SUBSTRATE.md` §3.1 row 5, religion's coupled-fields set
is `{T, M, E, F, B}` (5 cells of the 40-cell matrix). This spec also relies on
**reads** of `P` (population pressure) for group-size, because `P` carries
the contact/kinship/cluster signal that Norenzayan's group-size term depends
on. Read-only consumers do not count against the matrix count (§3.2), but
they do need to be enumerated for traceability.

### 3.1 Reads

| Field | What religion reads it for | Bounded by |
|---|---|---|
| **∇T** (temperature gradient) | Climate-stress: cold snaps, heat waves, monsoon shifts. Drives *mythic_coherence* (people integrate stories under environmental volatility). | `ClimateParams::co2_sensitivity` × `feedback_factor` (substrate invariant, not ours). |
| **∇M** (moisture gradient) | Famine pressure. Drives *uncertainty_reduction* demand (rain-call rituals). | `SEA_LEVEL_SENSITIVITY_M_PER_C` (substrate invariant). |
| **∇B** (biomass gradient) | Local scarcity → "hardship" signal Norenzayan needs. Drives *monitoring* (food-sharing rules become moralized). | Biomass regrowth rate × carrying capacity (substrate invariant). |
| **∇P** (population pressure gradient) | Group-size + contact: dense populations generate more monitoring because detection is cheaper. | `MIN_AGENTS = 2` guard (existing engine constant); per-cluster `member_count` scan. |
| **unrest** (macro scalar on `Simulation`) | Aggregate societal uncertainty — *not* a substrate field, but the macro social-conservation law (ADR-018 row 27/28). Feeds *uncertainty_reduction* demand. | `MAX_MISERY_UNREST = 30`; `MAX_RISE = 15`; `DECAY = 5`. |

> Note on "unrest": the substrate has no `unrest` field by design — unrest is
> the macro conservation law that aggregates the substrate's effects on agents
> (ADR-018 row 17 `agent_misery_unrest`). Religion reads it as a derived
> scalar, not as a substrate field, because unrest is *the* conserved
> social-state quantity the substrate sum collapses to.

### 3.2 Writes

| Field | What religion writes | Rate cap (per religion, per tick) | Enforcement |
|---|---|---|---|
| **E** (energy) | Ritual fires (offerings, vigils, cremation) emit joules into local cells. | `MAX_RITUAL_E_PER_TICK = 50_000 J` (config) | Substrate setter refuses writes above the cap with `physics: ritual write exceeds MAX_RITUAL_E_PER_TICK`. |
| **F** (material flux) | Burnt offerings, feast waste, incense smoke (carries carbon aerosols → microclimate). | `MAX_RITUAL_F_PER_TICK = 1.0 kg/cell/tick` | Same. |
| **B** (biomass) | Sacrifice / burnt offerings consume local biomass; tithe-gathered food is *redirected*, not destroyed (substrate sees the redirected flux, not the religion's intent). | `MAX_RITUAL_B_PER_TICK = 0.5 kg/cell/tick` | Same. |
| **P** (population pressure) | Pilgrimage, exile, martyrdom shifts population gradients. | `MAX_RITUAL_P_PER_TICK = 10 agents/cell/tick` | Substrate setter; same refusal path. |
| **belief** (macro scalar on `Simulation`) | Each successful ritual mints a bounded `belief` increment (the macro-faith budget, ADR-018 row 9 "Religion / Belief → Cohesion" — `add_belief` is `saturating_add`). | `MAX_RITUAL_BELIEF_PER_TICK = 5` (per religion) | `add_belief` already saturates; cap is upstream of the writer, not at the substrate. |
| **cohesion** (macro scalar on `Simulation`) | Sustained monitoring + shared ritual mints bounded cohesion. Existing `cohesion_delta(belief, unrest)` (engine.rs:2698) absorbs this. | `COHESION_PER_RITUAL = 1`, bounded by per-tick `MAX_RITUAL_COHESION_PER_TICK = 8` (new const). | Existing saturation in `add_cohesion`; new const enforces upstream. |
| **law.compliance** (per-faction) | Religious rule-keepers update `law.compliance[ReligionTaboo::Action(t)]` in `civ-laws`. This is *not* a substrate write — it is the religion's hook into `civ-laws` (see §6). | `MAX_LAW_COMPLIANCE_DELTA_PER_TICK = 0.05` | `civ-laws` already rate-limits compliance updates. |

The four substrate writes + three macro writes are **the only** outward
effects a religion has. There is no `religion.broadcast()`. The substrate
diffuses the writes; downstream layers (`civ-economy`, `civ-climate`,
`civ-tactics`) read the *consequences* (`B` fell, `T` rose near a ritual
site, `P` shifted to a pilgrimage cell) — never a religion event object.

---

## 4. The per-tick mechanism

The religion mechanism runs **once per `phase_emergence`** tick, after
`phase_emergence_culture` and before `phase_emergence_social` (so cluster
membership is current but social graph updates still see the religion's writes
through the substrate's next-tick lag — the interleave pattern from
`PHYSICS_COUPLING_SUBSTRATE.md` §4.1).

```
fn tick_religion(world: &hecs::World, fields: &mut PhysicsFields,
                 sim: &mut Simulation, tick: u64) -> ReligionTickReport {
    // 1. Sample substrate gradients at each religion's centroid.
    let religions = scan_religion_clusters(world); // BTreeMap<ClusterId, ReligiousProfile>
    let gradients = sample_substrate_gradients(fields, &religions);

    // 2. Apply the Big-Gods response curve (per religion, per axis).
    for (cluster_id, profile) in religions.iter_mut() {
        let g = &gradients[cluster_id];
        apply_big_gods_response(profile, g, tick);

        // 3. Cap all per-tick deltas (the invariant).
        profile.enforce_caps();
    }

    // 4. Convert response into bounded substrate + macro writes.
    let writes = plan_substrate_writes(&religions, &gradients);
    let macro_deltas = plan_macro_writes(&religions, &gradients);

    // 5. Apply writes through the typed substrate setters + macro mutators.
    apply_substrate_writes(fields, &writes);    // rate-limited, refuses > cap
    apply_macro_deltas(sim, &macro_deltas);      // add_belief/add_cohesion saturate

    // 6. Hand off taboo list to civ-laws (the religion → law hook).
    update_law_compliance(world, &religions);

    ReligionTickReport { religions, writes, macro_deltas }
}
```

### 4.1 The Big-Gods response curve (the only authored content)

There is exactly **one** authored function in this spec: the response curve
that maps substrate gradients to the three religion scalars. This is the
"rule, not the outcome" of religion emergence — every other quantity is
derived.

```
/// FR-CIV-EMERGENCE-RELIGION-1 — Norenzayan Big-Gods response.
///
/// Inputs:
///   hardship       = clamp01(|∇T|_p · w_T + |∇B|_p · w_B + |∇M|_p · w_M)   ∈ [0, 1]
///   group_size     = cluster.member_count,                              ∈ [2, ∞)
///   uncertainty    = clamp01(unrest / MAX_MISERY_UNREST),                ∈ [0, 1]
///   kinship_density = mean(SocialGraph.ties[*].kinship) over members,   ∈ [0, 1]
///
/// Outputs:
///   Δmonitoring,  Δmythic_coherence,  Δuncertainty_reduction  ∈ [-0.05, +0.05]
///
/// Mechanic:
///   monitoring rises with (hardship × group_size_factor) and falls with
///   kinship_density.  This is the Big-Gods prediction: in larger, more
///   stressed, less kin-bonded populations, surveillance & sanctioning god-
///   concepts outcompete private-cope ones.
///
///   mythic_coherence rises with sustained monitoring + low migration
///   (cluster stability).  Falls on contact with phonemically-distant
///   language (proxy for foreign-belief contact; see ADR-014 phoneme drift).
///
///   uncertainty_reduction rises with uncertainty * (1 - monitoring):
///   people seek relief proportional to felt anxiety, damped by how much
///   monitoring already supplies structure.  Returns toward zero when
///   unrest falls (it is *relief*, not *stock*).
fn apply_big_gods_response(profile: &mut ReligiousProfile,
                           g: &SubstrateGradients,
                           tick: u64) {
    let hardship       = clamp01(g.grad_T * W_HARDSHIP_T
                                + g.grad_B * W_HARDSHIP_B
                                + g.grad_M * W_HARDSHIP_M);
    let group_factor   = clamp01(profile.population as f32 / GROUP_NORM);
    let kinship_factor = clamp01(g.kinship_density);
    let uncertainty    = clamp01(g.unrest / MAX_MISERY_UNREST as f32);

    // Monitoring: Big-Gods term.  Larger + harder + less kin → more.
    let d_monitoring = 0.05 * (
        + 0.55 * hardship * group_factor
        + 0.30 * (1.0 - kinship_factor) * group_factor
        + 0.15 * uncertainty * group_factor
    ) - 0.02 * kinship_factor;

    // Mythic coherence: integration under stress + low migration.
    let d_coherence = 0.04 * (
        + 0.50 * profile.monitoring.max(0.3) // sustained monitoring integrates myth
        + 0.30 * (1.0 - g.migration_rate)
        + 0.20 * hardship
    ) - 0.03 * g.language_distance; // foreign belief contact frays canon

    // Uncertainty reduction: relief proportional to felt uncertainty,
    // damped by how much monitoring already supplies structure.
    let relief = uncertainty * (1.0 - profile.monitoring * 0.6);
    let d_uncertainty = 0.06 * relief - 0.05 * profile.uncertainty_reduction;
    // Returns toward zero when unrest falls — it is *relief*, not *stock*.

    profile.monitoring           = (profile.monitoring + d_monitoring).clamp(0.0, 1.0);
    profile.mythic_coherence     = (profile.mythic_coherence + d_coherence).clamp(0.0, 1.0);
    profile.uncertainty_reduction = (profile.uncertainty_reduction + d_uncertainty).clamp(0.0, 1.0);
}
```

The three constants `W_HARDSHIP_T`, `W_HARDSHIP_B`, `W_HARDSHIP_M` are config
(scenario YAML, RON overlay per `civ-laws` mod-friendliness). Default
`(0.4, 0.4, 0.2)` weights biomass-and-temperature over moisture — biome-tunable
per `PHYSICS_COUPLING_SUBSTRATE.md` §7.5 (the four reference scenarios
Genesis / Stress / Famine / Ice each commit a tuned triplet).

### 4.2 Per-tick caps (the invariant)

Every per-tick delta is bounded by a `const`:

```
MAX_D_MONITORING_PER_TICK    = 0.05;
MAX_D_COHERENCE_PER_TICK     = 0.04;
MAX_D_UNCERT_REDUCTION_TICK  = 0.06;
MAX_RITUAL_E_PER_TICK        = 50_000.0;   // joules per religion per tick
MAX_RITUAL_F_PER_TICK        = 1.0;        // kg/cell/tick
MAX_RITUAL_B_PER_TICK        = 0.5;        // kg/cell/tick
MAX_RITUAL_P_PER_TICK        = 10.0;       // agents/cell/tick
MAX_RITUAL_BELIEF_PER_TICK   = 5;          // macro belief units
MAX_RITUAL_COHESION_PER_TICK = 8;          // macro cohesion units
MAX_LAW_COMPLIANCE_DELTA_PER_TICK = 0.05;  // passed to civ-laws
```

A profile mutation that would exceed the cap is **clamped** (not refused);
a substrate write that would exceed the cap is **refused** with a logged
warning. The difference is intentional: clamping a profile is the same as the
existing `add_belief` saturating; refusing a substrate write is the
substrate's conservation invariant. Both tripwires fire the
`civ-emergence-metrics::branching::classify_regime` monitor so the dashboard
sees them.

### 4.3 Substrate write planning (the downward causation)

A religion produces a substrate write only when its three scalars cross a
threshold AND the substrate has enough "budget" to absorb the write without
tripping the conservation invariant. The plan is per-cell, per-field, capped.

```
struct SubstrateWrite {
    field: Field,        // E | F | B | P
    cell: IVec3,
    delta: f32,          // joules / kg / agents
    source: ClusterId,
}

fn plan_substrate_writes(profile: &ReligiousProfile,
                         centroid: IVec3) -> Vec<SubstrateWrite> {
    let mut writes = Vec::new();
    let intensity = profile.monitoring * profile.mythic_coherence;

    // Ritual fire: requires monitoring * coherence > 0.3 and an active ritual
    // (we model "active ritual" as monitoring*coherence; real schedules are
    // a future extension — see §7 Open Questions).
    if intensity > 0.30 && profile.uncertainty_reduction > 0.05 {
        // 3x3x3 block around the centroid; joules scaled by intensity.
        let e_per_cell = (intensity * 1000.0).min(MAX_RITUAL_E_PER_TICK / 27.0);
        for offset in BLOCK_3X3X3 {
            writes.push(SubstrateWrite { field: Field::E,
                                         cell: centroid + offset,
                                         delta: e_per_cell,
                                         source: profile.cluster });
        }
    }

    // Burnt offering: requires coherence > 0.5 (organized sacrifice, not
    // chaos); consumes B, emits F (ash + CO2 aerosols).
    if profile.mythic_coherence > 0.50 {
        for offset in BLOCK_3X3X3 {
            writes.push(SubstrateWrite { field: Field::B,
                                         cell: centroid + offset,
                                         delta: -0.5,
                                         source: profile.cluster });
            writes.push(SubstrateWrite { field: Field::F,
                                         cell: centroid + offset,
                                         delta: 0.1,
                                         source: profile.cluster });
        }
    }

    // Pilgrimage / martyrdom: requires uncertainty_reduction > 0.4 (people
    // are sufficiently relieved of anxiety that some move toward the site);
    // shifts P by a small amount.
    if profile.uncertainty_reduction > 0.40 {
        writes.push(SubstrateWrite { field: Field::P,
                                     cell: centroid,
                                     delta: 5.0,
                                     source: profile.cluster });
    }

    writes
}
```

Crucially: **the writes are the religion's only causal output**. A religion
cannot tell `civ-economy` "the tithe is due"; it can only *redirect biomass*
(`B` ↓ in one cell, `B` ↑ in another — substrate advection does the rest) or
*vent heat* (`E` ↑ at the ritual site, climate advects). The economy then
*reads* `B` and sees a harvest shortfall, exactly as it would a drought.

---

## 5. Why this is "Big Gods" and not "any religion"

Norenzayan's prediction is that **larger, more stressed, less kin-bonded
populations produce moralizing, monitoring, norm-enforcing religions**; small,
dense-kin, low-stress populations produce private-cope / shamanic forms.
The response curve §4.1 has those three terms literally:

| Norenzayan predictor | Substrate gradient | Response term |
|---|---|---|
| Group size | `∇P` cluster density (`population / GROUP_NORM`) | `0.55 * hardship * group_factor` |
| Stress | `\|∇T\| + \|∇B\| + \|∇M\|` | `0.55 * hardship * group_factor` |
| Kin-bonding | `SocialGraph.ties[*].kinship` mean | `- 0.02 * kinship_factor` (lower kinship → higher monitoring) |

A society that is large, hardship-stricken, and weakly kin-bonded will see
`monitoring` rise; a small kin-dense village under low stress will see
`monitoring` decay toward zero. The model's prediction is the *curve*,
not a label: "this religion is Big-Gods" is derived from `monitoring > 0.7 &&
mythic_coherence > 0.6 && population > 150`, displayed as a tooltip, never
stored.

This is the *emergence* the charter demands: no author picks the form; the
substrate does.

---

## 6. The religion → `civ-laws` hook (compliance, not theology)

Norenzayan's other prediction is that Big-Gods religions reduce free-riding by
**moralizing the cost of defection** — turning economic / social norms into
*religious obligations*. The mechanism in this spec is the *only* religion →
law coupling, and it is **field-mediated, not API-called**: religion writes
to the substrate; the substrate's `B`/`E`/`F` shifts change the payoff matrix
that `civ-laws` evaluates.

But there is also a **bounded direct hook**: when a religion's `monitoring`
rises above `LAW_MONITORING_THRESHOLD = 0.6` (config), the religion emits a
`ReligionTaboo { action: String }` to `civ-laws` *as a suggestion* — `civ-laws`
already has a `taboo: Vec<String>` field on its policy entries. The religion
**does not** modify the law; it suggests a taboo label, `civ-laws` integrates
it through its existing compliance rate-limiter (cap of
`MAX_LAW_COMPLIANCE_DELTA_PER_TICK = 0.05` per religion per tick).

The action label is derived, not authored: `action = "<scarce_resource>_<prohibition>"`,
where `<scarce_resource>` is the substrate cell's dominant deficit (B / M /
T) and `<prohibition>` is one of a fixed enum `{kill, hoard, desecrate,
marry_out, eat_alone}`. The enum is **not theology**; it is the set of
actions a needs-driven monitoring system would mechanically police
(`civ-economy::institutions` already names these as anti-defection levers).
The spec refuses any new enum variant without a substrate gradient to drive it.

```
/// FR-CIV-EMERGENCE-RELIGION-2 — religion → law compliance hook.
fn update_law_compliance(world: &hecs::World, religions: &[ReligiousProfile]) {
    for profile in religions {
        if profile.monitoring < LAW_MONITORING_THRESHOLD { continue; }
        let taboo = derive_taboo_label(world, profile.centroid);
        // civ-laws receives the suggestion; it owns enforcement & caps.
        civ_laws::suggest_taboo(profile.cluster, taboo,
                                MAX_LAW_COMPLIANCE_DELTA_PER_TICK);
    }
}
```

The taboo label is generated locally each tick from the substrate's current
state, not stored on the religion. Storing it would be authoring theology.

---

## 7. Phase ordering and interleave

```
[tick n]
  1. SUBSTRATE-PHYSICS    evolve T,M,E,F (advection, diffusion, reaction)       dt_phys
  2. SUBSTRATE-ECOLOGY    evolve P,B given current T,M,E,F                       dt_eco
  3. phase_emergence_genetics                                                  (per ADR-018 row 11–12)
  4. phase_emergence_culture                                                    (drift)
  4a. phase_emergence_religion          <-- NEW: this spec                     (profiled here)
  5. phase_emergence_social
  6. phase_emergence_psyche
  7. phase_emergence_legends                                                      (records, doesn't generate)
  8. phase_emergence_civ_ai
  9. phase_law_compliance                                                         (civ-laws absorbs taboo suggestions)
 10. SUBSTRATE-LOG                                                                 (append invariants + legends ingest)
```

Religion runs **after culture, before social**: cultural traits of the cluster
are current (so `language_distance` for `d_coherence` is read correctly), but
the social graph's `kinship_density` is read *before* this tick's social
updates (so the religion's response is causally upstream of social
re-organization — the interleave that gives edge-of-chaos lag per
`PHYSICS_COUPLING_SUBSTRATE.md` §4.1).

---

## 8. Invariants (the tripwires)

The five `PHYSICS_COUPLING_SUBSTRATE.md` §5 tripwires all apply, plus three
religion-specific invariants:

| ID | Invariant | Tripwire |
|---|---|---|
| **REL-INV-1** | Per-tick cap respected for every religion every tick. | If any cap is exceeded, clamp and log `religion: <cluster> cap exceeded on axis <monitoring\|coherence\|uncertainty_reduction> by Δ > cap`. |
| **REL-INV-2** | Substrate conservation: total `Σ E`, `Σ F`, `Σ B` change attributed to religion writes ≤ per-tick budget. | If any field's religion-attributed change exceeds `0.05` of substrate sink rate, log `religion: substrate write exceeds β budget`. |
| **REL-INV-3** | No `BeliefConcept` enum anywhere in the codebase. | `coupling_audit.sh` grep `pub enum BeliefConcept` in `crates/` — must return zero. |
| **REL-INV-4** | No inter-layer API calls. | `coupling_audit.sh` grep `use civ_religion` / `use civ_agents` etc. inside other emergent crates. |
| **REL-INV-5** | Population scan is `MIN_AGENTS = 2`-guarded. | Cluster with `member_count < 2` does not produce a religion scan (avoids lone wanderers driving religion). |
| **REL-INV-6** | `mythic_coherence` does not exceed `monitoring + 0.2` (coherence cannot outpace monitoring without kin-density support; physical constraint, not authored rule). | Clamp + log. |
| **REL-INV-7** | `uncertainty_reduction * monitoring ≤ MAX_JOINT_RELIEF = 0.7` (a religion cannot simultaneously claim high structure AND high relief beyond a physical bound). | Clamp + log. |

All seven tripwires feed `civ-emergence-metrics::branching::classify_regime`
so the dashboard sees religion-side tripwires as well as substrate-side.

---

## 9. Why this drops the `BeliefConcept` enum entirely

The `BeliefConcept` enum at `crates/engine/src/religion.rs:4-10` is the
canonical "hardcoded pantheon" failure. The decision to **delete** it (rather
than extend it) follows from ADR-016 §"Why Norenzayan": religion must be
"a macro adaptation to chronic needs stress rather than a separate authored
ideology type." An enum with five named kinds is exactly that — a separate
authored ideology type. The replacement is the three-scalar profile (§2),
which is **measured, derived, and substrate-driven**.

Migration path (when this spec is implemented):

1. Add `crates/engine/src/religion/profile.rs` with the three scalars and
   `apply_big_gods_response`.
2. Replace `crates/engine/src/religion.rs:42-67` (`emerge_belief`) with
   `tick_religion(world, fields, sim, tick) -> ReligionTickReport`.
3. Replace `crates/engine/src/religion.rs:69-98` (`spread_religion`) with the
   `plan_substrate_writes` + `apply_substrate_writes` split (§4.3).
4. Mark `BeliefConcept` and `Religion { beliefs: Vec<Belief> }` as `#[deprecated]`
   for one release, then delete. The legends query API
   (`legends::query::EntityRef`) replaces the "what did this religion believe?"
   surface — the answer is `monitoring` + `mythic_coherence` +
   `uncertainty_reduction` + the substrate gradients that produced them.
5. The `civ-legends` producer contract gains one new event:
   `EventKind::ReligionShift { monitoring, coherence, uncertainty }` (an
   extension of the open taxonomy, §3.4 of `legends-engine.md`). Existing
   kinds are unchanged.

No religion content (deity names, pantheons, myths) is added — the inspector
derives display labels from the triple plus the substrate state at the
religion's centroid cell.

---

## 10. Observability (dashboard hooks)

Per ADR-011 §"Emergence dashboard observability", each coupling must
contribute to at least one metric:

| Metric | Source | What it tells the player |
|---|---|---|
| `religion.profile.monitoring` per cluster | `ReligiousProfile::monitoring` time series | Are Big-Gods forms emerging? |
| `religion.profile.mythic_coherence` per cluster | `ReligiousProfile::mythic_coherence` time series | Is shared narrative forming? |
| `religion.profile.uncertainty_reduction` per cluster | `ReligiousProfile::uncertainty_reduction` time series | Is the religion absorbing unrest? |
| `religion.substrate_writes` per (field, cluster) | Substrate write log | Which religions are physically affecting the world? |
| `religion.law_compliance_delta` per cluster | `civ-laws` hook log | Which religions are binding their members to law? |
| `religion.cap_violation_rate` per axis | Tripwire log | Are we near the edge-of-chaos regime? |

These feed `crates/civ-emergence-metrics/src/lib.rs` exactly like the existing
`branching::classify_regime` and `power_law::PowerLawFit` metrics.

---

## 11. Tests (acceptance criteria)

### 11.1 Unit tests (engine-side)

- `fr_civ_religion_001_monitoring_rises_with_hardship_and_group_size`:
  feed three populations {small + low-hardship, large + low-hardship,
  large + high-hardship} for 1000 ticks; assert
  `monitoring[large+high] > monitoring[large+low] > monitoring[small+low]`.
- `fr_civ_religion_002_monitoring_decays_with_kin_density`:
  feed two populations of equal size & hardship, one kin-dense, one kin-thin;
  assert `monitoring[kin-thin] > monitoring[kin-dense]` after 1000 ticks.
- `fr_civ_religion_003_uncertainty_reduction_returns_to_zero_when_unrest_falls`:
  feed a population with `unrest = 0.6`, then drop to `unrest = 0.0`; assert
  `uncertainty_reduction` decays toward zero (relief, not stock).
- `fr_civ_religion_004_mythic_coherence_frays_on_foreign_contact`:
  introduce a cluster with phoneme-distance > 0.5 from neighbours; assert
  `mythic_coherence` decays.
- `fr_civ_religion_005_per_tick_caps_are_respected`:
  drive a religion into saturation; assert every per-tick delta ≤ its const.
- `fr_civ_religion_006_no_belief_concept_enum_after_migration`:
  grep `crates/` for `pub enum BeliefConcept`; assert zero matches after the
  migration is complete.
- `fr_civ_religion_007_min_agents_guard`:
  a cluster with `member_count = 1` does not produce a religion scan.
- `fr_civ_religion_008_rel_invariant_6_coherence_le_monitoring_plus_0_2`:
  drive a religion toward the constraint boundary; assert clamp + log.

### 11.2 Integration tests (substrate-coupled)

- `fr_civ_religion_010_ritual_fire_writes_e_to_substrate`:
  drive `monitoring = 0.8`, `mythic_coherence = 0.7`, `uncertainty_reduction
  = 0.3`; assert `Σ E` in 3³ block around centroid increases by ≤
  `MAX_RITUAL_E_PER_TICK` and by ≥ 0.
- `fr_civ_religion_011_burnt_offering_consumes_b`:
  drive `mythic_coherence > 0.5`; assert `Σ B` in the 3³ block decreases by
  ≤ `MAX_RITUAL_B_PER_TICK`.
- `fr_civ_religion_012_substrate_conservation_under_religion_load`:
  run 10 000 ticks with three active religions; assert `Σ E`, `Σ F`, `Σ B`
  conserved within the existing substrate invariant (no religion-attributed
  leak).
- `fr_civ_religion_013_religion_shift_records_in_legends`:
  fire one `ReligionShift` event; assert the legends worker ingests it and
  `epoch_digest` includes the triple.

### 11.3 Property tests

- `prop_religion_profile_stays_in_unit_interval`: every scalar ∈ `[0, 1]` for
  any sequence of substrate inputs.
- `prop_religion_cap_violation_rate_is_bounded`: across 100 random seeds,
  cap-violation rate ≤ 0.01 (the cap almost never trips in normal play;
  frequent tripping signals a misconfigured scenario).
- `prop_big_gods_dominates_under_norenzayan_conditions`: across 200 random
  (hardship × group × kinship × unrest) parameter sets,
  `monitoring > 0.5` correlates with `hardship * group > 0.5 && kinship < 0.3`
  ≥ 95 % of the time.

### 11.4 End-to-end (scenario-driven)

- `fr_civ_religion_020_genesis_scenario_no_big_gods`:
  the Genesis reference scenario (no hardship, small populations, dense kin)
  produces `monitoring < 0.3` for every cluster across 50 000 ticks.
- `fr_civ_religion_021_famine_scenario_big_gods_emerge`:
  the Famine reference scenario (climate-driven M and B collapse, dense
  populations) produces `monitoring > 0.7` for ≥ 1 cluster within 20 000
  ticks, and the religion's taboo binds ≥ 1 civ-laws rule.

---

## 12. Migration & rollback

When this spec is implemented:

1. New module: `crates/engine/src/religion/profile.rs` (the three-scalar
   profile + `apply_big_gods_response` + `plan_substrate_writes`).
2. New module: `crates/engine/src/religion/coupling.rs` (the substrate
   gradient sampling + write application + macro-delta application).
3. New test module: `crates/engine/src/religion/tests_profile.rs` (covers
   §11.1–11.3).
4. Old `crates/engine/src/religion.rs` is *replaced* (not edited). The
   `BeliefConcept` enum and the `Religion { beliefs }` struct are deleted;
   the `emerge_belief` and `spread_religion` entry points are deleted.
5. The `civ-legends` producer contract gains `EventKind::ReligionShift`
   (extension of the open taxonomy, no engine change).
6. `civ-laws` gains `suggest_taboo(cluster, action, max_delta)` — a small
   bounded hook. No authored taboo list.

Rollback: revert the four module changes. The deletion of `BeliefConcept`
is the highest-risk part; for one release we keep a `#[deprecated]`
shim so external crates (e.g. web inspector reading religion details)
do not break, then remove in the release after.

---

## 13. Open questions for review

1. **Mythic coherence — language proxy.** `d_coherence` currently uses
   `language_distance` as a foreign-belief-contact proxy. Should we instead
   use a direct contact-network overlap (`SocialGraph::contact_intensity`)?
   Open: requires `civ-language` to expose contact intensity to the substrate.
   *Default for now*: language distance.
2. **Ritual cadence.** §4.3 triggers writes by threshold-crossing, not by a
   schedule. Should religions develop *cadence* (a derived fourth scalar)?
   Open: may require a `last_ritual_tick` field on `ReligiousProfile`, fed by
   the substrate's biomass-recovery rate at the centroid. *Defer* until
   scenario testing shows threshold-only is too uniform.
3. **Martyrdom / sacrifice as population writes.** §4.3 includes a small `P`
   shift under `uncertainty_reduction > 0.4`. Should martyrdom (mass `P`
   decrease at the site) be a separate, larger-amplitude write gated by
   `monitoring * uncertainty_reduction > 0.5`? *Defer* — the substrate
   invariant forbids single-tick `P` writes above `MAX_RITUAL_P_PER_TICK`,
   and mass martyrdom exceeds that; we want a separate ADR if we go there.
4. **Religion → faction identity.** Should religion tie-break `faction_id`
   for clustering (per ADR-015)? *No* — religion is a parallel axis, not a
   re-bucketing of factions. Faction clustering already runs in
   `phase_emergence_faction`; religion sits orthogonally.
5. **Inspector labels.** §2 mentions a *derived* display label. The label
   set (`moralizing sun cult`, `shamanic ancestor rite`, `ecstatic mystery`)
   is the only authored *cosmetic* content. Should we expose a config file
   for label rules per scenario? *Default*: ship a small fixed label table,
   overridable via `civ-laws` mod overlay.

---

## 14. References

- Norenzayan, A. (2013). *Big Gods: How Religion Transformed Cooperation and
  Conflict.* Princeton University Press. (The model this spec implements.)
- Atran, S. & Henrich, J. (2010). "The Evolution of Religion: How Cognitive
  By-Products, Adaptive Learning Heuristics, and Cultural Displays Evolved."
  *Trends in Cognitive Sciences*. (Substrate-driven religion framing.)
- Boyer, P. (2001). *Religion Explained.* Basic Books. (Why theological
  minimalism at the model layer is correct — concepts are minimal
  counterintuitive agents; the *triple* is the model's analogue.)
- Civis codebase:
  - `crates/engine/src/religion.rs:1-158` (the model this spec replaces).
  - `crates/engine/src/emergence.rs:159` (`phase_emergence` phase order).
  - `crates/needs/src/lib.rs:34-1012` (the needs vector ADR-016 cites).
  - `crates/climate/src/lib.rs:71-117` (`ClimateState::step` → T gradient).
  - `crates/planet/src/{geology,weather}.rs` (M, F substrate writes).
  - `crates/legends/src/{lib,model}.rs` (saga-graph producer contract).
  - `crates/laws/src/lib.rs:67-74` (the `LawDb` religion suggestion lands in).
  - `crates/civ-emergence-metrics/src/lib.rs` (the dashboard hooks).
- Civis ADRs:
  - ADR-011 — N-series emergence coupling contract.
  - ADR-016 — Religion emergence from needs vector (the substrate decision).
  - ADR-018 — Bidirectional coupling inventory (rows 9–11 are this spec's
    existing rows; this spec adds the substrate-coupled implementation).
- Civis design docs:
  - `docs/guides/emergence-charter.md` — only the substrate is authored.
  - `docs/design/PHYSICS_COUPLING_SUBSTRATE.md` — the only coupling channel.
  - `docs/design/legends-engine.md` — the saga-graph producer contract.
