### **ChatGPT**

SHADOW-STATE CAPTURE THRESHOLD THEOREM
(CivLab – Formal Conditions for Oligarchic/Captured Regime Emergence v1.0)
This is the missing theorem layer that explains why systems drift into capture, even when baseline welfare and good intentions exist.
We’ll formalize shadow capture as a supercritical growth process on a hidden influence network, coupled to:

rent extraction

opacity/surveillance asymmetry

war/emergency spending

enforcement selectivity

legitimacy and oversight

Goal: derive conditions under which capture inevitably grows vs conditions under which it decays.

0) Reduced model objects
For a region rrr (drop subscript), define:

Gt∈[0,1]G\_t \\in [0,1]Gt​∈[0,1]: governance integrity (higher is better)

It∈[0,1]I\_t \\in [0,1]It​∈[0,1]: inequality/stratification (proxy for elite leverage)

Rt∈[0,1]R\_t \\in [0,1]Rt​∈[0,1]: rent extraction intensity (structural extraction)

Ot∈[0,1]O\_t \\in [0,1]Ot​∈[0,1]: opacity (lack of transparency / auditability)

Wt∈[0,1]W\_t \\in [0,1]Wt​∈[0,1]: war/emergency intensity (procurement secrecy, emergency powers)

Et∈[0,1]E\_t \\in [0,1]Et​∈[0,1]: enforcement intensity

Selt∈[0,1]Sel\_t \\in [0,1]Selt​∈[0,1]: enforcement selectivity (0 = uniform rule of law, 1 = highly selective)

Lt∈[0,1]L\_t \\in [0,1]Lt​∈[0,1]: legitimacy

Let the shadow system have an aggregate “capture stock”:

Ct∈[0,1]C\_t \\in [0,1]Ct​∈[0,1]: capture level (0 none, 1 fully captured)

And a hidden influence network with adjacency matrix AAA (not fully observed by player).
Let ρ(A)\\rho(A)ρ(A) denote the spectral radius (largest eigenvalue magnitude), which captures how “amplifying” the influence network is (high ρ\\rhoρ ⇒ influence spreads and compounds easily).

1) Capture dynamics (macro form)
Assume capture evolves as:
Ct+1=Ct+Γt(1−Ct)⏟capture growth−ΔtCt⏟capture decayC\_{t+1} = C\_t + \\underbrace{\\Gamma\_t(1-C\_t)}\_{\\text{capture growth}} - \\underbrace{\\Delta\_t C\_t}\_{\\text{capture decay}}Ct+1​=Ct​+capture growthΓt​(1−Ct​)​​−capture decayΔt​Ct​​​
Where:

Γt\\Gamma\_tΓt​ is the effective capture growth rate (how fast shadow networks expand influence into institutions)

Δt\\Delta\_tΔt​ is the effective decay rate (oversight, exposure, reform, enforcement against elites)

This is a logistic-type growth with decay.

2) Structural forms for growth and decay
2.1 Capture growth rate Γt\\Gamma\_tΓt​
Model growth as the product of:

opportunity (rents, war budgets, opacity)

network amplification (shadow graph)

institutional susceptibility (weak governance, selective enforcement)

resource fuel (black markets/leakage)

A tractable functional form:
Γt=α  ρ(A)  (Rt+ωWt)⏟rents + emergency budgets  Ot⏟opacity  (1−Gt+κSelt)⏟weak integrity + selectivity\\Gamma\_t = \\alpha\\;\\rho(A)\\;\\underbrace{(R\_t + \\omega W\_t)}\_{\\text{rents + emergency budgets}}\\;\\underbrace{O\_t}\_{\\text{opacity}}\\;\\underbrace{(1-G\_t + \\kappa Sel\_t)}\_{\\text{weak integrity + selectivity}}Γt​=αρ(A)rents + emergency budgets(Rt​+ωWt​)​​opacityOt​​​weak integrity + selectivity(1−Gt​+κSelt​)​​
Interpretation:

Rent and emergency spending provide payoff

Opacity makes capture easier and reduces exposure

Low integrity and selective enforcement accelerate it

Network spectral radius controls amplification speed

2.2 Capture decay rate Δt\\Delta\_tΔt​
Decay depends on:

oversight strength

transparency

impartial rule of law

exposure/scandals

civic pressure (legitimacy + civic norms, optional)

A tractable form:
Δt=β  (1−Ot)⏟transparency  Gt⏟integrity  (1−Selt)⏟impartial enforcement  +  χ  Exposuret\\Delta\_t = \\beta\\;\\underbrace{(1-O\_t)}\_{\\text{transparency}}\\;\\underbrace{G\_t}\_{\\text{integrity}}\\;\\underbrace{(1-Sel\_t)}\_{\\text{impartial enforcement}}\\;+\\;\\chi\\;\\text{Exposure}\_tΔt​=βtransparency(1−Ot​)​​integrityGt​​​impartial enforcement(1−Selt​)​​+χExposuret​
Exposure events are stochastic and more likely when transparency is higher and shadow activity is larger.

