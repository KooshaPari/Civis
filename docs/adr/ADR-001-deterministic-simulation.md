# ADR-001: Deterministic Simulation Architecture

**Date**: 2026-04-04  
**Status**: Accepted  
**Deciders**: Civis Architecture Team  
**Related**: SOTA-CIVILIZATION-SIMULATION.md, SPEC.md  

## Context

Civis is a civilization simulation platform that prioritizes reproducibility for policy analysis and scientific research. Unlike most agent-based modeling (ABM) platforms which prioritize ease of use over determinism, Civis must guarantee that the same initial conditions always produce identical results across runs, platforms, and versions.

### Why Determinism Matters

| Use Case | Determinism Requirement |
|----------|------------------------|
| **Policy Analysis** | Prove that outcome X follows from policy Y |
| **Scientific Reproducibility** | Other researchers must replicate results |
| **Debugging** | Replay exact scenario that produced bug |
| **Regression Testing** | Verify that changes don't alter valid behavior |
| **Verification** | Compare two implementations for equivalence |

### Industry Gap Analysis

After reviewing 20+ ABM platforms (NetLogo, MASON, Repast, AnyLogic, Mesa, Agents.jl), we found:

- **None guarantee cross-platform determinism**
- **None provide formal replay systems**
- **All use floating-point in non-deterministic ways**
- **All rely on platform-dependent RNG or collections**

This represents a significant gap that Civis aims to fill.

## Decision

**Adopt a determinism-first architecture with the following mandatory properties:**

### Core Requirements

1. **Seeded RNG**: All randomness derived from a single simulation seed using ChaCha8Rng
2. **Fixed-Point Math**: Cross-platform deterministic arithmetic for critical calculations
3. **Ordered Collections**: Deterministic iteration order via IndexMap and BTree collections
4. **Replay Capability**: Full event log + periodic state checksums for verification
5. **No Undefined Behavior**: Leverage Rust's safety guarantees
6. **Explicit Ordering**: No implicit iteration dependencies

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Civis Determinism Stack                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │  Layer 5: Replay System                                               │ │
│  │  • Event log serialization                                            │ │
│  │  • Periodic checksums (every N ticks)                               │ │
│  │  • Verification mode (replay + compare)                               │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                   │                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │  Layer 4: Simulation Logic                                            │ │
│  │  • No system time dependence                                          │ │
│  │  • No thread scheduling dependence                                  │ │
│  │  • No floating-point equality comparisons                            │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                   │                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │  Layer 3: Data Structures                                             │ │
│  │  • IndexMap for insertion-order preservation                        │ │
│  │  • BTreeMap for sorted iteration                                    │ │
│  │  • Stable sort algorithms                                           │ │
│  │  • Deterministic hash seeds                                         │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                   │                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │  Layer 2: Arithmetic                                                  │ │
│  │  • Fixed-point for economic calculations                            │ │
│  │  • IEEE-754 strict mode for physics                                 │ │
│  │  • No FMA unless deterministic across targets                         │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                   │                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │  Layer 1: Randomness                                                  │ │
│  │  • ChaCha8Rng (seeded, parallel streams)                              │ │
│  │  • Jump-ahead for parallel system execution                         │ │
│  │  • Serialize RNG state for checkpoints                              │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Design

### Layer 1: Random Number Generation

**Selected Algorithm**: ChaCha8Rng

| Algorithm | Speed | Period | Parallel Streams | Quality | Determinism |
|-----------|-------|--------|------------------|-----------|-------------|
| MT19937 | Fast | 2^19937 | No | Good | Platform-dependent |
| PCG64 | Very Fast | 2^128 | Yes | Good | Platform-dependent |
| ChaCha8Rng | Fast | 2^256 | Yes | Excellent | Cross-platform |

**Why ChaCha8Rng**:
- Cryptographic quality ensures statistical randomness
- Cross-platform identical output (portable implementation)
- Jump-ahead enables parallel system execution without coordination
- Seeded state is 136 bytes (compact for serialization)

**Implementation**:
```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct Simulation {
    seed: u64,
    rng: ChaCha8Rng,
    // RNG state serializable for checkpoints
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
    
    pub fn checkpoint_rng(&self) -> Vec<u8> {
        // Serialize RNG state
        bincode::serialize(&self.rng).unwrap()
    }
}
```

