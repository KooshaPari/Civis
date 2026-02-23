### **ChatGPT**

We go with (2) Financial system & credit crises dynamics, because without it you’re missing the main “capitalism goes off the rails” engine and one of the biggest real-world drivers of instability, inequality, shadow capture, and war. Also, it plugs cleanly into your scarcity model: energy shocks + credit fragility is where empires actually break.
FINANCIAL SYSTEM & CREDIT CRISES DYNAMICS
(CivLab – Money, Credit, Leverage, and Crisis Layer v1.0)
0) Design intent
Model finance as a powerful amplifier:

boosts growth when stable

concentrates wealth via rents

creates bubbles and sudden collapses

interacts with sanctions, war, and scarcity

provides another axis for shadow capture

This is not “finance as a minigame.”
It’s a core engine of instability and oligarchy attractors.

1) Core objects and state variables (per region rrr)
1.1 Balance sheet aggregates

KtK\_tKt​: real productive capital stock

YtY\_tYt​: output

DtD\_tDt​: private debt stock (households + firms)

BtB\_tBt​: bank assets/liabilities (abstracted)

LVRtLVR\_tLVRt​: leverage ratio (systemwide)

HPItHPI\_tHPIt​: housing/land price index (asset price proxy)

APItAPI\_tAPIt​: broader asset price index

rtr\_trt​: interest rate / cost of credit

CStCS\_tCSt​: credit spread (risk premium)

NPLtNPL\_tNPLt​: non-performing loan ratio

LLPtLLP\_tLLPt​: loan loss provisions

CtC\_tCt​: credit creation rate (new lending)

RtrentR^{rent}\_tRtrent​: rent extraction share through finance

1.2 Distribution hooks (optional MVP+)

household debt burden distribution

firm debt service ratio distribution

2) Money/credit creation mechanism (macro)
We model credit as endogenously created by banking/finance under constraints.
Credit creation:
Ct=χ⋅RiskAppetitet⋅BankHealtht⋅CollateralValuetC\_{t} = \\chi \\cdot \\text{RiskAppetite}\_t \\cdot \\text{BankHealth}\_t \\cdot \\text{CollateralValue}\_tCt​=χ⋅RiskAppetitet​⋅BankHealtht​⋅CollateralValuet​
Where:

RiskAppetite rises in booms, falls in crises

BankHealth falls as NPL rises

CollateralValue depends on asset prices (HPI/API)

Debt evolves:
Dt+1=Dt+Ct−Repayt−DefaulttD\_{t+1} = D\_t + C\_t - \\text{Repay}\_t - \\text{Default}\_tDt+1​=Dt​+Ct​−Repayt​−Defaultt​

3) Asset price dynamics (bubble engine)
Asset prices respond to credit availability and expectations.
A simple positive feedback:
HPIt+1=HPIt⋅exp⁡(α⋅gC−β⋅rt+ϵt)HPI\_{t+1}=HPI\_t \\cdot \\exp(\\alpha \\cdot g\_C - \\beta \\cdot r\_t + \\epsilon\_t)HPIt+1​=HPIt​⋅exp(α⋅gC​−β⋅rt​+ϵt​)
where gCg\_CgC​ is credit growth rate.
Collateral value:
CollateralValuet∝HPIt\\text{CollateralValue}\_t \\propto HPI\_tCollateralValuet​∝HPIt​
This creates the classic loop:
credit ↑ → prices ↑ → collateral ↑ → credit ↑

4) Debt service and default
Debt service burden (aggregate proxy):
DSRt=rtDtYtDSR\_t = \\frac{r\_t D\_t}{Y\_t}DSRt​=Yt​rt​Dt​​
Default rate rises when households/firms are stressed:
DefaultRatet=σ(a⋅DSRt+b⋅St+c⋅Ut−d⋅IncomeGrowtht)\\text{DefaultRate}\_t = \\sigma(a\\cdot DSR\_t + b\\cdot S\_t + c\\cdot U\_t - d\\cdot \\text{IncomeGrowth}\_t)DefaultRatet​=σ(a⋅DSRt​+b⋅St​+c⋅Ut​−d⋅IncomeGrowtht​)
Defaults:
Defaultt=DefaultRatet⋅Dt\\text{Default}\_t = \\text{DefaultRate}\_t \\cdot D\_tDefaultt​=DefaultRatet​⋅Dt​
NPL evolves:
NPLt+1=(1−δ)NPLt+DefaultRatet−WriteOfftNPL\_{t+1} = (1-\\delta)NPL\_t + \\text{DefaultRate}\_t - \\text{WriteOff}\_tNPLt+1​=(1−δ)NPLt​+DefaultRatet​−WriteOfft​

5) Banking crisis threshold (nonlinear collapse)
Define a bank solvency/stability measure:
BankHealtht=1−θ1NPLt−θ2LVRt−θ3CSt\\text{BankHealth}\_t = 1 - \\theta\_1 NPL\_t - \\theta\_2 LVR\_t - \\theta\_3 CS\_tBankHealtht​=1−θ1​NPLt​−θ2​LVRt​−θ3​CSt​
When BankHealth drops below a threshold:

credit creation collapses

spreads spike

recession shock hits output

unemployment rises (optional)

political extremism rises

Crisis switch:
if BankHealtht<hcrit⇒CreditCrunch\\text{if } \\text{BankHealth}\_t < h\_{crit} \\Rightarrow \\text{CreditCrunch}if BankHealtht​<hcrit​⇒CreditCrunch
Credit crunch behavior:
Ct↓↓,rt↑,CSt↑,Yt↓C\_t \\downarrow\\downarrow,\\quad r\_t \\uparrow,\\quad CS\_t \\uparrow,\\quad Y\_t \\downarrowCt​↓↓,rt​↑,CSt​↑,Yt​↓

6) Government/central bank response (policy levers)
This is where regimes differ.
Levers:

rate policy rtr\_trt​ (or “tight/loose” regime)

bailout policy (recapitalize banks)

debt jubilee / restructuring

capital controls

macroprudential: leverage caps, LTV caps

land value tax / property tax (deflates housing bubble)

“boring finance” constraints

Different regime defaults:

capitalist: tends to bail out banks, protect asset prices

planned: credit is administratively allocated; crises manifest as shortages and misallocation

hybrid: finance is utility-like; bailouts conditional; land value tax strong; leverage capped

7) Finance as rent extraction (inequality engine)
Finance generates “rent” not tied to real productivity:
Rtrent=ρ1⋅InterestMargin+ρ2⋅Fees+ρ3⋅AssetAppreciationCaptureR^{rent}\_t = \\rho\_1 \\cdot \\text{InterestMargin} + \\rho\_2 \\cdot \\text{Fees} + \\rho\_3 \\cdot \\text{AssetAppreciationCapture}Rtrent​=ρ1​⋅InterestMargin+ρ2​⋅Fees+ρ3​⋅AssetAppreciationCapture
This flows primarily to:

owners of capital

politically connected institutions

Which raises inequality:
It+1=It+λ⋅Rtrent−μ⋅RedistributionI\_{t+1} = I\_t + \\lambda \\cdot R^{rent}\_t - \\mu \\cdot \\text{Redistribution}It+1​=It​+λ⋅Rtrent​−μ⋅Redistribution
And lowers mobility MMM.

8) Interaction with scarcity and energy shocks (the killer combo)
Energy scarcity shock increases:

production costs → lowers output YYY

raises inflation pressure (if modeled)

raises defaults

So:
St↑⇒DSRt↑⇒DefaultRate↑⇒BankHealth↓⇒CreditCrunchS\_t \\uparrow \\Rightarrow DSR\_t \\uparrow \\Rightarrow DefaultRate \\uparrow \\Rightarrow BankHealth \\downarrow \\Rightarrow CreditCrunchSt​↑⇒DSRt​↑⇒DefaultRate↑⇒BankHealth↓⇒CreditCrunch
This is the real-world amplifier.

9) Interaction with war and sanctions
Sanctions reduce trade flows → output falls → defaults rise.
War increases defense spending → can temporarily boost output, but:

destroys capital

increases risk premium

increases shadow capture via procurement

can trigger capital flight

Capital flight (optional):
Outflowt=f(Risk,Tt,Gt)\\text{Outflow}\_t = f(\\text{Risk}, T\_t, G\_t)Outflowt​=f(Risk,Tt​,Gt​)
Outflow reduces domestic investment and destabilizes currency/credit.

10) Shadow state integration
Shadow actors profit from:

procurement corruption (war spending)

bank bailouts (capture)

black market finance

illicit capital channels during sanctions

They can increase:

leakage

corruption

bailout bias

This creates the “oligarchic basin.”

11) LOD integration
Macro:

run the credit cycle on aggregates

produce crisis events

City layer:

housing prices and rents rise

homelessness risk rises

construction booms/busts visible

Micro (instanced districts):

household debt distress

foreclosures

protests

political radicalization

This is how finance becomes experiential.

12) Key new metrics

Credit/GDP ratio D/YD/YD/Y

Asset price index HPI/API

Default rate, NPL

BankHealth

Crisis probability

Rent extraction share from finance

Inequality shift attributable to finance

“Financial fragility index”:

FFI=ω1(D/Y)+ω2LVR+ω3NPL+ω4CSFFI = \\omega\_1(D/Y)+\\omega\_2 LVR+\\omega\_3 NPL+\\omega\_4 CSFFI=ω1​(D/Y)+ω2​LVR+ω3​NPL+ω4​CS

13) Attractor states created by finance
This layer creates new macro attractors:

stable “boring finance” equilibrium (low leverage, low rent)

boom-bust cycle equilibrium (oscillatory)

oligarchic asset inflation state (persistent inequality)

crisis spiral / lost decade state

capital flight collapse state

Now capitalism has real dynamics, not just parameters.

Next logical spec (my pick): Hegemonic cycles model (rise/fall), because finance + war + scarcity is how global orders shift.

---

