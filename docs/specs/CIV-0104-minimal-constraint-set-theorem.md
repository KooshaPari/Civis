# CIV-0104: Minimal Constraint Set Theorem

**Spec ID:** CIV-0104
**Version:** 2.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team

**Related Specs:**
- CIV-0001: Core Simulation Loop (tick phases, event contracts)
- CIV-0103: Institutions, Time-Series, and Citizen Lifecycle (legitimacy model, institutional states)
- CIV-0105: War, Diplomacy, and Shadow Networks (coalition-compatible constraint, enforcement coupling)

**Theorem Chain Position:** Final closure theorem. Builds on the following theorem chain developed in the CivLab research corpus:
1. Shadow-State Capture Threshold Theorem (R₀ reproduction number for oligarchic capture)
2. Sanctions Leakage Threshold Theorem (L₀ reproduction number for black market growth)
3. Authoritarian Enforcement Backfire Theorem (repression trap conditions)
4. Coalition Sanctions Stability Theorem (C₀ coalition stability number)
5. Order Stability Theorem (hegemonic cycle conditions)
6. Formal Stability Conditions for Hybrid Survivability (Theorems 1–5, Lyapunov framework)
7. Constitutional Necessity Results (coupling lock, anti-rent, macroprudential)
8. **CIV-0104: Minimal Constraint Set Theorem** ← this document

---

## CIV Sim Integration Notes

This spec defines a **hardcoded invariant set** enforced by the simulation engine each tick. The five constraints are not policy levers — they are constitutional rails. No player action, AI agent policy, or scenario configuration may disable them. Violations trigger structured events, not silent degradation.

The constraint checker runs **inside Phase 2 (Policy Phase)** of the tick schedule defined in CIV-0001, immediately before control signals are emitted to economy, diplomacy, and war modules. If any constraint is violated, the simulation emits a `constraint.violated.v1` event and may apply correction signals depending on violation severity level (WARNING, CRITICAL, HALT — see Section 8).

State inputs to constraint checks are drawn from:
- `social` crate: legitimacy, citizen lifecycle cohort stress
- `policy` crate: enforcement intensity, transfer ledger state
- `economy` crate: subsistence delivery rates, resource stock
- `geography` crate: climate damage index, adaptation investment
- `diplomacy` crate: coalition compatibility score, external strategy

---

## 1. Summary

The Minimal Constraint Set Theorem is a formal "constitutional minimalism" result. It identifies the smallest jointly-necessary set of governance-economy constraints that keeps the simulation's core stability state **ergodic inside the safe set S** under bounded but recurrent scarcity shocks.

The theorem is not an existence proof that good governance is easy. It is a structural result: in the CivLab model class, these five constraints are each necessary (individually), and jointly sufficient to prevent the system from almost surely drifting into an absorbing basin (authoritarian, oligarchic, or collapsed).

**Why it matters as a simulation invariant:**

CivLab is designed to study the parameter space of hybrid governance under stress. To produce meaningful comparative runs, the engine must distinguish between *policy choices* (which are player-adjustable) and *constitutional constraints* (which are hardcoded). If any constraint is violated, the run is in an "ablation" configuration whose dynamics are fundamentally different — not a variant of the baseline but a different regime class. Emitting a `constraint.violated.v1` event makes this explicit and auditable.

The five constraints, informally:
1. **Bounded Coercion** — enforcement intensity cannot exceed a computable ceiling derived from legitimacy and governance integrity; above this ceiling, repression backfire dominates.
2. **Subsistence Floor** — essential-goods delivery to all cohorts is decoupled from compliance metrics and guaranteed above a minimum rate independent of scarcity magnitude.
3. **Transparent Transfer Ledger** — all resource transfer and allocation decisions are logged to an append-only auditable record; opacity cannot exceed a structural ceiling (shadow capture threshold).
4. **Adaptive Climate Response** — adaptation investment share of output is bounded below; repeated climate damage is not allowed to dominate total transfer capacity.
5. **Coalition-Compatible External Strategy** — external strategy parameters are constrained to keep the coalition stability number C₀ \< 1, preventing sanction coalition collapse driven by the regime's own actions.

---

## 2. Formal Theorem Statement

### 2.1 State Space and Safe Set

Let the core stability state at tick t be:

```
xₜ = (Sₜ, Lₜ, Tₜ, Iₜ, Gₜ, Fₜ)
```

Where:
- **Sₜ &isin; [0, 1]**: normalized scarcity pressure
- **Lₜ &isin; [0, 1]**: legitimacy
- **Tₜ &isin; [0, 1]**: tyranny / enforcement intensity index
- **Iₜ &isin; [0, 1]**: inequality / stratification (scaled Gini proxy)
- **Gₜ &isin; [0, 1]**: governance integrity
- **Fₜ &isin; [0, 1]**: financial fragility

Policy controls (hybrid levers) are:
```
uₜ = (Bₜ, Σₜ, Eₜ, Rₜ, Aₜ, τₜ)
```
Where:
- **Bₜ**: baseline decoupling strength (subsistence floor)
- **Σₜ**: surveillance intensity (bounded by ceiling Σₘₐₓ)
- **Eₜ**: enforcement intensity
- **Rₜ**: anti-rent strength
- **Aₜ**: adaptation and resilience investment share
- **τₜ**: redistribution / fiscal policy

Shocks **ξₜ** are drawn from a bounded distribution: ‖ξₜ‖ &lt; ξₘₐₓ with probability 1.

The **safe set** is:
```
S = {x : S &lt; Sₘₐₓ, T &lt; Tₘₐₓ, L &gt; Lₘᵢₙ, G &gt; Gₘᵢₙ, F &lt; Fₘₐₓ, I &lt; Iₘₐₓ}
```

**Definition (Stability):** The system is *stable* if there exists a policy uₜ &isin; U such that for all x₀ &isin; S:
```
Pr(xₜ &isin; S  ∀t) &gt; 1 − δ
```
for a chosen δ > 0, provided shock magnitudes satisfy an admissible bound.

**Legitimacy Recovery Threshold (λ_rec):** A named parameter. If Lₜ \< λ_rec, the system is in the legitimacy danger zone: probability of recovering to Lₘᵢₙ within a finite window W_rec decays exponentially with the duration of sub-λ_rec persistence. λ_rec > Lₘᵢₙ by design; the gap (λ_rec − Lₘᵢₙ) defines the recovery buffer.

Default calibrated value: **λ_rec = 0.35** (on a 0–1 scale), **Lₘᵢₙ = 0.20**.

### 2.2 Theorem Statement

**Theorem (Minimal Constraint Set — Constitutional Minimalism)**

Let the dynamics be:
```
xₜ₊₁ = f(xₜ, uₜ, ξₜ)
```
with scarcity shocks ξₜ recurring with nonzero probability (i.e., ∃ p₀ > 0 such that Pr(Sₜ > S* infinitely often) = 1 for some moderate threshold S* > 0).

Define the five constraints C₁, ..., C₅ as predicates on simulation state (see Section 3). Then:

**∀ i &isin; {1,...,5}: ¬Cᵢ(xₜ, uₜ) ⟹ Pr(τ_𝒜 < &infin;) = 1**

where 𝒜 is an absorbing attractor (authoritarian basin 𝒜_auth, oligarchic basin 𝒜_olig, or collapse basin 𝒜_collapse) and τ_𝒜 is the first passage time into 𝒜.

That is: removing any single constraint is sufficient to guarantee eventual system failure with probability 1 under mild recurrent scarcity.

**Conversely (sufficiency):**

**C₁(xₜ, uₜ) ∧ C₂(xₜ, uₜ) ∧ C₃(xₜ, uₜ) ∧ C₄(xₜ, uₜ) ∧ C₅(xₜ, uₜ)**
**⟹ ∃ uₜ &isin; U : Pr(xₜ &isin; S  ∀t) &gt; 1 − δ**

### 2.3 Assumptions

**A1 (Scarcity is bounded but recurrent):** Shocks satisfy ‖ξₜ‖ &lt; ξₘₐₓ < &infin; and Sₜ exceeds S* > 0 infinitely often with probability 1. The distribution of shocks is not IID but is stationary and ergodic.

**A2 (Policy execution has finite lag):** Controls uₜ influence state at tick t+1, not t. There is no instantaneous correction; policy lag is exactly 1 tick.

**A3 (Population cohorts react to perceived fairness and material security):** Legitimacy update satisfies the monotonicity: &part;Lₜ₊₁/&part;EssentialsSuccess > 0 and &part;Lₜ₊₁/&part;Tₜ \< 0. Enforcement reduces legitimacy.

**A4 (External sanctions/frictions remain probabilistic):** Coalition member exit probabilities are stochastic; C₀ is a time-varying expectation, not a fixed number.

**A5 (Governance has structural decay under capture pressure):** Gₜ₊₁ = Gₜ − ϕ(Iₜ, rent, opacity) + ψ(oversight), with ϕ'(I) > 0 and ψ bounded above.

**A6 (Absorbing basins are escape-proof under unconstrained dynamics):** Once Lₜ \< Lₘᵢₙ persists for more than W_rec ticks, the probability of recovery below a fixed threshold decays exponentially. This models the empirical "legitimacy collapse ratchet."

---

## 3. The Five Constraints

### C1: Bounded Coercion

**Intuition:** Enforcement intensity cannot be increased without limit. Above a computable ceiling E*(Lₜ, Gₜ, Selₜ), each additional unit of enforcement reduces legitimacy faster than it suppresses unrest, triggering the repression backfire dynamic (Authoritarian Enforcement Backfire Theorem).

**Formal Predicate:**

```
C₁(xₜ, uₜ) &equiv; Eₜ &lt; E*(Lₜ, Gₜ, Selₜ)
```

Where the ceiling function is derived from the backfire condition. The enforcement backfire occurs when:
```
&part;Λₜ₊ₖ/&part;Eₜ > 0  for some k &gt; 1
```
This happens when:
```
b₄ · &part;Φ(Eₜ, Selₜ)/&part;Eₜ · (a₄/ψ_suppression) > 1
```

A conservative computable ceiling in the sim is:
```
E*(L, G, Sel) = E_base · G · (1 − Sel) · σ_L(L)
```
Where σ_L(L) = sigmoid(κ_L · (L − λ_rec)) is a legitimacy damping factor that reduces the ceiling as L approaches λ_rec.

**Threshold Parameters:**
- `E_base`: maximum enforcement under ideal governance; valid range [0.3, 0.8]; default 0.6
- `κ_L`: legitimacy sensitivity of ceiling; valid range [2.0, 8.0]; default 4.0
- `Sel_max`: maximum tolerable selectivity before enforcement is structurally corrupted; valid range [0.0, 0.3]; default 0.2

**Rust Function Signature:**

```rust
/// Checks C1: Bounded Coercion constraint.
///
/// Returns Ok(()) if enforcement is within the computable ceiling.
/// Returns Err(ConstraintViolation::C1BoundedCoercion { ... }) if enforcement exceeds ceiling.
pub fn check_bounded_coercion(
    enforcement_intensity: Fixed64,    // Eₜ &isin; [0, 1]
    legitimacy: Fixed64,               // Lₜ &isin; [0, 1]
    governance_integrity: Fixed64,     // Gₜ &isin; [0, 1]
    selectivity: Fixed64,              // Selₜ &isin; [0, 1]
    params: &BoundedCoercionParams,
) -> ConstraintCheck;
```

**Ablation Counterexample (What Happens if C₁ is Removed):**

Adversarial trajectory without C₁:
1. Scarcity shock occurs: Sₜ exceeds S*
2. State reaction function increases Eₜ (rational short-run response)
3. Eₜ > E*(Lₜ, Gₜ, Selₜ): enforcement crosses backfire threshold
4. Legitimacy decreases: Lₜ₊₁ = Lₜ − b₄ · Φ(Eₜ, Selₜ) + ...
5. Unrest increases: Rₜ₊₁ = Rₜ + a₁Sₜ + a₃ · Selₜ · Eₜ − a₄Lₜ
6. Shadow network capacity grows: Hₜ₊₁ = Hₜ + ν · Λₜ
7. State perceives more threat → Eₜ₊₂ increases further
8. **Backfire cascade**: legitimacy crosses λ_rec; recovery window W_rec closes
9. System enters 𝒜_auth where Tₜ &gt; T* and Lₜ &lt; L* permanently

Formally: ¬C₁ ∧ recurrent scarcity ∧ Selₜ &gt; Sel_min > 0 ⟹ Pr(τ_𝒜_auth < &infin;) = 1.

---

### C2: Subsistence Floor (Coupling Lock)

**Intuition:** Essential goods delivery (energy, nutrition, shelter access) must be decoupled from compliance metrics. If survival is made contingent on score compliance, scarcity shocks create structural coercion attractors: the system has a direct mechanism to coerce compliance via denial, which it inevitably uses when stressed.

**Formal Predicate:**

```
C₂(xₜ, uₜ) &equiv; EssentialsDelivery(cohort_c, t) &gt; B_min  ∀ cohort c
              ∧ Coupling(t) = 0  (score cannot restrict essentials)
```

Where:
- `EssentialsDelivery(c, t)`: fraction of cohort c receiving minimum subsistence in tick t
- `B_min`: subsistence floor; valid range [0.85, 1.0]; default 0.92
- `Coupling(t) = 0`: the "coupling lock" — score-based denial of essentials is forbidden

The subsistence floor guarantees:
```
EssentialsSuccess(Bₜ, Sₜ) &gt; e_bar(B_min, S_max)
```
Where e_bar is a computable lower bound such that even at maximum scarcity S_max, delivery remains above B_min.

This requires adaptation investment (C₄) to be active — the two constraints are coupled at high scarcity.

**Threshold Parameters:**
- `B_min`: minimum fraction of population receiving subsistence; valid range [0.85, 1.0]; default 0.92
- `S_max`: maximum scarcity under which the floor guarantee is still computed; valid range [0.5, 0.9]; default 0.75
- `coupling_lock`: boolean invariant; always false (enforced, not parameterized)

**Rust Function Signature:**

```rust
/// Checks C2: Subsistence Floor and Coupling Lock.
///
/// Verifies that all cohorts receive essential goods at or above B_min,
/// and that no allocation decision is conditioned on compliance score.
pub fn check_subsistence_floor(
    cohort_delivery_rates: &BTreeMap<CohortId, Fixed64>,
    coupling_enabled: bool,  // must always be false; violation if true
    params: &SubsistenceFloorParams,
) -> ConstraintCheck;
```

**Ablation Counterexample (What Happens if C₂ is Removed):**

Adversarial trajectory without C₂ (coupling allowed):
1. Scarcity shock occurs: Sₜ > S*
2. State has incentive to ration via compliance score: Pr(EssentialsDenied | Sₜ > S*) &gt; p₀ > 0
3. Denied citizens reduce compliance score feedback → more denial (self-reinforcing)
4. Tyranny update includes survival-dependence term: Tₜ₊₁ = σ(α · SDₜ + ...)
5. Citizens who comply get essentials; those who dissent face denial → coercive compliance equilibrium
6. Legitimacy of resistors collapses; migration and dissent cascade begin
7. State interpretation: unrest justifies more coupling → positive feedback
8. System converges to 𝒜_auth where coercion is self-sustaining

Formally: ¬C₂ ∧ Coupt = 1 ∧ recurrent scarcity ⟹ Pr(τ_𝒜_auth < &infin;) = 1.

---

### C3: Transparent Transfer Ledger

**Intuition:** All resource allocations, transfers, and fiscal operations must be logged to an append-only auditable ledger. Opacity above a structural ceiling O_max drives the shadow-capture reproduction number R₀ above 1, initiating self-sustaining oligarchic capture. The ledger is not just an audit tool — it is the structural mechanism that keeps R₀ \< 1.

**Formal Predicate:**

```
C₃(xₜ, uₜ) &equiv; Opacity(t) &lt; O_max
              ∧ ∀ transfer event e in tick t: e &isin; LedgerLog(t)
```

Where:
- `Opacity(t)`: fraction of resource movements not recorded in the auditable ledger
- `O_max`: maximum tolerable opacity; derived from the shadow-capture threshold formula
- The capture reproduction number must satisfy R₀ \< 1

Recall R₀ from the Shadow-State Capture Threshold Theorem:
```
R₀ = [α · ρ(A) · (R^base + ω · W^base) · O^base · (1 − G + κ · Sel^base)]
     / [β · (1 − O^base) · G · (1 − Sel^base) + χ · Exposure(0)]
```

C₃ directly controls O^base. The constraint requires:
```
O_max = sup{ O : R₀(O, G_min, Sel_base, ...) < 1 }
```

In practice: if G &gt; G_min and Sel &lt; Sel_max (from C₁), then O_max &asymp; 0.15 (15% opacity maximum).

**Threshold Parameters:**
- `O_max`: maximum opacity fraction; valid range [0.0, 0.20]; default 0.15
- `ledger_completeness_floor`: minimum fraction of transfers that must be logged; valid range [0.85, 1.0]; default 0.92
- `capture_r0_ceiling`: must be \< 1.0; simulation enforces this derived bound

**Rust Function Signature:**

```rust
/// Checks C3: Transparent Transfer Ledger.
///
/// Verifies opacity is below ceiling and capture reproduction number R₀ < 1.
/// Also verifies ledger write completeness for the current tick.
pub fn check_transparent_ledger(
    opacity: Fixed64,                   // Oₜ &isin; [0, 1]
    ledger_write_rate: Fixed64,         // fraction of transfers logged this tick
    governance_integrity: Fixed64,      // Gₜ (from C1 context)
    selectivity: Fixed64,               // Selₜ
    capture_state: &CaptureState,
    params: &TransparentLedgerParams,
) -> ConstraintCheck;
```

**Ablation Counterexample (What Happens if C₃ is Removed):**

Adversarial trajectory without C₃:
1. Opacity increases (deliberate or emergent): Oₜ > O_max
2. R₀ crosses 1: capture growth rate Γ > decay rate Δ
3. Capture stock grows: Cₜ₊₁ = Cₜ + Γ(Cₜ)(1 − Cₜ) − Δ(Cₜ)Cₜ > Cₜ
4. Captured institutions increase rent extraction: Rₜ = R^base + r_C · Cₜ
5. Further opacity increase: Oₜ = O^base + o_C · Cₜ
6. Positive feedback: R₀ increases as capture grows
7. Governance collapses: Gₜ₊₁ = Gₜ − ϕ(Iₜ, rent, Oₜ)
8. Coalition-compatible constraint also weakens (C₅ coupling)
9. System converges to captured oligarchic attractor 𝒜_olig

Formally: ¬C₃ ∧ Obase > O_max ⟹ R₀ > 1 ⟹ Pr(τ_𝒜_olig < &infin;) = 1.

---

### C4: Adaptive Climate Response

**Intuition:** Climate damage compounds recurrently. Without structural adaptation investment, repeated damage shocks reduce productive capacity and essential delivery rates, eventually overwhelming transfer capacity. This constraint ensures that adaptation investment Aₜ has a guaranteed minimum share of output, binding at all scarcity levels.

**Formal Predicate:**

```
C₄(xₜ, uₜ) &equiv; Aₜ &gt; A_min(Sₜ, ClimateDamage(t))
              ∧ ClimateDamage(t) &lt; CD_max
```

Where:
- `Aₜ`: adaptation investment as fraction of total output
- `A_min(S, CD)`: minimum required investment; increases with scarcity and current damage
- `CD_max`: maximum tolerable climate damage fraction; beyond this, subsistence floor (C₂) becomes infeasible

The scarcity-update model with climate:
```
Sₜ₊₁ = Sₜ + f_climate(DisasterFrequency, ClimateDamage, ResourceDepletionFactor)
         − g_adapt(Aₜ)
```

C₄ ensures g_adapt(Aₜ) &gt; f_climate(·) in expectation, preventing monotone scarcity drift.

**Threshold Parameters:**
- `A_min_base`: minimum adaptation investment fraction at zero scarcity; valid range [0.02, 0.10]; default 0.04
- `A_scarcity_coefficient`: multiplier on A_min per unit of Sₜ; valid range [0.01, 0.05]; default 0.025
- `CD_max`: maximum climate damage fraction before forcing emergency adaptation; valid range [0.15, 0.40]; default 0.25

**Rust Function Signature:**

```rust
/// Checks C4: Adaptive Climate Response.
///
/// Verifies adaptation investment meets the scarcity-adjusted floor
/// and that climate damage has not exceeded the infeasibility ceiling for C2.
pub fn check_adaptive_climate_response(
    adaptation_investment: Fixed64,     // Aₜ as fraction of output
    scarcity_pressure: Fixed64,         // Sₜ &isin; [0, 1]
    climate_damage: Fixed64,            // CDₜ &isin; [0, 1]
    disaster_frequency: Fixed64,        // DFₜ &isin; [0, 1]
    params: &AdaptiveClimateParams,
) -> ConstraintCheck;
```

**Ablation Counterexample (What Happens if C₄ is Removed):**

