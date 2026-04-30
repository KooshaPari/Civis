# Civis Charter

## 1. Mission Statement

**Civis** (CivLab) is a headless Civilization simulation engine designed for deterministic, large-scale, multi-agent computational social science research. The mission is to provide a high-performance, reproducible simulation platform that enables researchers to model complex civilizations, test hypotheses about societal development, and generate synthetic data for AI training while guaranteeing bit-identical results across all platforms and runs.

The project exists to be the gold standard for computational social science simulation—where determinism is non-negotiable, scale is limited only by hardware, and every simulation tick is fully reproducible from event logs. Civis enables researchers to ask "what if" questions about civilization development with scientific rigor.

---

## 2. Tenets (Unless You Know Better Ones)

### Tenet 1: Determinism Above All

Identical input must produce identical output across platforms, compiler versions, and runs. Floating-point arithmetic is forbidden in simulation logic—fixed-point i64 with 10^6 scale factor is mandatory. RNG is seeded once per run (ChaCha8Rng) with all calls logged for replay. Determinism is the foundation; everything else is secondary.

### Tenet 2. Data-Oriented Design

ECS (Entity Component System) architecture with Hecs ensures cache-friendly memory layout. Components are plain data structs. Systems are pure functions over component queries. No OOP inheritance. No virtual dispatch in hot paths. Memory layout optimized for SIMD and vectorization.

### Tenet 3. Fixed-Point Precision

All simulation arithmetic uses fixed-point i64 @ 10^6 scale. Six decimal places provide sufficient precision for economic calculations (prices, production rates). Multiplication and division use i128 intermediates to prevent overflow. No floating-point rounding errors accumulate over millions of ticks.

### Tenet 4. Deterministic Randomness

Random number generation is fully deterministic and reproducible. ChaCha8Rng provides cryptographic-quality randomness with deterministic seeding. Every RNG call is logged with tick number, entity ID, and result. Replays reproduce identical RNG sequences.

### Tenet 5. Reproducible Research

Every simulation run produces a complete event log enabling full replay. Researchers can reconstruct any simulation state at any tick. Event logs are the primary research artifact. Snapshots enable fast-forward to arbitrary points.

### Tenet 6. Performance Within Determinism

Target <16ms tick budget at standard simulation scale. Determinism constraints accepted as performance trade-offs. No compromises on determinism for speed. Profile-guided optimization within deterministic constraints.

### Tenet 7. Scientific Rigor

Simulation results must be interpretable by social scientists. Clear documentation of assumptions. Validated against historical data where possible. Uncertainty quantified and reported. Peer-reviewable methodology.

---

## 3. Scope & Boundaries

### In Scope

**Simulation Core:**
- ECS world management with Hecs
- Fixed-point arithmetic primitives
- Deterministic RNG subsystem
- Event logging and replay system
- Snapshot save/load for fast-forward
- Tick scheduler with <16ms budget

**Civilization Systems:**
- Population dynamics (birth, death, migration)
- Resource production and consumption
- Economy (trade, prices, markets)
- Technology research and diffusion
- Cultural development and spread
- Political systems and governance
- Warfare and conflict resolution
- Diplomacy and international relations

**Research Infrastructure:**
- Scenario definition format
- Parameter sweep configuration
- Experiment orchestration
- Results aggregation and export
- Statistical analysis utilities

**Integration:**
- Python bindings for researcher workflows
- Jupyter notebook integration
- Data export (CSV, Parquet, Arrow)
- Visualization data generation

### Out of Scope

- Real-time graphics rendering (headless only)
- User interface for direct interaction
- Game mechanics or player agency
- 3D terrain or spatial simulation
- Networked multiplayer
- Machine learning model training (exports data for external training)

### Boundaries

- Simulation is headless: no display required
- Output is data: visualization is external
- Scale is configurable: from village to planet
- Determinism is absolute: no compromises
- Research is primary: entertainment is secondary

---

## 4. Target Users & Personas

### Primary Persona: Computational Social Scientist Dr. Sarah Chen

**Role:** Researcher studying civilization development patterns
**Goals:** Run reproducible simulations, test hypotheses, publish findings
**Pain Points:** Non-deterministic results, difficult parameter sweeps, opaque simulation logic
**Needs:** Deterministic output, parameter configuration, statistical analysis tools
**Tech Comfort:** High, comfortable with Python and data analysis

### Secondary Persona: AI Researcher Dr. Marcus Johnson

