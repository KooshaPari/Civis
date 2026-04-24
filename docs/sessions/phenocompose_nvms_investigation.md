# Repository Investigation: phenocompose and nvms

## Executive Summary

You own **one unified repository** on GitHub:

- **phenocompose** (PRIMARY) - `https://github.com/KooshaPari/phenocompose.git`
- **nvms** (DOES NOT EXIST separately) - Referenced as "BytePort/nvms" in historical context, but fully merged into phenocompose

The README.md explicitly states: "**Merged Implementation**: KooshaPari/nanovms + BytePort/nvms + PhenoCompose Driver"

---

## PHENOCOMPOSE DETAILED ANALYSIS

### Purpose
**PhenoCompose** is a **unified multi-tier application isolation platform** for:
- AI agents requiring ephemeral desktop environments
- Game automation testing with GPU passthrough
- CI/CD pipelines with high-density containers
- Edge computing on consumer hardware
- Research HPC with GPU acceleration

### Architecture Overview
```
┌─────────────────────────────────────────┐
│     PhenoCompose (Unified Interface)     │ Rust driver
├─────────────────────────────────────────┤
│              NVMS Core (Go)              │ 3-tier orchestration
├─────────────────────────────────────────┤
│  Tier 1 (WASM)  │ Tier 2 (gVisor)       │ Tier 3 (Firecracker)
│  ~1ms startup   │ ~90ms startup         │ ~125ms startup
│  Fast, trusted  │ Browser automation    │ Full isolation
└─────────────────────────────────────────┘
```

### Technology Stack

| Component | Language | Purpose | Status |
|-----------|----------|---------|--------|
| **nanovms core** | Go | VM orchestration, hypervisor abstractions, CLI | ✅ Complete |
| **pheno-compose-driver** | Rust | Unified interface, FFI bindings | ✅ Complete |
| **Firecracker adapter** | Go | MicroVM (<125ms) management | ✅ Complete |
| **gVisor adapter** | Go | Container sandboxing (~90ms) | ✅ Complete |
| **WASM runtime** | Go + Wasmtime | Language-level isolation (~1ms) | ✅ Complete |
| **Linux/macOS/Windows adapters** | Go | Platform-specific hypervisor support | ✅ Complete |
| **FFI bindings** | Rust | C, Go, Rust, Zig, Mojo interop | ✅ Complete (recent: ebb5744) |
| **GPU passthrough** | Go | VFIO support for gaming VMs | Implemented |
| **Documentation** | TypeScript/VitePress | Architecture, journeys, ADRs | ✅ VitePress site |

### Key Features

1. **3-Tier Isolation Model**
   - Tier 1 (WASM): ~1ms startup, Wasmtime runtime, language-level sandboxing
   - Tier 2 (gVisor): ~90ms startup, syscall isolation, browser automation support
   - Tier 3 (Firecracker): ~125ms startup, full VM isolation, GPU passthrough capable

2. **Platform Support**
   - macOS: Native + Lima/Colima + Virtualization.framework
   - Linux: Native Firecracker + KVM
   - Windows: WSL2 integration + Hyper-V support

3. **Game Automation** (High relevance to DINOForge)
   - Parallel game test fleet (100+ VMs)
   - GPU passthrough (VFIO) for rendering tests
   - Steam headless integration
   - BepInEx plugin loader support (modding framework)
   - Snapshot-based cloning (<2s VM restore)
   - Looking Glass (low-latency remote display)

4. **Consumer Hardware Optimization**
   - Budget profile: 4C/8T, 16GB RAM
   - Mid profile: 8C/16T, 32GB RAM
   - Enthusiast profile: 16C/32T, 64GB RAM
   - CPU P-state tuning, huge pages, NUMA binding

### File Structure
```
phenocompose/
├── cmd/nanovms/              # CLI entry point
├── internal/
│   ├── adapters/             # Platform implementations
│   │   ├── linux/
│   │   ├── mac/
│   │   ├── windows/
│   │   ├── sandbox/
│   │   └── wasm/
│   ├── domain/               # Core models (VM, Sandbox, etc.)
│   └── ports/                # Interfaces (hexagonal architecture)
├── bindings/
│   ├── rust-ffi/             # Rust FFI to Go
│   ├── go-c-export/          # C export from Go
│   └── mojo/                 # Mojo bindings (GPU)
├── pheno-compose-driver/     # Rust driver (lib + staticlib + cdylib)
├── integrations/
│   └── pheno-compose/        # Driver integration docs
├── docs/                     # VitePress documentation
│   ├── journeys/             # User journeys (game-automation.md, etc.)
│   ├── adr/                  # Architecture Decision Records (14 ADRs)
│   └── guides/               # Quickstart, architecture
├── tests/                    # Unit + integration tests
├── SPEC.md                   # 91KB comprehensive specification
├── PLAN.md                   # Implementation roadmap (6 phases)
├── AGENTS.md                 # Language policy, agent rules
├── LANGUAGES.md              # 22KB language selection criteria
└── go.mod                    # Go module: github.com/kooshapari/nanovms v1.23
```

