# Comparison Matrix

## Feature Comparison

This document compares **civ** with similar tools in the deterministic simulation and policy-driven architecture space.

| Repository | Purpose | Key Features | Language/Framework | Maturity | Comparison |
|------------|---------|--------------|-------------------|----------|------------|
| **civ (this repo)** | Deterministic simulation & policy-driven architecture | Simulation, Policy enforcement, Deterministic execution | Rust | Stable | Research-grade simulation |
| [TLA+](https://github.com/tlaplus/tlaplus) | Formal specifications | Model checking, Temporal logic | Java | Stable | Industry standard formal method |
| [Alloy](https://github.com/alloy-lang/alloy) | Lightweight formal methods | Relational modeling, BMC | Java | Stable | Formal verification |
| [ cadCAD](https://github.com/cadCAD-org/cadCAD) | Policy simulation | Multi-agent simulation, Jupyter | Python | Stable | Agent-based simulation |
| [FBP](https://github.com/flow-based-programming/flow-based-programming) | Dataflow programming | Async messaging, Components | Various | Stable | Dataflow paradigm |
| [Pony](https://github.com/ponylang/pony) | Actor model | Capabilities, No race conditions | Pony | Beta | Actor-based |
| [Elixir](https://github.com/elixir-lang/elixir) | Concurrent runtime | OTP, Fault tolerance | Erlang VM | Stable | Production-grade actors |

## Detailed Feature Comparison

### Determinism & Reproducibility

| Feature | civ | cadCAD | TLA+ | Alloy |
|---------|-----|--------|------|-------|
| Deterministic Execution | ✅ | ✅ | ✅ | ✅ |
| Reproducible Simulations | ✅ | ✅ | ✅ | ✅ |
| Time-travel Debugging | ❌ | ✅ | ✅ | ❌ |
| Event Replay | ✅ | ✅ | ✅ | ❌ |

### Policy Enforcement

| Feature | civ | TLA+ | cadCAD | Alloy |
|---------|-----|------|--------|-------|
| Policy as Code | ✅ | ✅ | ✅ | ❌ |
| Runtime Enforcement | ✅ | ❌ | ✅ | ❌ |
| Formal Verification | ❌ | ✅ | ❌ | ✅ |
| Invariant Checking | ❌ | ✅ | ✅ | ✅ |

### Architecture Patterns

| Feature | civ | FBP | Pony | Elixir |
|---------|-----|-----|------|--------|
| Dataflow | ✅ | ✅ | ❌ | ✅ |
| Actor Model | ❌ | ❌ | ✅ | ✅ |
| Message Passing | ✅ | ✅ | ✅ | ✅ |
| Formal Specification | ❌ | ❌ | ❌ | ❌ |

## Unique Value Proposition

civ provides:

1. **Deterministic Simulation**: Reproducible execution for testing and verification
2. **Policy-Driven**: Architecture-level policy enforcement
3. **Rust-Based**: Memory safety, performance, and type system guarantees
4. **Phenotype Ecosystem**: Part of the Phenotype design system

## Use Cases

| Use Case | Recommended Tool |
|----------|-----------------|
| Complex distributed systems verification | TLA+ |
| Multi-agent simulation with policies | cadCAD |
| Formal verification of invariants | Alloy |
| Production actor systems | Elixir |
| Research-grade simulation | civ |

## Documentation Structure

civ provides extensive documentation:

```
civ/
├── src/                    # Core implementation
├── docs/
│   ├── wiki/              # Concept & architecture knowledge
│   ├── development-guide/ # Contributor guides
│   ├── api/              # API documentation
│   └── roadmap/         # Planning artifacts
└── Cargo.toml            # Rust project
```

## References

- TLA+: [tlaplus/tlaplus](https://github.com/tlaplus/tlaplus)
- Alloy: [alloy-lang/alloy](https://github.com/alloy-lang/alloy)
- cadCAD: [cadCAD-org/cadCAD](https://github.com/cadCAD-org/cadCAD)
