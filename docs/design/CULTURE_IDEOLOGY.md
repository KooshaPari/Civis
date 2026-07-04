# CIV-CULTURE-IDEOLOGY: Emergent Norms, Values, Art, and Identity — Design Spec

> **Status:** Design (planner-only, 2026-06-24). No implementation code in this document.
> **Spec ID:** `civ-culture-ideology` | **Layer:** Macro read-out + integrated composition on top of the existing emergence substrate.
> **Governing canon:** [`docs/adr/ADR-emergence-charter.md`](../adr/ADR-emergence-charter.md) (Layer-0 only is authored; everything above emerges), [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) §3 (E2 psyche/social, E5 language, E6 religion), [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) (dialect / script / ritual / tradition classifiers — *companion*, do not duplicate), [`docs/design/psyche-social.md`](psyche-social.md) (psyche + social substrate), [`docs/design/LANGUAGE_EMERGENCE.md`](LANGUAGE_EMERGENCE.md) (language drift), [`docs/design/legends-engine.md`](legends-engine.md) (saga-graph read-out), [`docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md`](../specs/CIV-0106-social-ideology-health-insurgency-v1.md) (6-axis `IdeologyVector` + diffusion).
> **Pattern ancestor:** `civ-culture-emergent.md` §1.2 (pure read-out functions, additive, never mutate substrate).
>
> **What this spec owns:** the four macro phenomena that `civ-culture-emergent.md` does **not** cover — **Norms**, **Values**, **Art**, and **Identity** — plus the **Ideology** read-out that integrates all four into a single inspectable bundle. The companion spec (`civ-culture-emergent.md`) owns dialect / script / ritual / tradition; this spec is its sibling, not its replacement.

---

## 0. Why this spec exists

The Civis substrate is *broadly complete* for emergence: `Psyche::beliefs` drifts via `belief_culture_exposure` (psyche-social.md §3.3), `SocialGraph::Tie` carries kinship / affinity / trust / familiarity / last_seen (social.rs), `CultureProfile::traits` and `language` drift + creolize through a contact graph (culture.rs), `ClusterId` partitions agents by co-location + payoff (cluster.rs), `FactionSeed` k-means clusters agents by an 8-dim `AgentIdeology` (faction_emergence.rs), `Religion` accumulates `Belief`s with `concept ∈ {NaturalAgent, MoralOverseer, Afterlife, Taboo, Ritual}` (religion.rs), and `CIV-0106` defines a 6-axis `IdeologyVector` with diffusion over an influence graph.

What is *missing* — and what this spec specifies — is the **integrated macro layer** that lets the player (and the inspector, the legends engine, the emergence dashboard, the 3D / web overlays) perceive culture and ideology as *composed phenomena*:

- **Norms** — the unwritten rules a community enforces through tie-affinity collapse (not the `civ-laws` physics-DB; norms are emergent proto-social-rules that *precede* any codified law).
- **Values** — the *shared evaluative orientation* of a community (what it considers good, sacred, dangerous, beautiful) — distinct from per-agent `Psyche::beliefs`, distinct from the 6-axis `IdeologyVector` of CIV-0106, derived from the *centroid* of lived belief + religion + kinship.
- **Art** — the *aesthetic traditions* of a community: a recurring style signature over the cultural trait vector + the material affordances of the cluster's biome (music is explicitly out of scope, owned by `audio-direction.md` / E6).
- **Identity** — the *in-group / out-group* marker signature: who counts as "us," what signals that membership (kinship density, dialect distance, value-axis distance, norm-distance), and how it stiffens or relaxes.
- **Ideology** — the *integration* of the four above plus the CIV-0106 6-axis vector into a single inspectable `IdeologyBundle`. The ideology of a faction is *not* its `FactionSeed::ideology_centroid` (that is the 8-dim cluster k-means input); it is the bundle composed from the macro read-outs.

The charter test applied to every field below: *can this emerge from Layer-0 rules?* Yes — norms from repeated tie-affinity collapse on the same `Tie::affinity` polarity, values from the weighted belief centroid + religion's `Belief` distribution, art from the cluster's `CultureProfile::traits` projected through a `MaterialPalette` data table, identity from a partition of clusters by a composite distance metric. Nothing in this spec is a `Norm`, `Value`, `Art`, `Identity`, or `Ideology` enum stored on any agent, cluster, or faction.

This spec **does not** re-implement `drift_populations`, `belief_culture_exposure`, `update_beliefs`, `apply_social_event`, `decay_social_graph`, `spread_religion`, `cluster_into_factions`, or the CIV-0106 `IdeologyField` diffusion step. The substrate is correct as of this spec's date; this spec adds read-out + composition + bidirectional coupling + criticality knobs + dashboard metrics, mirroring the structure of `civ-culture-emergent.md` so the two specs are siblings.

---

## 1. Substrate inventory (what we read, what we do not write)

| Substrate field | Crate + module | Role in this spec |
|---|---|---|
| Per-agent psyche | `crates/agents/src/psyche.rs` (`Psyche { drives[4], temperament, mood, beliefs[4], maturity }`) | Source of personal belief samples; weighted into the cluster belief centroid (§3.1). Not mutated. |
| Per-agent social graph | `crates/agents/src/social.rs` (`Tie { kinship, familiarity, affinity, trust, last_seen }`, `SocialGraph`, `relation_label`) | The channel through which norms propagate (§2.2) and through which identity markers are scored (§5.3). Not mutated. |
| Per-cluster culture | `crates/agents/src/culture.rs` (`CultureProfile { traits[4], language[4], contact, kinship }`) | Read for the value centroid (§3.1) and the art signature (§4.2). Not mutated. |
| Per-cluster cluster ID | `crates/agents/src/cluster.rs` (`ClusterId`, `ClusterMember`, `MembershipPayoff`) | The aggregator key for norms / values / art / identity — the unit of "community." Not mutated. |
| Per-cluster religion | `crates/engine/src/religion.rs` (`Religion { beliefs: Vec<Belief>, cohesion, member_count, age_ticks }`, `BeliefConcept`) | Read for the sacred-vs-profane axis of values (§3.4) and the ritual component of art / norms (§4.3). Not mutated. |
| Per-cluster faction | `crates/engine/src/faction_emergence.rs` (`FactionSeed { ideology_centroid: [f32; 8], territory_center, member_count }`) | The 8-dim cluster centroid is the *input*; this spec derives the *integrated ideology bundle* on top of it (§6). Not mutated. |
| Per-cluster language | `culture.rs` (`CultureProfile::language`) + `LANGUAGE_EMERGENCE.md` | Read for the identity marker (§5.4) and for value-clarity scoring (§3.5). Not mutated. |
| Per-region ideology field | `CIV-0106` (`IdeologyField { vector: [market, state, liberty, equality, security, tradition], integrity_damping }`) | Read as the *institutional* ideology signal; the per-cluster bundle in this spec sits *above* the CIV-0106 region-level field and refines it. Not mutated. |
| Per-cohort cohesion | `CIV-0106` (`cohesion`, `polarization`) | Read as the trust / solidarity substrate for norms (§2.4) and identity (§5.5). Not mutated. |
| Co-located cooperation events | `social.rs` (`apply_social_event(Interaction::Cooperated)`, `Coexisted`) | Read for the ritual-as-frequency-pattern channel that *nurtures* norms (§2.4). Not mutated. |
| Freehand / player input | (none yet) | The freehand tool path is the *one* legitimate source of authored cultural content (per `civ-culture-emergent.md` §2.2 final row); this spec does **not** introduce any new authored cultural enum, but the existing freehand pathway remains the player-visible lever. |