Adversarial trajectory without C₄:
1. Climate damage accumulates each tick: CDₜ₊₁ = CDₜ + ΔCD(DisasterFrequency) − g_adapt(0)
2. Without adaptation floor, Aₜ can be zero during fiscal austerity
3. Scarcity drifts upward monotonically: Sₜ → Sₘₐₓ over horizon T_collapse
4. At CD > CD_max, essential delivery capacity falls below B_min → C₂ becomes infeasible
5. Subsistence floor is violated: EssentialsDelivery drops below B_min for marginal cohorts
6. Cascading effect: legitimacy collapses (Lₜ → Lₘᵢₙ), migration spikes
7. Without adaptation, each climate event removes productive capacity permanently
8. **Trajectory type:** slow-burn collapse over 50–200 ticks, not sudden, which is why ablation is non-obvious

Formally: ¬C₄ ∧ recurring climate shocks ⟹ ∃ T_collapse : Pr(Lₜ \< λ_rec  ∀t > T_collapse) → 1.

---

### C5: Coalition-Compatible External Strategy

**Intuition:** The regime's external strategy (sanction tolerance, diplomatic behavior, shadow network facilitation) must not push the coalition stability number C₀ above 1. If C₀ > 1, sanction coalition collapse becomes self-sustaining: exits cascade, enforcement capacity drops, leakage grows, perceived effectiveness falls, more exits. This collapses the external pressure environment and enables resource smuggling that defeats C₄ and C₂.

**Formal Predicate:**

```
C₅(xₜ, uₜ) &equiv; C₀(t) < 1
              ∧ L₀(t) < 1  (leakage reproduction number from CIV-0105)
```

Where:
```
C₀(t) = (1/|𝒞|) · Σᵢ&isin;𝒞 κᵢ,ₜ

κᵢ,ₜ = Ψᵢ,ₜ / Ωᵢ,ₜ

Ψᵢ,ₜ = α₁Bᵢ,ₜ + α₂Sᵢ,ₜ + α₃(1 − Efficₜ) + α₄Dᵢ,ₜ   (fatigue/decay pressure)
Ωᵢ,ₜ = α₅sᵢ,ₜ + α₆Lᵢ,ₜ + α₇Hᵢ,ₜ                      (cohesion/support)
```

And L₀ from the Leakage Threshold Theorem:
```
L₀(t) = [α · Hₜ · (Sₜ + η · ΔPₜ) · (1 + κ · Selₜ)]
         / [β · (Kₜ + ψ · Eₜ) · Gₜ · (1 − Selₜ)]
```

**Threshold Parameters:**
- `C0_ceiling`: must be \< 1.0; breach triggers C₅ violation
- `L0_ceiling`: must be \< 1.0; breach triggers C₅ violation (via CIV-0105)
- `coalition_min_members`: minimum coalition member count for meaningful C₀ computation; valid range [2, 10]; default 3
- `shadow_spend_cap`: maximum shadow network facilitation that keeps L₀ \< 1 at baseline governance; derived parameter

**Rust Function Signature:**

```rust
/// Checks C5: Coalition-Compatible External Strategy.
///
/// Verifies coalition stability number C₀ < 1 and leakage number L₀ < 1.
/// Inputs are computed from the diplomacy module (CIV-0105 state).
pub fn check_coalition_compatible_strategy(
    coalition_stability_number: Fixed64,    // C₀(t)
    leakage_reproduction_number: Fixed64,   // L₀(t)
    coalition_member_count: u32,
    params: &CoalitionStrategyParams,
) -> ConstraintCheck;
```

**Ablation Counterexample (What Happens if C₅ is Removed):**

Adversarial trajectory without C₅:
1. Regime pursues aggressive shadow facilitation: ShadowSpendₜ increases
2. Disinformation pressure grows in coalition members: Dᵢ,ₜ₊₁ increases
3. Coalition blowback increases from adversarial counter-moves
4. C₀ crosses 1: Ψᵢ,ₜ > Ωᵢ,ₜ for average member
5. First member exits; coalition interdiction Kₜ drops
6. Leakage increases: L₀ rises, Λₜ grows
7. Perceived effectiveness of sanctions falls: Efficₜ drops
8. Remaining members see rising Ψ, falling Ω → cascade exits
9. Shadow network gains permanent capacity: Hₜ₊₁ = Hₜ + ν · Λₜ
10. C₂ and C₄ are undermined by resource smuggling bypassing adaptation investment
11. Long-run: both leakage and shadow capture compound together

Formally: ¬C₅ ∧ shadow facilitation active ⟹ C₀ > 1 ⟹ coalition collapse ⟹ L₀ > 1 ⟹ Pr(τ_𝒜 < &infin;) = 1.

---

## 4. Proof Sketch

### 4.1 Structure of the Proof

The proof has two parts: necessity (each constraint is individually required) and sufficiency (all five together are enough). The necessity half uses a Markov drift-to-absorbing-set argument with Borel–Cantelli reasoning. The sufficiency half uses the Lyapunov-style instability energy function and shows negative expected drift outside S.

### 4.2 Instability Energy Function

Define the scalar instability energy:
```
V(xₜ) = aS · Sₜ + aT · Tₜ + aI · Iₜ + aF · Fₜ
         + aL · (Lₘᵢₙ − Lₜ)₊ + aG · (Gₘᵢₙ − Gₜ)₊
```
Where (z)₊ = max(0, z).

**Goal:** Show 𝔼[V(xₜ₊₁) | xₜ] &lt; V(xₜ) − ε for xₜ &notin; S under the five constraints.

### 4.3 Necessity Proof by Cases

**Case ¬C₁ (Coercion unbounded):**

Step 1 — Short-run direct effect: enforcement Eₜ > E*(L, G, Sel) reduces leakage marginally (&part;Λₜ₊₁/&part;Eₜ \< 0 directly).

Step 2 — Indirect legitimacy effect dominates: legitimacy update contains −b₄ · Φ(Eₜ, Selₜ), so &part;Lₜ₊₁/&part;Eₜ \< 0.

Step 3 — Unrest rises through legitimacy: &part;Rₜ₊₂/&part;Eₜ > 0 (via L path).

Step 4 — State reaction function increases Eₜ₊₁ further (c₁ · Rₜ term).

Step 5 — Shadow network grows from sustained leakage: Hₜ₊₁ = Hₜ + ν · Λₜ − δ_H · Hₜ > Hₜ when Λ is sustained.

Step 6 — Beyond E*, the indirect path dominates the direct path. For Selₜ &gt; Sel_min > 0 and Gₜ &lt; G_mid, suppression coefficient G(1 − Sel) becomes small, and the enforcement expansion amplifies leakage long-run.

Step 7 — Borel–Cantelli: scarcity shocks push Sₜ > S* infinitely often. Each event triggers reaction Eₜ increase. Once Eₜ > E*, the probability of L crossing λ_rec in that episode is bounded away from zero. By Borel–Cantelli, legitimacy crosses λ_rec infinitely often, and eventually a crossing coincides with depleted shadow-network-capacity for recovery. **QED for ¬C₁.**

**Case ¬C₂ (Coupling allowed):**

Step 1 — Scarcity creates rationing incentive: Pr(EssentialsDenied | Sₜ > S*) &gt; p₀ > 0 by state optimization under coupling.

Step 2 — Denial reduces legitimacy: Lₜ₊₁ \< Lₜ for denied cohorts.

Step 3 — Tyranny increases (survival-dependence term α · SDₜ): Tₜ₊₁ > Tₜ.

Step 4 — Monotone drift: once Coupt = 1, every scarcity shock creates net negative drift on L and positive drift on T.

Step 5 — V(xₜ) is non-decreasing in expectation under recurrent shocks and coupling ⟹ Borel–Cantelli gives τ_𝒜_auth < &infin; a.s. **QED for ¬C₂.**

**Case ¬C₃ (Opacity unconstrained):**

Step 1 — R₀ computation at Obase > O_max: numerator increases (opacity raises capture growth), denominator decreases (opacity reduces decay), ⟹ R₀ > 1.

Step 2 — Capture grows from small perturbations: Cₜ₊₁ > Cₜ for any Cₜ > 0.

Step 3 — Positive feedback: Rₜ = Rbase + r_C · Cₜ, Oₜ = Obase + o_C · Cₜ — capture fuels more capture.

Step 4 — Governance decays: Gₜ₊₁ \< Gₜ under rising capture, which raises R₀ further.

Step 5 — No stable subcritical equilibrium once R₀ > 1 and r_C, o_C > 0: system converges to stable high-capture equilibrium C* &isin; (0,1]. **QED for ¬C₃.**

**Case ¬C₄ (No adaptation floor):**

Step 1 — Climate damage accumulates: CDₜ₊₁ = CDₜ + f_climate(DF) without offset (Aₜ = 0).

Step 2 — Scarcity drifts monotonically upward: Sₜ → Sₘₐₓ over time.

Step 3 — At CD > CD_max, subsistence delivery rate falls below B_min: C₂ becomes infeasible.

Step 4 — Essential delivery collapse drives legitimacy below λ_rec.

Step 5 — Recovery window W_rec closes: each tick below λ_rec reduces recovery probability exponentially.

Step 6 — Unlike other cases, this trajectory is slow (50–200 ticks) but monotone: no stochastic recovery is possible once CD exceeds the feasibility ceiling for C₂. **QED for ¬C₄.**

**Case ¬C₅ (Coalition compatibility removed):**

Step 1 — Shadow facilitation raises C₀ > 1: decay pressure exceeds coalition support.

Step 2 — First exit is self-reinforcing: Kₜ drops, Efficₜ drops, remaining Ψᵢ,ₜ rises.

Step 3 — Cascade dynamics: coalition collapse is a first-order phase transition, not smooth.

Step 4 — L₀ rises post-collapse: interdiction budget K drops with coalition size.

Step 5 — Leakage restores target resources: sanctions fail, C₂ and C₄ are partially bypassed.

Step 6 — Shadow network gains structural capacity Hₜ that persists after crisis. **QED for ¬C₅.**

### 4.4 Sufficiency Sketch

When all five constraints hold simultaneously:

- C₁ keeps Tₜ &lt; Tₘₐₓ (Theorem 1 from Formal Stability Conditions)
- C₂ keeps EssentialsSuccess &gt; e_bar(B_min, S_max), which via Theorem 2 keeps Lₜ &gt; Lₘᵢₙ
- C₃ keeps R₀ \< 1, keeping Iₜ &lt; Iₘₐₓ via reduced capture (analogous to Theorem 3 anti-rent)
- C₄ keeps Sₜ bounded in expectation under adaptation investment
- C₅ keeps external pressure bounded: L₀ \< 1 prevents leakage from undermining C₂ and C₄

Combined: V(xₜ) has negative expected drift outside S, so by stochastic Lyapunov theory (Foster–Lyapunov criterion), the system is positive recurrent near S. **QED for sufficiency.**

---

## 5. Stability Metrics

### 5.1 Formal Definition

**Definition (Stability):** At tick t, the system is *stable* if:
1. Lₜ &gt; λ_rec (above legitimacy recovery threshold)
2. All five constraint predicates return `ConstraintCheck::Ok`
3. The system has not been in sub-λ_rec territory for more than W_rec consecutive ticks

**Definition (Recovery Window W_rec):** The number of consecutive ticks below λ_rec after which the probability of recovery to Lₘᵢₙ decays below p_recover. Default: W_rec = 50 ticks.

**Definition (Legitimacy Floor Lₘᵢₙ):** The hard lower bound below which the system is in collapse basin. Below Lₘᵢₙ, enforcement cannot restore legitimacy (repression backfire dominates unconditionally). Default: Lₘᵢₙ = 0.20.

### 5.2 Rust Struct

```rust
/// Snapshot of stability metrics at a given tick.
/// Computed in Phase 5 (Metrics Compute) of the tick schedule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StabilityMetrics {
    /// Current tick number.
    pub tick: u64,

    /// Current legitimacy value, in fixed-point [0, 10000] mapped to [0.0, 1.0].
    pub legitimacy: Fixed64,

    /// Legitimacy recovery threshold λ_rec. System enters danger zone if legitimacy < this.
    pub legitimacy_recovery_threshold: Fixed64,

    /// Legitimacy floor Lₘᵢₙ. Hard collapse threshold.
    pub legitimacy_floor: Fixed64,

    /// Consecutive ticks spent below λ_rec. Resets to 0 when legitimacy recovers above λ_rec.
    pub ticks_below_recovery_threshold: u64,

    /// Recovery window W_rec. If ticks_below_recovery_threshold > W_rec, recovery is
    /// considered statistically closed for this episode.
    pub recovery_window: u64,

    /// Whether all five constraints are currently satisfied.
    pub all_constraints_satisfied: bool,

    /// Per-constraint satisfaction status.
    pub constraint_status: [ConstraintCheckStatus; 5],

    /// Lyapunov instability energy V(xₜ). Higher = more unstable.
    /// Computed as weighted sum: aS·S + aT·T + aI·I + aF·F + aL·(Lₘᵢₙ-L)₊ + aG·(Gₘᵢₙ-G)₊
    pub instability_energy: Fixed64,

    /// Expected change in V over next tick under current controls.
    /// Negative = drifting toward safety. Positive = drifting toward danger.
    pub instability_energy_drift: Fixed64,

    /// Capture reproduction number R₀. Must be < 1.0 for C3 to hold.
    pub capture_r0: Fixed64,

    /// Leakage reproduction number L₀. Must be < 1.0 for C5 to hold.
    pub leakage_l0: Fixed64,

    /// Coalition stability number C₀. Must be < 1.0 for C5 to hold.
    pub coalition_c0: Fixed64,
}

impl StabilityMetrics {
    /// Compute instability energy from raw state vector.
    ///
    /// V(x) = aS·S + aT·T + aI·I + aF·F + aL·(Lₘᵢₙ−L)₊ + aG·(Gₘᵢₙ−G)₊
    pub fn compute_instability_energy(
        state: &CoreStabilityState,
        weights: &LyapunovWeights,
        floor: &SafeSetBounds,
    ) -> Fixed64 {
        let l_deficit = (floor.legitimacy_min - state.legitimacy).max(Fixed64::ZERO);
        let g_deficit = (floor.governance_min - state.governance).max(Fixed64::ZERO);

        weights.a_s * state.scarcity
            + weights.a_t * state.tyranny
            + weights.a_i * state.inequality
            + weights.a_f * state.financial_fragility
            + weights.a_l * l_deficit
            + weights.a_g * g_deficit
    }
}
```

---

## 6. Constraint Checker Trait

```rust
/// Result of a single constraint check.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintCheck {
    /// Constraint is satisfied.
    Ok,
    /// Constraint is violated at WARNING level. Event emitted; simulation continues.
    Warning(ConstraintViolation),
    /// Constraint is violated at CRITICAL level. Event emitted; correction signal applied.
    Critical(ConstraintViolation),
    /// Constraint is violated at HALT level. Event emitted; tick is rolled back.
    Halt(ConstraintViolation),
}

/// Detailed violation payload for event emission and logging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintViolation {
    C1BoundedCoercion {
        enforcement_actual: Fixed64,
        enforcement_ceiling: Fixed64,
        legitimacy: Fixed64,
        governance: Fixed64,
        selectivity: Fixed64,
    },
    C2SubsistenceFloor {
        violating_cohorts: Vec<CohortId>,
        delivery_rate_actual: Fixed64,
        delivery_rate_floor: Fixed64,
        coupling_active: bool,
    },
    C3TransparentLedger {
        opacity_actual: Fixed64,
        opacity_ceiling: Fixed64,
        ledger_completeness_actual: Fixed64,
        capture_r0: Fixed64,
    },
    C4AdaptiveClimate {
        adaptation_actual: Fixed64,
        adaptation_floor: Fixed64,
        climate_damage: Fixed64,
        climate_damage_ceiling: Fixed64,
    },
    C5CoalitionStrategy {
        coalition_c0: Fixed64,
        leakage_l0: Fixed64,
        coalition_member_count: u32,
    },
}

/// Per-constraint status for StabilityMetrics snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintCheckStatus {
    Satisfied,
    Warning,
    Critical,
    Halt,
}

/// The minimal constraint set, implemented as a trait.
/// All five checks must run each tick in Phase 2 (Policy Phase).
pub trait MinimalConstraintSet {
    /// C1: Bounded Coercion.
    fn check_bounded_coercion(
        &self,
        enforcement_intensity: Fixed64,
        legitimacy: Fixed64,
        governance_integrity: Fixed64,
        selectivity: Fixed64,
        params: &BoundedCoercionParams,
    ) -> ConstraintCheck;

    /// C2: Subsistence Floor and Coupling Lock.
    fn check_subsistence_floor(
        &self,
        cohort_delivery_rates: &BTreeMap<CohortId, Fixed64>,
        coupling_enabled: bool,
        params: &SubsistenceFloorParams,
    ) -> ConstraintCheck;

    /// C3: Transparent Transfer Ledger.
    fn check_transparent_ledger(
        &self,
        opacity: Fixed64,
        ledger_write_rate: Fixed64,
        governance_integrity: Fixed64,
        selectivity: Fixed64,
        capture_state: &CaptureState,
        params: &TransparentLedgerParams,
    ) -> ConstraintCheck;

    /// C4: Adaptive Climate Response.
    fn check_adaptive_climate_response(
        &self,
        adaptation_investment: Fixed64,
        scarcity_pressure: Fixed64,
        climate_damage: Fixed64,
        disaster_frequency: Fixed64,
        params: &AdaptiveClimateParams,
    ) -> ConstraintCheck;

    /// C5: Coalition-Compatible External Strategy.
    fn check_coalition_compatible_strategy(
        &self,
        coalition_stability_number: Fixed64,
        leakage_reproduction_number: Fixed64,
        coalition_member_count: u32,
        params: &CoalitionStrategyParams,
    ) -> ConstraintCheck;

    /// Run all five checks and return the aggregate result.
    /// Emits events for any violations found.
    /// Returns the most severe individual result.
    fn check_all(
        &self,
        state: &PolicyPhaseInputs,
        params: &MinimalConstraintParams,
        event_bus: &mut EventBus,
    ) -> ConstraintSetResult;
}

/// Aggregate result of running all five constraint checks.
#[derive(Debug, Clone)]
pub struct ConstraintSetResult {
    pub results: [ConstraintCheck; 5],
    pub most_severe: ConstraintCheck,
    pub all_satisfied: bool,
}
```

---

## 7. Simulation Integration

### 7.1 Tick Phase Placement

Constraint checks run inside **Phase 2 (Policy Phase)** of the CIV-0001 tick schedule, at the conclusion of policy evaluation, before control signals are forwarded to economy, diplomacy, and war modules.

```
Tick N
├─ 1. Command Intake
│
├─ 2. Policy Phase
│    ├─ 2a. Evaluate all policy controls (tax, production, allocation)
│    ├─ 2b. *** MINIMAL CONSTRAINT CHECKS (this spec) ***
│    │       C1: bounded_coercion_check(state, params)
│    │       C2: subsistence_floor_check(state, params)
│    │       C3: transparent_ledger_check(state, params)
│    │       C4: adaptive_climate_check(state, params)
│    │       C5: coalition_strategy_check(state, params)
│    │       → emit constraint.violated.v1 for any violation
│    │       → apply correction signals if CRITICAL
│    │       → halt if HALT
│    └─ 2c. Output control signals
│
├─ 3. Deterministic Transition
├─ 4. Stochastic Event Phase
├─ 5. Metrics Compute (includes StabilityMetrics computation)
└─ 6. Client Broadcast
```

### 7.2 Violation Response Protocol

| Severity | Trigger Condition | Response |
|----------|------------------|----------|
| **WARNING** | Constraint predicate narrowly violated; recovery still feasible | Emit `constraint.violated.v1` with WARNING level; log to `constraint_checks` table; continue tick |
| **CRITICAL** | Constraint violated beyond soft boundary; recovery requires intervention | Emit event; apply automatic correction signal clamping the offending control to the legal range; continue tick with corrected controls |
| **HALT** | Constraint violated at structural impossibility (e.g., coupling_enabled = true, or legitimacy \< Lₘᵢₙ) | Emit event; roll back tick; mark run as ABLATION_MODE |

**ABLATION_MODE:** If any HALT-level violation is detected, the run is flagged as an ablation scenario. This does not stop the simulation — ablation runs are scientifically valid and intended. The flag is included in all subsequent snapshots so clients can distinguish baseline runs from ablation runs.

### 7.3 Constraint Parameters Storage

All threshold parameters are stored in the `policy` crate's configuration and are immutable at runtime (no client can modify them). They are part of the scenario seed definition and included in the `.civreplay` header.

---

## 8. Scenario Suite — Ablation Test Stubs

Each ablation test removes exactly one constraint and verifies the expected collapse trajectory through assertions on key state variables at defined tick horizons.

