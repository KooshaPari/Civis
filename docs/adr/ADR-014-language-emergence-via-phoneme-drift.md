# ADR-014: Language Emergence via Phoneme Drift

## Status
Accepted

## Context
Civis needs language to emerge from repeated social transmission, not from a predefined grammar tree. The charter treats language as a drifting, diffusing layer: dialects, creoles, and family-specific speech forms should arise from contact, imitation, compression, and turnover. If language is authored as a fixed syntax tree, the system becomes a taxonomy exercise instead of an emergent process.

## Decision
Use a Kirby-style iterated learning model with phoneme drift as the core language mechanism.

The model is:

1. Agents express language as a compact phoneme-and-feature vector, not as a preset grammar tree.
2. Each transmission step introduces small drift, compression, and category pressure.
3. Listener reconstruction is lossy under population bottlenecks, so stable categories emerge only when they are transmissible.
4. Over time, contact zones blend neighboring variants into dialect continua and creoles.

This keeps the mechanism local and generative: language is learned, copied, mutated, and re-learned, rather than assembled from authored syntax objects.

## Consequences
- Language can diversify without needing hand-authored trees for every family or register.
- Drift is explainable: retention, loss, and simplification all come from transmission pressure.
- The model supports dialect formation, prestige shift, and creolization without special-case logic.
- Because the state is compact, the emergence layer stays cheap enough for per-tick simulation.

## Alternatives Considered
- **Predefined grammar trees.** Rejected because they encode language structure up front and make the result look emergent without actually being emergent.
- **L-systems / Lindenmayer grammars.** Good for recursive form generation, but they are better at syntax-like expansion than social transmission under bottlenecks.
- **Stochastic CFGs.** More flexible than fixed trees, but they still start from authored grammar topology and bias the system toward formal sentence generation instead of population drift.

## Cross-References
- ADR-011, which requires emergence couplings to be shared gradients rather than isolated theater systems.
- The emergence charter, which requires language to arise from drift and diffusion over populations.