**The substrate contract:** every macro label defined in §2–§6 is a *pure function* over the substrate above. No new state on any ECS entity, no new component, no new event type beyond what the legends engine already ingests.

---

## 2. Norms — the unwritten rules

### 2.1 What a "norm" is in this spec

A **norm** is a recurring *tie-affinity collapse pattern* in a cluster: when a given *behavioral hypothesis* (e.g. "eating the flesh of kin," "sharing food with strangers," "fighting unarmed opponents") is repeatedly reinforced by *the same sign* of `Tie::affinity` mutation across many intra-cluster ties, the behavior becomes a *norm*. The norm is **never stored** on any agent; it is a measured pattern of how the cluster's social graph responds to the behavior.

The norm space is the same enumeration used by `BeliefConcept` (`religion.rs:4-10`) extended with two social-only categories. Specifically, the *candidate hypothesis* set is *not* authored as an enum of behaviors; instead, it is a **taxonomy of behavioral consequences** the social graph can observe:

```
NormDomain
├── Kin   (interaction with kinship > 0.5 tie)
├── Strangers (interaction with kinship < 0.05 tie)
├── Deceased (interaction with dead / corpse location)
├── Property (interaction with material possession / hoarding)
├── Violence (interaction with conflict initiation)
├── Deception (interaction with broken trust events)
├── Sacred (interaction with religion-flagged object / location)
└── Authority (interaction with emergent dominance pattern)
```

The **list of domains** is the only authored content in this section (eight domains — derived from the substrate categories the graph already observes: kinship from `Tie::kinship`, religion from `Religion::beliefs`, violence / deception from `Interaction::{Competed, Defected}`, etc.). The **content** of each norm (e.g. "kin-flesh is forbidden") is *measured* from the substrate, never declared.

### 2.2 Norm classification — pure function

```text
// DIAGNOSTIC — pure read-out, never stored on a cluster
fn classify_norms(
    cluster_id:      ClusterId,
    events:          &[SocialEvent],
    religion:        Option<&Religion>,
    sample_window:   &SampleWindow,
    params:          &CultureIdeologyParams,
) -> Vec<NormPattern>

struct NormPattern {
    domain:   NormDomain,                     // which domain the pattern is in
    polarity: NormPolarity,                   // Forbidden | Expected | Honored | Neutral
    strength: f32,                            // [0, 1] — fraction of ties collapsing on this polarity
    evidence_count: u32,                      // number of supporting SocialEvent observations
    stable_ticks: u32,                        // how many consecutive windows this polarity held
}

enum NormPolarity { Forbidden, Expected, Honored, Neutral }
```