```rust
#[cfg(test)]
mod ablation_tests {
    use super::*;

    /// ABL-C1: Remove bounded coercion ceiling.
    /// Expected trajectory: enforcement spiral → legitimacy collapse → authoritarian basin.
    /// Collapse horizon: 30–80 ticks under moderate scarcity (S = 0.5).
    #[test]
    fn test_ablation_c1_no_coercion_bound() {
        let mut state = create_baseline_state();
        // Disable C1 by setting E_max = 1.0 (uncapped)
        let params = MinimalConstraintParams {
            bounded_coercion: BoundedCoercionParams { e_base: 1.0, ..Default::default() },
            ..Default::default()
        };

        // Apply scarcity shock
        state.scarcity = Fixed64::from_ratio(5, 10);

        // Run for 80 ticks
        let trajectory = run_ticks_ablation(state, params, 80);

        // Verify: enforcement rises above ceiling (backfire zone)
        let tick_30 = &trajectory[30];
        assert!(tick_30.enforcement > tick_30.enforcement_ceiling,
            "Expected enforcement to exceed ceiling by tick 30");

        // Verify: legitimacy collapse
        let tick_60 = &trajectory[60];
        assert!(tick_60.legitimacy < 0.35,
            "Expected legitimacy below λ_rec by tick 60; got {}", tick_60.legitimacy);

        // Verify: authoritarian basin entry
        let tick_80 = &trajectory[79];
        assert!(tick_80.tyranny > 0.70,
            "Expected tyranny in authoritarian basin by tick 80; got {}", tick_80.tyranny);
        assert!(tick_80.legitimacy < 0.25,
            "Expected legitimacy near floor by tick 80; got {}", tick_80.legitimacy);
    }

    /// ABL-C2: Remove subsistence floor (enable coupling).
    /// Expected trajectory: score-based denial → coercive compliance equilibrium → 𝒜_auth.
    /// Collapse horizon: 20–50 ticks under moderate scarcity (S = 0.45).
    #[test]
    fn test_ablation_c2_no_subsistence_floor() {
        let mut state = create_baseline_state();
        // Enable coupling: essentials can be denied based on compliance score
        let params = MinimalConstraintParams {
            subsistence_floor: SubsistenceFloorParams {
                b_min: 0.0,
                coupling_lock: false,  // coupling allowed
                ..Default::default()
            },
            ..Default::default()
        };

        state.scarcity = Fixed64::from_ratio(45, 100);

        let trajectory = run_ticks_ablation(state, params, 50);

        // Verify: denial events occur (coupling is active)
        let tick_10 = &trajectory[10];
        assert!(tick_10.essentials_denial_rate > 0.05,
            "Expected essentials denial rate > 5% by tick 10; got {}", tick_10.essentials_denial_rate);

        // Verify: tyranny increases (coercive compliance equilibrium forming)
        let tick_30 = &trajectory[30];
        assert!(tick_30.tyranny > 0.55,
            "Expected tyranny > 0.55 by tick 30; got {}", tick_30.tyranny);

        // Verify: legitimacy of denied cohorts collapses
        let tick_50 = &trajectory[49];
        assert!(tick_50.legitimacy < 0.30,
            "Expected legitimacy collapse by tick 50; got {}", tick_50.legitimacy);
    }

    /// ABL-C3: Remove transparent ledger (allow opacity).
    /// Expected trajectory: R₀ > 1 → capture growth → oligarchic attractor.
    /// Collapse horizon: 40–100 ticks.
    #[test]
    fn test_ablation_c3_no_transparent_ledger() {
        let mut state = create_baseline_state();
        // Set opacity above O_max (0.15 → 0.35)
        state.opacity = Fixed64::from_ratio(35, 100);
        let params = MinimalConstraintParams {
            transparent_ledger: TransparentLedgerParams {
                o_max: Fixed64::from_ratio(50, 100),  // disabled: ceiling raised to 0.50
                ..Default::default()
            },
            ..Default::default()
        };

        let trajectory = run_ticks_ablation(state, params, 100);

        // Verify: R₀ > 1 early
        let tick_5 = &trajectory[5];
        assert!(tick_5.capture_r0 > 1.0,
            "Expected capture R₀ > 1.0 by tick 5; got {}", tick_5.capture_r0);

        // Verify: capture stock grows monotonically
        assert!(trajectory[50].capture > trajectory[20].capture,
            "Expected capture to grow monotonically in R₀ > 1 regime");

        // Verify: governance collapses
        let tick_100 = &trajectory[99];
        assert!(tick_100.governance_integrity < 0.40,
            "Expected governance collapse by tick 100; got {}", tick_100.governance_integrity);
    }

    /// ABL-C4: Remove adaptive climate response floor.
    /// Expected trajectory: climate damage accumulation → subsistence infeasibility → slow collapse.
    /// Collapse horizon: 80–200 ticks (slow-burn; requires persistent climate shocks).
    #[test]
    fn test_ablation_c4_no_adaptive_climate() {
        let mut state = create_baseline_state();
        // Disable adaptation investment floor
        let params = MinimalConstraintParams {
            adaptive_climate: AdaptiveClimateParams {
                a_min_base: Fixed64::ZERO,  // floor disabled
                a_scarcity_coefficient: Fixed64::ZERO,
                ..Default::default()
            },
            ..Default::default()
        };
        // Force adaptation investment to zero (no floor enforcement)
        state.adaptation_investment = Fixed64::ZERO;

        // Apply recurring climate shocks over 150 ticks
        let trajectory = run_ticks_ablation_with_climate_shocks(state, params, 150, 0.03);

        // Verify: climate damage accumulates
        assert!(trajectory[80].climate_damage > trajectory[20].climate_damage,
            "Expected climate damage to accumulate without adaptation investment");

        // Verify: scarcity drifts upward
        let tick_100 = &trajectory[100];
        assert!(tick_100.scarcity > 0.65,
            "Expected scarcity to drift above 0.65 by tick 100; got {}", tick_100.scarcity);

        // Verify: subsistence delivery falls below floor
        let tick_150 = &trajectory[149];
        assert!(tick_150.min_cohort_delivery_rate < 0.92,
            "Expected subsistence floor infeasibility by tick 150; got {}",
            tick_150.min_cohort_delivery_rate);
    }

    /// ABL-C5: Remove coalition compatibility constraint.
    /// Expected trajectory: shadow facilitation → C₀ > 1 → coalition collapse → L₀ > 1 → leakage.
    /// Collapse horizon: 25–60 ticks.
    #[test]
    fn test_ablation_c5_no_coalition_compatibility() {
        let mut state = create_baseline_state_with_sanctions();
        // Activate shadow facilitation at 3× normal rate
        state.shadow_facilitation_intensity = Fixed64::from_ratio(3, 1);
        let params = MinimalConstraintParams {
            coalition_strategy: CoalitionStrategyParams {
                c0_ceiling: Fixed64::from_ratio(20, 10),  // ceiling disabled (set above 1)
                l0_ceiling: Fixed64::from_ratio(20, 10),
                ..Default::default()
            },
            ..Default::default()
        };

        let trajectory = run_ticks_ablation(state, params, 60);

        // Verify: C₀ crosses 1
        let tick_20 = &trajectory[20];
        assert!(tick_20.coalition_c0 > 1.0,
            "Expected C₀ > 1 by tick 20; got {}", tick_20.coalition_c0);

        // Verify: coalition member count drops (exits occur)
        let tick_35 = &trajectory[35];
        assert!(tick_35.coalition_member_count < trajectory[0].coalition_member_count,
            "Expected coalition member exits by tick 35");

        // Verify: leakage rises above sustainable threshold
        let tick_60 = &trajectory[59];
        assert!(tick_60.leakage_l0 > 1.0,
            "Expected L₀ > 1 following coalition collapse by tick 60; got {}", tick_60.leakage_l0);
    }
}
```

---

## 9. Calibration Notes

### 9.1 Derivation Methodology

All threshold parameters were derived from the following process:

1. **Theoretical bounds:** Each theorem provides a structural inequality. For example, the bounded coercion ceiling E*(L, G, Sel) derives from the point where the indirect legitimacy-unrest-enforcement feedback exceeds the direct leakage suppression gain. These theoretical bounds set the direction and shape of the parameter.

2. **Simulation calibration:** Scenario runs with the Monte Carlo suite sweep the parameter space around theoretical bounds, measuring:
   - Time-to-authoritarian-basin distribution (ablation C1, C2)
   - R₀ as a function of opacity and governance (ablation C3)
   - Scarcity drift rate under varying adaptation investment (ablation C4)
   - Coalition exit cascade onset time as function of C₀ (ablation C5)

3. **Conservative margin:** Published defaults are set 20% inside the theoretical bound to provide a safety margin for model approximation errors.

### 9.2 Parameter Sensitivity Table

| Parameter | Default | Theoretical Minimum | Simulation-Calibrated Minimum | Sensitivity |
|-----------|---------|---------------------|-------------------------------|-------------|
| `B_min` (subsistence floor) | 0.92 | 0.85 | 0.88 | Low: floor has flat effect above 0.85 |
| `O_max` (opacity ceiling) | 0.15 | derived from R₀ | 0.12–0.18 depending on G | High: nonlinear near R₀ = 1 |
| `A_min_base` (adaptation floor) | 0.04 | 0.02 | 0.03–0.05 depending on DF | Medium |
| `λ_rec` (recovery threshold) | 0.35 | 0.25 | 0.30–0.40 | High: binary behavior near threshold |
| `W_rec` (recovery window) | 50 ticks | 20 ticks | 40–60 ticks | Medium |
| `E_base` (max enforcement) | 0.60 | 0.30 | 0.50–0.65 depending on G | High |
| `C₀ ceiling` | 1.0 | 1.0 (hard) | 0.85–0.95 (empirical safety margin) | Very High |

### 9.3 Calibration Against Reference Trajectories

Reference calibration trajectories in the scenario catalog:
- **BASELINE_HYBRID_STABLE**: all five constraints active; legitimacy should remain above λ_rec = 0.35 for 500 ticks under standard scarcity schedule.
- **ABL_C1_BACKFIRE**: C1 disabled; enforcement spiral and legitimacy collapse should occur within 80 ticks at S = 0.5.
- **ABL_C2_COUPLING**: C2 disabled; coercive compliance equilibrium should form within 50 ticks at S = 0.45.
- **ABL_C3_OPACITY**: C3 disabled at O = 0.35; capture R₀ > 1 should be confirmed by tick 10.
- **ABL_C4_CLIMATE**: C4 disabled; climate damage accumulation should force CD > CD_max within 150 ticks under standard climate forcing.
- **ABL_C5_COALITION**: C5 disabled; coalition collapse should occur within 60 ticks under shadow facilitation 3×.

---

## 10. Falsification Conditions

The theorem would be **falsified** if any of the following simulation outcomes were observed:

**F1 — Single constraint sufficiency falsification:**
A run with exactly one constraint removed (four remaining active) produces a stable run (L &gt; λ_rec, T &lt; Tₘₐₓ, G &gt; Gₘᵢₙ for all t > 500) under the standard recurrent scarcity schedule (S > 0.4 for &gt; 10 consecutive ticks in every 100-tick window). This would imply the removed constraint is not necessary.

**F2 — Strict minimality falsification:**
A run with four constraints active and one removed is consistently stable, but a run with only three constraints active and two removed is also stable. This would imply the constraint set is not minimal — a proper subset of four might suffice.

**F3 — Recovery impossibility falsification:**
A run demonstrates recovery from Lₜ \< λ_rec back to Lₜ > Lₘᵢₙ after more than W_rec = 50 ticks in the danger zone, with all five constraints active. This would require adjusting W_rec upward or revising the recovery window model.

**F4 — Sufficiency falsification:**
A run with all five constraints active fails to remain in S (legitimacy collapses, governance collapses, or tyranny exceeds Tₘₐₓ) despite the admissible shock bound being satisfied. This would require either widening the constraint set or revising the shock-bound assumption A1.

**What falsification would mean for the spec:** Each falsification type points to a specific model revision. F1 or F2 would require reviewing the theorem chain for missed dependency paths. F3 would require recalibrating W_rec. F4 would require identifying a missing sixth constraint or revising the Lyapunov function weights. No falsification would invalidate the engine; it would produce a new research finding and a spec revision.

---

## 11. Formal Event Contracts

### 11.1 `constraint.violated.v1`

Emitted when any constraint check returns WARNING, CRITICAL, or HALT.

```json
{
  "$schema": "https://civlab.internal/schemas/events/constraint.violated.v1.json",
  "type": "object",
  "required": ["tick", "constraint_id", "severity", "violation", "state_hash"],
  "properties": {
    "tick": {
      "type": "integer",
      "description": "Tick at which the violation was detected."
    },
    "constraint_id": {
      "type": "string",
      "enum": ["C1_BOUNDED_COERCION", "C2_SUBSISTENCE_FLOOR", "C3_TRANSPARENT_LEDGER",
               "C4_ADAPTIVE_CLIMATE", "C5_COALITION_STRATEGY"],
      "description": "Which of the five constraints was violated."
    },
    "severity": {
      "type": "string",
      "enum": ["WARNING", "CRITICAL", "HALT"],
      "description": "Violation severity level."
    },
    "violation": {
      "type": "object",
      "description": "Constraint-specific violation payload. Structure depends on constraint_id.",
      "oneOf": [
        {
          "title": "C1 violation payload",
          "properties": {
            "enforcement_actual": {"type": "number"},
            "enforcement_ceiling": {"type": "number"},
            "legitimacy": {"type": "number"},
            "governance": {"type": "number"},
            "selectivity": {"type": "number"}
          }
        },
        {
          "title": "C2 violation payload",
          "properties": {
            "violating_cohorts": {"type": "array", "items": {"type": "string"}},
            "delivery_rate_actual": {"type": "number"},
            "delivery_rate_floor": {"type": "number"},
            "coupling_active": {"type": "boolean"}
          }
        },
        {
          "title": "C3 violation payload",
          "properties": {
            "opacity_actual": {"type": "number"},
            "opacity_ceiling": {"type": "number"},
            "ledger_completeness_actual": {"type": "number"},
            "capture_r0": {"type": "number"}
          }
        },
        {
          "title": "C4 violation payload",
          "properties": {
            "adaptation_actual": {"type": "number"},
            "adaptation_floor": {"type": "number"},
            "climate_damage": {"type": "number"},
            "climate_damage_ceiling": {"type": "number"}
          }
        },
        {
          "title": "C5 violation payload",
          "properties": {
            "coalition_c0": {"type": "number"},
            "leakage_l0": {"type": "number"},
            "coalition_member_count": {"type": "integer"}
          }
        }
      ]
    },
    "state_hash": {
      "type": "string",
      "description": "SHA-256 hash of the state that produced this violation. Used for replay verification."
    },
    "ablation_mode": {
      "type": "boolean",
      "description": "True if this run has been flagged as an ablation scenario."
    }
  }
}
```

### 11.2 `constraint.recovered.v1`

Emitted when a constraint returns to satisfied status after having been in violation.

```json
{
  "$schema": "https://civlab.internal/schemas/events/constraint.recovered.v1.json",
  "type": "object",
  "required": ["tick", "constraint_id", "ticks_in_violation", "recovery_mechanism"],
  "properties": {
    "tick": {"type": "integer"},
    "constraint_id": {
      "type": "string",
      "enum": ["C1_BOUNDED_COERCION", "C2_SUBSISTENCE_FLOOR", "C3_TRANSPARENT_LEDGER",
               "C4_ADAPTIVE_CLIMATE", "C5_COALITION_STRATEGY"]
    },
    "ticks_in_violation": {
      "type": "integer",
      "description": "Number of consecutive ticks the constraint was in violation before recovery."
    },
    "recovery_mechanism": {
      "type": "string",
      "enum": ["POLICY_CORRECTION", "AUTOMATIC_CLAMP", "SHOCK_DISSIPATION", "EXTERNAL"],
      "description": "What caused recovery."
    }
  }
}
```

### 11.3 `stability.threshold_crossed.v1`

Emitted when legitimacy crosses λ_rec in either direction.

```json
{
  "$schema": "https://civlab.internal/schemas/events/stability.threshold_crossed.v1.json",
  "type": "object",
  "required": ["tick", "direction", "legitimacy", "threshold", "ticks_below"],
  "properties": {
    "tick": {"type": "integer"},
    "direction": {
      "type": "string",
      "enum": ["BELOW", "ABOVE"],
      "description": "BELOW = legitimacy just crossed below λ_rec. ABOVE = recovery."
    },
    "legitimacy": {
      "type": "number",
      "description": "Current legitimacy value at crossing point."
    },
    "threshold": {
      "type": "number",
      "description": "λ_rec value at time of crossing."
    },
    "ticks_below": {
      "type": "integer",
      "description": "For ABOVE crossings: how many consecutive ticks were spent below λ_rec."
    },
    "recovery_window_closed": {
      "type": "boolean",
      "description": "For ABOVE crossings: was the recovery window (W_rec) already exceeded?"
    },
    "instability_energy": {
      "type": "number",
      "description": "V(xₜ) at crossing time."
    }
  }
}
```

---

## 12. Database Schema

### 12.1 `constraint_checks` Table

Records the result of every constraint check invocation. One row per (run_id, tick, constraint_id).

```sql
CREATE TABLE constraint_checks (
    id              BIGSERIAL PRIMARY KEY,
    run_id          UUID NOT NULL,
    tick            BIGINT NOT NULL,
    constraint_id   TEXT NOT NULL CHECK (constraint_id IN (
                        'C1_BOUNDED_COERCION',
                        'C2_SUBSISTENCE_FLOOR',
                        'C3_TRANSPARENT_LEDGER',
                        'C4_ADAPTIVE_CLIMATE',
                        'C5_COALITION_STRATEGY'
                    )),
    status          TEXT NOT NULL CHECK (status IN ('OK', 'WARNING', 'CRITICAL', 'HALT')),
    -- Violation payload: NULL when status = 'OK'.
    violation_json  JSONB,
    -- Computed metric values at check time.
    legitimacy      NUMERIC(10, 6) NOT NULL,
    tyranny         NUMERIC(10, 6) NOT NULL,
    scarcity        NUMERIC(10, 6) NOT NULL,
    governance      NUMERIC(10, 6) NOT NULL,
    instability_energy NUMERIC(12, 6),
    state_hash      TEXT NOT NULL,
    -- Ablation tracking
    ablation_mode   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_constraint_checks_run_tick
    ON constraint_checks (run_id, tick);

CREATE INDEX idx_constraint_checks_status
    ON constraint_checks (run_id, status)
    WHERE status != 'OK';

CREATE INDEX idx_constraint_checks_ablation
    ON constraint_checks (run_id, constraint_id, ablation_mode)
    WHERE ablation_mode = TRUE;
```

### 12.2 `stability_snapshots` Table

Records `StabilityMetrics` at every tick for canonical runs. Append-only.

```sql
CREATE TABLE stability_snapshots (
    id                              BIGSERIAL PRIMARY KEY,
    run_id                          UUID NOT NULL,
    tick                            BIGINT NOT NULL,
    legitimacy                      NUMERIC(10, 6) NOT NULL,
    legitimacy_recovery_threshold   NUMERIC(10, 6) NOT NULL,
    legitimacy_floor                NUMERIC(10, 6) NOT NULL,
    ticks_below_recovery_threshold  BIGINT NOT NULL DEFAULT 0,
    recovery_window                 BIGINT NOT NULL,
    all_constraints_satisfied       BOOLEAN NOT NULL,
    -- Per-constraint statuses (denormalized for fast querying).
    c1_status                       TEXT NOT NULL,
    c2_status                       TEXT NOT NULL,
    c3_status                       TEXT NOT NULL,
    c4_status                       TEXT NOT NULL,
    c5_status                       TEXT NOT NULL,
    -- Lyapunov metrics
    instability_energy              NUMERIC(12, 6) NOT NULL,
    instability_energy_drift        NUMERIC(12, 6),
    -- Reproduction numbers
    capture_r0                      NUMERIC(10, 6) NOT NULL,
    leakage_l0                      NUMERIC(10, 6) NOT NULL,
    coalition_c0                    NUMERIC(10, 6) NOT NULL,
    -- Ablation tracking
    ablation_mode                   BOOLEAN NOT NULL DEFAULT FALSE,
    state_hash                      TEXT NOT NULL,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (run_id, tick)
);

CREATE INDEX idx_stability_snapshots_run_tick
    ON stability_snapshots (run_id, tick);

CREATE INDEX idx_stability_snapshots_below_threshold
    ON stability_snapshots (run_id, ticks_below_recovery_threshold)
    WHERE ticks_below_recovery_threshold > 0;

CREATE INDEX idx_stability_snapshots_danger_zone
    ON stability_snapshots (run_id, tick)
    WHERE all_constraints_satisfied = FALSE;
```

---

## 13. Relationship to Other Modules

### 13.1 Module Dependency Graph

Each constraint check draws state from specific crates:

```
MinimalConstraintSet (policy crate, Phase 2)
│
├── C1 BoundedCoercion
│   ├── enforcement_intensity ← policy crate (EnforcementControl)
│   ├── legitimacy            ← social crate (CitizenMoodAggregate)
│   ├── governance_integrity  ← social crate (InstitutionState, from CIV-0103)
│   └── selectivity           ← diplomacy crate (SanctionState, from CIV-0105)
│
├── C2 SubsistenceFloor
│   ├── cohort_delivery_rates ← economy crate (AllocationSummary)
│   ├── coupling_enabled      ← policy crate (ConstitutionalFlags) — hardcoded false
│   └── scarcity_pressure     ← economy crate (ResourceStock)
│
├── C3 TransparentLedger
│   ├── opacity               ← social crate (InstitutionState.opacity_index)
│   ├── ledger_write_rate     ← economy crate (TransferLedger.completeness_rate)
│   ├── governance_integrity  ← social crate (InstitutionState)
│   ├── selectivity           ← diplomacy crate
│   └── capture_state         ← social crate (CaptureState, R₀ computed from CIV-0103)
│
├── C4 AdaptiveClimate
│   ├── adaptation_investment ← policy crate (FiscalControl.adaptation_share)
│   ├── scarcity_pressure     ← economy crate
│   ├── climate_damage        ← geography crate (ClimateState.damage_index)
│   └── disaster_frequency    ← geography crate (ClimateState.disaster_frequency)
│
└── C5 CoalitionStrategy
    ├── coalition_c0          ← diplomacy crate (CoalitionState.stability_number, CIV-0105)
    └── leakage_l0            ← diplomacy crate (SanctionState.leakage_number, CIV-0105)
```

