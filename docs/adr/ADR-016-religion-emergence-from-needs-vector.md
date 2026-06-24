# ADR-016: Religion Emergence from Needs Vector

## Status
Accepted

## Context
Civis needs religion to emerge from lived pressure, not from authored gods, creeds, or templated cult trees. The emergence charter already frames culture, belief, and social structure as systems that should arise from local conditions. Religion is the same problem at a higher level: when needs remain unmet, populations should produce narratives, ritual, authority, and sacred obligation as a coping and coordination layer.

## Decision
Model religion as an emergent response to the population needs vector, using a Norenzayan-style Big Gods framing.

The needs vector is the input signal:

1. Track population-level unmet needs such as survival, security, belonging, uncertainty reduction, and status.
2. Convert persistent deficits into religious intensity, ritual density, and authority-seeking behavior.
3. Let groups under stronger needs pressure converge on more monitoring, punishment, and coalition-binding religious forms.
4. Allow the resulting belief system to feed back into cohesion, compliance, and anxiety reduction.

This makes religion a macro adaptation to chronic needs stress rather than a separate authored ideology type.

## Why Norenzayan
The Big Gods model is a good fit because it explains why religions scale from private coping to social coordination. As groups grow and social trust weakens, high-monitoring, punitive, norm-enforcing religion becomes more likely. That maps cleanly to Civis because the simulation already tracks need pressure, cohesion, and institutional stability.

Needs-driven religion is optimal for emergence charter compliance because it starts from lower-level state and allows multiple outcomes. A society under low stress may remain secular or ritual-light; a society under high stress may generate intense sacred authority. The rule is mechanistic, not pre-scripted.

## Consequences
- Religion can emerge in response to hardship, insecurity, and group fragmentation without authored deity catalogs.
- Different environmental pressures can yield different religious forms while still using the same mechanism.
- The model gives downstream systems a clear causal chain from needs pressure to belief, ritual, and cohesion.
- Because the driver is a vector, the system can support mixed and evolving religions instead of one fixed doctrine tree.

## Alternatives Considered
- **Agent-based contagion model.** Rejected because pure contagion explains spread but not why a population develops religion in the first place; it is transmission without a strong structural cause.
- **Axelrod cultural dissemination.** Rejected because it is useful for trait diffusion, but religion here needs a stronger explanatory link to unmet needs and coalition formation.
- **Preauthored theology system.** Rejected because it violates the emergence charter by hardcoding religious content instead of deriving it from population state.

## Cross-References
- ADR-011, for the coupling architecture that keeps emergence layers compositional and bounded.
- The emergence charter, which requires belief systems to arise from local conditions instead of authored doctrine.