**Role:** Machine learning researcher generating synthetic training data
**Goals:** Generate diverse civilization scenarios for model training
**Pain Points:** Limited training data, expensive human labeling, data bias
**Needs:** Bulk simulation runs, diverse scenario generation, export formats
**Tech Comfort:** Very high, ML pipeline expert

### Tertiary Persona: Historian Dr. Elena Rodriguez

**Role:** Historian testing counterfactual scenarios
**Goals:** Explore "what if" historical questions with simulation
**Pain Points:** Lack of rigorous tools, difficulty validating against history
**Needs:** Historical parameter sets, validation tools, uncertainty quantification
**Tech Comfort:** Medium, learning computational methods

### Quaternary Persona: Game Designer Greg

**Role:** Strategy game designer researching realistic mechanics
**Goals:** Understand real civilization dynamics for game design
**Pain Points:** Unrealistic game mechanics, lack of historical grounding
**Needs:** Accessible simulation parameters, visualization data, documentation
**Tech Comfort:** High, game development background

### Quinary Persona: Student Researcher Sam

**Role:** Graduate student learning computational social science
**Goals:** Learn simulation methodology, reproduce published results
**Pain Points:** Steep learning curve, complex setup, unclear documentation
**Needs:** Tutorials, example scenarios, clear documentation
**Tech Comfort:** Medium, learning Rust and simulation concepts

---

## 5. Success Criteria (Measurable)

### Determinism Metrics

- **Bit-Identical Replays:** 100% of runs with identical seeds produce identical event logs
- **Cross-Platform Consistency:** Identical output on Linux, macOS, Windows, ARM, x86_64
- **Compiler Independence:** Consistent output across Rust compiler versions
- **Replay Fidelity:** Reconstruction from event log matches original with 100% accuracy

### Performance Metrics

- **Tick Budget:** <16ms per tick at standard scale (10,000 entities)
- **Scale Limits:** Support for 1M+ entities on high-end hardware
- **Memory Efficiency:** <1GB RAM per 100,000 entities
- **Event Log Throughput:** 1M+ events/second write performance

### Research Quality Metrics

- **Parameter Coverage:** All documented parameters affect simulation output
- **Validation Data:** Published comparisons with historical data where applicable
- **Documentation:** 100% of systems documented with scientific assumptions
- **Reproducibility:** Published scenarios reproduceable by external researchers

### Adoption Metrics

- **Research Citations:** Target 10+ academic citations within 2 years
- **Scenario Library:** 50+ documented example scenarios
- **Integration Usage:** 5+ external tools consuming Civis output
- **Tutorial Completeness:** Step-by-step tutorials for common use cases

---

## 6. Governance Model

### Component Organization

```
Civis/
├── core/                # Simulation engine
│   ├── ecs/             # ECS world and components
│   ├── fixed/           # Fixed-point arithmetic
│   ├── rng/             # Deterministic randomness
│   └── scheduler/       # Tick scheduling
├── systems/             # Civilization systems
│   ├── population/      # Demographics
│   ├── economy/         # Markets and trade
│   ├── technology/      # Research and diffusion
│   ├── culture/         # Cultural dynamics
│   ├── politics/        # Governance
│   └── warfare/         # Conflict
├── simulation/            # Scenario and experiment management
├── replay/              # Event log and reconstruction
├── bindings/            # Python and other language bindings
└── research/            # Research utilities and validation
```

### Research Collaboration Process

**New System Development:**
- Scientific literature review
- Model specification with assumptions
- Implementation with determinism verification
- Validation against reference data if available
- Documentation for peer review

**Parameter Addition:**
- Sensitivity analysis required
- Documentation of parameter effects
- Default value justification
- Range validation rules

### Determinism Verification

- CI runs with multiple seeds verify identical output
- Cross-platform CI matrix (Linux, macOS, Windows)
- Replay tests for every PR
- Event log hash verification

---

## 7. Charter Compliance Checklist

### For New Simulation Systems

- [ ] Fixed-point arithmetic used throughout
- [ ] No floating-point in simulation logic
- [ ] Deterministic RNG with logged calls
- [ ] Event logging for all state changes
- [ ] Scientific assumptions documented
- [ ] Validation against reference data attempted
- [ ] Performance within tick budget

### For System Modifications

- [ ] Determinism regression tests pass
- [ ] Event log format compatibility maintained
- [ ] Replay compatibility verified
- [ ] Performance benchmarked
- [ ] Documentation updated

### For Release Preparation

- [ ] All determinism tests pass on all platforms
- [ ] Event logs from reference scenarios match
- [ ] Documentation complete
- [ ] Tutorial scenarios verified
- [ ] Benchmark results documented