The classifier walks the `(cluster_id, window)` slice of `SocialEvent`s, groups by `NormDomain` (using the event's `a → b` tie properties at the time of the event), and for each domain measures:

- **Affinity collapse direction**: when an interaction of domain *d* fires, what is the mean `Δ affinity` across the ties it touches? Sustained negative collapse on kin-domain events → `Forbidden`. Sustained positive on stranger-domain → `Honored`. Mixed → `Expected`. Flat → `Neutral`.
- **Religion reinforcement**: if the cluster has a `Religion` whose `Belief`s include `Taboo { action }`, the `Forbidden` polarity on the matching domain is amplified by `religion.cohesion * belief.strength`.
- **Stability**: how many consecutive windows this polarity held without flipping. Below `params.norm_stability_threshold` (default 2 windows) the pattern is *noise*, not a *norm*.

The output `Vec<NormPattern>` is the cluster's *norm surface* at the sample tick — typically 1–6 entries, never 0 (every cluster has at least one observable polarity across the 8 domains).

### 2.3 What produces each norm phenomenon

**Taboos** are `Forbidden` patterns with `evidence_count ≥ params.taboo_min_evidence` (default 8) AND `religion.cohesion > 0.4` (a religious reinforcement is present). The classifier labels them `taboo.<domain>` (descriptive string, not enum).

**Hospitality norms** are `Honored` patterns on the `Strangers` domain with `strength ≥ 0.6`. Labeled `hospitality.<style>` where `<style>` is derived from the contact-edge pattern (regular trading partner → `hospitality.trade`; kin-by-marriage network → `hospitality.allied`; etc.).

**Authority norms** are `Expected` patterns on the `Authority` domain where the polarity is sustained AND `faction_emergence::should_faction_split` is false (the cluster has a stable authority gradient). Labeled `authority.<style>` derived from `FactionSeed::territory_center` variance.

**Deference norms** are `Honored` patterns on the `Deceased` domain (treatment of the dead) sustained across ≥ 1 generation (params.norm_generation_ticks, default 20 in-game years). Labeled `deference.<form>` derived from the spatial distribution of post-mortem cooperation events.

All labels are *derived strings* — the function set is the contract, the exact identifiers are illustrative.

### 2.4 The norm-emergence substrate channel

Norms emerge from the *same* substrate `civ-culture-emergent.md` §1.4 uses for rituals — `Interaction::Coexisted` + `Interaction::Cooperated` events at a `(location_bin, window_ticks)` — but with the *norm classifier's* polarity-and-domain lens. The two specs are complementary: `civ-culture-emergent.md` projects rituals as *frequency patterns* (descriptive of the cooperation shape); this spec projects norms as *polarity patterns* (descriptive of the *evaluative response*). A "feast" (ritual label) and a "hospitality norm" (norm label) can co-exist on the same cooperation pattern — the feast is the *form*, the norm is the *rule*.

The cohesion substrate (`CIV-0106` `cohesion` per region/cohort) is the *enforcement mechanism*: low cohesion ⇒ norms are unstable (the `stable_ticks` field rarely crosses the threshold); high cohesion + high `religion.cohesion` ⇒ norms stabilize faster and reach `Honored` / `Forbidden` more readily.

### 2.5 Norms do not become `civ-laws`

The `civ-laws` crate is the **physical-law database** (mass conservation, material properties, fictional-physics extensions). It is *not* the social-law layer. Norms are *pre-institutional*: they exist as graph patterns long before any institution codifies them. The eventual `policy` / `institution` crate (per CIV-0103) is the layer that *codifies* a subset of stabilized norms into enforceable rules — but **this spec does not write that codification path**. Norms as defined here are the *substrate* the institution layer would later read; the institution layer is owned by a separate spec (tracked as a follow-up in §8).

---

## 3. Values — the shared evaluative orientation

### 3.1 What a "value" is in this spec

A **value** in this spec is the *centroid of lived belief + religion* within a cluster, projected through a small `ValueAxesProfile` data table. It is *not* the per-agent `Psyche::beliefs` (those are personal samples) and *not* the 6-axis `IdeologyVector` of CIV-0106 (that is the institutional / region-level orientation). The value system is the *community-level* evaluative orientation — what this community, taken as a whole, considers good / bad / sacred / dangerous / beautiful.

### 3.2 The four value axes

The axes are *runtime-derived*, not authored enums. They are defined by a `ValueAxesProfile` data struct — a small RON-loadable table that maps `belief_axis` (the 4 axes of `Psyche::beliefs`) + `religion::BeliefConcept` to four named evaluative axes:

```
ValueAxesProfile
├── provision_axis:    weight_belief_axis[0]   // food / material security
├── security_axis:     weight_belief_axis[1]   // safety / threat response
├── affiliation_axis:  weight_belief_axis[2]   // kinship / in-group warmth
└── novelty_axis:      weight_belief_axis[3]   // exploration / change-tolerance
```

The default `ValueAxesProfile` is a unit-weighted projection (each `belief_axis` maps 1:1 to the same-named value axis). Mods / scenarios can load alternative projections (e.g. an insectoid civ might flip the security axis). **The four axes are the only authored content**; every downstream label is derived.

### 3.3 Value classification — pure function

```text
// DIAGNOSTIC — pure read-out, never stored on a cluster
fn classify_values(
    cluster_id:         ClusterId,
    members:            &[Entity],                    // cluster members
    psyches:            &Query<&Psyche>,
    social_graphs:      &Query<&SocialGraph>,
    culture:            &CultureProfile,
    religion:           Option<&Religion>,
    axes:               &ValueAxesProfile,
    params:             &CultureIdeologyParams,
) -> ValueSystemBundle

struct ValueSystemBundle {
    centroid:           [f32; 4],                     // the weighted belief centroid projected to value axes
    sacred_axes:        SmallBitSet<4>,                // which axes the religion reinforces
    forbidden_axes:     SmallBitSet<4>,                // which axes the religion forbids (Taboo)
    clarity:            f32,                           // [0, 1] — how tightly clustered the beliefs are
    tradition_id:       Option<String>,                // name carried from civ-culture-emergent TraditionLabel
}

struct ValueAxesProfile {
    provision:    AxisProjection,    // (belief_axis_index, weight)
    security:     AxisProjection,
    affiliation:  AxisProjection,
    novelty:      AxisProjection,
}
```

The classifier:

1. **Computes the weighted belief centroid** over cluster members, weighted by `Tie::familiarity` for the member's strongest intra-cluster ties (a loner does not anchor the cluster's value-system; a high-familiarity node does). This is the same `weighted_belief_centroid` primitive that `civ-culture-emergent.md` §3 specifies — it is the *shared* primitive for the values and the culture traits feedback.
2. **Projects** the centroid through `axes` to the four value axes (the `centroid: [f32; 4]` field).
3. **Marks sacred / forbidden axes** from the religion: if any `Religion::beliefs` is `BeliefConcept::MoralOverseer` and `strength > params.sacred_threshold` (default 0.5), the corresponding axis is `sacred`; if any `Belief::concept` is `Taboo { action }` and `strength > params.taboo_threshold` (default 0.6), the corresponding axis is `forbidden`.
4. **Computes clarity** as `1.0 - weighted_variance_of_centroid` — high clarity means the cluster is unanimous on its values; low clarity means the cluster is in a schism (this is the schism signal `civ-culture-emergent.md` §2.2 second-from-bottom row uses).
5. **Carries the tradition label** from the existing `classify_religion` read-out (a *soft* string ID, not a hard enum), so the value bundle can be cross-referenced with the legend.

### 3.4 What produces each value phenomenon

**Sacred values** are axes in `sacred_axes` with `religion.cohesion > 0.6` AND `clarity > 0.7`. Labeled `sacred.<axis_name>`. Treated as *non-negotiable* by the norm classifier — a `Forbidden` norm on a sacred axis carries the religion's full taboo force.

**Forbidden values** are axes in `forbidden_axes` with `religion.cohesion > 0.5`. Labeled `taboo.<axis_name>`. The norm classifier uses these as a *floor* for `Forbidden` polarity.

**Schism** is `clarity < params.schism_clarity_threshold` (default 0.3). The value classifier returns a bundle with empty `centroid` axes (zeroed) — a *signal to downstream consumers* (the polity cohesion graph in `polities-markets.md`, the cluster `reconcile_membership` path) that the cluster is in crisis and likely to split.

**Value drift** is measured over consecutive samples: a per-axis delta of magnitude > `params.value_drift_threshold` (default 0.05 per sample) is flagged `drift.<axis_name>.<direction>` (rising / falling). The dashboard (§7) consumes these as time-series.

### 3.5 Value–language coupling (the intelligibility channel)

A cluster's `CultureProfile::language` vector distance from a *reference* (the cluster's founding language state, recorded once in the cluster's first sample) provides a *clarity discount*: the further the language has drifted, the harder it is to communicate the cluster's values to outsiders, and the harder it is to coordinate on value-aligned action. The `classify_values` function applies a clarity discount `clarity *= (1.0 - language_drift_from_founding * params.language_clarity_discount)` (default discount coefficient 0.3). This is a *shared-gradient* coupling to `LANGUAGE_EMERGENCE.md`: no direct call, just the same `language_distance` primitive both specs use.

---

## 4. Art — the aesthetic traditions

### 4.1 What "art" is in this spec

**Art** in this spec is the *aesthetic signature* of a cluster — a recurring style pattern over the cluster's `CultureProfile::traits` + the `MaterialPalette` of the cluster's biome + the cluster's ritual frequency distribution. It is *not* a stored `Art` enum, *not* a `StyleId`, *not* a hand-coded glyph atlas. It is the *measured recurrence pattern* that the renderer can read to materialize a runtime visual signature (color palette, stroke curvature, ornamentation density).

The art signature is the *visual analogue* of the script signature in `civ-culture-emergent.md` §1.3 — but where the script signature is a *byte-vector* the renderer feeds to a glyph composer, the art signature is a *style-parameter bundle* the renderer feeds to the 3D / 2D client (color palette + ornamentation density + symmetry preference + stroke curvature bias).

