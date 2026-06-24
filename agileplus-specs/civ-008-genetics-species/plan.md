# Plan: Genetics and Species Diversity (civ-008)

## Phased WBS (v2 target — all tasks planned)

### Phase 1: Species registry and YAML schema (FR-CIV-BIO-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| G1.1 | `SpeciesType` enum + `BaseTraits` struct (all Fixed) | civ-003 P2.2 | Planned |
| G1.2 | Species YAML schema extension; registry loaded at scenario init | G1.1 | Planned |

### Phase 2: Genetic inheritance (FR-CIV-BIO-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| G2.1 | Birth event triggers trait inheritance sampling from parental distribution | G1.2 | Planned |
| G2.2 | Mutation rate applied via `ChaCha20Rng`; draw logged to event stream | G2.1 | Planned |
| G2.3 | Trait bounds enforcement: clamp to `[0, 2]` | G2.2 | Planned |

### Phase 3: Trait-simulation couplings (FR-CIV-BIO-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| G3.1 | `strength` → military unit effectiveness multiplier | G2.3 | Planned |
| G3.2 | `intelligence` → research speed multiplier | G3.1 | Planned |
| G3.3 | `longevity` → retirement age threshold | G3.2 | Planned |
| G3.4 | `disease_resistance` → health decrement reduction during plague | G3.3 | Planned |

## DAG Dependencies

```
(civ-003 P2.2) → G1.1 → G1.2 → G2.1 → G2.2 → G2.3 → G3.1 → G3.2 → G3.3 → G3.4
(civ-006 W1.2) → G3.1 [military coupling]
(civ-005 C3.3) → G3.4 [plague coupling]
```