### Layer 2: Arithmetic

**Challenge**: IEEE-754 floating-point has implementation-defined behavior across platforms.

**Solution**: Hybrid approach

| Calculation Type | Implementation | Justification |
|-----------------|----------------|---------------|
| Economic values (prices, wealth) | Fixed-point (I64F64) | Deterministic, exact |
| Physical simulation | IEEE-754 with strict mode | Performance, acceptable variance |
| Probabilities | Fixed-point [0, 1] | Deterministic thresholds |
| Geometric calculations | Fixed-point | Cross-platform consistency |

**Fixed-Point Implementation**:
```rust
use fixed::types::I64F64;

/// Economic values use 64.64 fixed-point
/// Range: ±9.2e18 with precision ~5.4e-20
pub type Currency = I64F64;

impl Currency {
    pub fn from_f64(v: f64) -> Self {
        I64F64::from_num(v)
    }
    
    pub fn to_f64(self) -> f64 {
        self.to_num()
    }
}

// Deterministic operations
let price_a = Currency::from_f64(100.50);
let price_b = Currency::from_f64(200.75);
let total = price_a + price_b; // Always exact
```

### Layer 3: Data Structures

**Challenge**: HashMap iteration order varies across platforms and runs.

**Solution**: Ordered collections for deterministic iteration

| Use Case | Collection | Ordering | Performance |
|----------|------------|----------|-------------|
| Agent storage | IndexMap | Insertion order | O(1) access |
| Sorted lookup | BTreeMap | Key order | O(log n) |
| Random access | Vec | Index order | O(1) |
| Event queue | BinaryHeap | Priority (tick) | O(log n) |

**Implementation**:
```rust
use indexmap::IndexMap;

pub struct World {
    // Deterministic iteration order
    citizens: IndexMap<CitizenId, Citizen>,
    institutions: IndexMap<InstitutionId, Institution>,
}

impl World {
    pub fn tick(&mut self, rng: &mut ChaCha8Rng) {
        // Always processes in insertion order
        for (id, citizen) in &mut self.citizens {
            citizen.update(rng);
        }
    }
}
```

### Layer 4: Simulation Logic

**Rules for Deterministic Code**:

1. **No `Instant::now()` in simulation logic** — Use simulation tick
2. **No thread scheduling dependence** — Explicit parallelization with ordered joins
3. **No `HashMap` without sorting** — Use ordered collections
4. **No floating-point equality** — Use epsilon or fixed-point
5. **No external I/O during tick** — Buffer and batch

**Example**:
```rust
// ❌ Non-deterministic
let random_citizen = citizens.values().choose(&mut rng);

// ✅ Deterministic (sorted selection)
let idx = rng.gen_range(0..citizens.len());
let random_citizen = citizens.get_index_mut(idx);
```

### Layer 5: Replay System

**Architecture**:

```rust
pub struct Replay {
    pub seed: u64,
    pub initial_state: WorldState,
    pub events: Vec<LoggedEvent>,
    pub checksums: Vec<(Tick, u64)>, // Periodic verification
}

pub struct LoggedEvent {
    pub tick: Tick,
    pub event_type: EventType,
    pub data: Vec<u8>, // Serialized event data
}

impl Replay {
    /// Record an event during simulation
    pub fn record(&mut self, tick: Tick, event: &dyn Event) {
        self.events.push(LoggedEvent {
            tick,
            event_type: event.type_id(),
            data: event.serialize(),
        });
    }
    
    /// Compute checksum of current state
    pub fn checksum(&self, world: &WorldState) -> u64 {
        let serialized = bincode::serialize(world).unwrap();
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        hasher.update(&serialized);
        hasher.digest()
    }
    
    /// Verify replay matches original
    pub fn verify(&self, simulation: &mut Simulation) -> Result<(), ReplayError> {
        // Reset to initial state
        simulation.load_state(&self.initial_state);
        
        // Replay events
        for event in &self.events {
            while simulation.tick() < event.tick {
                simulation.step();
            }
            simulation.apply_event(event)?;
        }
        
        // Compare checksums
        for (tick, expected) in &self.checksums {
            simulation.run_to(*tick);
            let actual = self.checksum(&simulation.world_state());
            if actual != *expected {
                return Err(ReplayError::ChecksumMismatch {
                    tick: *tick,
                    expected: *expected,
                    actual,
                });
            }
        }
        
        Ok(())
    }
}
```