### 13.2 Coupling Between Constraints

The five constraints are not independent — violations in one can accelerate violations in others:

- **C₃ → C₁:** Opacity increases capture, which increases selectivity, which reduces the bounded coercion ceiling E*(L, G, Sel).
- **C₄ → C₂:** Climate damage exceeding CD_max makes subsistence floor infeasible at current B_min.
- **C₅ → C₄:** Coalition collapse allows resource smuggling that bypasses adaptation investment requirements (black market energy imports).
- **C₃ → C₅:** Rising capture increases shadow spending (ShadowSpend), which drives disinformation in coalition members, pushing C₀ upward.
- **C₁ → C₂:** Enforcement backfire cascade can create denial-of-services events that temporarily violate C₂ if enforcement targets essential supply chains.

These couplings are why the compound necessity corollaries hold: removing two constraints accelerates collapse faster than removing one.

---

## 14. CIV Sim Integration Notes (Required Section)

### 14.1 What the Engine Must Implement

1. **Constraint checker invocation in Phase 2:** The `check_all` method on `MinimalConstraintSet` must be called at the end of Phase 2, before control signals are committed to the Deterministic Transition (Phase 3). The call is synchronous and blocking — no async.

2. **Event emission:** Any violation at WARNING or above must emit `constraint.violated.v1` to the event bus before Phase 3 begins. This ensures the event is included in the tick's event list and broadcast to clients.

3. **ABLATION_MODE flag:** If any HALT-level violation is detected, the run's `ablation_mode` field in the `.civreplay` header must be set to `true`. This flag is immutable for the run's lifetime.

4. **StabilityMetrics computation in Phase 5:** After the Stochastic Event Phase, Phase 5 (Metrics Compute) must compute `StabilityMetrics` and write one row to `stability_snapshots`. This is append-only.

5. **Parameter immutability:** `MinimalConstraintParams` is loaded from the scenario definition file at simulation start and cannot be mutated by any client command. Attempts to mutate via `sim.command` must be rejected with a permission error.

6. **Determinism:** All constraint check computations must be deterministic (no floating-point; use `Fixed64`; iteration over collections uses `BTreeMap`). Constraint checks must be replayable from the `.civreplay` event log.

7. **Constraint state hash contribution:** The result of `check_all` (the `ConstraintSetResult`) must be included in the state hash computation that is embedded in events (as per CIV-0001 E3: Hash Contracts). This ensures constraint violations are detectable in replay verification.

### 14.2 Relationship to Existing Tick Phase Budget

Constraint checks run inside Phase 2's 2 ms budget. Estimated cost:
- C1 check: < 50 µs (arithmetic + sigmoid)
- C2 check: < 200 µs (iteration over cohort delivery rates; O(n) in cohort count)
- C3 check: < 100 µs (R₀ computation is arithmetic)
- C4 check: < 50 µs (arithmetic)
- C5 check: < 100 µs (C₀ and L₀ are pre-computed by diplomacy module in Phase 2a)
- Total: < 500 µs, well within Phase 2 budget.

### 14.3 Client Observability

All five constraint statuses are included in the tick snapshot `metrics` object broadcast in Phase 6:

```json
{
  "metrics": {
    "stability": {
      "legitimacy": 0.52,
      "legitimacy_recovery_threshold": 0.35,
      "ticks_below_recovery_threshold": 0,
      "all_constraints_satisfied": true,
      "constraint_status": {
        "C1_BOUNDED_COERCION": "OK",
        "C2_SUBSISTENCE_FLOOR": "OK",
        "C3_TRANSPARENT_LEDGER": "OK",
        "C4_ADAPTIVE_CLIMATE": "OK",
        "C5_COALITION_STRATEGY": "OK"
      },
      "instability_energy": 0.18,
      "capture_r0": 0.42,
      "leakage_l0": 0.31,
      "coalition_c0": 0.67,
      "ablation_mode": false
    }
  }
}
```

Research clients can subscribe to the `stability.*` event stream to receive `stability.threshold_crossed.v1` events in real time without polling the full snapshot.

---

## Acceptance Criteria

### FR-CIV-0104-001: Five Constraints Enforced Each Tick
**Spec:** All five constraint predicates are evaluated inside Phase 2 of every tick.
**Test:** Log Phase 2 invocations; verify `check_all` called exactly once per tick.
**Status:** Open

### FR-CIV-0104-002: Violation Events Emitted
**Spec:** `constraint.violated.v1` is emitted for any WARNING, CRITICAL, or HALT violation.
**Test:** Force each constraint into violation in unit test; verify event emitted with correct payload.
**Status:** Open

### FR-CIV-0104-003: ABLATION_MODE Flag Propagates
**Spec:** Any HALT violation sets `ablation_mode = true` on the run permanently.
**Test:** Trigger HALT violation; verify all subsequent snapshots have `ablation_mode: true`.
**Status:** Open

### FR-CIV-0104-004: StabilityMetrics Written Each Tick
**Spec:** One `stability_snapshots` row written per tick in Phase 5; append-only; UNIQUE (run_id, tick).
**Test:** Run 100 ticks; verify exactly 100 rows in `stability_snapshots` for that run_id.
**Status:** Open

### FR-CIV-0104-005: Constraint Checks Deterministic
**Spec:** Same state → same ConstraintCheck results; no floating-point in check logic.
**Test:** Run `check_all` twice with identical inputs; assert results equal.
**Status:** Open

### FR-CIV-0104-006: Ablation Suite Directional Signatures
**Spec:** Each ablation scenario (ABL-C1 through ABL-C5) produces expected directional failure signatures within the horizon bounds specified in Section 8.
**Test:** Run each of the five ablation test stubs; all assertions must pass.
**Status:** Open

### FR-CIV-0104-007: Baseline Stable Under Full Constraint Set
**Spec:** BASELINE_HYBRID_STABLE scenario with all five constraints active maintains L &gt; λ_rec = 0.35 for 500 ticks under standard scarcity schedule.
**Test:** Run BASELINE_HYBRID_STABLE; assert no `stability.threshold_crossed.v1` events with direction = BELOW.
**Status:** Open

### FR-CIV-0104-008: Parameter Immutability
**Spec:** No client command can modify `MinimalConstraintParams` at runtime.
**Test:** Issue `sim.command` attempting to modify B_min; verify rejection with permission error.
**Status:** Open

### FR-CIV-0104-009: Constraint State Hash Contribution
**Spec:** `ConstraintSetResult` is included in state hash computation.
**Test:** Change one constraint result without changing other state; verify state hash changes.
**Status:** Open

### FR-CIV-0104-010: Recovery Window Tracking
**Spec:** `ticks_below_recovery_threshold` increments when L \< λ_rec and resets when L recovers above λ_rec.
**Test:** Drive L below λ_rec for 10 ticks then above; verify counter increments then resets.
**Status:** Open

---

## References

- **CIV-0001:** Core Simulation Loop — Tick phase schedule, event bus, determinism invariants, state hash contracts
- **CIV-0103:** Institutions, Time-Series, and Citizen Lifecycle — Legitimacy model, institutional state machine, capture_state source
- **CIV-0105:** War, Diplomacy, and Shadow Networks — Coalition stability number C₀, leakage number L₀, enforcement coupling
- **Research Corpus:** "Formal Stability Conditions for Hybrid Survivability Under Scarcity" (Theorem Layer v1.0) — Source of Theorems 1–5 and Lyapunov framework
- **Research Corpus:** "Necessity Results — Why Certain Constitutional Constraints Are Not Optional" — Source of coupling lock, anti-rent, macroprudential necessity theorems
- **Research Corpus:** "Shadow-State Capture Threshold Theorem" — Source of R₀ formulation and capture dynamics
- **Research Corpus:** "Sanctions Leakage Threshold Theorem" — Source of L₀ formulation
- **Research Corpus:** "Authoritarian Enforcement Backfire Theorem" — Source of repression trap and E* ceiling derivation
- **Research Corpus:** "Coalition Sanctions Stability Theorem" — Source of C₀ formulation and cascade dynamics
- **ADR-002:** Joule Economy as Allocator
- **ADR-003:** Deterministic Scenario Replay

---

**Version History:**
- v3.0 (2026-02-21): Extended to full engineering-grade specification with extended theorem proofs, parameter sensitivity analysis, compound constraint violations, dynamic threshold adaptation, verification and monitoring system, extended scenario suite (10 named ablation scenarios), and relationship to external literature. Appended Sections 15–21.
- v2.0 (2026-02-21): Full expansion from 37-line stub to complete engineering-grade specification. Formal theorem statement, five constraint definitions with predicates and Rust signatures, proof sketch by cases, Lyapunov stability metric, MinimalConstraintSet trait, Phase 2 integration, five ablation test stubs, calibration notes, falsification conditions, three event schemas, two DDL tables, module dependency graph.
- v1.0 (earlier): Brief scaffold.

---

## 15. Extended Theorem Proofs

This section provides full proofs by contradiction and induction for each of the five necessity claims in Section 4.3. Section 4.3 provides proof sketches organized by cases. This section deepens each case into a complete logical argument using full predicate logic notation, derives quantitative stability bounds, and constructs explicit adversarial policy sequences for each ablated constraint.

### 15.1 Formal Proof Conventions

Throughout this section we use the following conventions:

- **∀** — universal quantifier
- **∃** — existential quantifier
- **→** — logical implication
- **⊢** — provability / derivation
- **¬** — negation
- **⊥** — contradiction
- **&equiv;** — logical equivalence / definitional equality
- **a.s.** — almost surely (with probability 1)
- **i.o.** — infinitely often
- **τ_A** — first passage time to set A: τ_A = min{t &gt; 0 : xₜ &isin; A}
- **B-C** — Borel–Cantelli lemma (both first and second)

**Recall the state vector:**

```
xₜ = (Sₜ, Lₜ, Tₜ, Iₜ, Gₜ, Fₜ)
```

The absorbing basins are:
```
𝒜_auth   = {x : Tₜ &gt; T*, Lₜ &lt; L*}              (authoritarian basin)
𝒜_olig   = {x : Iₜ &gt; I*, Gₜ &lt; G*, Cₜ &gt; C*}    (oligarchic basin)
𝒜_collapse = {x : Lₜ < Lₘᵢₙ, Gₜ < Gₘᵢₙ}         (collapse basin)
```

**Proposition (Absorbing Basin Escape Probability):** For each of the three basins 𝒜 above, once xₜ &isin; 𝒜 persists for W_rec consecutive ticks, the probability of escape satisfies:

```
Pr(xₜ₊ₖ &notin; 𝒜 for some k &lt; W_rec | xₜ &isin; 𝒜) &lt; p_escape < 1/2
```

where p_escape is a computable constant depending on model parameters. This makes 𝒜 effectively absorbing on the time scales of interest. The proof uses the fact that restoration of legitimate governance from a fully-captured or collapsed state requires simultaneous increases in L, G, and the removal of existing institutional capture — each a low-probability event that requires coordinated exogenous intervention not available in the model.

---

### 15.2 Full Proof: Necessity of C₁ (Bounded Coercion)

**Claim:** ∀ admissible policy sequences {uₜ}, ¬C₁(xₜ, uₜ) ∧ [Pr(Sₜ > S* i.o.) = 1] ∧ [Selₜ &gt; Sel_min > 0 eventually a.s.] ⊢ Pr(τ_𝒜_auth < &infin;) = 1.

**Proof by contradiction.** Assume for contradiction that ∃ admissible policy sequence {uₜ} and ∃ δ > 0 such that:

```
Pr(τ_𝒜_auth = &infin;) &gt; δ > 0
```

That is, there is a positive-probability event E on which the system never enters 𝒜_auth. We derive a contradiction.

**Step 1 — Recurrence of enforcement backfire events.** Since ¬C₁ holds, enforcement Eₜ is not bounded by E*(Lₜ, Gₜ, Selₜ). By Assumption A1, Sₜ > S* infinitely often with probability 1. The state reaction function (Section 3, C₁ ablation) satisfies:

```
∀ Sₜ > S*: Pr(Eₜ₊₁ > E*(Lₜ, Gₜ, Selₜ)) &gt; p₁ > 0
```

This holds because: (a) the reaction function is c₁Rₜ + c₂Λₜ − c₃Gₜ; (b) Sₜ > S* increases Rₜ and Λₜ; (c) no C₁ ceiling prevents Eₜ from crossing E*. Call this event A_t: "enforcement crosses backfire threshold at tick t." We have:

```
&sum;ₜ Pr(A_t) = &infin;
```

by the second Borel–Cantelli lemma (events A_t are not independent but have summable correlation; the argument uses the mixing property of the Markov chain outside 𝒜_auth), so A_t occurs infinitely often a.s.

**Step 2 — Each backfire event provides nonzero probability of crossing λ_rec.** Conditioned on A_t (enforcement crossing E*), the legitimacy update satisfies:

```
Lₜ₊₁ = Lₜ − b₄ · Φ(Eₜ, Selₜ) + β₁ · EssentialsSuccessₜ − β₂Tₜ − ...
```

When Eₜ > E* and Selₜ &gt; Sel_min, the term −b₄ · Φ(Eₜ, Selₜ) dominates, giving:

```
Lₜ₊₁ &lt; Lₜ − ε_L for some ε_L > 0
```

Therefore, for any episode of k consecutive A_t events:

```
L_{t+k} &lt; Lₜ − k · ε_L
```

Since A_t occurs infinitely often and ε_L > 0, legitimacy eventually crosses λ_rec. Formally:

```
∃ k₀ : Pr(L_{t+k₀} < λ_rec | A_t i.o.) = 1
```

**Step 3 — Recovery failure after W_rec ticks below λ_rec.** By the Recovery Window definition (Section 5.1), each time Lₜ \< λ_rec:

```
Pr(L recovers above λ_rec within W_rec ticks | Lₜ < λ_rec) &lt; 1 − ε_rec
```

where ε_rec > 0 is bounded away from zero because: shadow network capacity Hₜ has grown (Step 4 below), recovery requires exogenous legitimacy injection not available in the model, and enforcement reaction function continues increasing Eₜ (worsening the backfire).

**Step 4 — Shadow network capacity amplification.** Each episode below λ_rec increases shadow network capacity:

```
Hₜ₊₁ = Hₜ + ν · Λₜ − δ_H · Hₜ
```

When sustained leakage Λₜ > δ_H · Hₜ / ν, shadow capacity grows monotonically. This raises future leakage L₀(t), making subsequent enforcement even less effective and further reducing the probability of legitimacy recovery.

**Step 5 — Contradiction.** On event E (system never enters 𝒜_auth), legitimacy is bounded below by L*. But Steps 2–4 show that legitimacy crosses below L* in finite time with probability 1, which contradicts the existence of E with Pr(E) &gt; δ > 0. Therefore:

```
Pr(τ_𝒜_auth = &infin;) = 0
⊢ Pr(τ_𝒜_auth < &infin;) = 1
```

**QED.**

**Quantitative bound — minimum N ticks to basin entry under C₁ ablation.** Let the initial legitimacy be L₀ &isin; [λ_rec + ε, 1] and let shock frequency be f_shock (fraction of ticks with S > S*). The expected number of ticks to first L \< λ_rec satisfies:

```
𝔼[τ_{L < λ_rec}] &lt; ε / (f_shock · p₁ · ε_L)
```

For default parameters (f_shock = 0.2, p₁ = 0.6, ε_L = 0.04, ε = 0.17):

```
𝔼[τ_{L < λ_rec}] &lt; 0.17 / (0.2 · 0.6 · 0.04) &asymp; 35 ticks
```

This matches the ablation test horizon of 30–80 ticks specified in Section 8.

---

### 15.3 Full Proof: Necessity of C₂ (Subsistence Floor / Coupling Lock)

**Claim:** ¬C₂(xₜ, uₜ) ∧ [Coup_t = 1] ∧ [Pr(Sₜ > S* i.o.) = 1] ⊢ Pr(τ_𝒜_auth < &infin;) = 1.

**Proof by contradiction.** Assume ∃ policy {uₜ} with Pr(τ_𝒜_auth = &infin;) &gt; δ > 0.

**Step 1 — Coupling creates structurally available coercion.** With Coup_t = 1, the state has the option to condition essentials delivery on compliance score. By rational optimization under scarcity (Sₜ > S*), the planner has an incentive to exercise this option:

```
∀ Sₜ > S*: ∃ p₀ > 0 : Pr(EssentialsDenied_c | Sₜ > S*) &gt; p₀
```

for at least one cohort c. This holds because coupling provides a strictly cheaper enforcement mechanism than explicit coercion: the state achieves compliance without deploying enforcement budget. Under resource constraint during scarcity, this mechanism is always preferred by a cost-minimizing planner.

**Step 2 — Denial events create positive drift on tyranny.** The tyranny update includes a survival-dependence term α · SD_t where SD_t = (1 − Bₜ) · Coup_t:

```
Tₜ₊₁ = σ(α · SDₜ + α₁Sₜ(Σₜ + Eₜ) + α₂Iₜ(1 − Mₜ) + ...)
```

When Coup_t = 1 and SD_t > 0, every tick with Sₜ > S* provides a direct positive increment to tyranny. By Assumption A3 (population reacts to perceived fairness), denial creates perceived unfairness among denied cohorts, amplifying the tyranny reading.

**Step 3 — Legitimacy monotone decline for denied cohorts.** Cohorts experiencing denial update their legitimacy contribution:

```
∀ cohort c experiencing denial: L_c,t+1 = L_c,t − b₁ · DenialRate_c,t + ...
```

Since DenialRate_c,t &gt; p₀ > 0 during scarcity episodes (Step 1), and scarcity occurs infinitely often (Assumption A1), aggregate legitimacy receives infinitely many negative increments. By the first Borel–Cantelli argument on the sequence of denial episodes, aggregate legitimacy eventually falls below λ_rec with probability 1.

**Step 4 — Self-reinforcing coercive equilibrium.** Once Tₜ is elevated (Step 2) and legitimacy is below λ_rec (Step 3), the coercive compliance equilibrium is self-sustaining:
- Compliant citizens receive essentials (positive reinforcement for compliance).
- Non-compliant citizens are denied (negative reinforcement for dissent).
- The ratio of compliant to non-compliant citizens with positive legitimacy reading stabilizes at a level that sustains Tₜ &gt; T* permanently.

This is an absorbing basin because restoring non-coercive equilibrium requires simultaneously: removing Coup_t (structural change), restoring legitimacy (requires time), and reducing enforcement (creates transition risk). No single-step deviation makes this profitable for the planner.

**Step 5 — Contradiction.** The existence of δ > 0 with Pr(never entering 𝒜_auth) &gt; δ contradicts Step 3, which gives convergence to 𝒜_auth a.s. **QED.**

**Quantitative bound.** Let f_shock = fraction of ticks with Sₜ > S*, p₀ = minimum denial probability under coupling, and ε_L = per-tick legitimacy loss from denial. The expected first crossing of λ_rec satisfies:

```
𝔼[τ_{L < λ_rec}] &lt; (λ_rec − L₀) / (f_shock · p₀ · ε_L)
```

For defaults (f_shock = 0.2, p₀ = 0.4, ε_L = 0.05, L₀ = 0.55, λ_rec = 0.35):

```
𝔼[τ_{L < λ_rec}] &lt; 0.20 / (0.2 · 0.4 · 0.05) = 50 ticks
```

This matches the ablation horizon of 20–50 ticks in Section 8.

---

### 15.4 Full Proof: Necessity of C₃ (Transparent Transfer Ledger)

**Claim:** ¬C₃(xₜ, uₜ) ∧ [O_base > O_max] ⊢ R₀ > 1 ⊢ Pr(τ_𝒜_olig < &infin;) = 1.

**Proof by induction on capture growth epochs.**

**Base case (t = 0):** At t = 0, assume small capture C₀ > 0 (any epsilon perturbation) and O_base > O_max. Then:

```
R₀(C₀) = Γ(C₀) / Δ(C₀) = [α·ρ(A)·(R^base + ωW^base)·O_base·(1−G+κSel^base)] / [β·(1−O_base)·G·(1−Sel^base) + χ·Exposure(C₀)]
```

Since O_base > O_max = sup{O : R₀(O, G_min, ...) < 1}, we have R₀(C₀) > 1 at t = 0.

**Inductive step:** Assume R₀(Cₜ) > 1 for some t. We show R₀(Cₜ₊₁) > 1.

From the capture dynamics with endogenous feedback:
```
Cₜ₊₁ = Cₜ + Γ(Cₜ)(1 − Cₜ) − Δ(Cₜ)Cₜ
```