### 4.2 The art signature — derived projection

```text
// DIAGNOSTIC — pure read-out, cached like ScriptSignature
fn project_art(
    culture:    &CultureProfile,
    palette:    &MaterialPalette,
    rituals:    &BTreeMap<RitualLabel, f32>,
    params:     &CultureIdeologyParams,
) -> ArtSignature

struct ArtSignature {
    /// Warm-cool color bias in [-1, 1] (warm to cool). Derived from culture.traits[2] (novelty axis).
    color_warmth: f32,
    /// Saturation in [0, 1]. Derived from palette.material_richness * (1 - culture.kinship).
    saturation:   f32,
    /// Ornamentation density in [0, 1]. Derived from ritual-frequency entropy * culture.traits[3].
    ornamentation: f32,
    /// Symmetry preference in [0, 1] (asymmetric to symmetric). Derived from culture.kinship.
    symmetry:     f32,
    /// Stroke curvature bias in [-1, 1] (angular to curvilinear). Derived from culture.traits[1] (security axis).
    curvature:    f32,
    /// Material palette index (a hash into the MaterialPalette table — NOT an authored StyleId).
    material_idx: u32,
}
```

The projection is a pure function: a `CultureProfile` + a `MaterialPalette` + a `ritual-frequencies` map produces a single `ArtSignature`. The signature is **cached** on `CultureProfile` as a *derived* field (mirroring the `ScriptSignature` caching pattern in `civ-culture-emergent.md` §1.3) — computed lazily on first read, invalidated when `cultural_distance` from the prior signature exceeds `art_invalidation_threshold` (default 0.15).

### 4.3 What produces each art phenomenon

**Style families** are single-link clusters of `ArtSignature`s with `signature_distance < art_family_split_threshold` (default 0.20). The family is the *macro* phenomenon; the signature is the *per-cluster* instance. Two clusters in the same family share enough aesthetic vocabulary that the renderer can use the same glyph-composer pipeline; two clusters in different families need separate composer pipelines.

**Style drift** is the per-tick change in the signature: any axis change > `art_axis_drift_threshold` (default 0.02) per sample is a `drift` signal. Sustained drift for > `art_drift_stability_ticks` (default 4 samples) constitutes a *style family migration* — the cluster's signature has moved enough that it crosses the family-split boundary and joins a different family.

**Material constraint** is the `palette: MaterialPalette` input — the biome's available materials (pigments, clays, woods, metals, stones) bound the achievable palette. A desert cluster cannot produce a rainforest palette regardless of its culture vector; the `MaterialPalette` is the data-side constraint that makes art *grounded* in the substrate. The palette table is a RON-loadable data struct (not authored per-cluster), parallel to the `PhonotacticProfile` in `civ-culture-emergent.md` §1.3.

**Ritual ornamentation** is the `rituals: &BTreeMap<RitualLabel, f32>` input — a cluster with high ritual-frequency entropy (many ritual shapes, no dominant one) produces higher `ornamentation` (visual complexity); a cluster with one dominant ritual produces lower ornamentation (visual simplicity). This is the *visual* expression of the same ritual substrate the legends engine reads for the cultural register.

### 4.4 Art is not music

`audio-direction.md` / `emergent-systems-spec.md` §2.5 (E6) owns the `MusicalTradition` derivation from `(culture_vec, available_materials)`. The art signature in this spec is the *visual* counterpart — color, ornamentation, symmetry, curvature, material. Music is a separate substrate read-out that does not require culture-language co-emergence; art in this spec does (the art signature uses the culture + material + ritual substrate). This spec notes the boundary and stops.

### 4.5 The art cache and the renderer

The art signature is cached on `CultureProfile` as the field `art_cache: Option<ArtSignature>`. The cache is invalidated by the drift loop in the same tick that mutates the culture vector — a stored signature can never outlive a substantial culture change. The cache lifetime is identical to the `ScriptSignature` cache lifetime in `civ-culture-emergent.md`; the two caches are invalidated by the same drift pass (one of the few legitimate synchronization points between the two read-out specs).

---

## 5. Identity — the in-group / out-group marker signature

### 5.1 What "identity" is in this spec

**Identity** in this spec is the *composite in-group marker signature* of a cluster — the bundle of signals by which the cluster's members recognize each other as "us" and recognize outsiders as "them." Identity is *not* a stored `Identity` enum, *not* a `FactionId`, *not* a `TeamTag`. It is a *partition* of clusters by a composite distance metric over the substrate, plus a *strength* score for the partition.

Identity has two facets:

1. **Membership signal** (within-cluster): how tightly do the cluster's agents cluster on the identity dimensions? Measured by the weighted variance of the per-agent identity dimensions.
2. **Boundary signal** (between-cluster): how distinct is the cluster's identity from neighboring clusters? Measured by the composite distance to the nearest neighbor cluster on the identity dimensions.

### 5.2 The four identity dimensions

Identity is composed from four substrate dimensions, *each derived, each read-only*:

```
IdentityDimensions
├── kinship_density       // mean Tie::kinship for intra-cluster ties (read social.rs)
├── dialect_distance      // mean language_distance to neighboring clusters (read culture.rs + LANGUAGE_EMERGENCE)
├── value_axis_distance   // mean l2 distance of ValueSystemBundle.centroid to neighboring clusters (read §3)
└── norm_distance         // mean symmetric difference of norm polarity patterns to neighboring clusters (read §2)
```

These four are the *only* dimensions used by identity. The *list* is authored (as is the case with `NormDomain` in §2.1); the *values* are measured.

### 5.3 Identity classification — pure function

```text
// DIAGNOSTIC — pure read-out, never stored on a cluster
fn classify_identity(
    cluster_id:         ClusterId,
    cluster_cultures:   &BTreeMap<ClusterId, CultureProfile>,
    cluster_values:     &BTreeMap<ClusterId, ValueSystemBundle>,
    cluster_norms:      &BTreeMap<ClusterId, Vec<NormPattern>>,
    cluster_kin_density: &BTreeMap<ClusterId, f32>,    // precomputed mean Tie::kinship per cluster
    params:             &CultureIdeologyParams,
) -> IdentitySignature

struct IdentitySignature {
    membership_strength: f32,                  // [0, 1] — tightness of within-cluster identity
    boundary_strength:   f32,                  // [0, 1] — distinctness from nearest neighbor cluster
    composite_distance:  f32,                  // [0, 1] — weighted mean of the four dimensions
    in_group_label:      Option<String>,       // soft descriptive label (e.g. "us-of-the-river-bend")
    boundary_marker:     IdentityMarker,       // the dominant marker dimension (Kinship | Dialect | Value | Norm)
}

enum IdentityMarker { Kinship, Dialect, Value, Norm }
```

The classifier:

1. Computes `kinship_density = mean_tie_kinship(intra_cluster)` from the precomputed map (the map is a derived field; this spec does not store it).
2. Computes `dialect_distance = mean(language_distance(self, neighbor) for neighbor in nearest_neighbors)` where `nearest_neighbors` are the 3 nearest clusters in `cluster_cultures` by `language_distance`.
3. Computes `value_axis_distance = mean(l2_distance(values.centroid, neighbor.values.centroid) for neighbor in nearest_neighbors_by_value)`.
4. Computes `norm_distance = mean(symmetric_difference(norms, neighbor.norms) for neighbor in nearest_neighbors_by_norm)` where symmetric difference counts `NormPattern` entries that exist in only one of the two clusters OR have differing polarity.
5. `composite_distance = w_k * kinship_distance + w_d * dialect_distance + w_v * value_axis_distance + w_n * norm_distance` with default weights `w_k = 0.35, w_d = 0.25, w_v = 0.25, w_n = 0.15` (kinship heaviest because it is the substrate-level signal).
6. `membership_strength = 1.0 - weighted_variance(per_agent_identity_dimensions)` where the per-agent dimensions are computed inline.
7. `boundary_strength = composite_distance` (the more distant from neighbors, the stronger the boundary).
8. `boundary_marker` is the dimension with the highest individual contribution to `composite_distance`.

### 5.4 What produces each identity phenomenon

**Kinship-based identity** is when `boundary_marker = Kinship` AND `kinship_density > 0.5` AND `composite_distance > params.identity_distinctness_threshold` (default 0.5). Labeled `kin-clan.<descriptor>` where `<descriptor>` is derived from the cluster's territory. The label is descriptive (a string ID), not an enum.

**Dialect-based identity** is when `boundary_marker = Dialect` AND `dialect_distance > params.dialect_identity_threshold` (default 0.5). Labeled `dialect-people.<soft_label>` where `<soft_label>` comes from the existing `classify_dialect` (the macro read-out in `civ-culture-emergent.md`).

**Value-based identity** is when `boundary_marker = Value` AND `value_axis_distance > params.value_identity_threshold` (default 0.4) AND `value_bundle.clarity > 0.7` (the cluster is not in schism). Labeled `value-community.<axis_name>` (the dominant divergent axis).

**Norm-based identity** is when `boundary_marker = Norm` AND `norm_distance > params.norm_identity_threshold` (default 0.4). Labeled `norm-people.<dominant_norm_domain>`.

**Schism identity** is when `value_bundle.clarity < params.schism_clarity_threshold` AND `composite_distance > 0.7`. Labeled `schism.<in_group_label>` (signals an impending cluster split).

All labels are *soft* — they are descriptive strings the renderer can display, not authoritative enums. The partition is the real identity, not the label.

### 5.5 Identity stiffness — the feedback to the substrate

The *substrate* (social graph + culture drift) does not change in response to identity. Identity is a *read-out*, not a feedback loop. **However**, the *civ-emergence-metrics* dashboard (per `emergence-dashboard.md` §3) consumes `boundary_strength` as a polity-cohesion signal: high boundary strength across many adjacent clusters means polities are about to harden; low boundary strength across adjacent clusters means polities are about to blend. This is the *legitimate* downward coupling — through the shared substrate gradients, not through an API edge.

### 5.6 Identity never becomes `FactionSeed`

The existing `faction_emergence::cluster_into_factions` produces `FactionSeed`s from k-means on 8-dim `AgentIdeology`. That is the *emergent polity* (per `emergence-charter.md` §"Polities / states — decentralized"). Identity in this spec is a *prior* to faction formation: clusters with strong identity are *more likely* to form stable factions, clusters with weak identity are *more likely* to merge. The classifier emits this signal into the dashboard (the `identity_to_faction_signal` metric in §7), but **does not mutate** `faction_emergence`. The existing k-means remains the *only* path to faction creation; identity is an *advisory* read-out.

---

## 6. Ideology — the integrated macro bundle

### 6.1 What "ideology" is in this spec

**Ideology** in this spec is the *integrated macro bundle* that composes the four read-out layers (norms, values, art, identity) plus the CIV-0106 6-axis `IdeologyVector` (when a `RegionCohesion` is available for the cluster) plus the faction's 8-dim `ideology_centroid` into a single inspectable bundle. The bundle is what the inspector, the legends engine, the dashboard, and the 3D / web clients consume when the player asks "what is this faction's ideology?"

The bundle is *not* a replacement for the CIV-0106 6-axis `IdeologyVector`. The CIV-0106 vector is the *institutional / region-level* signal — how the institutional layer (state, shadow networks, foreign actors) is oriented. The bundle in this spec is the *community-level* signal — how the *community of agents* in a cluster is oriented. The two are related (the community orientation feeds upward into the institutional orientation via shared gradients) but they are not identical.

### 6.2 The ideology bundle — composed projection

```text
// DIAGNOSTIC — pure read-out, never stored on a faction or cluster
fn classify_ideology(
    cluster_id:        ClusterId,
    faction:           &FactionSeed,                    // may be absent if no faction has formed
    norms:             &[NormPattern],
    values:            &ValueSystemBundle,
    art:               &ArtSignature,
    identity:          &IdentitySignature,
    civ_0106_vector:   Option<&IdeologyVector>,         // region-level if available
    region_cohesion:   Option<&RegionCohesion>,
    tradition_label:   Option<&TraditionLabel>,         // from civ-culture-emergent
    params:            &CultureIdeologyParams,
) -> IdeologyBundle

struct IdeologyBundle {
    // The four macro read-out projections
    norms:             Vec<NormPattern>,
    values:            ValueSystemBundle,
    art:               ArtSignature,
    identity:          IdentitySignature,

    // The institutional layer signal (CIV-0106 — may be absent)
    civ_0106_vector:   Option<IdeologyVector>,
    civ_0106_distance_to_community: Option<f32>,   // how far the institutional orientation is from the community

    // The faction primitive (faction_emergence — may be absent)
    faction_centroid:  Option<[f32; 8]>,            // 8-dim AgentIdeology k-means input

    // The integrated composite
    cohesion_alignment: f32,                        // [0, 1] — does the community orientation align with institutional orientation?
    schism_pressure:   f32,                        // [0, 1] — schism signal from clarity (§3.4) + polarization (CIV-0106)
    tradition_label:   Option<String>,             // soft label carried from civ-culture-emergent
    tradition_strength: f32,                       // [0, 1] — bundle's combined weight (norms + values + rituals + identity)
}
```

### 6.3 What produces each ideology phenomenon

**Aligned ideology** is when `cohesion_alignment > params.ideology_alignment_threshold` (default 0.7) AND `schism_pressure < 0.3`. The community's norms + values agree with the institutional `IdeologyVector`; the faction is *cohesive* on its ideology. Labeled `aligned.<dominant_value_axis>`. The dashboard flags `MT-IDEO-001` (healthy alignment sustained > 4096 ticks).

