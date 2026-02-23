### **ChatGPT**

COALITION SANCTIONS STABILITY THEOREM
(CivLab – When Sanction Coalitions Hold vs Fracture Under Blowback and Disinformation v1.0)
This theorem formalizes coalition collapse. It’s the missing link between:

sanctions design,

leakage,

domestic legitimacy,

and geopolitical outcomes.

In reality, sanctions fail most often because coalitions fracture, not because interdiction is impossible.
We’ll model coalition cohesion as a dynamic system with a threshold reproduction number C0\\mathcal{C}\_0C0​ (coalition stability number), analogous to your capture and leakage thresholds.

0) Objects and variables
Coalition C\\mathcal{C}C sanctions target jjj. Members i∈Ci \\in \\mathcal{C}i∈C.
For each member iii, define:

Bi,t≥0B\_{i,t} \\ge 0Bi,t​≥0: sanction blowback cost (trade loss, inflation, shortages, political pain)

Li,t∈[0,1]L\_{i,t} \\in [0,1]Li,t​∈[0,1]: domestic legitimacy of the government

Fi,t∈[0,1]F\_{i,t} \\in [0,1]Fi,t​∈[0,1]: sanction fatigue (political exhaustion)

Di,t∈[0,1]D\_{i,t} \\in [0,1]Di,t​∈[0,1]: disinformation/propaganda pressure undermining support

Si,t∈[0,1]S\_{i,t} \\in [0,1]Si,t​∈[0,1]: scarcity pressure in coalition member (may rise due to blowback)

Gi,t∈[0,1]G\_{i,t} \\in [0,1]Gi,t​∈[0,1]: governance integrity (resists capture, improves messaging trust)

Hi,t∈[0,1]H\_{i,t} \\in [0,1]Hi,t​∈[0,1]: “commitment propensity” (culture/ideology alignment; slow-moving)

si,t≥0s\_{i,t} \\ge 0si,t​≥0: side-payments/compensation from leader (aid, energy shipments, subsidies)

pi,t∈{0,1}p\_{i,t} \\in \\{0,1\\}pi,t​∈{0,1}: participation indicator (1 = stays in coalition, 0 = exits)

Coalition-level:

KtK\_tKt​: coalition interdiction effort/budget (external enforcement)

Λt\\Lambda\_tΛt​: leakage level (if leakage is high, sanctions look ineffective → fatigue rises)

EffictEffic\_tEffict​: perceived effectiveness of sanctions (narrative + measured impact)

1) Participation decision rule (micro foundation)
Member iii stays in coalition if perceived net payoff is nonnegative:
Ui,tstay=Ai,t⏟avoided threat−Bi,t⏟blowback−Ri,t⏟retaliation risk−Φ(Fi,t,Di,t)⏟fatigue + narrative collapse+si,t⏟side-payments  ≥0U^{stay}\_{i,t} =
\\underbrace{A\_{i,t}}\_{\\text{avoided threat}}
-\\underbrace{B\_{i,t}}\_{\\text{blowback}}
-\\underbrace{R\_{i,t}}\_{\\text{retaliation risk}}
-\\underbrace{\\Phi(F\_{i,t},D\_{i,t})}\_{\\text{fatigue + narrative collapse}}
+\\underbrace{s\_{i,t}}\_{\\text{side-payments}}
\\;\\ge 0Ui,tstay​=avoided threatAi,t​​​−blowbackBi,t​​​−retaliation riskRi,t​​​−fatigue + narrative collapseΦ(Fi,t​,Di,t​)​​+side-paymentssi,t​​​≥0
This is the decision kernel.
But for stability analysis we need dynamics for Fi,tF\_{i,t}Fi,t​ and Di,tD\_{i,t}Di,t​.

2) Fatigue dynamics (the true coalition killer)
Fatigue rises with:

sustained blowback

lack of visible effectiveness

domestic scarcity

time duration

political opposition

A tractable update:
Fi,t+1=Fi,t+α1Bi,t+α2Si,t+α3(1−Effict)+α4Di,t−α5si,t−α6Li,tF\_{i,t+1} =
F\_{i,t}
+ \\alpha\_1 B\_{i,t}
+ \\alpha\_2 S\_{i,t}
+ \\alpha\_3 (1 - Effic\_t)
+ \\alpha\_4 D\_{i,t}
- \\alpha\_5 s\_{i,t}
- \\alpha\_6 L\_{i,t}Fi,t+1​=Fi,t​+α1​Bi,t​+α2​Si,t​+α3​(1−Effict​)+α4​Di,t​−α5​si,t​−α6​Li,t​
Interpretation:

if sanctions hurt you and don’t seem to work, fatigue grows

if you compensate people and maintain legitimacy, fatigue can be contained

3) Disinformation dynamics (shadow warfare on coalition)
Disinformation pressure rises when:

shadow actors invest (target + third parties)

polarization is high

trust is low

media capture is high

Update:
Di,t+1=(1−δD)Di,t+β1ShadowSpendt+β2Polarizationi,t−β3Gi,t−β4Transparencyi,tD\_{i,t+1} =
(1-\\delta\_D)D\_{i,t}
+ \\beta\_1 \\text{ShadowSpend}\_{t}
+ \\beta\_2 \\text{Polarization}\_{i,t}
- \\beta\_3 G\_{i,t}
- \\beta\_4 \\text{Transparency}\_{i,t}Di,t+1​=(1−δD​)Di,t​+β1​ShadowSpendt​+β2​Polarizationi,t​−β3​Gi,t​−β4​Transparencyi,t​
Disinformation is a force multiplier for fatigue.