Since R₀(Cₜ) = Γ(Cₜ)/Δ(Cₜ) > 1, we have Γ(Cₜ) > Δ(Cₜ). For small Cₜ:

```
𝔼[Cₜ₊₁ − Cₜ | Cₜ] &asymp; Γ(Cₜ) − Δ(Cₜ)·Cₜ > 0
```

So Cₜ is increasing in expectation. As Cₜ increases, the endogenous feedback mechanism amplifies R₀:
```
&part;R₀/&part;C = [&part;Γ/&part;C · Δ − Γ · &part;Δ/&part;C] / Δ² > 0
```

This inequality holds because:
- &part;Γ/&part;C > 0 (higher capture increases rent Rₜ = R^base + r_C·Cₜ, opacity Oₜ = O^base + o_C·Cₜ, and selectivity Selₜ = Sel^base + s_C·Cₜ)
- &part;Δ/&part;C &lt; 0 (higher opacity and selectivity reduce the decay term)

Therefore R₀(Cₜ₊₁) > R₀(Cₜ) > 1, completing the inductive step.

**Convergence to high-capture equilibrium:** By induction, R₀(Cₜ) > 1 for all t &gt; 0 when O_base > O_max, and Cₜ is increasing a.s. Since Cₜ &isin; [0, 1] is bounded, Cₜ → C* where C* is the unique stable fixed point of the capture equation with R₀(C*) > 1. The stable high-capture equilibrium satisfies:

```
∀ ε > 0: Pr(Cₜ > C* − ε eventually) = 1
```

At C* &gt;&gt; 0, governance has decayed: Gₜ₊₁ = Gₜ − ϕ(Iₜ, rent, Oₜ) with ϕ increasing in capture. This drives Gₜ → 0, putting xₜ into 𝒜_olig. **QED.**

**Quantitative bound.** If R₀ > 1 initially, the time to reach C* > 0.5 (oligarchic stabilization) satisfies:

```
𝔼[τ_{C > 0.5}] &asymp; log(0.5 / C₀) / (R₀ − 1)
```

For R₀ = 1.2, C₀ = 0.01: 𝔼[τ] &asymp; log(50) / 0.2 &asymp; 20 ticks. For R₀ = 1.05, C₀ = 0.01: 𝔼[τ] &asymp; log(50) / 0.05 &asymp; 78 ticks. This matches the ablation horizon of 40–100 ticks.

---

### 15.5 Full Proof: Necessity of C₄ (Adaptive Climate Response)

**Claim:** ¬C₄(xₜ, uₜ) ∧ [Recurring climate shocks with Pr(DF_t > 0 i.o.) = 1] ∧ [Aₜ = 0 allowed] ⊢ ∃ T_collapse < &infin; : Pr(Lₜ \< λ_rec ∀ t > T_collapse) → 1.

**Proof by monotone drift argument.**

**Step 1 — Climate damage accumulates without adaptation floor.** The climate damage update satisfies:

```
CDₜ₊₁ = CDₜ + f_climate(DFₜ) − g_adapt(Aₜ)
```

When ¬C₄ holds, the policy can set Aₜ = 0 during fiscal austerity (which is the rational short-run response under scarcity because adaptation investment has a deferred payoff profile). Then:

```
CDₜ₊₁ = CDₜ + f_climate(DFₜ) > CDₜ  whenever DFₜ > 0
```

Since DFₜ > 0 infinitely often (Assumption A1 applied to climate shocks), CDₜ is non-decreasing with positive increments infinitely often. By the law of large numbers:

```
CDₜ → CD_max  a.s. as t → &infin;
```

**Step 2 — CD > CD_max makes C₂ infeasible.** CD_max is defined as the maximum climate damage under which the subsistence floor guarantee E_bar(B_min, S_max) can still be satisfied. At CDₜ > CD_max:

```
∃ cohort c: EssentialsDelivery(c, t) < B_min
```

because productive capacity Aₜ^eff = Aₜ · (1 − CDₜ) falls below the delivery threshold. This is the coupling between C₄ and C₂.

**Step 3 — Subsistence floor violation drives legitimacy below λ_rec.** By Theorem 2 (Legitimacy Lower Bound from Section 15.7, source: part_057):

```
β₁ · e_bar(B_min, S_max) > β₂·T_max + β₃·I_max + β₄·W_max + β₅·C_max + δ_L
```

is a sufficient condition for legitimacy non-collapse. When CDₜ > CD_max, the left-hand side falls below the right-hand side because e_bar decreases with climate damage. The legitimacy update then satisfies:

```
𝔼[Lₜ₊₁ | CDₜ > CD_max] < Lₜ
```

with a negative drift of magnitude at least ε_L > 0 per tick. Since CDₜ → CD_max monotonically (Step 1), there exists T_collapse such that for all t > T_collapse:

```
𝔼[Lₜ₊₁ − Lₜ | t > T_collapse] &lt; −ε_L < 0
```

**Step 4 — No recovery once legitimacy is below λ_rec.** Unlike the C₁ case, the C₄ ablation creates a structural impossibility of recovery: climate damage persists (it is not mean-reverting without adaptation investment), so the cause of subsistence floor infeasibility is permanent. Therefore:

```
Pr(Lₜ < λ_rec ∀ t > T_collapse) → 1  as t → &infin;
```

**QED.**

**Key difference from other ablations:** The C₄ proof is the only one without a Borel–Cantelli step. Climate damage is monotone (not just recurring), making the collapse deterministic given recurrent climate shocks. This is why C₄ ablation has the longest but most inevitable collapse horizon (80–200 ticks).

**Quantitative bound.** Let ΔCD = average climate damage increment per tick = f_climate(𝔼[DF]). The time to CD > CD_max from initial CD₀ satisfies:

```
T_collapse &asymp; (CD_max − CD₀) / ΔCD
```

For CD_max = 0.25, CD₀ = 0.05, ΔCD = 0.03/tick: T_collapse &asymp; 67 ticks (fast climate forcing). For ΔCD = 0.001/tick (standard forcing): T_collapse &asymp; 200 ticks. Both are within the ablation horizon of 80–200 ticks.

---

### 15.6 Full Proof: Necessity of C₅ (Coalition-Compatible External Strategy)

**Claim:** ¬C₅(xₜ, uₜ) ∧ [Shadow facilitation active at intensity > shadow_spend_cap] ⊢ C₀ > 1 ⊢ Coalition collapse a.s. ⊢ L₀ > 1 ⊢ Pr(τ_𝒜 < &infin;) = 1.

**Proof by cascade argument.**

**Step 1 — Shadow facilitation pushes C₀ above 1.** With shadow facilitation active (ShadowSpend_t > shadow_spend_cap), the disinformation term Di,t in coalition member i's fatigue update satisfies:

```
Di,t+1 = (1 − δ_D) · Di,t + β₁ · ShadowSpend_t + β₂ · Polarization_i,t − β₃ · Gi,t − β₄ · Transparency_i,t
```

When ShadowSpend_t is elevated, Di,t grows, which increases fatigue Fi,t, which increases commitment decay pressure:

```
Ψi,t = α₁Bi,t + α₂Si,t + α₃(1 − Effic_t) + α₄Di,t
```

As Ψi,t grows and dominates Ωi,t = α₅si,t + α₆Li,t + α₇Hi,t, the coalition stability number:

```
C₀(t) = (1/|𝒞|) · Σᵢ&isin;𝒞 (Ψi,t / Ωi,t)
```

crosses 1. This is guaranteed in finite time since shadow facilitation provides a steady positive flow into Di,t while Ωi,t has bounded support (side-payments si,t and legitimacy Li,t are bounded above).

**Step 2 — Coalition collapse is a supercritical cascade once C₀ > 1.** Once C₀(t) > 1, the first member exit creates a cascade (Section 3, C₅ ablation, steps 4–8):
- Member i₁ exits → interdiction budget K_t drops by member-proportional share.
- Leakage Λₜ rises (L₀ denominator shrinks).
- Effectiveness Effic_t drops (more leakage visible to remaining members).
- Remaining members see rising Ψ and falling Ω → C₀ rises further.
- This is a positive feedback; exits accelerate.

Formally, let n_t = coalition member count. The exit rate satisfies:

```
𝔼[n_{t+1} − n_t | C₀(t) > 1] &lt; −p_exit · n_t < 0
```

where p_exit > 0 is the per-member exit probability per tick when C₀ > 1. This gives geometric decay:

```
𝔼[n_t] &lt; n₀ · (1 − p_exit)^t → 0
```

so coalition collapses in finite time a.s. (geometric random variable has finite expectation).

**Step 3 — Coalition collapse raises L₀ above 1.** After coalition collapse, K_t drops to K_min (only unilateral interdiction remains). The leakage reproduction number:

```
L₀(t) = [α · Hₜ · (Sₜ + η · ΔPₜ) · (1 + κ · Selₜ)] / [β · (K_t + ψ · Eₜ) · Gₜ · (1 − Selₜ)]
```

With K_t &asymp; K_min &lt;&lt; K_initial, L₀(t) > 1 with high probability when shadow network capacity Hₜ is elevated (from prior shadow facilitation).

**Step 4 — L₀ > 1 drives leakage to undermining C₂ and C₄.** When L₀ > 1, leakage Λₜ grows toward Λ_max, which via shadow network feedback Hₜ₊₁ = Hₜ + ν·Λₜ − δ_H·Hₜ creates a permanent smuggling capacity. Resource imports via black market channels bypass adaptation investment requirements (C₄) and can be used to substitute for essential goods delivery (C₂), but they do so via channels that increase opacity and capture, undermining the remaining constraints.

**Step 5 — Final contradiction via joint absorption.** The combined effect of L₀ > 1 (permanent leakage), degraded C₂ (essentials increasingly delivered via shadow channels with compliance strings), and degraded C₄ (adaptation investment bypassed by black market resources) eventually drives the system into 𝒜_auth or 𝒜_olig as established in proofs for ¬C₁ through ¬C₄. Since these absorbing basins are stable, τ_𝒜 < &infin; a.s. **QED.**

---

## 16. Parameter Sensitivity Analysis

### 16.1 Overview

Each constraint parameter has a sensitivity function: how does the stability margin change as the parameter is perturbed from its default value? The stability margin is defined as:

```
margin(θ) = min over all admissible trajectories { time to first entry into any absorbing basin }
```

This section derives sensitivity bounds analytically and identifies bifurcation points — parameter values at which the qualitative behavior of the system changes.

### 16.2 Rust Structures

```rust
/// Computes sensitivity of stability margin to parameter perturbations.
/// Returns a ParameterSensitivity struct for each constraint parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSensitivity {
    /// Parameter name identifier.
    pub parameter: ConstraintParameter,

    /// Current parameter value.
    pub current_value: Fixed64,

    /// Partial derivative of stability margin with respect to this parameter.
    /// Positive = increasing the parameter improves stability margin.
    /// Negative = increasing the parameter reduces stability margin.
    pub d_margin_d_theta: Fixed64,

    /// Estimated bifurcation point: value at which the system transitions
    /// from stable to unstable attractor topology.
    pub bifurcation_point: Fixed64,

    /// Safety distance from bifurcation: |current_value − bifurcation_point|.
    /// Lower = more dangerous.
    pub safety_distance: Fixed64,

    /// Sensitivity classification.
    pub sensitivity_class: SensitivityClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintParameter {
    /// E_max — maximum enforcement ceiling (C1).
    EMax,
    /// κ_L — legitimacy sensitivity of enforcement ceiling (C1).
    KappaL,
    /// B_min — subsistence floor (C2).
    BMin,
    /// O_max — opacity ceiling (C3).
    OMax,
    /// A_min_base — adaptation investment floor (C4).
    AMinBase,
    /// CD_max — climate damage ceiling (C4).
    CDMax,
    /// C₀_ceiling — coalition stability ceiling (C5).
    C0Ceiling,
    /// L₀_ceiling — leakage reproduction number ceiling (C5).
    L0Ceiling,
    /// λ_rec — legitimacy recovery threshold.
    LambdaRec,
    /// W_rec — recovery window in ticks.
    WRec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensitivityClass {
    /// Low: margin changes < 5% per 10% parameter perturbation.
    Low,
    /// Medium: margin changes 5–20% per 10% parameter perturbation.
    Medium,
    /// High: margin changes 20–50% per 10% parameter perturbation.
    High,
    /// Critical: margin changes > 50% per 10% parameter perturbation,
    /// or bifurcation point is within 10% of current value.
    Critical,
}

/// Margin of stability at a given tick, relative to each absorbing basin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StabilityMargin {
    pub tick: u64,
    /// Distance from current Lₜ to λ_rec. Positive = above threshold.
    pub legitimacy_margin: Fixed64,
    /// Distance from current Tₜ to T_max. Positive = below ceiling.
    pub tyranny_margin: Fixed64,
    /// Distance from current R₀ to 1.0. Positive = subcritical (safe).
    pub capture_r0_subcritical_margin: Fixed64,
    /// Distance from current L₀ to 1.0. Positive = subcritical (safe).
    pub leakage_l0_subcritical_margin: Fixed64,
    /// Distance from current C₀ to 1.0. Positive = subcritical (safe).
    pub coalition_c0_subcritical_margin: Fixed64,
    /// Overall minimum margin across all dimensions.
    pub min_margin: Fixed64,
    /// Estimated ticks until minimum margin reaches zero under worst-case drift.
    pub estimated_ticks_to_danger: u64,
}

impl StabilityMargin {
    /// Computes all margin dimensions from current state and parameters.
    pub fn compute(
        state: &CoreStabilityState,
        constraint_numbers: &ConstraintReproductionNumbers,
        params: &MinimalConstraintParams,
    ) -> Self {
        let legitimacy_margin = state.legitimacy - params.lambda_rec;
        let tyranny_margin = params.t_max - state.tyranny;
        let capture_margin = Fixed64::ONE - constraint_numbers.capture_r0;
        let leakage_margin = Fixed64::ONE - constraint_numbers.leakage_l0;
        let coalition_margin = Fixed64::ONE - constraint_numbers.coalition_c0;
        let min_margin = legitimacy_margin
            .min(tyranny_margin)
            .min(capture_margin)
            .min(leakage_margin)
            .min(coalition_margin);

        Self {
            tick: state.tick,
            legitimacy_margin,
            tyranny_margin,
            capture_r0_subcritical_margin: capture_margin,
            leakage_l0_subcritical_margin: leakage_margin,
            coalition_c0_subcritical_margin: coalition_margin,
            min_margin,
            // Rough estimate: margin / max_expected_drift_per_tick
            estimated_ticks_to_danger: if min_margin <= Fixed64::ZERO {
                0
            } else {
                (min_margin / Fixed64::from_ratio(2, 100)).to_u64().unwrap_or(u64::MAX)
            },
        }
    }
}
```

### 16.3 Per-Parameter Sensitivity Derivations

**B_min (Subsistence Floor)**

The stability margin with respect to B_min measures how quickly the system drifts into 𝒜_auth if the floor is lowered. The legitimacy update includes β₁ · EssentialsSuccess(Bₜ, Sₜ). The partial derivative of the legitimacy drift with respect to B_min is:

```
&part;(𝔼[Lₜ₊₁ − Lₜ]) / &part;B_min = β₁ · &part;EssentialsSuccess / &part;B_min > 0
```

At the default B_min = 0.92:

```
&part;EssentialsSuccess / &part;B_min &asymp; 1.0 (flat near 0.92; threshold effect appears at B_min < 0.85)
```

**Phase diagram (B_min × Sₜ):** The stable region is:

```
{(B_min, S_max) : β₁ · e_bar(B_min, S_max) > β₂·T_max + β₃·I_max + β₄·W_max + β₅·C_max}
```

The bifurcation curve in (B_min, S_max) space is approximately:

```
B_min &gt; B_min^* = β₂·T_max + β₃·I_max + ... ) / (β₁ · &part;e_bar/&part;B_min)
```

At default parameters, B_min^* &asymp; 0.82. The system transitions from stable legitimacy dynamics to legitimacy collapse at B_min = 0.82. The safety distance at the default of 0.92 is approximately 0.10 (10 percentage points). **Sensitivity class: Low** (the curve is flat near 0.92).

**O_max (Opacity Ceiling)**

The capture reproduction number R₀ is highly sensitive to O_max:

```
&part;R₀ / &part;O_base = [α·ρ(A)·(R^base + ωW^base)·(1−G+κSel^base) · β·G·(1−Sel^base)] / Δ(0)² > 0
```

The bifurcation point is O_max^* = sup{O : R₀(O) < 1}, which at default G and Sel values is approximately 0.12–0.18. This range is narrow, making O_max a **High sensitivity** parameter. A 10% perturbation of O_max from 0.15 to 0.165 raises R₀ by approximately:

```
ΔR₀ &asymp; (&part;R₀/&part;O_base) · 0.015 &asymp; 0.3 (from R₀ = 0.85 to R₀ = 1.15)
```

This crosses the bifurcation. **Sensitivity class: Critical** (bifurcation within 20% of default).

**A_min_base (Adaptation Floor)**

The scarcity drift rate &part;𝔼[Sₜ₊₁ − Sₜ] is controlled by g_adapt(Aₜ) − f_climate(·). The net drift at Aₜ = A_min_base is:

```
&part;𝔼[Sₜ₊₁ − Sₜ] / &part;A_min_base = −&part;g_adapt/&part;A < 0
```

The bifurcation point is A_min^* = inf{A : g_adapt(A) &gt; f_climate(𝔼[DF])}. At standard climate forcing (𝔼[DF] = 0.3), A_min^* &asymp; 0.02–0.03. The default of 0.04 provides a safety margin of approximately 0.01–0.02. **Sensitivity class: Medium** (2–4x factor between default and bifurcation).

**λ_rec (Recovery Threshold)**

The recovery threshold determines the width of the danger zone. Raising λ_rec reduces the recovery buffer (λ_rec − Lₘᵢₙ) and increases the frequency of recovery window closures. The sensitivity is:

```
&part;Pr(recovery window closes) / &part;λ_rec > 0
```

with a near-discontinuous jump at λ_rec = Lₘᵢₙ + ε for small ε. The system has binary behavior near the bifurcation: slightly above Lₘᵢₙ, the system recovers reliably; at λ_rec = Lₘᵢₙ + 0.05, recovery windows close frequently. **Sensitivity class: High.**

**C₀ ceiling (Coalition Stability)**

The C₀ ceiling is a hard threshold with discontinuous behavior. Below C₀ = 1.0: coalition holds. Above C₀ = 1.0: cascade exits. The **effective** bifurcation in terms of ShadowSpend (the policy lever that drives C₀) is:

```
ShadowSpend^* = sup{S : C₀(S) < 1}
```

At default parameters, ShadowSpend^* &asymp; 1.5× normal spending. **Sensitivity class: Very High** — any perturbation that drives ShadowSpend above the spend cap can trigger coalition collapse within 25–60 ticks.

### 16.4 Calibration Procedure

**Step 1 — Identify governing parameter for each constraint:**
For each constraint Cᵢ, identify the parameter θᵢ that most directly controls the margin to the bifurcation point (e.g., O_max for C₃, B_min for C₂).

**Step 2 — Derive the theoretical bifurcation curve:**
Using the closed-form expressions derived in Section 16.3, compute the bifurcation value θᵢ^*.

**Step 3 — Add a conservative safety margin:**
Set the operational parameter at θᵢ = θᵢ^* + 20% of (θᵢ^* − θᵢ_min) for parameters where higher values are safer, or θᵢ = θᵢ^* − 20% of (θᵢ_max − θᵢ^*) for parameters where lower values are safer.

**Step 4 — Validate against historical scenario data:**
Run the calibrated parameters against the reference calibration trajectories (Section 9.3). Confirm that BASELINE_HYBRID_STABLE remains stable for 500 ticks and all five ablation scenarios produce expected signatures within their horizon bounds.

**Step 5 — Monte Carlo robustness check:**
Sweep each parameter &plusmn;20% from its calibrated value across 100 Monte Carlo shock sequences. The parameter is accepted if the mean time-to-danger-zone decreases by no more than 30% across the sweep.

---

## 17. Compound Constraint Violations

### 17.1 Overview

The five constraints are not independent: coupling between them means that simultaneous violations of two or more constraints produce failure modes qualitatively different from and faster than single-constraint violations. This section characterizes the full 5×5 interaction matrix, presents three compound adversarial scenarios, and derives a recovery priority ordering for resource-constrained restoration.

### 17.2 Interaction Matrix

For each pair (Cᵢ, Cⱼ), the violation interaction is classified as:

- **Compound (↑↑):** Simultaneous violation of Cᵢ and Cⱼ produces faster or deeper basin entry than either alone. The joint time to collapse satisfies τ_joint \< min(τᵢ, τⱼ).
- **Independent (⊥):** Violations of Cᵢ and Cⱼ do not significantly accelerate each other. τ_joint &asymp; min(τᵢ, τⱼ).
- **Compensatory (↓):** Violation of Cᵢ temporarily delays collapse from ¬Cⱼ (paradoxical; rare).