### Key Files & Their Purpose

| File | Size | Purpose |
|------|------|---------|
| **SPEC.md** | 91KB | Comprehensive spec covering SOTA cloud infra, WASM, eBPF, DPDK, io_uring, Firecracker, gVisor |
| **PLAN.md** | 15KB | 6-phase roadmap (Core, Game Automation, Consumer HW, Production) |
| **AGENTS.md** | 7KB | Language policy (Go/Rust/Zig tier 1; Python/C# tier 2), agent rules |
| **LANGUAGES.md** | 22KB | Deep language selection matrix for ML/HPC/gaming workloads |
| **docs/journeys/game-automation.md** | Key journey showing parallel game testing, GPU passthrough, fleet orchestration |
| **Makefile.go** | Build configuration |
| **go.mod** | Go 1.23 module (NOT 1.22) |

### Current Development Status

**Latest commits** (as of clone 2026-04-23):
1. `ebb5744` - "feat: Mojo ML bindings with full GPU optimizations" (GPU ML)
2. `52c6718` - "feat: complete FFI bindings with GPU optimizations"
3. `9ee0908` - "feat: add FFI bindings for Go/Rust/Zig/Mojo integration"
4. `0689e22` - "feat: add comprehensive language policy"
5. `74555d7` - "feat: unified NVMS with PhenoCompose integration"

**Phase Status** (per PLAN.md):
- Phase 1 (Core Foundation): Mostly complete, scaffolding + CI/CD done
- Phase 2 (Game Automation): In progress (Week 14+)
- Phase 3+ (Consumer HW, Production): Planned

### License
**Apache-2.0**

### Dependencies
- Go 1.23
- Firecracker (binary dependency)
- gVisor/runsc (binary dependency)
- Wasmtime (for WASM tier)
- containerd (for container orchestration)
- Rust (for pheno-compose-driver)
- LibC / system libraries (Linux, macOS, Windows)

---

## NVMS REPOSITORY STATUS

### The Situation
**nvms repository does NOT exist as a separate GitHub repo.**

The README references it as:
- `BytePort/nvms` - Historical source (AWS deployment patterns)
- `KooshaPari/nanovms` - Original implementation (3-tier isolation)
- **Merged into phenocompose** - Current unified codebase

The intention was to merge:
1. **KooshaPari/nanovms** -> 3-tier isolation core
2. **BytePort/nvms** -> AWS Firecracker orchestration
3. **PhenoCompose** -> Rust driver + unified interface

**Result**: Single phenocompose repo containing everything. The "nvms" references are CLI command names, not separate repos.

---

## INTEGRATION POTENTIAL WITH DINOFORGE

### 1. As MCP Server Tool (HIGHEST VALUE)
**Current Status**: phenocompose is perfect for DINOForge's game automation via MCP

**Use Case**: Replace/augment the current `game_launch`, `game_launch_test` tools with phenocompose capabilities

**Integration Points**:
```
DINOForge MCP Server (Python)
    ↓
pheno-compose-driver (Rust FFI)
    ↓
nanovms (Go) [Firecracker backend]
    ↓
Game automation fleet (100 parallel DINO instances)
```

**Advantages over current setup**:
- Unified CLI for VM fleet management
- 3-tier isolation (WASM for fast tools, gVisor for browsers, Firecracker for full VMs)
- GPU passthrough support (VFIO) for visual regression tests
- Snapshot-based cloning (instant VM restore from baseline)
- Looking Glass remote display (low-latency headless game viewing)
- Steam integration (BepInEx-compatible)
- Cross-platform (macOS, Linux, Windows via WSL2)

**Proposed MCP Commands**:
```
game_launch_fleet       # Launch N parallel game instances
game_fleet_status       # Monitor fleet health
game_snapshot_template  # Create game baseline snapshot
game_restore_snapshot   # Clone new VM from snapshot
game_tier1_wasm         # Run lightweight tool in WASM (1ms)
game_tier2_gvisor       # Run browser automation in gVisor (90ms)
game_gpu_passthrough    # Direct GPU access for rendering tests
```

### 2. As CLI Tool for Dev Workflow
**Status**: Directly usable in `scripts/game/` as standalone tool

**Use Case**: Developers can script multi-VM test scenarios without DINOForge MCP

**Integration**:
- Add `nanovms` binary to DINOForge installer scripts
- Document in ASSET_PIPELINE_CLI.md (game automation section)
- Wrap in PowerShell/Bash convenience scripts

### 3. As Rust Library (MODERATE VALUE)
**Status**: pheno-compose-driver is published on crates.io (likely)

**Use Case**: Embed in DINOForge's Rust tooling (if you have Rust components)

**Current DINOForge Rust**: None identified in .NET codebase
**Impact**: Low unless DINOForge adds Rust layer (e.g., for bare-cua integration)

### 4. As Reference Architecture (DOCUMENTATION)
**Status**: Excellent educational value

**Use Case**: Copy patterns from phenocompose into DINOForge docs

**Patterns to adopt**:
- ADR (Architecture Decision Records) structure — phenocompose has 14 ADRs
- User journeys (game-automation.md, agent-desktop.md) — matches DINOForge's proof-of-features style
- 3-tier isolation model -> could inform DINOForge's backend selection strategy
- Language policy (AGENTS.md) — already similar to CLAUDE.md

### 5. Potential Conflicts/Dependencies

**Potential Issue #1: VFIO GPU Binding**
- phenocompose uses `nanovms vfio bind --gpu=01:00.0` to bind GPU to test VMs
- DINO currently runs on bare metal without VFIO
- Risk: Binding GPU to VFIO removes it from host OS temporarily
- Mitigation: Use second GPU or reserve phases (DINOForge runs on GPU 0, test fleet uses GPU 1)

**Potential Issue #2: Windows + WSL2 Complexity**
- phenocompose relies on WSL2 for Windows support
- DINOForge installer already handles WSL2
- No conflict; phenocompose would extend it

**Potential Issue #3: Port/Resource Conflicts**
- phenocompose might use same ports/resources as DINOForge MCP
- Mitigation: Run in separate process/container tier (gVisor tier 2)

**Potential Issue #4: Go vs .NET Runtime**
- phenocompose is Go-based, DINOForge is .NET 11
- No mutual exclusion — can coexist, different processes
- nanovms CLI is installed separately (no .NET dependency)

---

## RECOMMENDATION

### Short Term (v0.24.0)
1. **Evaluate phenocompose as backend** for `game_launch_test(hidden=True)` fleet expansion
   - Current: One TEST instance via CreateDesktop
   - Proposed: 100 parallel TEST instances via phenocompose + Firecracker
2. **Document integration in CLAUDE.md** (add phenocompose to game automation section)
3. **Add phenocompose to installer** (optional, advanced feature flag)

### Medium Term (v0.25+)
1. **Wrap nanovms CLI in MCP server** (pheno-compose MCP adapter)
   - Callable from Claude Code: `game_launch_fleet(count=50)`
2. **Add GPU passthrough detection** to game automation
   - Fallback: CreateDesktop if GPU unavailable
   - Preferred: VFIO passthrough if GPU available + configured

### Long Term (v0.26+)
1. **Adopt phenocompose's ADR pattern** for DINOForge architecture decisions
2. **Reference phenocompose journeys** in your own game automation docs
3. **Consider unifying under "agent desktop" umbrella** (phenocompose focuses on agent environments too)

### Do NOT Do
- Fork phenocompose (maintain as external dependency)
- Rewrite nanovms in C# (violates "wrap, don't handroll" rule)
- Use phenocompose's Mojo ML bindings (out of scope for DINOForge)

---

## Summary Table

| Aspect | phenocompose | nvms (separate) | Integration Risk |
|--------|--------------|-----------------|-----------------|
| **Repository Status** | Active, merged | Does not exist | N/A |
| **Primary Language** | Go + Rust | N/A | Low (cross-platform) |
| **License** | Apache-2.0 | N/A | Compatible (Apache) |
| **DINOForge Integration** | MCP backend, CLI tool | N/A | Medium (requires testing) |
| **Game Automation** | ✅ Tier 1 feature | N/A | ✅ Perfect fit |
| **Conflicts** | GPU binding, WSL2 deps | N/A | Manageable |
| **Maintenance Burden** | Monitor phenocompose upstream | N/A | Low (external) |

---

## Key Findings

### phenocompose Repository Details
- **GitHub**: https://github.com/KooshaPari/phenocompose.git
- **Primary Language**: Go (nanovms) + Rust (pheno-compose-driver)
- **API Type**: CLI (`nanovms` command), Rust FFI bindings, HTTP API (MCP-ready)
- **License**: Apache-2.0
- **Stability**: Actively maintained (latest commit 2026-04-23)
- **Game Automation Relevance**: ⭐⭐⭐⭐⭐ (Tier 1 feature)

### NVMS Repository Details
- **Status**: Does not exist as separate repo
- **Integration**: Fully merged into phenocompose
- **What it was**: 3 upstream projects merged (nanovms + BytePort/nvms + PhenoCompose)
- **Current form**: CLI commands like `nanovms deploy`, `nanovms game test`, etc.

### Integration Strategy
**Best Path Forward**: Use phenocompose as external CLI dependency, wrap key commands in MCP tools for seamless game fleet automation. Does not require code changes to phenocompose; just orchestration from DINOForge side.

