### **ChatGPT**

SANCTIONS LEAKAGE THRESHOLD THEOREM
(CivLab – When Black Markets Make Sanctions Inevitably Fail v1.0)
This is the sibling of the capture theorem. It treats sanctions leakage like an epidemic: if “leakage reproduction” exceeds 1, black/gray trade grows faster than interdiction can suppress it, so sanctions become structurally ineffective (or require tyranny-level enforcement to sustain).
We’ll formalize:

leakage growth dynamics

enforcement and coalition effects

shadow-network facilitation

legitimacy/tyranny constraints

a threshold condition L0\\mathcal{L}\_0L0​ analogous to R0\\mathcal{R}\_0R0​

0) Reduced objects
Consider:

Target polity jjj

Sanctioning coalition C\\mathcal{C}C

Trade/energy corridor network G\\mathcal{G}G

Let:

xe&isin;{0,1}x\_e\\in\\{0,1\\}xe​&isin;{0,1}: interdiction on edge eee by coalition (formal sanctions)

KKK: coalition interdiction budget (enforcement effort, naval patrols, compliance)

StS\_tSt​: scarcity pressure inside target jjj

ΔPt\\Delta P\_tΔPt​: price wedge between shadow and official markets in target (arbitrage incentive)

EtE\_tEt​: target internal enforcement intensity (policing/customs)

SeltSel\_tSelt​: selectivity (corruption/elite bypass)

GtG\_tGt​: governance integrity in target

HtH\_tHt​: shadow network facilitation capacity (smuggling networks)

BtB\_tBt​: baseline rights floor (affects legitimacy tolerance)

LtL\_tLt​: legitimacy

Define target’s shadow import capacity (leakage throughput):

Λt&gt;0\\Lambda\_t \\ge 0Λt​&gt;0: total effective black/gray inflow (energy + critical inputs)

Sanctions “fail” operationally if:
Λt&asymp;Λreq\\Lambda\_t \\approx \\Lambda\_{req}Λt​&asymp;Λreq​
i.e., leakage restores enough throughput that scarcity remains below intended level.

1) Leakage dynamics (macro logistic with suppression)
Model leakage as:
Λt+1=Λt+gt(Λt)⏟growth−st(Λt)⏟suppression\\Lambda\_{t+1}=\\Lambda\_t + \\underbrace{g\_t(\\Lambda\_t)}\_{\\text{growth}} - \\underbrace{s\_t(\\Lambda\_t)}\_{\\text{suppression}}Λt+1​=Λt​+growthgt​(Λt​)​​−suppressionst​(Λt​)​​
1.1 Growth mechanism
Leakage grows with:

scarcity StS\_tSt​ (demand desperation)

price wedge ΔPt\\Delta P\_tΔPt​ (profit motive)

network capacity HtH\_tHt​ (smuggler infrastructure)

enforcement selectivity/corruption (easier passage)

A tractable form:
gt(Λ)=α Ht (St+ηΔPt) (1+κSelt) (1−ΛΛmax)g\_t(\\Lambda)= \\alpha \\, H\_t \\,(S\_t + \\eta \\Delta P\_t)\\,(1+\\kappa Sel\_t)\\,(1-\\frac{\\Lambda}{\\Lambda\_{max}})gt​(Λ)=αHt​(St​+ηΔPt​)(1+κSelt​)(1−Λmax​Λ​)

logistic term ensures saturation at Λmax\\Lambda\_{max}Λmax​ (geographic/route limits)

1.2 Suppression mechanism
Suppression increases with:

coalition interdiction intensity (external)

target enforcement (internal)

governance integrity (reduces bribery, improves targeting)
And decreases with:

selectivity/corruption

shadow sophistication

A tractable form:
st(Λ)=β (Kt+ψEt) Gt (1−Selt) Λs\_t(\\Lambda)=\\beta \\,(K\_t + \\psi E\_t)\\,G\_t\\,(1-Sel\_t)\\,\\Lambdast​(Λ)=β(Kt​+ψEt​)Gt​(1−Selt​)Λ
This makes suppression proportional to current leakage volume.