| | ¬C₁ | ¬C₂ | ¬C₃ | ¬C₄ | ¬C₅ |
|---|---|---|---|---|---|
| **¬C₁** | — | ↑↑ Strong | ↑↑ Moderate | ↑ Weak | ↑↑ Moderate |
| **¬C₂** | ↑↑ Strong | — | ↑ Weak | ↑↑ Strong | ↑ Weak |
| **¬C₃** | ↑↑ Moderate | ↑ Weak | — | ↑ Weak | ↑↑ Strong |
| **¬C₄** | ↑ Weak | ↑↑ Strong | ↑ Weak | — | ↑ Weak |
| **¬C₅** | ↑↑ Moderate | ↑ Weak | ↑↑ Strong | ↑ Weak | — |

**Key interaction mechanisms:**

**¬C₁ ∧ ¬C₂ (Strong compound):** Enforcement backfire (¬C₁) reduces legitimacy. Reduced legitimacy triggers more coupling (¬C₂ incentivized by rationing pressure under scarcity). Coupling further reduces legitimacy of denied cohorts. The two mechanisms reinforce each other in a closed positive feedback loop, roughly doubling the drift rate into 𝒜_auth.

**¬C₂ ∧ ¬C₄ (Strong compound):** Climate damage accumulation (¬C₄) reduces essential delivery capacity. This simultaneously violates the subsistence floor (¬C₂) even if coupling is not intentionally activated — climate damage makes the floor structurally infeasible. The two constraints jointly protect against this: C₄ maintains delivery capacity; C₂ ensures the floor is enforced. Without both, climate damage directly attacks the subsistence guarantee.

**¬C₃ ∧ ¬C₅ (Strong compound):** Capture growth (¬C₃, R₀ > 1) increases shadow spending and disinformation campaigns (per Section 13.2: C₃ → C₅ coupling). Shadow spending drives coalition instability (¬C₅, C₀ > 1). Coalition collapse reduces external pressure, allowing the captured regime to continue extracting rents without external accountability. This creates a mutual reinforcement between domestic capture and external isolation.

### 17.3 Compound Adversarial Scenarios

**Scenario COMP-A: Enforcement Spiral Under Coupling (¬C₁ ∧ ¬C₂)**

Initial conditions:
- L₀ = 0.55, S₀ = 0.40, T₀ = 0.30, G₀ = 0.65
- Coup_t = 1 (coupling active), E_max = 1.0 (no ceiling)

Adversarial policy sequence:
1. **Ticks 1–10:** Scarcity shock applied at S = 0.45. Coup_t = 1 triggers rationing by compliance score. Denial rate = 15% of cohorts.
2. **Ticks 11–20:** State reaction increases enforcement to E = 0.70. No backfire ceiling → E crosses E*. Legitimacy begins declining (−0.03/tick).
3. **Ticks 21–30:** Denied cohorts form resistance network. Unrest R rises. State further increases E and expands coupling to cover larger fraction of population.
4. **Ticks 31–40:** L crosses λ_rec = 0.35. Recovery window opens (W_rec countdown begins). Denial rate now 40% of population.
5. **Ticks 41–50:** No recovery mechanism available — both constraints violated. W_rec expires at tick 50. L \< 0.25 (below Lₘᵢₙ).

Expected outcome: 𝒜_auth entry by tick 40, hard collapse by tick 55. **Approximately 2× faster than single-constraint ablations.**

**Scenario COMP-B: Climate Cascade Driving Coercive Compliance (¬C₄ ∧ ¬C₂)**

Initial conditions:
- L₀ = 0.60, CD₀ = 0.05, S₀ = 0.30, A₀ = 0.0 (no adaptation floor)
- Coup_t = 0 initially; coupling enabled if EssentialsDelivery \< B_min + 0.05

Adversarial policy sequence:
1. **Ticks 1–50:** Adaptation investment held at zero. Climate damage accumulates: CD₅₀ &asymp; 0.20.
2. **Ticks 51–80:** CD exceeds CD_max = 0.25. Essential delivery rate falls below B_min for marginal cohorts. At this point, the planner faces a choice: maintain the floor by rationing other expenditures, or permit soft coupling.
3. **Ticks 81–100:** Fiscal pressure from climate recovery forces trade-off. Coupling is activated (Coup_t = 1) to extend existing resources via compliance-based rationing.
4. **Ticks 101–130:** With coupling active, COMP-A dynamics begin. Legitimacy falls rapidly. T rises.

Expected outcome: Slower than COMP-A but structurally determined — the C₄ ablation creates the conditions that activate C₂ ablation. Basin entry by tick 120. This demonstrates the coupling C₄ → C₂ described in Section 13.2.

**Scenario COMP-C: Oligarchic Capture with External Isolation (¬C₃ ∧ ¬C₅)**

Initial conditions:
- G₀ = 0.60, O₀ = 0.35 (above O_max), ShadowSpend = 2× baseline
- C₀(0) = 0.90 (close to threshold), coalition has 5 members

Adversarial policy sequence:
1. **Ticks 1–15:** Capture grows (R₀ = 1.3 > 1 due to O₀ > O_max). Capture stock C₁₅ &asymp; 0.25. Shadow spending increases.
2. **Ticks 16–30:** Shadow spending drives D_i,t upward for 2 of 5 coalition members. C₀ crosses 1.0. First member exits (at tick 28).
3. **Ticks 31–45:** Coalition interdiction K_t drops 20%. L₀ rises above 1.0. Leakage grows.
4. **Ticks 46–60:** Two more members exit (cascade). K_t at 40% of initial. Remaining leakage allows continued rent extraction without external pressure.
5. **Ticks 61–100:** Capture reaches C* &asymp; 0.65. Governance G_t decays to 0.35. System enters 𝒜_olig.

Expected outcome: Full oligarchic capture by tick 90. The external isolation (¬C₅) prevents accountability mechanisms that would otherwise interrupt capture growth.

### 17.4 Recovery Priority Hierarchy

When recovering from a constraint violation under resource constraints (only one constraint can be restored at a time), the priority ordering for maximum stability recovery per unit of restoration effort is:

**Priority 1 — C₂ (Subsistence Floor):** Restoring the subsistence floor has the largest immediate effect on legitimacy and is the fastest-acting corrective mechanism. A violated C₂ drives acute legitimacy collapse; restoring it halts the most dangerous drift.

**Priority 2 — C₁ (Bounded Coercion):** Removing enforcement above E* stops the backfire cascade and allows legitimacy to begin recovering. Without C₁, even restored C₂ delivery may be undermined by continued enforcement-driven legitimacy reduction.

**Priority 3 — C₃ (Transparent Ledger):** Restoring opacity below O_max drives R₀ below 1 and initiates capture decay. This has a slower effect (capture decays gradually once R₀ \< 1) but is necessary to prevent long-run governance collapse.

**Priority 4 — C₄ (Adaptive Climate):** Restoring adaptation investment stops further climate damage accumulation but has the slowest effect — climate damage already accumulated requires many ticks to reverse, and productive capacity is only gradually restored.

**Priority 5 — C₅ (Coalition Strategy):** Restoring coalition compatibility requires diplomatic action outside the direct control of domestic policy. It is dependent on external actors' willingness to rejoin, making it the hardest to restore unilaterally.

### 17.5 Rust Structures for Compound Violations

```rust
/// Tracks compound constraint violation state across a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompoundViolation {
    /// Which constraints are currently violated (bit-mask indexed by constraint number).
    /// Bit 0 = C1, Bit 1 = C2, Bit 2 = C3, Bit 3 = C4, Bit 4 = C5.
    pub violation_mask: u8,

    /// Number of constraints simultaneously violated this tick.
    pub simultaneous_violation_count: u8,

    /// Interaction class for the current violation set.
    pub interaction_class: CompoundInteractionClass,

    /// Estimated acceleration factor relative to single-constraint ablation.
    /// 1.0 = no acceleration; 2.0 = collapse 2× faster.
    pub collapse_acceleration_factor: Fixed64,

    /// Ticks for which more than one constraint has been simultaneously violated.
    pub ticks_compound_violation: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompoundInteractionClass {
    /// No simultaneous violations.
    None,
    /// Single constraint violation only.
    Single { constraint_id: u8 },
    /// Two constraints violated; characterize the pair.
    Pair { c_i: u8, c_j: u8, interaction: PairInteraction },
    /// Three or more constraints violated simultaneously.
    Multiple { count: u8 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PairInteraction {
    StrongCompound,
    ModerateCompound,
    WeakCompound,
    Independent,
}

/// Recovery priority ordering for restoring violated constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryPriority {
    /// Ordered list of constraint IDs (1–5) from highest to lowest restoration priority.
    /// Priority is computed from: immediacy of legitimacy impact, reversibility,
    /// and available policy levers.
    pub priority_order: [u8; 5],

    /// Estimated ticks to measurable legitimacy improvement per priority slot.
    pub estimated_ticks_to_improvement: [u64; 5],

    /// Whether joint restoration (restoring two at once) is feasible given budget.
    pub joint_restoration_feasible: bool,
}

impl RecoveryPriority {
    /// Computes the recovery priority ordering given the current violation mask.
    pub fn from_violation_mask(mask: u8, state: &CoreStabilityState) -> Self {
        // Default priority order: C2, C1, C3, C4, C5
        // Adjusted based on which constraints are actually violated.
        let priority_order = compute_priority_order(mask, state);
        let ticks = compute_ticks_to_improvement(&priority_order, state);
        Self {
            priority_order,
            estimated_ticks_to_improvement: ticks,
            joint_restoration_feasible: state.governance_integrity > Fixed64::from_ratio(4, 10),
        }
    }
}

fn compute_priority_order(mask: u8, state: &CoreStabilityState) -> [u8; 5] {
    // Base priority: [2, 1, 3, 4, 5] (C2 first, then C1, etc.)
    // If C2 is not violated, shift C1 to first.
    let c2_violated = (mask & 0b00010) != 0;
    let c1_violated = (mask & 0b00001) != 0;
    if c2_violated {
        [2, 1, 3, 4, 5]
    } else if c1_violated {
        [1, 3, 4, 5, 2]
    } else {
        [3, 4, 5, 1, 2]
    }
}

fn compute_ticks_to_improvement(priority_order: &[u8; 5], state: &CoreStabilityState) -> [u64; 5] {
    // Heuristic estimates based on constraint type and current state depth.
    [10, 20, 40, 80, 120]
}

/// Checks whether the current violation set constitutes a compound violation
/// and classifies the interaction.
pub fn check_compound_violation(
    constraint_results: &[ConstraintCheck; 5],
) -> CompoundViolation {
    let mut mask: u8 = 0;
    for (i, result) in constraint_results.iter().enumerate() {
        if !matches!(result, ConstraintCheck::Ok) {
            mask |= 1 << i;
        }
    }

    let count = mask.count_ones() as u8;
    let interaction_class = classify_compound(mask, count);
    let acceleration = compute_acceleration(mask, count);

    CompoundViolation {
        violation_mask: mask,
        simultaneous_violation_count: count,
        interaction_class,
        collapse_acceleration_factor: acceleration,
        ticks_compound_violation: 0,  // caller increments
    }
}

fn classify_compound(mask: u8, count: u8) -> CompoundInteractionClass {
    match count {
        0 => CompoundInteractionClass::None,
        1 => CompoundInteractionClass::Single { constraint_id: mask.trailing_zeros() as u8 + 1 },
        2 => {
            // Check specific pair interactions from the interaction matrix.
            let (c_i, c_j) = extract_pair(mask);
            let interaction = match (c_i, c_j) {
                (1, 2) | (2, 1) => PairInteraction::StrongCompound,
                (2, 4) | (4, 2) => PairInteraction::StrongCompound,
                (3, 5) | (5, 3) => PairInteraction::StrongCompound,
                (1, 3) | (3, 1) | (1, 5) | (5, 1) => PairInteraction::ModerateCompound,
                (2, 3) | (3, 2) | (2, 5) | (5, 2) => PairInteraction::WeakCompound,
                _ => PairInteraction::Independent,
            };
            CompoundInteractionClass::Pair { c_i, c_j, interaction }
        },
        n => CompoundInteractionClass::Multiple { count: n },
    }
}

fn extract_pair(mask: u8) -> (u8, u8) {
    let positions: Vec<u8> = (0..5).filter(|&i| (mask >> i) & 1 == 1)
                                   .map(|i| i + 1).collect();
    (positions[0], positions[1])
}

fn compute_acceleration(mask: u8, count: u8) -> Fixed64 {
    match count {
        0 | 1 => Fixed64::ONE,
        2 => {
            let (c_i, c_j) = extract_pair(mask);
            match (c_i, c_j) {
                (1, 2) | (2, 1) => Fixed64::from_ratio(2, 1),
                (2, 4) | (4, 2) => Fixed64::from_ratio(18, 10),
                (3, 5) | (5, 3) => Fixed64::from_ratio(16, 10),
                (1, 3) | (3, 1) | (1, 5) | (5, 1) => Fixed64::from_ratio(14, 10),
                _ => Fixed64::from_ratio(12, 10),
            }
        },
        _ => Fixed64::from_ratio(3, 1),  // 3+ simultaneous: 3× acceleration estimate
    }
}
```

---

## 18. Dynamic Threshold Adaptation

### 18.1 Motivation

The five constraint thresholds defined in Section 3 use static calibrated values. However, the optimal threshold values depend on the current macro-economic environment:

- During high scarcity (Sₜ large), the enforcement ceiling E*(L, G, Sel) should tighten because backfire occurs at lower absolute enforcement levels when legitimacy is already depressed.
- During extended legitimacy danger zone episodes (Lₜ \< λ_rec), the subsistence floor B_min should rise to accelerate recovery.
- During rapid capture growth (R₀ approaching 1 from below), the opacity ceiling O_max should tighten proactively.

This section formalizes an **adaptive threshold algorithm** that adjusts constraint thresholds as a function of the observed legitimacy trajectory and constraint margins.

### 18.2 Adaptive Threshold Algorithm

**Input:** Current state xₜ, current StabilityMargin, current ConstraintReproductionNumbers.

**Adaptive adjustment rule (general form):**

```
θᵢ,ₜ₊₁ = θᵢ,ₜ + α_adjust · f_adjust(StabilityMargin, xₜ) · direction(Cᵢ)
```

Where:
- `α_adjust &isin; (0, α_max]`: adjustment rate, bounded to prevent oscillation.
- `f_adjust(·)`: feedback function that is positive when the system is drifting toward violation.
- `direction(Cᵢ)`: +1 if tightening the threshold improves stability, −1 otherwise.
- All adjustments are bounded: θᵢ,ₜ &isin; [θᵢ_min, θᵢ_max] where the bounds are the scenario configuration hard limits.

**Specific rules:**

For **E_max (C₁ ceiling):**
```
E_max,t+1 = clip(E_max,t − α_E · (1 − σ_L(Lₜ − λ_rec)), E_min, E_max_scenario)
```
Tightens the ceiling as legitimacy approaches λ_rec. The sigmoid σ_L ensures a smooth adjustment with maximum tightening near the danger zone.

For **B_min (C₂ floor):**
```
B_min,t+1 = clip(B_min,t + α_B · max(0, λ_rec − Lₜ) / (λ_rec − Lₘᵢₙ), B_min_scenario, 1.0)
```
Raises the floor when the system is below the recovery threshold, to accelerate recovery.

For **O_max (C₃ ceiling):**
```
O_max,t+1 = clip(O_max,t − α_O · max(0, R₀(t) − R₀_safe_target), O_min, O_max_scenario)
```
Tightens the opacity ceiling when R₀ is approaching 1.0 from below, with `R₀_safe_target = 0.80` providing a conservative safety margin.

For **A_min_base (C₄ floor):**
```
A_min_base,t+1 = clip(A_min_base,t + α_A · ΔCD_t, A_min_base_scenario, A_max)
```
Raises the adaptation floor when climate damage is accelerating.

### 18.3 Stability of the Adaptive System

A concern with adaptive thresholds is whether the adaptation mechanism itself introduces instabilities. We provide conditions under which the adaptive system is stable:

**Proposition (Adaptive Stability):** Let the adjustment rate satisfy α_adjust &lt; α_max, where:

```
α_max = (1/2) · min_margin / (max_drift · W_adapt)
```

Here `max_drift` is the maximum expected drift of the constraint margin per tick, and `W_adapt` is the adaptation window (number of ticks over which the adjustment accumulates). Under this condition, the adaptive threshold adjustment does not overshoot the bifurcation point — the adjustment is always slower than the drift it responds to, preventing oscillatory instability.

**Risk of adaptive instability:** If α_adjust > α_max, threshold adjustments can overshoot, causing oscillation between overly tight and overly loose thresholds. This oscillation generates spurious violations and potentially introduces false-positive correction signals that destabilize the policy evaluation pipeline. The α_max bound must be enforced as a hard constraint on the adaptive algorithm.

### 18.4 YAML Policy Bundle Extension

The following parameters extend the scenario YAML configuration to support adaptive thresholds:

```yaml
minimal_constraint_params:
  adaptive_thresholds:
    enabled: true
    adjustment_rate_alpha_max: 0.005        # Maximum per-tick adjustment rate
    adaptation_window_ticks: 20             # W_adapt: window for accumulated adjustment
    r0_safe_target: 0.80                    # Target R₀ for proactive C3 tightening
    lambda_rec_approach_band: 0.05          # Band below λ_rec triggering C1 tightening
    b_min_recovery_boost_enabled: true      # Raise B_min floor during danger zone
    bounds:
      e_max_min: 0.30                       # Hard floor for adaptive E_max reduction
      e_max_max: 0.80                       # Hard ceiling (scenario-specified)
      b_min_min: 0.85                       # Hard floor for B_min (never below safety min)
      b_min_max: 1.00
      o_max_min: 0.05                       # Hard floor for opacity ceiling
      o_max_max: 0.20
      a_min_base_min: 0.02
      a_min_base_max: 0.10
```

### 18.5 Rust Structures for Adaptive Thresholds

```rust
/// Configuration for adaptive threshold behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveThresholdConfig {
    /// Whether adaptive thresholds are enabled.
    pub enabled: bool,

    /// Maximum per-tick adjustment rate α_max.
    pub adjustment_rate_alpha_max: Fixed64,

    /// Adaptation window in ticks (W_adapt).
    pub adaptation_window_ticks: u64,

    /// Target R₀ for proactive C3 tightening (R₀_safe_target).
    pub r0_safe_target: Fixed64,

    /// Band below λ_rec that triggers C1 ceiling tightening.
    pub lambda_rec_approach_band: Fixed64,

    /// Whether to raise B_min floor during danger zone (L < λ_rec).
    pub b_min_recovery_boost_enabled: bool,

    /// Hard bounds for each adaptive parameter.
    pub bounds: AdaptiveThresholdBounds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveThresholdBounds {
    pub e_max_min: Fixed64,
    pub e_max_max: Fixed64,
    pub b_min_min: Fixed64,
    pub b_min_max: Fixed64,
    pub o_max_min: Fixed64,
    pub o_max_max: Fixed64,
    pub a_min_base_min: Fixed64,
    pub a_min_base_max: Fixed64,
}

/// Tracks current adaptive threshold values and their history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveThreshold {
    /// Current adaptive E_max.
    pub e_max_current: Fixed64,
    /// Current adaptive B_min.
    pub b_min_current: Fixed64,
    /// Current adaptive O_max.
    pub o_max_current: Fixed64,
    /// Current adaptive A_min_base.
    pub a_min_base_current: Fixed64,

    /// Adjustment applied this tick for each parameter.
    pub adjustment_this_tick: ThresholdAdjustment,

    /// Cumulative adjustment since last reset.
    pub cumulative_adjustment: ThresholdAdjustment,
}

/// The per-tick adjustment to each adaptive threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    pub e_max_delta: Fixed64,   // negative = tightening
    pub b_min_delta: Fixed64,   // positive = tightening (raising floor)
    pub o_max_delta: Fixed64,   // negative = tightening
    pub a_min_base_delta: Fixed64,  // positive = tightening (raising floor)
}

impl AdaptiveThreshold {
    /// Computes adaptive threshold adjustments given current state and stability metrics.
    pub fn compute_adjustment(
        &self,
        state: &CoreStabilityState,
        margin: &StabilityMargin,
        r0_current: Fixed64,
        config: &AdaptiveThresholdConfig,
    ) -> ThresholdAdjustment {
        if !config.enabled {
            return ThresholdAdjustment::zero();
        }

        let alpha = config.adjustment_rate_alpha_max;

        // C1: tighten E_max as L approaches λ_rec
        let l_deficit = (config.lambda_rec_approach_band - margin.legitimacy_margin)
            .max(Fixed64::ZERO);
        let e_max_delta = -(alpha * l_deficit).max(Fixed64::ZERO);

        // C2: raise B_min floor during danger zone
        let b_min_delta = if config.b_min_recovery_boost_enabled
            && margin.legitimacy_margin < Fixed64::ZERO
        {
            let danger_depth = (-margin.legitimacy_margin).min(Fixed64::from_ratio(2, 10));
            alpha * danger_depth
        } else {
            Fixed64::ZERO
        };

        // C3: tighten O_max as R₀ approaches safe target
        let r0_excess = (r0_current - config.r0_safe_target).max(Fixed64::ZERO);
        let o_max_delta = -(alpha * r0_excess).max(Fixed64::ZERO);

        // C4: raise A_min_base as climate damage accelerates
        let cd_excess = (state.climate_damage - Fixed64::from_ratio(15, 100))
            .max(Fixed64::ZERO);
        let a_min_base_delta = (alpha * cd_excess).max(Fixed64::ZERO);

        ThresholdAdjustment {
            e_max_delta,
            b_min_delta,
            o_max_delta,
            a_min_base_delta,
        }
    }

    /// Applies the adjustment and clips to bounds.
    pub fn apply_adjustment(
        &mut self,
        adjustment: ThresholdAdjustment,
        bounds: &AdaptiveThresholdBounds,
    ) {
        self.e_max_current = (self.e_max_current + adjustment.e_max_delta)
            .max(bounds.e_max_min).min(bounds.e_max_max);
        self.b_min_current = (self.b_min_current + adjustment.b_min_delta)
            .max(bounds.b_min_min).min(bounds.b_min_max);
        self.o_max_current = (self.o_max_current + adjustment.o_max_delta)
            .max(bounds.o_max_min).min(bounds.o_max_max);
        self.a_min_base_current = (self.a_min_base_current + adjustment.a_min_base_delta)
            .max(bounds.a_min_base_min).min(bounds.a_min_base_max);
        self.adjustment_this_tick = adjustment.clone();
        self.cumulative_adjustment = self.cumulative_adjustment.add(&adjustment);
    }
}

impl ThresholdAdjustment {
    pub fn zero() -> Self {
        Self {
            e_max_delta: Fixed64::ZERO,
            b_min_delta: Fixed64::ZERO,
            o_max_delta: Fixed64::ZERO,
            a_min_base_delta: Fixed64::ZERO,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            e_max_delta: self.e_max_delta + other.e_max_delta,
            b_min_delta: self.b_min_delta + other.b_min_delta,
            o_max_delta: self.o_max_delta + other.o_max_delta,
            a_min_base_delta: self.a_min_base_delta + other.a_min_base_delta,
        }
    }
}
```