---

## 8. Decision Authority Levels

### Level 1: System Maintainer Authority

**Scope:** Bug fixes, performance optimizations within determinism constraints, documentation updates
**Process:** Maintainer approval
**Examples:** Tick scheduler tuning, memory layout optimization

### Level 2: System Development Authority

**Scope:** New civilization systems, new components, new systems
**Process:** RFC with determinism impact assessment, peer review
**Examples:** New economic model, cultural system addition

### Level 3: Architecture Authority

**Scope:** ECS changes, fixed-point precision changes, RNG algorithm changes
**Process:** Written ADR, determinism verification protocol, steering approval
**Examples:** Precision increase, new RNG algorithm

### Level 4: Research Integration Authority

**Scope:** Python bindings changes, export format changes, research workflow modifications
**Process:** Researcher feedback, technical review, steering approval
**Examples:** New export format, breaking API changes

### Level 5: Scientific Authority

**Scope:** Core simulation assumptions, validation methodology, research partnerships
**Process:** Academic advisory input, executive approval
**Examples:** Paradigm shift in simulation approach, major research collaboration

---

## 9. Security & Compliance Considerations

### Simulation Integrity

- Event logs cryptographically signed for research integrity
- Determinism verification prevents tampering
- Reproduction requirement prevents result fabrication

### Data Management

- Event logs may contain sensitive research data
- Access controls for unpublished research
- Retention policies for large simulation datasets
- Export controls for potentially sensitive scenarios

### Reproducibility Standards

- Version pinning for all dependencies
- Containerized execution environments
- Complete environment documentation
- Determinism audit logs

---

## 10. Operational Guidelines

### Simulation Execution

- Containerized execution recommended
- Resource limits prevent runaway simulations
- Checkpointing for long-running simulations
- Parallel execution for parameter sweeps

### Data Management

- Event log compression for storage efficiency
- Tiered storage: hot (SSD), warm (disk), cold (archive)
- Snapshot lifecycle management
- Backup strategies for valuable simulations

### Research Workflows

- Jupyter integration for interactive analysis
- Batch execution for parameter sweeps
- Statistical analysis utilities included
- Visualization data export to external tools

---

## 11. Integration Points

### Phenotype Ecosystem

- **AgilePlus:** Feature tracking and research milestones
- **pheno-evaluation:** Benchmark and result aggregation
- **phenodocs:** Research documentation and publication

### External Research Tools

- **Python/Pandas:** Data analysis workflows
- **Jupyter:** Interactive research notebooks
- **Gnuplot/Matplotlib:** Visualization
- **R:** Statistical analysis
- **TensorFlow/PyTorch:** ML training on synthetic data

### Data Formats

- **Event Log:** Custom binary format with text export
- **Snapshots:** MessagePack compressed
- **Exports:** CSV, Parquet, Arrow
- **Scenarios:** YAML configuration files

---

*This charter governs Civis, the deterministic civilization simulation engine. Reproducible research requires rigorous engineering.*

*Last Updated: April 2026*
*Next Review: July 2026*

---

## 12. Development Workflows

### Local Development Setup

1. Clone the repository
2. Install required dependencies per project documentation
3. Run initial setup scripts if available
4. Verify setup by running tests
5. Configure IDE/editor with project settings
6. Set up pre-commit hooks if applicable

### Contribution Process

1. Create feature branch from main
2. Implement changes with tests
3. Ensure all quality checks pass
4. Update documentation for API changes
5. Create pull request with description
6. Address review feedback
7. Merge after approval and CI pass

### Testing Requirements

- Unit tests for all new functionality
- Integration tests for feature workflows
- Performance benchmarks for critical paths
- End-to-end tests for user scenarios
- Regression tests for bug fixes

### Release Management

1. Update version according to semver
2. Update CHANGELOG with all changes
3. Run full test suite
4. Create release tag
5. Build and publish artifacts
6. Update documentation references

---

## 13. Quality Standards

### Code Quality

- Follow project style guidelines
- Maintain test coverage thresholds
- No linting errors
- Static analysis passes
- Security scan clean

### Documentation Quality

- All public APIs documented
- README accurate and current
- Architecture decisions in ADRs
- Examples for common use cases
- Troubleshooting guides maintained

### Performance Standards

- Benchmarks meet targets
- No performance regressions
- Resource usage optimized
- Scalability tested

---

*Last Updated: April 2026*
*Next Review: July 2026*