2) The leakage reproduction number L0\\mathcal{L}\_0L0​
Linearize at small leakage Λ&asymp;0\\Lambda\\approx 0Λ&asymp;0.
Then growth is approximately:
gt(Λ)&asymp;αHt(St+ηΔPt)(1+κSelt)g\_t(\\Lambda)\\approx \\alpha H\_t (S\_t + \\eta\\Delta P\_t)(1+\\kappa Sel\_t)gt​(Λ)&asymp;αHt​(St​+ηΔPt​)(1+κSelt​)
and suppression is approximately:
st(Λ)&asymp;β(Kt+ψEt)Gt(1−Selt)Λs\_t(\\Lambda)\\approx \\beta (K\_t+\\psi E\_t)G\_t(1-Sel\_t)\\Lambdast​(Λ)&asymp;β(Kt​+ψEt​)Gt​(1−Selt​)Λ
To define a threshold like an epidemic, we focus on whether leakage can grow from small perturbations. A standard way is to compare marginal growth vs marginal suppression at low Λ\\LambdaΛ.
Define:
L0(t)=αHt(St+ηΔPt)(1+κSelt)β(Kt+ψEt)Gt(1−Selt)\\mathcal{L}\_0(t)=
\\frac{
\\alpha H\_t (S\_t + \\eta \\Delta P\_t)(1+\\kappa Sel\_t)
}{
\\beta (K\_t+\\psi E\_t)G\_t(1-Sel\_t)
}L0​(t)=β(Kt​+ψEt​)Gt​(1−Selt​)αHt​(St​+ηΔPt​)(1+κSelt​)​
Interpretation:

numerator = incentives + network facilitation + corruption bypass

denominator = external interdiction + internal enforcement × integrity

3) Sanctions Leakage Threshold Theorem
Theorem (Leakage threshold).
Assume leakage dynamics as above, with bounded parameters and Λmax>0\\Lambda\_{max}>0Λmax​>0. Let L0(t)\\mathcal{L}\_0(t)L0​(t) be defined as:
L0(t)=αHt(St+ηΔPt)(1+κSelt)β(Kt+ψEt)Gt(1−Selt)\\mathcal{L}\_0(t)=
\\frac{
\\alpha H\_t (S\_t + \\eta \\Delta P\_t)(1+\\kappa Sel\_t)
}{
\\beta (K\_t+\\psi E\_t)G\_t(1-Sel\_t)
}L0​(t)=β(Kt​+ψEt​)Gt​(1−Selt​)αHt​(St​+ηΔPt​)(1+κSelt​)​
Then:

If L0(t)<1\\mathcal{L}\_0(t) < 1L0​(t)<1 uniformly for t&gt;t0t\\ge t\_0t&gt;t0​, leakage decays to a low steady level and sanctions remain effective (up to residual leakage).

If L0(t)>1\\mathcal{L}\_0(t) > 1L0​(t)>1 for sustained periods, leakage grows toward a high-leakage equilibrium Λ\\\*\\Lambda^\\\*Λ\\\* close to Λmax\\Lambda\_{max}Λmax​, and sanctions effectiveness collapses (unless coalition escalates enforcement).

Under endogenous feedback where StS\_tSt​ and ΔPt\\Delta P\_tΔPt​ increase when sanctions tighten (they will), L0\\mathcal{L}\_0L0​ tends to rise over time, producing runaway leakage unless enforcement increases superlinearly.

Plain meaning:
If scarcity and profit incentives outpace combined interdiction and honest enforcement, black markets will grow until they neutralize sanctions.