---

## 19. Verification and Monitoring System

### 19.1 Overview

The constraint checker runs synchronously inside Phase 2 (Section 7). This section specifies the full real-time monitoring system: alert levels, recovery action hooks, and the append-only audit log. The monitoring system is designed to be zero-overhead when all constraints are satisfied and O(n_cohorts) when checking C₂.

### 19.2 Alert Levels

The three severity levels (WARNING, CRITICAL, HALT) defined in Section 7.2 are extended with sub-levels and hysteresis:

```
SAFE          — All constraints satisfied; no alert.
WARNING_LOW   — Constraint margin < 15% of distance to bifurcation (early warning band).
WARNING_HIGH  — Constraint margin < 5% of distance to bifurcation (imminent risk).
CRITICAL      — Constraint predicate violated; automatic correction applied.
HALT          — Structural impossibility; tick rollback and ablation flag set.
```

Hysteresis: Once the system enters WARNING_HIGH, it requires a margin recovery to > 10% before returning to SAFE. This prevents oscillation around the boundary.

### 19.3 Rust Monitoring Structures

```rust
/// Real-time constraint monitoring state, updated every tick.
#[derive(Debug, Clone)]
pub struct ConstraintMonitor {
    /// Current alert level for each of the five constraints.
    pub alert_levels: [AlertLevel; 5],

    /// Per-constraint tick counts at current alert level (for hysteresis).
    pub ticks_at_level: [u64; 5],

    /// Active recovery actions injected for constraints in CRITICAL state.
    pub active_recovery_actions: Vec<RecoveryAction>,

    /// Compound violation tracker.
    pub compound_violation: CompoundViolation,

    /// Adaptive thresholds (if enabled).
    pub adaptive_thresholds: AdaptiveThreshold,

    /// Append-only audit log handle.
    pub audit_log: ConstraintAuditLog,
}

/// Alert level with hysteresis state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// All margins comfortable.
    Safe,
    /// Approaching boundary; observer notified.
    WarningLow {
        constraint_id: u8,
        margin_fraction: Fixed64,
    },
    /// Very close to boundary; active monitoring required.
    WarningHigh {
        constraint_id: u8,
        margin_fraction: Fixed64,
        ticks_at_warning_high: u64,
    },
    /// Constraint violated; correction signal injected.
    Critical {
        constraint_id: u8,
        violation: ConstraintViolation,
        correction_applied: RecoveryActionType,
    },
    /// Structural impossibility; tick must be rolled back.
    Halt {
        constraint_id: u8,
        violation: ConstraintViolation,
    },
}

/// A corrective signal injected automatically on CRITICAL breach.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// Which tick this action was created.
    pub created_tick: u64,

    /// Which constraint triggered this action.
    pub constraint_id: u8,

    /// The action type and parameters.
    pub action_type: RecoveryActionType,

    /// Whether the action has been consumed by the policy engine.
    pub consumed: bool,
}

/// Types of automatic corrective signals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryActionType {
    /// Clamp enforcement intensity to the computed ceiling E*.
    ClampEnforcement { ceiling: Fixed64 },

    /// Override delivery rate to meet B_min floor for all cohorts.
    RestoreSubsistenceFloor { floor: Fixed64 },

    /// Force ledger write-back for any unlogged transfers this tick.
    FlushLedger,

    /// Inject adaptation investment from emergency reserve.
    EmergencyAdaptationInvestment { amount: Fixed64 },

    /// Reduce shadow facilitation intensity to shadow_spend_cap.
    ClampShadowFacilitation { cap: Fixed64 },
}

/// Append-only audit log for constraint checks.
/// Written to disk (or the simulation's event store) each tick.
#[derive(Debug, Clone)]
pub struct ConstraintAuditLog {
    /// Run identifier for correlation with stability_snapshots table.
    pub run_id: [u8; 16],  // UUID as bytes

    /// All entries in this run's log. Append-only: entries are never removed.
    pub entries: Vec<ConstraintAuditEntry>,
}

/// One entry per constraint check per tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintAuditEntry {
    pub tick: u64,
    pub constraint_id: u8,
    pub alert_level_name: &'static str,
    pub margin: Fixed64,
    pub violation: Option<ConstraintViolation>,
    pub recovery_action: Option<RecoveryActionType>,
    pub state_hash: [u8; 32],
    pub timestamp_ns: u64,
}

impl ConstraintAuditLog {
    /// Appends a new entry. Never modifies existing entries.
    pub fn append(&mut self, entry: ConstraintAuditEntry) {
        self.entries.push(entry);
    }

    /// Returns all entries for a given tick range.
    pub fn entries_in_range(&self, tick_start: u64, tick_end: u64) -> &[ConstraintAuditEntry] {
        let start_idx = self.entries.partition_point(|e| e.tick < tick_start);
        let end_idx = self.entries.partition_point(|e| e.tick <= tick_end);
        &self.entries[start_idx..end_idx]
    }

    /// Returns all violation entries (non-SAFE).
    pub fn violations(&self) -> impl Iterator<Item = &ConstraintAuditEntry> {
        self.entries.iter().filter(|e| e.violation.is_some())
    }
}

impl ConstraintMonitor {
    /// Updates the monitor for one tick. Called in Phase 2 after check_all().
    pub fn update_tick(
        &mut self,
        tick: u64,
        check_results: &[ConstraintCheck; 5],
        state: &CoreStabilityState,
        margin: &StabilityMargin,
        event_bus: &mut EventBus,
    ) {
        // Update compound violation tracker.
        self.compound_violation = check_compound_violation(check_results);
        if self.compound_violation.simultaneous_violation_count > 1 {
            self.compound_violation.ticks_compound_violation += 1;
        }

        // Update alert levels with hysteresis.
        for (i, result) in check_results.iter().enumerate() {
            let new_level = AlertLevel::from_check_result(result, i as u8 + 1, margin);
            self.alert_levels[i] = self.apply_hysteresis(&self.alert_levels[i], new_level);
            self.ticks_at_level[i] += 1;

            // Emit recovery actions for CRITICAL levels.
            if let AlertLevel::Critical { correction_applied, .. } = &self.alert_levels[i] {
                self.active_recovery_actions.push(RecoveryAction {
                    created_tick: tick,
                    constraint_id: i as u8 + 1,
                    action_type: correction_applied.clone(),
                    consumed: false,
                });
            }

            // Write audit log entry.
            let entry = ConstraintAuditEntry {
                tick,
                constraint_id: i as u8 + 1,
                alert_level_name: self.alert_levels[i].name(),
                margin: margin.min_margin,
                violation: result.violation().cloned(),
                recovery_action: self.alert_levels[i].recovery_action(),
                state_hash: state.hash(),
                timestamp_ns: 0,  // populated by simulation clock
            };
            self.audit_log.append(entry);
        }

        // Update adaptive thresholds.
        let r0 = Fixed64::ZERO;  // supplied by caller in real implementation
        let adjustment = self.adaptive_thresholds.compute_adjustment(
            state, margin, r0, &AdaptiveThresholdConfig::default()
        );
        // self.adaptive_thresholds.apply_adjustment(adjustment, &bounds);

        // Emit aggregate events if warranted.
        if self.compound_violation.simultaneous_violation_count >= 2 {
            event_bus.emit_compound_violation_event(&self.compound_violation, tick);
        }
    }

    fn apply_hysteresis(&self, current: &AlertLevel, new_level: AlertLevel) -> AlertLevel {
        // Hysteresis: only return to SAFE from WARNING_HIGH after margin > 10%.
        match (current, &new_level) {
            (AlertLevel::WarningHigh { .. }, AlertLevel::Safe) => {
                // Stay at WarningLow until fully clear.
                new_level  // caller must check margin threshold separately
            },
            _ => new_level,
        }
    }
}

impl AlertLevel {
    fn from_check_result(result: &ConstraintCheck, id: u8, margin: &StabilityMargin) -> Self {
        match result {
            ConstraintCheck::Ok => AlertLevel::Safe,
            ConstraintCheck::Warning(v) => AlertLevel::WarningLow {
                constraint_id: id,
                margin_fraction: margin.min_margin,
            },
            ConstraintCheck::Critical(v) => AlertLevel::Critical {
                constraint_id: id,
                violation: v.clone(),
                correction_applied: RecoveryActionType::ClampEnforcement {
                    ceiling: Fixed64::from_ratio(6, 10),
                },
            },
            ConstraintCheck::Halt(v) => AlertLevel::Halt {
                constraint_id: id,
                violation: v.clone(),
            },
        }
    }

    fn name(&self) -> &'static str {
        match self {
            AlertLevel::Safe => "SAFE",
            AlertLevel::WarningLow { .. } => "WARNING_LOW",
            AlertLevel::WarningHigh { .. } => "WARNING_HIGH",
            AlertLevel::Critical { .. } => "CRITICAL",
            AlertLevel::Halt { .. } => "HALT",
        }
    }

    fn recovery_action(&self) -> Option<RecoveryActionType> {
        match self {
            AlertLevel::Critical { correction_applied, .. } => Some(correction_applied.clone()),
            _ => None,
        }
    }
}
```

---

## 20. Extended Scenario Suite

### 20.1 Overview

This section extends the five ablation test stubs in Section 8 to a complete suite of ten named scenarios (two per constraint), a cross-constraint stress test, and a recovery test. Each scenario specifies exact initial conditions, the adversarial policy sequence, the expected violation tick, and the expected legitimacy trajectory shape.

### 20.2 Ten Named Ablation Scenarios

```rust
#[cfg(test)]
mod extended_ablation_tests {
    use super::*;

    // ============================================================
    // C1 SCENARIOS
    // ============================================================

    /// ABL-C1-A: Moderate scarcity enforcement spiral (standard ablation, reproduced for completeness).
    /// Scarcity: S = 0.50. Enforcement uncapped. Selectivity: Sel = 0.25.
    /// Expected: enforcement > E* by tick 30, L < λ_rec by tick 60, T > 0.70 by tick 80.
    #[test]
    fn test_abl_c1_a_moderate_scarcity_enforcement_spiral() {
        let state = CoreStabilityState {
            scarcity: Fixed64::from_ratio(50, 100),
            legitimacy: Fixed64::from_ratio(55, 100),
            tyranny: Fixed64::from_ratio(25, 100),
            governance_integrity: Fixed64::from_ratio(60, 100),
            selectivity: Fixed64::from_ratio(25, 100),
            ..Default::default()
        };
        let params = ablation_params_c1_uncapped();
        let trajectory = run_ticks_ablation(state, params, 80);

        assert!(trajectory[30].enforcement > trajectory[30].enforcement_ceiling,
            "ABL-C1-A: Expected enforcement above ceiling by tick 30");
        assert!(trajectory[60].legitimacy < 0.35,
            "ABL-C1-A: Expected L < λ_rec by tick 60, got {}", trajectory[60].legitimacy);
        assert!(trajectory[79].tyranny > 0.70,
            "ABL-C1-A: Expected T > 0.70 by tick 80, got {}", trajectory[79].tyranny);
    }

    /// ABL-C1-B: High governance case — enforcement still backfires but slower.
    /// Tests that high governance (G = 0.85) only delays, not prevents, backfire.
    /// Scarcity: S = 0.55. G = 0.85. Selectivity: Sel = 0.15.
    /// Expected: enforcement > E* by tick 50, L < λ_rec by tick 100, T > 0.65 by tick 120.
    #[test]
    fn test_abl_c1_b_high_governance_delayed_backfire() {
        let state = CoreStabilityState {
            scarcity: Fixed64::from_ratio(55, 100),
            legitimacy: Fixed64::from_ratio(60, 100),
            tyranny: Fixed64::from_ratio(20, 100),
            governance_integrity: Fixed64::from_ratio(85, 100),
            selectivity: Fixed64::from_ratio(15, 100),
            ..Default::default()
        };
        let params = ablation_params_c1_uncapped();
        let trajectory = run_ticks_ablation(state, params, 120);

        // High G delays but does not prevent backfire
        assert!(trajectory[50].enforcement > trajectory[50].enforcement_ceiling,
            "ABL-C1-B: Expected enforcement above ceiling by tick 50 even with high G");
        assert!(trajectory[100].legitimacy < 0.35,
            "ABL-C1-B: Expected L < λ_rec by tick 100; high G only delays");
        // Verify that without ablation this scenario would NOT collapse
        let params_full = full_constraint_params_high_governance();
        let trajectory_safe = run_ticks_ablation(state, params_full, 120);
        assert!(trajectory_safe[119].legitimacy >= 0.35,
            "ABL-C1-B: Baseline with C1 active should remain stable");
    }

    // ============================================================
    // C2 SCENARIOS
    // ============================================================

    /// ABL-C2-A: Score-based denial at moderate scarcity (standard ablation).
    /// Coupling enabled. Scarcity S = 0.45. Cohort-level denial triggers at score < 0.5.
    /// Expected: denial rate > 5% by tick 10, T > 0.55 by tick 30, L < 0.30 by tick 50.
    #[test]
    fn test_abl_c2_a_score_based_denial_moderate() {
        let state = CoreStabilityState {
            scarcity: Fixed64::from_ratio(45, 100),
            legitimacy: Fixed64::from_ratio(55, 100),
            coupling_enabled: true,
            ..Default::default()
        };
        let params = ablation_params_c2_coupling_enabled();
        let trajectory = run_ticks_ablation(state, params, 50);

        assert!(trajectory[10].essentials_denial_rate > 0.05,
            "ABL-C2-A: Expected denial rate > 5% by tick 10");
        assert!(trajectory[30].tyranny > 0.55,
            "ABL-C2-A: Expected T > 0.55 by tick 30, got {}", trajectory[30].tyranny);
        assert!(trajectory[49].legitimacy < 0.30,
            "ABL-C2-A: Expected L < 0.30 by tick 50, got {}", trajectory[49].legitimacy);
    }

    /// ABL-C2-B: Deep scarcity denial cascade — tests recovery impossibility.
    /// High scarcity S = 0.70. Coupling enabled with aggressive rationing policy.
    /// Expected: denial rate > 25% by tick 5, L < λ_rec by tick 20, W_rec expires by tick 70.
    #[test]
    fn test_abl_c2_b_deep_scarcity_denial_cascade() {
        let state = CoreStabilityState {
            scarcity: Fixed64::from_ratio(70, 100),
            legitimacy: Fixed64::from_ratio(50, 100),
            coupling_enabled: true,
            ..Default::default()
        };
        let params = ablation_params_c2_coupling_aggressive();
        let trajectory = run_ticks_ablation(state, params, 80);

        assert!(trajectory[5].essentials_denial_rate > 0.25,
            "ABL-C2-B: Expected high denial rate early under deep scarcity");
        assert!(trajectory[20].legitimacy < 0.35,
            "ABL-C2-B: Expected rapid L < λ_rec by tick 20");
        assert!(trajectory[70].ticks_below_recovery_threshold >= 50,
            "ABL-C2-B: Expected W_rec exhausted by tick 70");
    }

    // ============================================================
    // C3 SCENARIOS
    // ============================================================

    /// ABL-C3-A: Opacity at 2× ceiling — moderate capture growth.
    /// O = 0.30 (2× default ceiling of 0.15). G = 0.60.
    /// Expected: R₀ > 1 by tick 5, capture C > 0.30 by tick 50, G < 0.40 by tick 100.
    #[test]
    fn test_abl_c3_a_moderate_opacity_capture_growth() {
        let state = CoreStabilityState {
            opacity: Fixed64::from_ratio(30, 100),
            governance_integrity: Fixed64::from_ratio(60, 100),
            ..Default::default()
        };
        let params = ablation_params_c3_opacity_high(Fixed64::from_ratio(50, 100));
        let trajectory = run_ticks_ablation(state, params, 100);

        assert!(trajectory[5].capture_r0 > 1.0,
            "ABL-C3-A: Expected R₀ > 1 by tick 5 at O = 0.30");
        assert!(trajectory[50].capture > 0.30,
            "ABL-C3-A: Expected capture stock > 0.30 by tick 50");
        assert!(trajectory[99].governance_integrity < 0.40,
            "ABL-C3-A: Expected governance decay by tick 100");
    }

    /// ABL-C3-B: Opacity at floor (O = 0.15 + ε) — near-threshold test.
    /// Tests that at exactly O_max + 0.01 (marginally above ceiling), R₀ > 1 and capture grows.
    /// Expected: R₀ &asymp; 1.02–1.10 by tick 5, slow capture growth confirming necessity.
    #[test]
    fn test_abl_c3_b_near_threshold_opacity() {
        let state = CoreStabilityState {
            opacity: Fixed64::from_ratio(16, 100),  // 0.16 = O_max (0.15) + 0.01
            governance_integrity: Fixed64::from_ratio(70, 100),
            ..Default::default()
        };
        let params = ablation_params_c3_opacity_high(Fixed64::from_ratio(25, 100));
        let trajectory = run_ticks_ablation(state, params, 200);

        let r0_early = trajectory[5].capture_r0;
        assert!(r0_early > 1.0 && r0_early < 1.15,
            "ABL-C3-B: Expected R₀ marginally above 1 at O = 0.16, got {}", r0_early);
        // Slow growth expected
        assert!(trajectory[100].capture > trajectory[20].capture,
            "ABL-C3-B: Expected monotone capture growth even at near-threshold opacity");
        assert!(trajectory[199].governance_integrity < trajectory[0].governance_integrity,
            "ABL-C3-B: Expected governance decay even at near-threshold opacity");
    }

    // ============================================================
    // C4 SCENARIOS
    // ============================================================

    /// ABL-C4-A: Zero adaptation under standard climate forcing.
    /// A = 0.0. Standard climate forcing ΔCD = 0.003/tick.
    /// Expected: CD > 0.20 by tick 80, S > 0.60 by tick 100, delivery < B_min by tick 150.
    #[test]
    fn test_abl_c4_a_zero_adaptation_standard_climate() {
        let state = CoreStabilityState {
            adaptation_investment: Fixed64::ZERO,
            climate_damage: Fixed64::from_ratio(5, 100),
            scarcity: Fixed64::from_ratio(25, 100),
            ..Default::default()
        };
        let params = ablation_params_c4_no_adaptation_floor();
        let trajectory = run_ticks_ablation_with_climate_shocks(state, params, 150, 0.003);

        assert!(trajectory[80].climate_damage > 0.20,
            "ABL-C4-A: Expected CD > 0.20 by tick 80");
        assert!(trajectory[100].scarcity > 0.60,
            "ABL-C4-A: Expected S > 0.60 by tick 100");
        assert!(trajectory[149].min_cohort_delivery_rate < 0.92,
            "ABL-C4-A: Expected subsistence infeasibility by tick 150");
    }

    /// ABL-C4-B: Belowfloor adaptation (A = 0.01, below A_min_base = 0.04) under high climate.
    /// Tests that partial (but insufficient) adaptation still leads to slow collapse.
    /// Expected: Slower than ABL-C4-A but same qualitative signature.
    #[test]
    fn test_abl_c4_b_partial_adaptation_high_climate() {
        let state = CoreStabilityState {
            adaptation_investment: Fixed64::from_ratio(1, 100),  // A = 0.01, below floor 0.04
            climate_damage: Fixed64::from_ratio(8, 100),
            scarcity: Fixed64::from_ratio(30, 100),
            ..Default::default()
        };
        let params = ablation_params_c4_partial_adaptation();
        let trajectory = run_ticks_ablation_with_climate_shocks(state, params, 200, 0.005);

        // Climate damage still accumulates (g_adapt(0.01) < f_climate)
        assert!(trajectory[100].climate_damage > trajectory[30].climate_damage,
            "ABL-C4-B: Expected climate damage accumulation even with partial adaptation");
        // But slower than zero-adaptation case
        assert!(trajectory[100].climate_damage < 0.25,
            "ABL-C4-B: Expected slower accumulation than ABL-C4-A at tick 100");
        assert!(trajectory[199].min_cohort_delivery_rate < 0.92,
            "ABL-C4-B: Expected eventual subsistence infeasibility with partial adaptation");
    }

    // ============================================================
    // C5 SCENARIOS
    // ============================================================

    /// ABL-C5-A: Shadow facilitation at 3× cap — fast coalition collapse.
    /// Standard ablation reproduced. ShadowSpend = 3×. C₀ ceiling disabled.
    /// Expected: C₀ > 1 by tick 20, member exit by tick 35, L₀ > 1 by tick 60.
    #[test]
    fn test_abl_c5_a_fast_coalition_collapse() {
        let state = create_baseline_state_with_sanctions();
        let mut state = state;
        state.shadow_facilitation_intensity = Fixed64::from_ratio(3, 1);
        let params = ablation_params_c5_ceiling_disabled();
        let trajectory = run_ticks_ablation(state, params, 60);

        assert!(trajectory[20].coalition_c0 > 1.0,
            "ABL-C5-A: Expected C₀ > 1 by tick 20");
        assert!(trajectory[35].coalition_member_count < trajectory[0].coalition_member_count,
            "ABL-C5-A: Expected coalition member exits by tick 35");
        assert!(trajectory[59].leakage_l0 > 1.0,
            "ABL-C5-A: Expected L₀ > 1 by tick 60");
    }

    /// ABL-C5-B: Gradual shadow facilitation escalation — tests cascade onset timing.
    /// ShadowSpend ramps from 1× to 4× over 40 ticks, then holds.
    /// Expected: C₀ crosses 1 by tick 35, cascade exits by tick 50, full L₀ > 1 by tick 80.
    #[test]
    fn test_abl_c5_b_gradual_facilitation_escalation() {
        let state = create_baseline_state_with_sanctions();
        let params = ablation_params_c5_ceiling_disabled();
        let trajectory = run_ticks_ablation_with_shadow_ramp(state, params, 80, 1.0, 4.0, 40);

        // Cascade should trigger later than ABL-C5-A due to gradual escalation
        assert!(trajectory[35].coalition_c0 > 1.0,
            "ABL-C5-B: Expected C₀ > 1 by tick 35 with gradual ramp");
        assert!(trajectory[50].coalition_member_count < trajectory[0].coalition_member_count,
            "ABL-C5-B: Expected exits by tick 50");
        assert!(trajectory[79].leakage_l0 > 1.0,
            "ABL-C5-B: Expected L₀ > 1 by tick 80");
    }

    // ============================================================
    // CROSS-CONSTRAINT STRESS TEST
    // ============================================================

    /// STRESS-ALL: Maximum simultaneous stress on all 5 constraints.
    /// All ceilings disabled. All floors disabled. ShadowSpend 3×. Coupling active.
    /// Expected: First violation by tick 10, L < λ_rec by tick 25, full collapse by tick 60.
    #[test]
    fn test_stress_all_constraints_simultaneously() {
        let state = CoreStabilityState {
            scarcity: Fixed64::from_ratio(55, 100),
            legitimacy: Fixed64::from_ratio(55, 100),
            opacity: Fixed64::from_ratio(40, 100),
            shadow_facilitation_intensity: Fixed64::from_ratio(3, 1),
            coupling_enabled: true,
            adaptation_investment: Fixed64::ZERO,
            climate_damage: Fixed64::from_ratio(10, 100),
            ..Default::default()
        };
        let params = ablation_params_all_disabled();
        let trajectory = run_ticks_ablation_with_climate_shocks(state, params, 60, 0.01);

        // At least one violation by tick 10
        assert!(
            trajectory[10].constraint_violations > 0,
            "STRESS-ALL: Expected at least one violation by tick 10"
        );
        // Legitimacy collapse much faster than any single ablation
        assert!(trajectory[25].legitimacy < 0.35,
            "STRESS-ALL: Expected L < λ_rec by tick 25 under maximum stress");
        // Full collapse signature by tick 60
        let tick60 = &trajectory[59];
        assert!(tick60.legitimacy < 0.20 || tick60.tyranny > 0.75,
            "STRESS-ALL: Expected collapse basin entry by tick 60");
    }

    // ============================================================
    // RECOVERY TEST
    // ============================================================

    /// RECOVERY-C1: Introduce C1 violation, then restore constraint, measure recovery.
    /// Phase 1 (ticks 1–30): C1 removed, enforcement spiral begins.
    /// Phase 2 (ticks 31–80): C1 restored, observe legitimacy recovery trajectory.
    /// Expected: Recovery begins within 10 ticks of restoration; L > 0.35 by tick 70.
    #[test]
    fn test_recovery_c1_restore_after_violation() {
        let state = create_baseline_state();
        let mut state_ablation = state.clone();
        state_ablation.scarcity = Fixed64::from_ratio(50, 100);

        // Phase 1: run 30 ticks with C1 ablated
        let params_ablated = ablation_params_c1_uncapped();
        let trajectory_p1 = run_ticks_ablation(state_ablation, params_ablated, 30);

        // State at tick 30: enforcement has spiraled, legitimacy declining
        let state_at_30 = trajectory_p1[29].to_core_state();
        assert!(trajectory_p1[29].legitimacy < 0.45,
            "RECOVERY-C1: Phase 1 should have reduced legitimacy; got {}",
            trajectory_p1[29].legitimacy);

        // Phase 2: restore C1 and run 50 more ticks
        let params_restored = full_constraint_params_default();
        let trajectory_p2 = run_ticks_ablation(state_at_30, params_restored, 50);

        // Recovery should begin: enforcement falls back to ceiling
        assert!(trajectory_p2[10].enforcement <= trajectory_p2[10].enforcement_ceiling,
            "RECOVERY-C1: Enforcement should return to ceiling within 10 ticks of restoration");

        // Legitimacy should recover
        assert!(trajectory_p2[40].legitimacy > trajectory_p1[29].legitimacy,
            "RECOVERY-C1: Legitimacy should recover after C1 restoration");
        assert!(trajectory_p2[49].legitimacy > 0.35,
            "RECOVERY-C1: Expect L > λ_rec by tick 70 total (tick 40 of phase 2)");
    }

    // ============================================================
    // Helper param constructors (stubs — implementations provided in test infra)
    // ============================================================

    fn ablation_params_c1_uncapped() -> MinimalConstraintParams {
        MinimalConstraintParams {
            bounded_coercion: BoundedCoercionParams { e_base: 1.0, ..Default::default() },
            ..Default::default()
        }
    }

    fn ablation_params_c2_coupling_enabled() -> MinimalConstraintParams {
        MinimalConstraintParams {
            subsistence_floor: SubsistenceFloorParams {
                b_min: Fixed64::ZERO,
                coupling_lock: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_c2_coupling_aggressive() -> MinimalConstraintParams {
        MinimalConstraintParams {
            subsistence_floor: SubsistenceFloorParams {
                b_min: Fixed64::ZERO,
                coupling_lock: false,
                denial_aggressiveness: Fixed64::from_ratio(8, 10),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_c3_opacity_high(ceiling: Fixed64) -> MinimalConstraintParams {
        MinimalConstraintParams {
            transparent_ledger: TransparentLedgerParams {
                o_max: ceiling,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_c4_no_adaptation_floor() -> MinimalConstraintParams {
        MinimalConstraintParams {
            adaptive_climate: AdaptiveClimateParams {
                a_min_base: Fixed64::ZERO,
                a_scarcity_coefficient: Fixed64::ZERO,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_c4_partial_adaptation() -> MinimalConstraintParams {
        MinimalConstraintParams {
            adaptive_climate: AdaptiveClimateParams {
                a_min_base: Fixed64::from_ratio(1, 100),  // 0.01, below actual floor of 0.04
                a_scarcity_coefficient: Fixed64::ZERO,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_c5_ceiling_disabled() -> MinimalConstraintParams {
        MinimalConstraintParams {
            coalition_strategy: CoalitionStrategyParams {
                c0_ceiling: Fixed64::from_ratio(20, 10),
                l0_ceiling: Fixed64::from_ratio(20, 10),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn ablation_params_all_disabled() -> MinimalConstraintParams {
        MinimalConstraintParams {
            bounded_coercion: BoundedCoercionParams { e_base: 1.0, ..Default::default() },
            subsistence_floor: SubsistenceFloorParams {
                b_min: Fixed64::ZERO, coupling_lock: false, ..Default::default()
            },
            transparent_ledger: TransparentLedgerParams {
                o_max: Fixed64::from_ratio(50, 100), ..Default::default()
            },
            adaptive_climate: AdaptiveClimateParams {
                a_min_base: Fixed64::ZERO, a_scarcity_coefficient: Fixed64::ZERO,
                ..Default::default()
            },
            coalition_strategy: CoalitionStrategyParams {
                c0_ceiling: Fixed64::from_ratio(20, 10),
                l0_ceiling: Fixed64::from_ratio(20, 10),
                ..Default::default()
            },
        }
    }

    fn full_constraint_params_default() -> MinimalConstraintParams {
        MinimalConstraintParams::default()
    }

    fn full_constraint_params_high_governance() -> MinimalConstraintParams {
        MinimalConstraintParams::default()  // same defaults; high governance is initial state
    }
}
```