**Split ideology** is when `cohesion_alignment < params.ideology_split_threshold` (default 0.3). The community and the institutional layer disagree; an *ideological split* is forming. The CIV-0106 insurgency propensity already reads its `polarization` input from this misalignment (per `CIV-0106` §1.6 — polarization is derived from inter-cohort variance). This spec adds the *cause* (the value-system drift that produced the split) to the *effect* (the polarization) the CIV-0106 model already measures.

**Schism ideology** is when `schism_pressure > params.ideology_schism_threshold` (default 0.6). The community is internally divided (low clarity from §3.4) AND/OR institutionally misaligned (low cohesion_alignment). A cluster split via `reconcile_membership` is likely.

**Authoritarian ideology** is when `coercion_intensity` (from CIV-0105) is high AND `value_axis_distance` between the cluster and its institutional layer is low AND `norm_density` (the fraction of domains with stable polarity) is high. The institutional layer has *codified* the cluster's norms; the cluster has lost its organic norm evolution and now follows institutional enforcement. Labeled `authoritarian.<codified_axis>`. The dashboard flags `MT-IDEO-002` (authoritarian drift).

**Anarchic ideology** is when `cohesion_alignment < 0.4` AND `coercion_intensity < 0.3` AND `value_axis_distance` between the cluster and its institutional layer is high. The cluster has *rejected* institutional ideology; norms are evolving freely. Labeled `anarchic.<dominant_value_axis>`.

All labels are *soft* — descriptive strings, not enums.

### 6.4 The ideology bundle is the inspector's primary read-out

The inspector (`crates/web` + `crates/civ-watch`) exposes the ideology bundle when the player clicks a cluster, faction, or region. The display is the bundle's fields rendered as a card: norms list, values card, art preview (color + ornamentation), identity label, CIV-0106 vector mini-chart, faction centroid if present, alignment + schism pressure gauges, tradition label if present. This is the single feature that makes emergent ideology *legible* (per `legends-engine.md` §0 the #1 moat is *legibility* of emergent depth).

### 6.5 The ideology bundle is the legends engine's cultural register input

`legends-engine.md` §3.4 lists `IdeologyShift` / `CulturalSpeciation` as `EventKind`s that the legends engine ingests. The bundle's `civ_0106_distance_to_community` + `schism_pressure` + `tradition_strength` time-series are the *magnitude drivers* for those events — when `civ_0106_distance_to_community` crosses `ideology_shift_threshold` (default 0.4 delta per sample), the bundle emits an `IdeologyShift` `RawSimEvent` into the legends bus. When `tradition_strength` rises above `tradition_formation_threshold` (default 0.7), the bundle emits a `CulturalSpeciation` event. These are *advisory* emissions — the legends engine ingests them as input; this spec does not write the legends-engine code.

---

## 7. Criticality knobs and dashboard metrics

### 7.1 Criticality knobs — edge of chaos

All knobs concentrate in a `CultureIdeologyParams` struct on `EmergenceState` (loaded from scenario RON, same pattern as `CultureEmergenceParams` in `civ-culture-emergent.md` §4). Defaults target weak emergence (Class 4): the system is in a *culturally-varied, ideologically-coherent, not-static, not-exploding* band — multiple stable identities, multiple aligned ideologies, norm surfaces that evolve, art signatures that vary by cluster but not so rapidly that the renderer cache thrashes.

| Parameter | Type | Default | Effect on norms/values/art/identity dynamics | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `norm_stability_threshold` | `u32` | `2` | Number of consecutive windows a polarity must hold to be labeled a norm (not noise) | `> 8` (almost nothing labeled a norm; cultural register silent on taboos) | `1` (every micro-flip labels; norm-spam) |
| `taboo_min_evidence` | `u32` | `8` | Minimum SocialEvent count for a Forbidden pattern to become a `Taboo` | `> 64` (no taboos ever form; sacred/forbidden values stay abstract) | `1` (every isolated negative event is a taboo; cultural chaos) |
| `sacred_threshold` | `f32` | `0.5` | Religion `Belief::strength` above which an axis is marked `sacred` | `> 0.95` (nothing is sacred; value system is purely pragmatic) | `0.0` (everything is sacred; no value can be traded off) |
| `taboo_threshold` | `f32` | `0.6` | Religion `Taboo::strength` above which an axis is marked `forbidden` | `> 0.95` (no forbidden axes; no sacred cows) | `0.0` (every belief is a taboo; cluster freezes) |
| `schism_clarity_threshold` | `f32` | `0.3` | Value clarity below which the cluster is in schism | `> 0.8` (almost never schism; cultural drift invisible) | `0.0` (constant schism; no stable value systems) |
| `value_drift_threshold` | `f32` | `0.05` | Per-sample per-axis delta flagged as drift | `> 0.5` (only massive shifts register; drift invisible) | `0.0` (every micro-drift is a drift event; dashboard noise) |
| `language_clarity_discount` | `f32` | `0.3` | How much language drift from founding discounts value clarity | `> 0.95` (values unreadable as language drifts; identity splits on language alone) | `0.0` (language has no effect on value; decoupling) |
| `art_invalidation_threshold` | `f32` | `0.15` | `cultural_distance` from cached signature at which the cache is invalidated | `> 0.5` (signature pins even as culture drifts; same art for diverged clusters) | `< 0.02` (signature thrashes every tick; renderer cache invalidates continuously) |
| `art_family_split_threshold` | `f32` | `0.20` | `signature_distance` at which two art signatures are different families | `> 0.8` (one art family per world) | `< 0.02` (every micro-drift is a new family; thousands of families) |
| `art_axis_drift_threshold` | `f32` | `0.02` | Per-axis change per sample flagged as drift | `> 0.5` (only massive shifts register) | `0.0` (every micro-change is drift) |
| `art_drift_stability_ticks` | `u32` | `4` | Consecutive drift samples before style family migration is declared | `> 32` (almost never migrates; stuck in founding family) | `1` (every drift is migration; family chaos) |
| `identity_distinctness_threshold` | `f32` | `0.5` | `composite_distance` above which identity is "distinct" (kinship/dialect/value/norm label fires) | `> 0.95` (almost no distinct identity; everyone is "us") | `< 0.05` (every micro-difference is distinct identity; identity chaos) |
| `dialect_identity_threshold` | `f32` | `0.5` | `dialect_distance` above which identity is dialect-based | `> 0.95` (no dialect-based identity) | `< 0.05` (every micro-dialect is an identity; thousands of identities) |
| `value_identity_threshold` | `f32` | `0.4` | `value_axis_distance` above which identity is value-based | `> 0.95` (no value-based identity) | `< 0.05` (every micro-value-difference is an identity) |
| `norm_identity_threshold` | `f32` | `0.4` | `norm_distance` above which identity is norm-based | `> 0.95` (no norm-based identity) | `< 0.05` (every micro-norm-difference is an identity) |
| `ideology_alignment_threshold` | `f32` | `0.7` | `cohesion_alignment` above which ideology is "aligned" | `> 0.95` (only perfect alignment labels; aligned label rare) | `< 0.3` (almost nothing aligned; permanent split narrative) |
| `ideology_split_threshold` | `f32` | `0.3` | `cohesion_alignment` below which ideology is "split" | `> 0.95` (no split ever labeled) | `0.0` (constant split; every cluster is fighting its institution) |
| `ideology_schism_threshold` | `f32` | `0.6` | `schism_pressure` above which ideology is "schism" | `> 0.95` (no schism ever labeled) | `0.0` (constant schism) |
| `ideology_shift_event_threshold` | `f32` | `0.4` | Per-sample delta in `civ_0106_distance_to_community` that emits an `IdeologyShift` event | `> 1.0` (no IdeologyShift events ever fire) | `0.0` (every micro-drift emits; legend-spam) |
| `tradition_formation_threshold` | `f32` | `0.7` | `tradition_strength` above which a `CulturalSpeciation` event fires | `> 1.0` (no CulturalSpeciation events ever fire) | `0.0` (every cluster forms a tradition immediately; tradition-spam) |

