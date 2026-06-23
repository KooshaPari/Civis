# ADR-015: Faction Emergence via k-means Ideology Clustering

## Status
Accepted

## Context
Civis needs factions to emerge from population ideology, not from explicit faction labels or hand-authored political trees. The emergence charter treats polities as clusterable social structures, with membership inferred from shared state rather than fixed enums. A faction model must also be stable enough for diplomacy, cohesion, and unrest to consume deterministically.

## Decision
Use k-means clustering over ideology vectors as the faction emergence layer.

The implementation is:

1. Represent each agent or cohort as an ideology vector derived from beliefs, grievances, loyalties, and contact history.
2. Run deterministic k-means with fixed `k`, fixed initialization, and stable tie-breaking.
3. Derive faction centroids from the resulting clusters and project them into diplomacy, cohesion, and unrest consumers.
4. Re-run the clustering on a cadence or when the ideology field shifts enough to justify a reassignment.

## Why k-means
k-means gives the right tradeoff for emergence: it is simple, fast, and has practical convergence guarantees. Under a fixed dataset and deterministic initialization, Lloyd-style updates monotonically reduce within-cluster distortion until assignments stabilize. That gives us a bounded and auditable emergence step instead of an open-ended heuristic.

For Civis, that matters more than perfect partition quality. Factions need to settle into a usable macro structure each tick or cadence window so downstream systems can reason about them.

## Consequences
- Faction membership is derived from actual ideological proximity rather than enum membership.
- Convergence is predictable and testable, which keeps the emergence layer suitable for simulation ticks.
- Cluster centroids give diplomacy a stable macro representative for each emergent bloc.
- The approach scales better than graph-heavy clustering once populations get large.

## Alternatives Considered
- **Hierarchical clustering.** Rejected because it is harder to recompute incrementally and produces nested trees that are more complex than the faction use case needs.
- **DBSCAN.** Rejected because density-based clustering is fragile when population ideology is continuous or unevenly sampled; it also does not naturally guarantee a fixed number of factions for downstream consumers.
- **SOM (self-organizing map).** Rejected because it is better as a visualization and dimensionality-reduction tool than as a direct faction assignment mechanism.

## Cross-References
- ADR-011, for the coupling contract that keeps emergence signals shared and bounded.
- The emergence charter, which requires polities to arise from co-location, kinship, culture, and payoff gradients rather than authored faction lists.