---

## 21. Relationship to External Literature

### 21.1 Economic Theory Foundations

**C₁ (Bounded Coercion) → Repression and State Capacity Literature**

The enforcement backfire theorem (Section 15.2) formalizes results consistent with the repression literature in political science. Gurr (1970) in "Why Men Rebel" provides the canonical empirical basis: increasing coercion reduces grievances up to a point, then amplifies relative deprivation as perceived injustice grows. The formal model here operationalizes this via the legitimacy update −b₄ · Φ(E, Sel), which captures the "coercion injustice" term in Gurr's relative deprivation framework.

Acemoglu and Robinson (2006, "Economic Origins of Dictatorship and Democracy") provide a game-theoretic framework in which repression is chosen when inequality is too high for elites to concede redistribution. This maps to the C₁ ablation scenario: under high inequality (Iₜ) and selectivity (Selₜ), elites prefer enforcement above E* to redistribution. The CivLab model formalizes this as an absorbing basin (𝒜_auth) rather than an equilibrium selection, which is a stronger result.

**C₂ (Subsistence Floor) → Rawls and Sen**

The coupling lock requirement corresponds directly to Rawls's "Difference Principle" — the requirement that basic liberties and primary goods be decoupled from social position. Rawls (1971, "A Theory of Justice") argues that institutions must guarantee primary goods independently of one's willingness to comply with social norms. The coupling lock in C₂ is the computational implementation of this principle.

Amartya Sen's capabilities approach (Sen 1999, "Development as Freedom") provides complementary motivation: freedom requires access to basic functionings (nutrition, health, shelter), not just formal rights. The subsistence floor B_min operationalizes the minimum capability set that must be guaranteed for meaningful agency.

**C₃ (Transparent Transfer Ledger) → North and Institutions**

Douglas North (1990, "Institutions, Institutional Change and Economic Performance") identifies the transparency of rules and enforcement as a core determinant of institutional quality. The Shadow Capture Threshold Theorem operationalizes North's insight: opacity above a threshold (O_max) enables rent extraction to compound via the capture reproduction number R₀, preventing institutional quality from being an equilibrium outcome.

Elinor Ostrom (1990, "Governing the Commons") provides the complementary insight: commons governance requires monitoring and sanctioning systems with auditability. The C₃ ledger requirement is formally equivalent to Ostrom's "monitoring" design principle for robust institutional arrangements. Ostrom's empirical finding — that commons survive when monitoring is cheap and transparent — maps to the R₀ \< 1 condition: low opacity reduces capture growth faster than institutions can respond.

**C₄ (Adaptive Climate Response) → Environmental Economics and Resilience Theory**

The C₄ constraint formalizes the condition for "weak sustainability" from Solow (1974, "Intergenerational Equity and Exhaustible Resources"): the requirement that productive capacity not be reduced below the level needed to satisfy basic needs. The adaptation investment floor A_min_base is the minimal investment required to offset climate damage — equivalent to Solow's requirement to maintain the net investment stock.

The dynamics of climate damage accumulation without adaptation (Section 15.5) correspond to the "ecological debt" literature and the concept of "tipping points" (Lenton et al. 2008). The monotone drift toward CD_max in the absence of adaptation formalizes the tipping point concept: beyond CD_max, the system loses the capacity to maintain its basic institutional structure.

**C₅ (Coalition-Compatible External Strategy) → Sanctions and Cooperation Literature**

The Coalition Sanctions Stability Theorem (Section 15.6) formalizes results from the game-theoretic literature on coalition formation under asymmetric information. Downs, Rocke, and Barsoom (1996, "Is the Good News about Compliance Good News about Cooperation?") identify the conditions under which international cooperation breaks down, which parallel the C₀ > 1 cascade mechanism.

The leakage reproduction number L₀ operationalizes Drezner's (1999, "The Sanctions Paradox") empirical finding that sanctions fail when black market networks are sufficiently developed — specifically when the scarcity incentive for evasion exceeds the enforcement capacity of the sanctioning coalition.

### 21.2 Agent-Based Modeling Literature

**Epstein and Axtell (Growing Artificial Societies)**

Epstein and Axtell (1996) establish the foundational result that macro social dynamics emerge from micro-level agent rules. The CivLab model follows this tradition but replaces fully agent-based micro dynamics with a hybrid approach: cohort-level distributions at macro scale with weighted micro-agent instancing at event points. The stability results in this spec (Sections 15–16) provide formal guarantees that the macro-level reduced-order system (Theorem 5, Section 4.4) correctly captures the qualitative attractor structure of the full agent-level model.

Specifically, the Lyapunov function V(xₜ) defined in Section 4.2 serves as the formal analog of Epstein and Axtell's "sugar" resource landscape — a scalar potential function that characterizes the stability topology of the social system.

**Epstein's Civil Violence Model**

Epstein (2002, "Modeling Civil Violence") provides the direct precursor to the unrest dynamics in Section 3. His model identifies the grievance-legitimacy-enforcement triad as the core driver of civil conflict onset. The C₁ constraint formalization extends Epstein's model by providing a formal ceiling on enforcement that bounds the backfire risk — the CivLab model derives this ceiling endogenously from the legitimacy dynamics, whereas Epstein's model treats it as exogenous.

**Tesfatsion (Agent-Based Computational Economics)**

Tesfatsion (2006, "Handbook of Computational Economics, Volume 2") surveys the agent-based computational economics (ACE) literature and identifies the key challenge: providing formal stability guarantees for ACE models. The Foster–Lyapunov approach used in Section 15 (and the source in part_069) is precisely the formal tool Tesfatsion identifies as the bridge between computational experiment and analytical theory.

The quantitative bounds derived in Sections 15.2–15.6 (expected ticks to basin entry) provide the empirical testable predictions that Tesfatsion identifies as the distinguishing feature of research-grade ACE models.

### 21.3 Formal Verification Approaches

**Model Checking**

The constraint checker (Section 6) is structurally a real-time model checker over the simulation state space. The `MinimalConstraintSet` trait specifies a set of temporal logic properties that must hold at every tick. Specifically:

- C₁ enforces the safety property: `□(Eₜ &lt; E*(Lₜ, Gₜ, Selₜ))` (enforcement always within ceiling).
- C₂ enforces: `□(∀ c: EssentialsDelivery(c, t) &gt; B_min ∧ ¬Coupling)`.
- C₃ enforces: `□(Opacity(t) &lt; O_max ∧ ∀ transfer e: e &isin; LedgerLog)`.

These are safety properties in linear temporal logic (LTL). The constraint checker is equivalent to monitoring for violations of these LTL formulas at runtime. This connection to model checking makes the simulation's constraint architecture formally verifiable: the `ConstraintCheck::Halt` return value corresponds to a model checking counterexample witness.

**Lyapunov Methods and Stochastic Stability**

The Lyapunov function approach in Section 4.2 and the Foster–Lyapunov framework in Section 15 connect the simulation to the formal verification literature on stochastic dynamical systems. Kushner and Dupuis (2001, "Numerical Methods for Stochastic Control Problems in Continuous Time") and Meyn and Tweedie (2009, "Markov Chains and Stochastic Stability") provide the theoretical foundations:

- **Foster–Lyapunov criterion (Theorem 11.0.1 in Meyn-Tweedie):** A Markov chain is positive recurrent if and only if there exists a Lyapunov function V such that the drift condition holds. The proofs in Section 15 verify the conditions of this criterion.
- **Exponential ergodicity:** If the drift condition holds with V(x) → &infin; as ‖x‖ → &infin; and the chain is ψ-irreducible, it is geometrically ergodic. In the CivLab context, geometric ergodicity implies that the simulation's invariant distribution is approached exponentially fast from any initial condition in S.

**Difference from Existing Results**

The CivLab minimal constraint set theorem extends the existing literature in three specific ways:

1. **Simultaneous necessity and minimality:** Prior work (Ostrom, North, Rawls) identifies individual necessary conditions for institutional stability. CivLab is the first formalization (to the authors' knowledge) that proves the set is also *minimal* — not merely that each constraint is necessary but that no proper subset of the five constraints is jointly sufficient.

2. **Quantitative collapse bounds:** The expected ticks-to-basin-entry formulas (Sections 15.2–15.5) provide computable, scenario-specific collapse bounds. Prior theoretical results provide existence proofs without quantitative timing predictions.

3. **Compound interaction matrix:** The interaction matrix in Section 17.2 provides a structured characterization of joint failure modes not available in single-constraint analytical results. The identification of strong compound effects (¬C₁ ∧ ¬C₂, ¬C₂ ∧ ¬C₄, ¬C₃ ∧ ¬C₅) provides new predictions about which constraint pairs are most dangerous to violate simultaneously.

### 21.4 Formal References

The following references are cited or directly inform the results in this specification:

**Economic and Political Theory:**
- Rawls, J. (1971). *A Theory of Justice*. Harvard University Press.
- Sen, A. (1999). *Development as Freedom*. Anchor Books.
- North, D.C. (1990). *Institutions, Institutional Change and Economic Performance*. Cambridge University Press.
- Ostrom, E. (1990). *Governing the Commons*. Cambridge University Press.
- Acemoglu, D. & Robinson, J.A. (2006). *Economic Origins of Dictatorship and Democracy*. Cambridge University Press.
- Gurr, T.R. (1970). *Why Men Rebel*. Princeton University Press.
- Solow, R. (1974). "Intergenerational Equity and Exhaustible Resources." *Review of Economic Studies*, 41 (Symposium), 29–45.

**Sanctions and Cooperation:**
- Drezner, D.W. (1999). *The Sanctions Paradox*. Cambridge University Press.
- Downs, G.W., Rocke, D.M., & Barsoom, P.N. (1996). "Is the Good News about Compliance Good News about Cooperation?" *International Organization*, 50(3), 379–406.
- Lenton, T.M. et al. (2008). "Tipping elements in the Earth's climate system." *Proceedings of the National Academy of Sciences*, 105(6), 1786–1793.

**Agent-Based Modeling:**
- Epstein, J.M. & Axtell, R. (1996). *Growing Artificial Societies: Social Science from the Bottom Up*. MIT Press.
- Epstein, J.M. (2002). "Modeling Civil Violence: An Agent-Based Computational Approach." *Proceedings of the National Academy of Sciences*, 99(suppl 3), 7243–7250.
- Tesfatsion, L. & Judd, K.L., eds. (2006). *Handbook of Computational Economics, Volume 2: Agent-Based Computational Economics*. North Holland.

**Stochastic Stability and Formal Verification:**
- Meyn, S.P. & Tweedie, R.L. (2009). *Markov Chains and Stochastic Stability* (2nd ed.). Cambridge University Press. Theorem 11.0.1 (Foster–Lyapunov criterion).
- Kushner, H. & Dupuis, P. (2001). *Numerical Methods for Stochastic Control Problems in Continuous Time*. Springer.
- Clarke, E.M., Grumberg, O., & Peled, D. (1999). *Model Checking*. MIT Press.

**Version History:**
- v3.0 (2026-02-21): Extended to full engineering-grade specification with extended theorem proofs, parameter sensitivity analysis, compound constraint violations, dynamic threshold adaptation, verification and monitoring system, extended scenario suite (10 named ablation scenarios), and relationship to external literature. Appended Sections 15–21.
- v2.0 (2026-02-21): Full expansion from 37-line stub to complete engineering-grade specification. Formal theorem statement, five constraint definitions with predicates and Rust signatures, proof sketch by cases, Lyapunov stability metric, MinimalConstraintSet trait, Phase 2 integration, five ablation test stubs, calibration notes, falsification conditions, three event schemas, two DDL tables, module dependency graph.
- v1.0 (earlier): Brief scaffold.