4) Coalition enforcement constraint: the tyranny/legitimacy bound
Coalition can increase external interdiction KtK\_tKt​, target can increase internal enforcement EtE\_tEt​. But both are politically limited.
4.1 Target enforcement is bounded by legitimacy
Higher EtE\_tEt​ increases tyranny and reduces legitimacy:
Tt+1↑ with Et,Lt+1↓ with EtT\_{t+1} \\uparrow \\text{ with } E\_t
\\quad,\\quad
L\_{t+1} \\downarrow \\text{ with } E\_tTt+1​↑ with Et​,Lt+1​↓ with Et​
If legitimacy falls too low, enforcement collapses due to revolt, fragmentation, or capture.
So EtE\_tEt​ has an effective upper bound:
Et&lt;E‾(Lt,Bt)E\_t \\le \\overline{E}(L\_t,B\_t)Et​&lt;E(Lt​,Bt​)
where higher baseline BtB\_tBt​ increases tolerance for enforcement (people endure hardship better), but coupling is forbidden in the hybrid.
4.2 Coalition interdiction is bounded by blowback and fatigue
Coalition members also suffer cost from sanctions (trade loss, price shocks, political fatigue). So:
Kt&lt;K‾(coalition blowback,domestic politics)K\_t \\le \\overline{K}(\\text{coalition blowback},\\text{domestic politics})Kt​&lt;K(coalition blowback,domestic politics)
This means even if L0>1\\mathcal{L}\_0>1L0​>1, the coalition may be unable to push it below 1 sustainably.

5) Corruption/selectivity creates “elite bypass”
If SeltSel\_tSelt​ increases, leakage rises even if enforcement increases, because enforcement becomes selective and the shadow economy consolidates into elite-controlled channels.
This yields a dark result:
Even very high EtE\_tEt​ can fail if GtG\_tGt​ is low and SeltSel\_tSelt​ is high:

enforcement targets small actors

elites and shadow networks route around

Formally, denominator contains Gt(1−Selt)G\_t(1-Sel\_t)Gt​(1−Selt​). If either goes to 0, suppression collapses.

6) Shadow-state facilitation feedback (leakage fuels itself)
Leakage increases shadow resources and network capacity:
Ht+1=Ht+νΛt−δHHtH\_{t+1} = H\_t + \\nu \\Lambda\_t - \\delta\_H H\_tHt+1​=Ht​+νΛt​−δH​Ht​
So if leakage grows, HHH grows, which further increases L0\\mathcal{L}\_0L0​. This is the “smuggling empire” attractor.

7) Practical policy insights (what the theorem implies)
To keep sanctions effective, you must reduce L0\\mathcal{L}\_0L0​ below 1 by changing:
Reduce numerator

Reduce HtH\_tHt​: disrupt networks (requires intelligence, not just patrols)

Reduce StS\_tSt​: allow humanitarian channels / reduce desperation

Reduce ΔPt\\Delta P\_tΔPt​: avoid extreme wedges (price controls can backfire; targeted supply helps)

Reduce SeltSel\_tSelt​: corruption control (hard but necessary)

Increase denominator

Increase coalition interdiction KtK\_tKt​ (but limited by fatigue)

Increase internal enforcement EtE\_tEt​ paired with integrity GtG\_tGt​

Increase governance integrity GtG\_tGt​ (anti-corruption, transparency)

This shows why “sanctions without governance integrity” fail.

8) CivLab integration: sanction effectiveness as a phase diagram
You can compute L^0(t)\\hat{\\mathcal{L}}\_0(t)L^0​(t) in-sim and display:

leakage outbreak risk meter

“sanctions will fail” warning if L0>1\\mathcal{L}\_0>1L0​>1

Plot phase map:

x-axis: coalition interdiction budget KKK

y-axis: target governance integrity GGG
Contours: L0=1\\mathcal{L}\_0=1L0​=1

This is a research-grade output and a great game UI element.

9) Link to war probability (bargaining failure)
If sanctions fail, coalition may escalate to war.
So sanctions failure increases crisis escalation probability:
Pr⁡(escalate to war)↑ with Λ\\\* and St\\Pr(\\text{escalate to war}) \\uparrow \\text{ with } \\Lambda^\\\* \\text{ and } S\_tPr(escalate to war)↑ with Λ\\\* and St​
and also with shadow provocation.
So this theorem plugs directly into your corridor war dynamics.

10) Summary in one line
Sanctions work only when:

scarcity isn’t desperate

black market networks are weak

governance integrity is strong

enforcement is impartial

coalition commitment is durable

Otherwise leakage becomes self-sustaining and sanctions collapse.

If you want the next theorem in this chain, the natural continuation is:
“Authoritarian enforcement backfire theorem” — conditions under which raising enforcement to suppress leakage increases instability and leads to fragmentation, making leakage worse long-run (a formalized “repression trap”).

---