All knobs are grouped in one `CultureIdeologyParams` struct (not scattered across modules) and loaded from the scenario RON config. The emergence dashboard (§7.2) plots a real-time criticality indicator so the designer can see whether the system is heading toward heat-death (one identity, one ideology, no norm labels) or explosion (thousands of micro-identities, ideology-spam, art-cache thrash) before adjusting.

### 7.2 Dashboard metrics for the culture-ideology read-out

These metrics extend the **Emergence Dashboard** (`crates/civ-emergence-metrics/src/dashboard.rs` extension; companion to the `culture: CultureDashboard` field added by `civ-culture-emergent.md` §5). The dashboard's existing 5 tiles (`cluster_entropy`, `ideology_homophily`, `sentience_fraction`, `psyche_stability`, `diplomacy_tension`) remain unchanged; this spec adds the **`ideology: IdeologyDashboard`** field.

| Metric | How to compute | Target signature (healthy ideology) | Failure mode |
|---|---|---|---|
| **Norm count** | `count(UniqueNormPattern across clusters)` at the sample tick | Power-law: a few major norms + many micro-norms (α ∈ [1.5, 2.0]) | 0 (cultural heat-death); > `cluster_count × 4` (= every domain in every cluster; norm explosion) |
| **Taboo rate** | `count(taboo.<domain> patterns) / cluster_count` | Low and stable; taboos are *rare* and *sacred* | Zero (no religious reinforcement of norms); > 0.5/cluster (taboo-spam, value freeze) |
| **Value clarity median** | `median(clarity across clusters)` | Moderate (0.5..0.8) — clear enough to act on, schism-detectable | Near zero (no shared values); near 1.0 (cult-like unanimity) |
| **Value drift rate** | `count(drift events per sample) / cluster_count` | Low (< 0.05 per sample) — values are sticky | High (> 0.20 per sample) = rapid value churn |
| **Art signature diversity** | `count(distinct ArtSignature across clusters)` | Power-law: a few major art families + many micro-families | 1 (one art style per world); > `cluster_count` (every cluster its own art; renderer thrash) |
| **Art cache invalidation rate** | `count(art_cache invalidations this sample) / cluster_count` | Low (< 0.05 per sample) — signatures are sticky | High (> 0.20 per sample) = cache thrash |
| **Identity distinctness median** | `median(boundary_strength across clusters)` | Moderate (0.4..0.7) — clusters are recognizable but not isolated | Near zero (no identity; total cultural fusion); near 1.0 (total isolation; no inter-cluster relations) |
| **Identity marker entropy** | Shannon entropy of the `boundary_marker` distribution | Moderate (0.5..0.8) — multiple kinds of identity, not one dominant | Near zero (kinship-only identity; culture is purely biological); near 1.0 (chaos) |
| **Schism pressure median** | `median(schism_pressure across clusters)` | Moderate (0.2..0.5) — schism is detectable but rare | Near zero (no schism ever; cluster splits are never ideological); near 1.0 (constant schism) |
| **Ideology alignment median** | `median(cohesion_alignment across clusters)` | Moderate-to-high (0.5..0.8) — community and institution usually agree | Near zero (constant institutional-community mismatch); near 1.0 (perfect alignment; no organic culture) |
| **Ideology × norm MI** | `MI(ideology_alignment_bin, norm_domain_bin)` normalised by `H(ideology_alignment_bin)` | Moderate (0.2..0.5) — ideology is shaped by norms but not determined by them | Near 0 (ideology is independent of norms — fine, charter warning that the value-system → institutional channel may be broken); near 1.0 (ideology *is* norms) |
| **Ideology shift event rate** | `count(IdeologyShift events per sample) / cluster_count` | Low (< 0.05 per sample) | Zero (no IdeologyShift events fire); > 0.20 per sample = ideology-churn spam |
| **Tradition strength distribution** | Histogram of `tradition_strength` across clusters | Moderate (0.3..0.7) — traditions are forming but not all clusters have one | All zero (no traditions); all 1.0 (every cluster has a tradition — tradition-spam) |
| **`ideology_alignment` × `value_clarity` correlation** | Pearson r over 4096 ticks | Moderate positive (0.3..0.6) — clear values → aligned ideology | Near 0 (institutions ignore community values); near 1.0 (institutions fully determined by community values) |

The `emergence.metrics.v1` replay-bus event (per `emergence-dashboard.md` §5) is extended with the 14 `ideology_*` fields above. The `emergence.alarm.v1` event is extended with three new alarm IDs derived from this spec:

- **MT-IDEO-001** — `ideology_alignment` stuck below `ideology_split_threshold` for 4096 ticks across > 30% of clusters (institutional-community split epidemic).
- **MT-IDEO-002** — `value_clarity` stuck above 0.95 for 4096 ticks across > 30% of clusters (cult-like unanimity — charter warning in the *other* direction).
- **MT-IDEO-003** — `art_cache_invalidation_rate` > 0.20 per sample for 256 ticks (renderer cache thrash from over-fast culture drift).

---

## 8. What this spec does NOT include