4) Coalition perceived effectiveness EffictEffic\_tEffict​
Coalitions survive if members believe sanctions are working.
Perceived effectiveness depends on:

measured target scarcity/damage

leakage (black market neutralization)

propaganda/narrative

A simple form:
Effict=σ(γ1Δtarget,tE−γ2Λt−γ3D‾C,t)Effic\_t = \\sigma\\Big(
\\gamma\_1 \\Delta^E\_{target,t}
- \\gamma\_2 \\Lambda\_t
- \\gamma\_3 \\overline{D}\_{\\mathcal{C},t}
\\Big)Effict​=σ(γ1​Δtarget,tE​−γ2​Λt​−γ3​DC,t​)
So high leakage and high disinfo make sanctions feel pointless.

5) Coalition stability number C0\\mathcal{C}\_0C0​
We want a threshold: when does fatigue/disinfo spread faster than cohesion mechanisms can contain?
Define “commitment decay pressure” for member iii:
Ψi,t=α1Bi,t+α2Si,t+α3(1−Effict)+α4Di,t\\Psi\_{i,t} = \\alpha\_1 B\_{i,t} + \\alpha\_2 S\_{i,t} + \\alpha\_3 (1-Effic\_t) + \\alpha\_4 D\_{i,t}Ψi,t​=α1​Bi,t​+α2​Si,t​+α3​(1−Effict​)+α4​Di,t​
Define “commitment support”:
Ωi,t=α5si,t+α6Li,t+α7Hi,t\\Omega\_{i,t} = \\alpha\_5 s\_{i,t} + \\alpha\_6 L\_{i,t} + \\alpha\_7 H\_{i,t}Ωi,t​=α5​si,t​+α6​Li,t​+α7​Hi,t​
Then local stability indicator for each member:
κi,t=Ψi,tΩi,t\\kappa\_{i,t} = \\frac{\\Psi\_{i,t}}{\\Omega\_{i,t}}κi,t​=Ωi,t​Ψi,t​​
Coalition stability number:
C0(t)=1∣C∣∑i∈Cκi,t\\mathcal{C}\_0(t) =
\\frac{1}{|\\mathcal{C}|}\\sum\_{i\\in\\mathcal{C}} \\kappa\_{i,t}C0​(t)=∣C∣1​i∈C∑​κi,t​
Interpretation:

if C0<1\\mathcal{C}\_0 < 1C0​<1, average support dominates decay → coalition tends to hold

if C0>1\\mathcal{C}\_0 > 1C0​>1, fatigue dominates → exits accelerate

This is an actionable computed metric.

6) Theorem statement
Theorem (Coalition sanctions stability).
Assume:

Members update fatigue Fi,tF\_{i,t}Fi,t​ and disinformation Di,tD\_{i,t}Di,t​ as above.

Participation is a monotone function of fatigue and perceived payoff: higher FFF and DDD decrease probability of staying.

Blowback Bi,tB\_{i,t}Bi,t​ is bounded below when sanctions are active (sanctions have cost).

Side-payments and legitimacy are bounded above by budget and domestic politics.

Then there exists a threshold C\\\*\\mathcal{C}^\\\*C\\\* such that:

If C0(t)<1\\mathcal{C}\_0(t) < 1C0​(t)<1 uniformly over a horizon, coalition membership remains stable (no cascade exits) and the coalition can sustain interdiction effort KtK\_tKt​.

If C0(t)>1\\mathcal{C}\_0(t) > 1C0​(t)>1 for sustained periods, coalition exits occur as a cascade, reducing KtK\_tKt​, which reduces EffictEffic\_tEffict​, which further increases Ψi,t\\Psi\_{i,t}Ψi,t​, producing a positive feedback loop and coalition collapse.

In other words: coalition collapse is a supercritical cascade once fatigue/disinfo overwhelms compensations and legitimacy.

7) Cascade mechanism (why it’s nonlinear)
When one member exits:

coalition interdiction KtK\_tKt​ drops

leakage Λt\\Lambda\_tΛt​ rises

effectiveness EffictEffic\_tEffict​ drops

fatigue in remaining members rises

more members exit

So coalition stability is not smooth; it flips.

8) Design corollaries (policy implications)
Corollary A: Sanctions must be designed to minimize coalition blowback, not just maximize target pain
This links directly to your robust interdiction planner.
Corollary B: Side-payments are not optional
Coalitions are held together by compensation and narrative.
Corollary C: High governance integrity is “coalition glue”
It reduces disinformation success and increases legitimacy durability.
Corollary D: Leakage control is coalition survival control
Even if sanctions hurt the target, if leakage makes it look ineffective, coalitions crack.

9) CivLab instrumentation (UI + research outputs)
Add a “Coalition Stability Meter”:

C0\\mathcal{C}\_0C0​ displayed with:

blowback contribution

disinformation contribution

leakage/ineffectiveness contribution

side-payment contribution

Warn when:

C0>1\\mathcal{C}\_0 > 1C0​>1

exit cascade risk high in next N quarters

This becomes a core 4X gameplay element.

10) Links to your other theorems

Leakage theorem: L0>1\\mathcal{L}\_0 > 1L0​>1 → leakage rises → EffictEffic\_tEffict​ falls → coalition collapses

Capture theorem: capture rises → disinfo rises → coalition collapses

Repression backfire theorem: target crackdown destabilizes → migration and war escalation follow

Hegemonic cycles: coalition collapse often triggers system war or order reset

Everything is now formally connected.

Next theorem (most natural continuation)
“Order Stability Theorem” — conditions under which a hegemon-maintained global order is stable vs collapses into blocs and system war (a threshold in parity + scarcity + coalition integrity).

---