3) The threshold condition (the core theorem)
Define the net capture multiplier around small capture levels Ct≈0C\_t \\approx 0Ct​≈0.
Linearize near C=0C=0C=0:
Ct+1≈Ct+Γt−ΔtCtC\_{t+1} \\approx C\_t + \\Gamma\_t - \\Delta\_t C\_tCt+1​≈Ct​+Γt​−Δt​Ct​
More precisely, because growth term is Γt(1−Ct)≈Γt\\Gamma\_t(1-C\_t)\\approx \\Gamma\_tΓt​(1−Ct​)≈Γt​ near 0, capture starts increasing if Γt\\Gamma\_tΓt​ is nontrivial and persists; but long-run behavior depends on whether the system admits a stable fixed point near 0.
A clearer stability criterion uses the expected drift:
E[Ct+1−Ct∣Ct]≈E[Γt]−E[Δt]Ct\\mathbb{E}[C\_{t+1}-C\_t\\mid C\_t] \\approx \\mathbb{E}[\\Gamma\_t] - \\mathbb{E}[\\Delta\_t] C\_tE[Ct+1​−Ct​∣Ct​]≈E[Γt​]−E[Δt​]Ct​
This implies:

if E[Γt]\\mathbb{E}[\\Gamma\_t]E[Γt​] stays positive, capture rises from 0

whether it stabilizes at low level or runs away depends on balance between growth and decay as CCC increases (because Γt\\Gamma\_tΓt​ also tends to increase with CCC in real systems via resource feedback). We incorporate that next.

4) Endogenous feedback: capture fuels more rents and opacity
In real systems, capture is self-reinforcing:

captured institutions increase rent extraction

increase procurement leakage

reduce transparency

increase selectivity

Model feedback:
Rt=Rbase+rCCtR\_t = R^{base} + r\_C C\_tRt​=Rbase+rC​Ct​
Ot=Obase+oCCtO\_t = O^{base} + o\_C C\_tOt​=Obase+oC​Ct​
Selt=Selbase+sCCtSel\_t = Sel^{base} + s\_C C\_tSelt​=Selbase+sC​Ct​
These are “capture → more capture” channels.
Now growth rate becomes increasing in CtC\_tCt​.

5) Shadow Capture Threshold Theorem
Theorem (Supercritical capture vs subcritical containment).
Assume capture evolves as:
Ct+1=Ct+Γ(Ct)(1−Ct)−Δ(Ct)CtC\_{t+1} = C\_t + \\Gamma(C\_t)(1-C\_t) - \\Delta(C\_t)C\_tCt+1​=Ct​+Γ(Ct​)(1−Ct​)−Δ(Ct​)Ct​
with:
Γ(C)=αρ(A) (Rbase+rCC+ω(Wbase+wCC)) (Obase+oCC) (1−G+κ(Selbase+sCC))\\Gamma(C)=\\alpha\\rho(A)\\,(R^{base}+r\_C C + \\omega(W^{base}+w\_C C))\\,(O^{base}+o\_C C)\\,(1-G + \\kappa(Sel^{base}+s\_C C))Γ(C)=αρ(A)(Rbase+rC​C+ω(Wbase+wC​C))(Obase+oC​C)(1−G+κ(Selbase+sC​C))
and
Δ(C)=β(1−(Obase+oCC)) G (1−(Selbase+sCC))+χ Exposure(C)\\Delta(C)=\\beta(1-(O^{base}+o\_C C))\\,G\\,(1-(Sel^{base}+s\_C C)) + \\chi\\,\\text{Exposure}(C)Δ(C)=β(1−(Obase+oC​C))G(1−(Selbase+sC​C))+χExposure(C)
where Exposure(C)\\text{Exposure}(C)Exposure(C) is nondecreasing in CCC but bounded above by institutional capacity and transparency.
Define the capture reproduction number at low capture:
R0=Γ(0)Δ(0)=αρ(A) (Rbase+ωWbase) Obase (1−G+κSelbase)β(1−Obase) G (1−Selbase)+χ Exposure(0)\\mathcal{R}\_0 =
\\frac{\\Gamma(0)}{\\Delta(0)}
=
\\frac{
\\alpha\\rho(A)\\,(R^{base}+\\omega W^{base})\\,O^{base}\\,(1-G+\\kappa Sel^{base})
}{
\\beta(1-O^{base})\\,G\\,(1-Sel^{base}) + \\chi\\,\\text{Exposure}(0)
}R0​=Δ(0)Γ(0)​=β(1−Obase)G(1−Selbase)+χExposure(0)αρ(A)(Rbase+ωWbase)Obase(1−G+κSelbase)​
Then:

If R0<1\\mathcal{R}\_0 < 1R0​<1, C=0C=0C=0 is locally stable: small capture decays back toward 0 (containment regime).

If R0>1\\mathcal{R}\_0 > 1R0​>1, C=0C=0C=0 is unstable: capture grows from small perturbations (supercritical regime).

Under positive feedback parameters rC,oC,sC>0r\_C,o\_C,s\_C>0rC​,oC​,sC​>0, if R0>1\\mathcal{R}\_0 > 1R0​>1, the system admits a stable high-capture equilibrium C\\\*∈(0,1]C^\\\* \\in (0,1]C\\\*∈(0,1] (oligarchic/captured attractor), unless exposure grows fast enough to offset feedback (rare in practice).

Interpretation (plain):
There is a sharp threshold—like epidemic spread. If the “capture reproduction number” exceeds 1, capture becomes self-sustaining and tends to a high-capture state.

6) What determines R0\\mathcal{R}\_0R0​ (design levers)
The theorem tells you exactly what to attack:
Increases capture (bad)

High network amplification: ρ(A)\\rho(A)ρ(A) large (dense elite networks)

High base rent share RbaseR^{base}Rbase

High emergency/war opacity WbaseW^{base}Wbase

High opacity ObaseO^{base}Obase

Low governance integrity GGG

High selectivity SelbaseSel^{base}Selbase

Decreases capture (good)

Transparency (reduces ObaseO^{base}Obase, increases 1−Obase1-O^{base}1−Obase)

Strong independent oversight (increases decay term)

Impartial enforcement (reduce SelbaseSel^{base}Selbase)

Anti-rent structure (lower RbaseR^{base}Rbase)

Constitutional emergency-power sunsets (lower WbaseW^{base}Wbase)

Anti-concentration policies that effectively reduce ρ(A)\\rho(A)ρ(A) (break network centrality)

7) Coupling to tyranny, scarcity, and war (the explosive part)
You now connect this to other core variables:

War increases WtW\_tWt​ and opacity OtO\_tOt​, pushing R0\\mathcal{R}\_0R0​ upward.

Scarcity increases emergency measures and procurement, similarly pushing R0\\mathcal{R}\_0R0​ up.

Rising tyranny can increase selectivity SeltSel\_tSelt​ (selective enforcement), feeding capture.

So under scarcity:
S↑⇒W↑,  O↑,  Sel↑⇒R0↑S\\uparrow \\Rightarrow W\\uparrow,\\; O\\uparrow,\\; Sel\\uparrow \\Rightarrow \\mathcal{R}\_0 \\uparrowS↑⇒W↑,O↑,Sel↑⇒R0​↑
This explains “crisis authoritarianism + oligarchy” as a predictable phase shift.

8) Hybrid constitutional implications (actionable constraints)
To keep R0<1\\mathcal{R}\_0 < 1R0​<1 in the hybrid, you need constitutional minima/maxima:

minimum transparency: Obase≤Omax⁡O^{base} \\le O\_{\\max}Obase≤Omax​

maximum emergency opacity: Wbase≤Wmax⁡W^{base} \\le W\_{\\max}Wbase≤Wmax​ with sunsets

minimum governance integrity: G≥Gmin⁡G \\ge G\_{\\min}G≥Gmin​ supported by institutions

maximum selectivity: Selbase≤Selmax⁡Sel^{base} \\le Sel\_{\\max}Selbase≤Selmax​

rent suppression floor: Rbase≤Rmax⁡R^{base} \\le R\_{\\max}Rbase≤Rmax​ (via LVT, antitrust, boring finance)

network deconcentration policies to reduce ρ(A)\\rho(A)ρ(A)

These become design inequalities exactly like the earlier survivability theorems.

9) Empirical validation in CivLab (how you use the theorem)
9.1 Phase diagram
Plot regions in parameter space where R0<1\\mathcal{R}\_0 < 1R0​<1 vs >1>1>1.
9.2 Attractor atlas confirmation
Sim runs should show:

If R0<1\\mathcal{R}\_0<1R0​<1: capture events fade and don’t accumulate.

If R0>1\\mathcal{R}\_0>1R0​>1: capture accumulates to a stable high level.

9.3 Early warning indicator
Compute R^0(t)\\hat{\\mathcal{R}}\_0(t)R^0​(t) from current state and show it as a “capture outbreak risk meter.”
This becomes a killer UI element: “deep state epidemic risk.”

10) The brutal takeaway
This theorem says something uncomfortable but useful:

If your baseline rent/opacity/selective enforcement conditions push capture above threshold, the system will drift oligarchic regardless of ideology.

That’s the point of formalization.

If you want to keep pushing theorem-territory, the natural next one is:
“Sanctions leakage threshold theorem” — conditions under which black markets guarantee sanctions failure (leakage reproduction number > 1), and how enforcement/legitimacy tradeoffs bound effectiveness.

---