- Any `enum Norm`, `enum Value`, `enum Art`, `enum Identity`, `enum Ideology` stored on any agent, cluster, faction, or polity. All macro labels are pure read-out functions over the substrate (§2.2, §3.3, §4.2, §5.3, §6.2).
- Any `set_culture`, `set_ideology`, `set_norm`, `set_value`, `set_art`, `set_identity` callable that mutates macro state independent of the underlying substrate. There is no way to "impose" a norm, a value, an art style, or an identity from code — they emerge (or they don't).
- Any direct `civ-agents` → `civ-engine` / `civ-diplomacy` / `civ-economy` API edge. All coupling is the shared substrate gradient (per the charter, mirroring `civ-culture-emergent.md` §2).
- Any re-implementation of `drift_populations`, `belief_culture_exposure`, `update_beliefs`, `apply_social_event`, `decay_social_graph`, `spread_religion`, `cluster_into_factions`, or the CIV-0106 `IdeologyField` diffusion step. The substrate is correct as of this spec's date.
- Any hardcoded `Style` enum, `Palette` enum, `Iconography` enum, `Symbol` enum. The art signature is a *style-parameter bundle*, not an authored taxonomy.
- Any music / audio component. Music is the domain of `audio-direction.md` / `emergent-systems-spec.md` §2.5 (E6); the `MusicalTradition` is a separate substrate read-out.
- Any change to the existing `civ-laws` crate. The `civ-laws` DB is the *physical-law* database; this spec's *norms* are the *proto-social-rules* that precede any institution. The codification path from norms to enforceable institution rules is a separate spec (CIV-0103 territory).
- Any change to `faction_emergence::cluster_into_factions`. The 8-dim k-means remains the *only* path to faction creation; identity is an *advisory* read-out that the dashboard consumes.
- Any change to `civ-culture-emergent.md` ownership. That spec owns dialect / script / ritual / tradition; this spec owns norms / values / art / identity / ideology. The two specs reference each other through shared primitives (`weighted_belief_centroid`, `language_distance`, `ritual_frequencies`, `TraditionLabel`) and shared substrate fields (`CultureProfile::traits`, `Psyche::beliefs`, `Tie::familiarity`, `SocialEvent`).
- Any LLM call in the culture-ideology path. The cultural register of the legends engine may use the existing LLM garnish for narrative embellishment (per `emergent-systems-spec.md` §2.6 / §2.10), but the *macro read-out* itself is a pure function over the substrate.

---

## 9. Out of scope / future work

The following are *adjacent* systems this spec acknowledges but does not specify:

- **Norm codification into institutional law** (CIV-0103 territory): the path from emergent norms → institutional rules → enforceable `policy` is owned by the institution spec. This spec produces the *substrate* that the institution spec would later read.
- **Identity-driven faction formation**: an *advisory* signal (the `identity_to_faction_signal` metric in §7.2) is emitted by the identity classifier; a future spec may wire that signal as a *k-means seeding input* to `faction_emergence` (still charter-safe — the k-means algorithm itself remains unchanged).
- **Cross-cluster identity negotiation**: two clusters with high identity may merge (a `IdentityMerger` event), three clusters may split (a `IdentitySchism` event). These are dashboard-level events; their causal modeling is owned by a future diplomacy spec.
- **Aesthetic cross-pollination**: two clusters in different art families may trade ornamentation patterns through a *measured* contact-edge process. The contact-edge builder in `civ-culture-emergent.md` P2-A may be extended to weight art-family pairs separately; that extension is owned by `civ-culture-emergent.md`, not this spec.
- **Per-civ phonotactic + material palette defaults**: the `MaterialPalette` data table is RON-loadable; a future PR will provide per-biome defaults (desert palette, rainforest palette, tundra palette, etc.) — owned by the `civ-planet` spec.

---

## 10. Phased implementation outline (deferred to the implementing agent)

This spec is *planner-only*. The implementing agent will define the exact file paths, function signatures, and test names. The phased outline below mirrors `civ-culture-emergent.md` §6 so the two specs can be implemented in parallel:

- **Phase 0** — Prerequisite audit: verify the substrate inventory in §1 is the complete read surface; confirm `weighted_belief_centroid` (per `civ-culture-emergent.md` P3-A) is the shared primitive.
- **Phase 1** — Add the five classifier functions: `classify_norms`, `classify_values`, `project_art` + `ArtSignature` cache, `classify_identity`, `classify_ideology`. Each is pure + read-only + additive.
- **Phase 2** — Add the `CultureIdeologyParams` RON-loadable struct + the `IdeologyDashboard` struct + `compute_ideology_metrics` pure fn. Extend the existing `EmergenceDashboard` with the new `ideology` field.
- **Phase 3** — Wire the metrics into `sample_emergence` + extend the `emergence.metrics.v1` replay-bus event + extend `emergence.alarm.v1` with the three new alarm IDs.
- **Phase 4** — Wire the `IdeologyShift` / `CulturalSpeciation` events into the legends engine ingest path (`legends` crate, `SourceCrate::Agents` `RawSimEvent` stream, new additive `EventKind` variants).
- **Phase 5** — Wire the `IdeologyBundle` into the inspector (`crates/web` + `crates/civ-watch`) as a primary read-out card.
- **Phase 6** — Acceptance criteria reachability: scenario tests for norm emergence from isolation, value clarity dynamics, art signature stability, identity distinctness, ideology alignment vs schism pressure. The dashboard regression test runs in CI as a performance-gated test.
- **Phase 7** — Coupling sanity audit: verify no production code path mutates the substrate fields read in §1 outside of their existing legitimate mutation sites. The CI invariant is the same as `civ-culture-emergent.md` P5-E.

The full DAG, file paths, test names, and CI integration steps are defined in the follow-up implementation PR (not this design spec).

---

## 11. Document authority

This spec defines the macro read-out + bidirectional coupling + criticality knobs + dashboard metrics for emergent **Norms**, **Values**, **Art**, **Identity**, and **Ideology**, on top of the existing `crates/agents/src/{psyche, social, culture, cluster}.rs` + `crates/engine/src/{religion, faction_emergence}.rs` + `CIV-0106 IdeologyField` + `LANGUAGE_EMERGENCE.md` substrate. The micro-drivers (`drift_populations`, `update_beliefs`, `apply_social_event`, `decay_social_graph`, `spread_religion`, `cluster_into_factions`, CIV-0106 diffusion step) are unchanged. The five classifier functions in §2–§6 are the only additions, and they are pure / read-only / never mutate the substrate. There is no other way for a norm, a value, an art style, an identity, or an ideology to exist — they are pure read-out functions over the substrate, per the charter.

The companion spec `civ-culture-emergent.md` owns dialect / script / ritual / tradition; this spec is its sibling. The two specs reference each other through shared primitives (`weighted_belief_centroid`, `language_distance`, `ritual_frequencies`, `TraditionLabel`) and shared substrate fields (`CultureProfile::traits`, `Psyche::beliefs`, `Tie::familiarity`, `SocialEvent`). Together, the two specs constitute the complete macro read-out for emergent culture and ideology in Civis.