## Consequences

### Positive

1. **Reproducibility**: Same seed always produces identical results
2. **Debugging**: Replay any scenario exactly
3. **Testing**: Property-based testing with known outcomes
4. **Verification**: Compare implementations byte-for-byte
5. **Scientific Rigor**: Results can be independently verified

### Negative

1. **Performance Cost**: Fixed-point math ~20% slower than floating-point
2. **Collection Overhead**: Ordered collections slightly slower than HashMap
3. **Complexity**: Developers must understand determinism requirements
4. **Ecosystem Friction**: Some libraries incompatible with deterministic requirements
5. **Platform Testing**: Must test determinism across platforms in CI

### Mitigations

| Concern | Mitigation |
|---------|------------|
| Performance | Use floating-point for non-critical paths; profile first |
| Complexity | Lint rules and code review checklist |
| Ecosystem | Wrap non-deterministic libraries in deterministic interfaces |
| Testing | CI determinism tests on Linux, macOS, Windows |

## Implementation Plan

### Phase 1: Foundation (Completed)

- [x] ChaCha8Rng integration with seed support
- [x] Fixed-point types for economic values
- [x] IndexMap for agent collections
- [x] Basic replay serialization

### Phase 2: Verification (In Progress)

- [ ] Cross-platform determinism tests
- [ ] Replay verification CI pipeline
- [ ] Determinism lint rules
- [ ] Fuzz testing for determinism edge cases

### Phase 3: Optimization

- [ ] Parallel system execution with jump-ahead RNG
- [ ] Incremental checksum computation
- [ ] Differential replay (store only differences)
- [ ] Network-distributed deterministic simulation

## Validation

### Test: Cross-Platform Determinism

```rust
#[test]
fn cross_platform_determinism() {
    let seed = 12345u64;
    let mut sim = Simulation::new(seed);
    
    // Run 1000 ticks
    for _ in 0..1000 {
        sim.step();
    }
    
    // This checksum must be identical on all platforms
    let checksum = sim.checksum();
    assert_eq!(checksum, EXPECTED_CHECKSUM_1000);
}
```

### Test: Replay Verification

```rust
#[test]
fn replay_verification() {
    let seed = 12345u64;
    let mut sim = Simulation::new(seed);
    
    // Record replay
    let mut replay = sim.start_recording();
    for _ in 0..1000 {
        sim.step();
        replay.checkpoint(&sim);
    }
    let replay_data = replay.finish();
    
    // Verify replay
    let mut new_sim = Simulation::new(seed);
    assert!(replay_data.verify(&mut new_sim).is_ok());
}
```

## Alternatives Considered

### Alternative 1: Relaxed Determinism (Rejected)

**Approach**: Allow minor non-determinism; focus on statistical equivalence

**Rejection Reason**: Policy analysis requires proving causality, not just correlation. Non-determinism breaks causal inference.

### Alternative 2: Record/Replay via System Capture (Rejected)

**Approach**: Use rr-project or similar to capture system execution

**Rejection Reason**: Too slow; requires Linux; doesn't enable cross-platform verification

### Alternative 3: Pure Functional Simulation (Rejected)

**Approach**: Immutable state, pure functions, no side effects

**Rejection Reason**: Performance penalty too high for 100K+ agents; difficult to express mutable agent state

## References

- [ChaCha RNG](https://cr.yp.to/chacha/chacha-20080128.pdf) — Bernstein, 2008
- [Fixed-Point Arithmetic](https://docs.rs/fixed/latest/fixed/) — Rust fixed crate
- [Deterministic Simulation](https://factorio.com/blog/post/fff-201) — Factorio blog
- [Floating-Point Determinism](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/) — Bruce Dawson
- [ECS Architecture](https://github.com/hecs-systems/hecs) — Rust ECS

---

*This ADR defines the core determinism architecture of Civis. All simulation code must adhere to these principles.*
